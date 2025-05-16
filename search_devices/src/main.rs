// 必要なクレートをインポート
use fltk::{
    prelude::*,
    app, window::Window, input::Input, button::Button,
    group::Pack, text::{TextDisplay, TextBuffer}, frame::Frame,
};
use std::{process::Command, net::Ipv4Addr, time::Duration};
use ipnetwork::Ipv4Network;  // CIDR表記のネットワーク操作

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

    // 結果表示用スクロール付きテキスト
    // 高さを調整してウィンドウ内に収める
    let mut display = TextDisplay::new(0, 0, 500, 310, "");
    let mut buff = TextBuffer::default();
    display.set_buffer(buff.clone());

    pack.end();
    wind.end();
    wind.show();

    // ボタン押下時、バッファをクリアし、スキャンを別スレッドで実行して per-IP status を送信
    let (s, r) = app::channel::<(Ipv4Addr, bool)>();
    btn.set_callback({
        let inp = input.clone();
        let s = s.clone();
        // コールバック内でバッファ用クローンを可変に定義
        let mut buff_clone = buff.clone();
        move |_| {
            // スキャン前にバッファをクリア
            buff_clone.set_text("");
            let seg = inp.value();
            let s = s.clone();
            std::thread::spawn(move || {
                if let Some(net) = parse_network(&seg) {
                    for ip in net.iter() {
                        if ip == net.network() || ip == net.broadcast() {
                            continue;
                        }
                        let alive = is_alive(&ip);
                        s.send((ip, alive));
                    }
                }
            });
        }
    });

    // イベントループ
    while app.wait() {
        // 各IPごとのステータスを受信して追記表示
        if let Some((ip, alive)) = r.recv() {
            let line = format!("{}: {}\n", ip, if alive { "alive" } else { "unreachable" });
            buff.append(&line);
        }
    }
}
