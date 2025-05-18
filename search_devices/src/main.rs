#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use fltk::{prelude::*, app, window::Window, group::Tabs, enums::FrameType};
use std::net::Ipv4Addr;
mod cidr_tab;
mod ip_list_tab;

fn main() {
    // FLTKアプリケーションを初期化
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 500, 400, "Ping Scanner GUI");
    let mut tabs = Tabs::new(0, 0, 500, 400, "");
    tabs.set_frame(FrameType::DownBox);
    tabs.begin();

    // チャネル初期化 (CIDR用のみ)
    let (s, r) = app::channel::<(Ipv4Addr, bool, String)>();

    // タブ構築
    let (_running, mut buff) = cidr_tab::build_cidr_tab(s);
    let (_running_list, mut buff_list, _list_input, _scan_list_btn, _stop_list_btn, _clear_list_btn, _display_list) = ip_list_tab::build_ip_list_tab();

    tabs.end();
    wind.end();
    wind.show();

    // イベントループ: CIDRタブの結果を処理
    while app.wait() {
        if let Some((ip, alive, host_info)) = r.recv() {
            let mark = if alive { "〇" } else { "×" };
            buff.append(&format!("{:<15} {:<7} {:<12} {}\n",
                ip, mark, if alive { "alive" } else { "unreachable" }, host_info));
        }
    }
}
