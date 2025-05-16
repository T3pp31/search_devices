// 必要なクレートをインポート
use egui::CentralPanel;  // egui の中心パネルのみ
// egui + glium バックエンド統合
use egui_glium::EguiGlium;
// glium Display と Surface トレイト
use glium::{Display, Surface};
// glutin (winit) を glium 経由でインポート
use glium::glutin::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    ContextBuilder,
};
use std::{process::Command, net::Ipv4Addr};  // ping実行とIPアドレス

use ipnetwork::Ipv4Network;  // CIDR表記のネットワーク操作

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
    let event_loop: EventLoop<()> = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Ping Scanner GUI");
    // ContextBuilderでWindowedContextを生成
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true)
        .build_windowed(wb, &event_loop)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let window = windowed_context.window().clone();
    // glium Displayを生成: WindowedContextを渡して作成
    let display = unsafe { Display::from_gl_window(windowed_context).unwrap() };

    // egui + glium 統合インスタンス
    let mut egui = EguiGlium::new(&display, &window, &event_loop);

    // アプリ状態
    let mut segment = String::new();
    let mut is_scanning = false;
    let mut results: Vec<Ipv4Addr> = Vec::new();

    // イベントループ開始
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: window_event, .. } => {
                egui.on_event(&window_event);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // egui フレームを実行し、UIを構築
                egui.run(&window, |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Ping Scanner GUI");
                        ui.horizontal(|ui| {
                            ui.label("Segment:");
                            ui.text_edit_singleline(&mut segment);
                            if ui.button("Scan").clicked() {
                                results.clear();
                                is_scanning = true;
                                results = scan_network(parse_network(&segment));
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

                // 描画準備
                let mut target = display.draw();
                // 背景をクリア
                target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);
                // UIを描画
                egui.paint(&display, &mut target);
                target.finish().unwrap();
                window.request_redraw();
            }
            // ウィンドウ閉じるリクエスト
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
