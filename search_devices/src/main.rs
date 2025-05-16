// 必要なクレートをインポート
use fltk::{
    prelude::*,
    app, window::Window, input::Input, button::Button,
    group::Pack, text::{TextDisplay, TextBuffer}, frame::Frame,
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
    let mut pack = Pack::new(0, 0, 500, 400, "");
    pack.set_spacing(5);
    // 入力例を示す簡潔な説明ラベル
    let _label = Frame::new(0, 0, 500, 30, "CIDR形式で入力 (例: 192.168.1.0/24)");

    // セグメント入力とスキャンボタン (初期例をセット)
    let mut input = Input::new(0, 0, 300, 30, "");
    input.set_value("192.168.1.0/24");
    let mut btn = Button::new(0, 0, 80, 30, "Scan");
    // 停止ボタンを追加
    let mut stop_btn = Button::new(100, 0, 80, 30, "Stop");

    // 結果表示用スクロール付きテキスト
    // 高さを調整してウィンドウ内に収める
    let mut display = TextDisplay::new(0, 0, 500, 310, "");
    let mut buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    // スキャン制御用フラグ
    let running = Arc::new(AtomicBool::new(false));

    pack.end();
    wind.end();
    wind.show();

    // ボタン押下時、バッファをクリアし、スキャンを別スレッドで実行して per-IP status を送信
    let (s, r) = app::channel::<(Ipv4Addr, bool, String)>();
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

    // イベントループ
    while app.wait() {
        // 各IPごとのステータスを受信して追記表示
        if let Some((ip, alive, host_info)) = r.recv() {
            let status = if alive { "alive" } else { "unreachable" };
            let line = format!("{:<15} {:<12} {}\n", ip, status, host_info);
            buff.append(&line);
        }
    }
}
