// 必要なクレートをインポート
use fltk::{
    prelude::*,
    app, window::Window, input::{Input, MultilineInput}, button::Button,
    group::{Pack, Tabs, Group}, text::{TextDisplay, TextBuffer}, frame::Frame,
};
use std::{process::Command, net::{Ipv4Addr, IpAddr}, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use ipnetwork::Ipv4Network;  // CIDR表記のネットワーク操作
use dns_lookup::lookup_addr;

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
    Command::new("ping").args(&args)
        .output().map(|o| o.status.success()).unwrap_or(false)
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
    let mut input = Input::new(10, 70, 300, 30, "");
    input.set_value("192.168.1.0/24");
    let mut btn = Button::new(320, 70, 80, 30, "Scan");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut display = TextDisplay::new(10, 110, 480, 260, "");
    let mut buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    let running = Arc::new(AtomicBool::new(false));
    cidr_group.end();

    // --- IPリストタブ ---
    // IPリストタブ (Tabバー下) 固定レイアウト
    let list_group = Group::new(0, 25, 500, 375, "IP List");
    list_group.begin();
    let _list_label = Frame::new(10, 30, 480, 30, "Enter IP addresses (one per line)");
    let list_input = MultilineInput::new(10, 70, 300, 100, "");
    let mut scan_list_btn = Button::new(320, 70, 80, 30, "Scan List");
    let mut stop_list_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut display_list = TextDisplay::new(10, 180, 480, 200, "");
    let mut buff_list = TextBuffer::default();
    display_list.set_buffer(buff_list.clone());
    let running_list = Arc::new(AtomicBool::new(false));
    // Note: コールバック内でバッファをクリアするため `buff2` を利用します

    list_group.end();

    tabs.end();
    wind.end();
    wind.show();

    // CIDR用チャネル
    let (s, r) = app::channel::<(Ipv4Addr, bool, String)>();
    // IPリスト用チャネル（非同期スキャン向け）
    let (s2, r2) = app::channel::<(Ipv4Addr, bool, String)>();

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
            // ヘッダー行
            let header = format!("{:<15} {:<12} {}\n", "IP Address", "Status", "Host Info");
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
        let s2 = s2.clone();
        let running_flag = running_list.clone();
        let mut buf_clone = buff_list.clone();
        move |_| {
            // スキャン開始
            running_flag.store(true, Ordering::SeqCst);
            // ヘッダー表示
            let header = format!("{:<15} {:<12} {}\n", "IP Address", "Status", "Host Info");
            buf_clone.set_text(&header);
            // 入力取得
            let lines: Vec<String> = input_cb.value()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if lines.is_empty() {
                buf_clone.append("[Error] IPアドレスが入力されていません\n");
                running_flag.store(false, Ordering::SeqCst);
                return;
            }
            // バックグラウンドでスキャン (Arc を内部でクローンして使用)
            {
                let flag_thread = running_flag.clone();
                let s2_thread = s2.clone();
                std::thread::spawn(move || {
                    for ip_str in lines {
                        if !flag_thread.load(Ordering::SeqCst) { break }
                        let (ip, alive, host) = if let Ok(addr) = ip_str.parse::<Ipv4Addr>() {
                            (addr, is_alive(&addr), lookup_addr(&IpAddr::V4(addr)).unwrap_or_default())
                        } else {
                            (Ipv4Addr::UNSPECIFIED, false, "Invalid IP".into())
                        };
                        // main スレッドへ送信
                        s2_thread.send((ip, alive, host));
                    }
                    flag_thread.store(false, Ordering::SeqCst);
                });
            }
        }
    });

    // イベントループ
    while app.wait() {
        // CIDRタブ結果
        if let Some((ip, alive, host)) = r.recv() {
            let status = if alive { "alive" } else { "unreachable" };
            buff.append(&format!("{:<15} {:<12} {}\n", ip, status, host));
        }
        // IPリストタブ結果
        if let Some((ip, alive, host)) = r2.recv() {
            let status = if alive { "alive" } else { "unreachable" };
            buff_list.append(&format!("{:<15} {:<12} {}\n", ip, status, host));
        }
    }
}
