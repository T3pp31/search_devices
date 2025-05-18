#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 必要なクレートをインポート
use fltk::{
    prelude::*,
    app, window::Window, input::{Input, MultilineInput}, button::Button,
    group::{Pack, Tabs, Group}, text::{TextDisplay, TextBuffer}, frame::Frame,
};
use std::{process::Command, net::{Ipv4Addr, IpAddr}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use ipnetwork::Ipv4Network;  // CIDR表記のネットワーク操作
use dns_lookup::lookup_addr;
use std::os::windows::process::CommandExt; // Windows向けプロセスフラグ用トレイト
const CREATE_NO_WINDOW: u32 = 0x08000000; // WinAPI: コンソールウィンドウを生成しない

/// ネットワーク文字列を解析し `Ipv4Network` 型を返します
fn parse_network(segment: &str) -> Option<Ipv4Network> {
    segment.parse().ok()
}

/// 指定したIPに ping を実行し、生存を判定します
fn is_alive(ip: &Ipv4Addr) -> bool {
    let ip_str = ip.to_string();
    let args = if cfg!(windows) {
        ["-n", "1", "-w", "1000", &ip_str]
    } else {
        ["-c", "1", "-W", "1", &ip_str]
    };
    // pingコマンドを生成し、Windowsではコンソールウィンドウを非表示に
    let mut cmd = Command::new("ping");
    #[cfg(windows)] {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// ネットワーク内の生存ホストを返します
fn scan_network(network: Ipv4Network) -> Vec<Ipv4Addr> {
    network.iter().filter_map(|ip| {
        if ip == network.network() || ip == network.broadcast() {
            None
        } else if is_alive(&ip) {
            Some(ip)
        } else {
            None
        }
    }).collect()
}

fn main() {
    // FLTKアプリケーションを初期化
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 500, 400, "Ping Scanner GUI");
    let mut tabs = Tabs::new(0, 0, 500, 400, "");
    // タブバーを枠付きで表示
    use fltk::enums::FrameType;
    tabs.set_frame(FrameType::DownBox);
    tabs.begin();

    // --- CIDRタブ ---
    // CIDRタブ (Tabバー下) 固定レイアウト
    let cidr_group = Group::new(0, 25, 500, 375, "CIDR");
    cidr_group.begin();
    let _label = Frame::new(10, 30, 480, 30, "CIDR形式で入力 (例: 192.168.1.0/24)");
    let mut input = Input::new(10, 70, 200, 30, "");
    input.set_value("192.168.1.0/24");
    let mut btn = Button::new(320, 70, 80, 30, "Scan");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    // クリアボタンを追加
    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut display = TextDisplay::new(10, 110, 480, 260, "");
    let mut buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    // クリアボタンのコールバック
    {
        let mut buff_clone = buff.clone();
        clear_btn.set_callback(move |_| {
            buff_clone.set_text("");
        });
    }
    let running = Arc::new(AtomicBool::new(false));
    cidr_group.end();

    // --- IPリストタブ ---
    // IPリストタブ (Tabバー下) 固定レイアウト
    let list_group = Group::new(0, 25, 500, 375, "IP List");
    list_group.begin();
    let _list_label = Frame::new(10, 30, 480, 30, "Enter IP addresses (one per line)");
    let list_input = MultilineInput::new(10, 70, 200, 100, "");
    let mut scan_list_btn = Button::new(320, 70, 80, 30, "Scan List");
    let mut stop_list_btn = Button::new(410, 70, 80, 30, "Stop");
    // クリアボタンを追加
    let mut clear_list_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut display_list = TextDisplay::new(10, 180, 480, 200, "");
    let buff_list = TextBuffer::default();
    display_list.set_buffer(buff_list.clone());
    // クリアボタンのコールバック
    {
        let mut buff_clone = buff_list.clone();
        clear_list_btn.set_callback(move |_| {
            buff_clone.set_text("");
        });
    }
    let running_list = Arc::new(AtomicBool::new(false));
    // Note: コールバック内でバッファをクリアするため `buff2` を利用します

    list_group.end();

    tabs.end();
    wind.end();
    wind.show();

    // CIDR用チャネル
    let (s, r) = app::channel::<(Ipv4Addr, bool, String)>();
    // IPリストでは同期スキャンを使用してチャンネルは不要

    // 停止ボタンのコールバック
    {
        let running_flag = running.clone();
        stop_btn.set_callback(move |_| {
            running_flag.store(false, Ordering::SeqCst);
        });
    }
    // スキャン開始ボタンのコールバック
    btn.set_callback({
        let inp = input.clone();
        let s = s.clone();
        let running_flag = running.clone();
        let mut buff_clone = buff.clone();
        move |_| {
            // スキャン開始
            running_flag.store(true, Ordering::SeqCst);
            // ヘッダー行：Result 列を追加
            let header = format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info");
            buff_clone.set_text(&header);
            let seg = inp.value();
            let s = s.clone();
            let running_thread = running_flag.clone();
            std::thread::spawn(move || {
                if let Some(net) = parse_network(&seg) {
                    for ip in net.iter() {
                        // 中断チェック
                        if !running_thread.load(Ordering::SeqCst) {
                            break;
                        }
                        if ip == net.network() || ip == net.broadcast() {
                            continue;
                        }
                        let alive = is_alive(&ip);
                        // IpAddr 型に変換して逆引き
                        let host_info = lookup_addr(&IpAddr::V4(ip)).unwrap_or_default();
                        s.send((ip, alive, host_info));
                    }
                    running_thread.store(false, Ordering::SeqCst);
                }
            });
        }
    });

    // IPリスト用停止ボタン
    {
        let running_flag2 = running_list.clone();
        stop_list_btn.set_callback(move |_| {
            running_flag2.store(false, Ordering::SeqCst);
        });
    }
    // IPリスト用スキャンボタン（非同期処理）
    scan_list_btn.set_callback({
        let input_cb = list_input.clone();
        let running_flag = running_list.clone();
        let mut buf_clone = buff_list.clone();
        move |_| {
            // スキャン開始
            running_flag.store(true, Ordering::SeqCst);
            // ヘッダー行：Result 列を追加
            let header = format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info");
            buf_clone.set_text(&header);
            // デバッグ: 入力行をUI上に表示
            let raw = input_cb.value();
            let lines: Vec<String> = raw
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if lines.is_empty() {
                buf_clone.append("[Error] IPアドレスが入力されていません\n");
                running_flag.store(false, Ordering::SeqCst);
                return;
            }
            // 同期スキャン: UI スレッド上で直接バッファに結果を追加
            for ip_str in lines.clone() {
                if !running_flag.load(Ordering::SeqCst) { break }
                if let Ok(addr) = ip_str.parse::<Ipv4Addr>() {
                    let alive = is_alive(&addr);
                    let status = if alive { "alive" } else { "unreachable" };
                    let host_info = lookup_addr(&IpAddr::V4(addr)).unwrap_or_default();
                    let mark = if alive { "〇" } else { "×" };
                    buf_clone.append(&format!("{:<15} {:<7} {:<12} {}\n",
                        addr, mark, status, host_info));
                } else {
                    buf_clone.append(&format!("{:<15} {:<7} {:<12} {}\n",
                        Ipv4Addr::UNSPECIFIED,
                        "×",
                        "invalid",
                        "Invalid IP"
                    ));
                }
            }
            running_flag.store(false, Ordering::SeqCst);
        }
    });

    // イベントループ
    while app.wait() {
        if let Some((ip, alive, host_info)) = r.recv() {
            let status = if alive { "alive" } else { "unreachable" };
            let mark = if alive { "〇" } else { "×" };
            buff.append(&format!("{:<15} {:<7} {:<12} {}\n",
                ip, mark, status, host_info));
        }
    }
}
