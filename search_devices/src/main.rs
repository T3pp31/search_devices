#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use fltk::{prelude::*, app, window::Window, group::{Tabs, Group}, enums::FrameType};
use std::net::Ipv4Addr;
mod cidr_tab;
mod ip_list_tab;
mod tracert_tab;
mod utils;

fn main() {
    // FLTKアプリケーションを初期化
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 500, 400, "Ping Scanner GUI");
    let mut tabs = Tabs::new(0, 0, 500, 400, "");
    tabs.set_frame(FrameType::DownBox);
    tabs.begin();

    // 単一チャネルでタブIDによる振り分け方式
    let (sender, receiver) = app::channel::<(String, Ipv4Addr, bool, String)>();
    println!("[Debug] Main: Shared channel - sender: {:p}, receiver: {:p}", &sender, &receiver);

    // CIDRタブの構築
    let cidr_group = Group::new(0, 25, 500, 375, "CIDR");
    cidr_group.begin();
    let (_running, mut buff) = cidr_tab::build_cidr_tab(sender.clone());
    println!("[Debug] Main received CIDR buffer: {:p}", &buff);
    cidr_group.end();
    
    // IP Listタブの構築
    let list_group = Group::new(0, 25, 500, 375, "IP List");
    list_group.begin();
    let (_running_list, mut buff_list, display_list) = ip_list_tab::build_ip_list_tab(sender.clone());
    println!("[Debug] Main received IP List buffer: {:p}", &buff_list);
    list_group.end();

    // Tracertタブの構築
    let tracert_group = Group::new(0, 25, 500, 375, "Tracert");
    tracert_group.begin();
    let (_running_tr, mut buff_tr, display_tr) = tracert_tab::build_tracert_tab(sender.clone());
    println!("[Debug] Main received Tracert buffer: {:p}", &buff_tr);
    tracert_group.end();

    tabs.end();
    wind.end();
    wind.show();

    // イベントループ
    println!("[Debug] Starting event loop");
    while app.wait() {
        // 単一チャンネルから受信してタブIDで振り分け
        if let Some((tab_id, ip, alive, host_info)) = receiver.recv() {
            println!("[Debug] Received result: tab_id={}, IP: {}, alive: {}", tab_id, ip, alive);
            
            let mark = if alive { "〇" } else { "×" };
            let status = if alive { "alive" } else { "unreachable" };
            
            match tab_id.as_str() {
                "CIDR" => {
                    println!("[Debug] Processing CIDR result");
                    buff.append(&format!("{:<15} {:<7} {:<12} {}\n",
                        ip, mark, status, host_info));
                }
                "IPLIST" => {
                    println!("[Debug] Processing IP List result");
                    // 無効なIPアドレスの場合の処理
                    if ip == Ipv4Addr::UNSPECIFIED && !alive && host_info == "Invalid IP" {
                        buff_list.append(&format!("{:<15} {:<7} {:<12} {}\n",
                            "Invalid", "×", "invalid", "Invalid IP"));
                    } else {
                        buff_list.append(&format!("{:<15} {:<7} {:<12} {}\n",
                            ip, mark, status, host_info));
                    }
                    println!("[Debug] Appended to buff_list - current text length: {}", buff_list.text().len());
                    
                    // TextDisplayを明示的に更新
                    if let Ok(mut display) = display_list.lock() {
                        display.redraw();
                        println!("[Debug] IP List display redrawn");
                    }
                    
                    // UIを更新
                    app::awake();
                    app::redraw();
                }
                "TRACERT" => {
                    println!("[Debug] Processing Tracert result");
                    if let Some(line) = crate::utils::sanitize_line(&host_info) {
                        buff_tr.append(&format!("{}\n", line));
                    }
                    if let Ok(mut display) = display_tr.lock() {
                        display.redraw();
                        println!("[Debug] Tracert display redrawn");
                    }
                    app::awake();
                    app::redraw();
                }
                _ => {
                    println!("[Debug] Unknown tab_id: {}", tab_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::sanitize_line;

    #[test]
    fn test_tracert_line_sanitization_in_main() {
        assert_eq!(sanitize_line("   abc   "), Some("abc".into()));
        assert!(sanitize_line("   ").is_none());
    }
}
