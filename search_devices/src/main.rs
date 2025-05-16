// 必要なクレートをインポート
use egui::{CentralPanel, Context};  // egui の中心パネルとコンテキスト
use egui_glium::EguiGlium;  // egui + glium バックエンド統合
use glium::{Display, Surface};
use glutin::event_loop::{EventLoop, ControlFlow};  // イベントループをglutinから使用
use glutin::window::WindowBuilder;    // WindowBuilderをglutinから使用
use glutin::ContextBuilder;
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
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Ping Scanner GUI");
    let gl_window = ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true)
        .build_windowed(wb, &event_loop)
        .unwrap();
    // OpenGLコンテキストをcurrentにする
    let gl_window = unsafe { gl_window.make_current().unwrap() };
    // winitのWindowを取得
    let window = gl_window.window().clone();
    // glium Displayを生成
    let display = unsafe { Display::from_context_surface(gl_window.context().unwrap(), gl_window).unwrap() };

    // egui + glium 統合インスタンス (Display, Window, EventLoop を渡す)
    let mut egui = EguiGlium::new(&display, window, &event_loop);

    // アプリ状態
    let mut segment = String::new();
    let mut is_scanning = false;
    let mut results: Vec<Ipv4Addr> = Vec::new();

    // イベントループ開始
    event_loop.run(move |event, _, control_flow| {
        match &event {
            // ウィンドウイベントはeguiに通知
            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(&event);
                // UI更新のためリクエスト
                window.request_redraw();
            }
            _ => {}
        }

        if let glutin::event::Event::RedrawRequested(_) = event {
            // egui フレームを実行し、UIを構築
            let needs_repaint = egui.run(&display, |ctx: &Context| {
                CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Ping Scanner GUI");
                    ui.horizontal(|ui| {
                        ui.label("Segment:");
                        ui.text_edit_singleline(&mut segment);
                        if ui.button("Scan").clicked() {
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

            // 描画準備
            let mut target = display.draw();
            target.clear_color(0.1, 0.1, 0.1, 1.0);
            // UIを描画
            egui.paint(&display, &mut target);
            target.finish().unwrap();

            // 再描画が必要ならリクエスト
            if needs_repaint {
                window.request_redraw();
            }
        }

        // ウィンドウ閉じるリクエスト
        if let glutin::event::Event::WindowEvent { event: glutin::event::WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit;
        }
    });
}
