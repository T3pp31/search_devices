#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use fltk::{prelude::*, app, window::Window, group::Tabs, enums::FrameType};
use std::net::Ipv4Addr;
mod cidr_tab;
mod ip_list_tab;
mod env_tab;

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
    let (_env_group, input_repeat, input_interval, input_block, input_timeout, input_ttl) = env_tab::build_env_tab();
    let (_running, mut buff) = cidr_tab::build_cidr_tab(
        s,
        input_repeat.clone(),
        input_interval.clone(),
        input_block.clone(),
        input_timeout.clone(),
        input_ttl.clone(),
    );
    let (_running_list, mut buff_list, _list_input, _scan_list_btn, _stop_list_btn, _clear_list_btn, _display_list) = ip_list_tab::build_ip_list_tab(
        input_repeat,
        input_interval,
        input_block,
        input_timeout,
        input_ttl,
    );

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
