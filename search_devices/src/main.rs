// 必要なクレートをインポート
use egui::{CentralPanel, Context};  // egui の中心パネルとコンテキスト
use egui_glium::EguiGlium;  // egui + glium バックエンド統合
use glium::{Display, glutin::{self, event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, ContextBuilder}, Surface};
use ipnetwork::Ipv4Network;  // CIDR表記のネットワーク操作
use std::{process::Command, net::Ipv4Addr};  // ping実行とIPアドレス

/// ネットワーク文字列を解析し `Ipv4Network` 型を返します
fn parse_network(segment: &str) -> Ipv4Network {
    segment.parse().expect("セグメントが不正です (例: 192.168.1.0/24)")
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
    // ウィンドウと OpenGL コンテキストのセットアップ
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Ping Scanner GUI");
    let cb = ContextBuilder::new().with_vsync(true).with_srgb(true);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    // egui + glium 統合インスタンス
    let mut egui = EguiGlium::new(&display);

    // アプリ状態
    let mut segment = String::new();
    let mut is_scanning = false;
    let mut results: Vec<Ipv4Addr> = Vec::new();

    // イベントループ開始
    event_loop.run(move |event, _, control_flow| {
        // egui にイベントを通知
        egui.on_event(&event);

        match event {
            // ウィンドウ閉じるリクエスト
            glutin::event::Event::WindowEvent { event: glutin::event::WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // 再描画要求
            glutin::event::Event::RedrawRequested(_) => {
                // egui フレームを実行
                let repaint = egui.run(&display, |ctx: &Context| {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Ping Scanner GUI");
                        ui.horizontal(|ui| {
                            ui.label("Segment:");
                            ui.text_edit_singleline(&mut segment);
                            if ui.button("Scan").clicked() {
                                // スキャン処理
                                results.clear();
                                is_scanning = true;
                                let network = parse_network(&segment);
                                results = scan_network(network);
                                is_scanning = false;
                            }
                        });
                        ui.separator();
                        if is_scanning {
                            ui.label("Scanning...");
                        }
                        ui.label("Alive Hosts:");
                        for ip in &results {
                            ui.label(ip.to_string());
                        }
                    });
                });
                // 再描画ループ継続
                if repaint {
                    display.gl_window().window().request_redraw();
                }
            }
            // その他: 必要に応じて再描画トリガ
            _ => {}
        }
    });
}
