use fltk::{
    prelude::*,
    group::Group,
    frame::Frame,
    input::MultilineInput,
    button::Button,
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{net::{Ipv4Addr, IpAddr}, process::Command, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use dns_lookup::lookup_addr;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 指定した IP に ping を実行し、生存を判定します
fn is_alive(ip: &Ipv4Addr, repeat: u32, timeout: u32, block_size: u32, ttl: u32) -> bool {
    let ip_str = ip.to_string();
    // Windows 用の引数
    let args = ["-n", &repeat.to_string(), "-w", &timeout.to_string(), "-l", &block_size.to_string(), "-i", &ttl.to_string(), &ip_str];
    let mut cmd = Command::new("ping");
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// IPリストタブを構築し、実行中フラグと結果バッファを返します
pub fn build_ip_list_tab(
    input_repeat: fltk::input::IntInput,
    input_interval: fltk::input::IntInput,
    input_block: fltk::input::IntInput,
    input_timeout: fltk::input::IntInput,
    input_ttl: fltk::input::IntInput,
) -> (Arc<AtomicBool>, TextBuffer, MultilineInput, Button, Button, Button, TextDisplay) {
    let list_group = Group::new(0, 25, 500, 375, "IP List");
    list_group.begin();
    Frame::new(10, 30, 480, 30, "Enter IP addresses (one per line)");
    let mut input = MultilineInput::new(10, 70, 200, 100, "");
    input.set_value("192.168.0.1\n192.168.0.2\n192.168.0.3");
    let mut scan_btn = Button::new(320, 70, 80, 30, "Scan List");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut display = TextDisplay::new(10, 180, 480, 200, "");
    let buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    // クリア処理
    {
        let mut b = buff.clone();
        clear_btn.set_callback(move |_| {
            b.set_text("")
        });
    }
    let running = Arc::new(AtomicBool::new(false));
    // スキャン開始処理
    {
        let inp = input.clone();
        let flag = running.clone();
        let mut buf_clone = buff.clone();
        // Envタブの設定取得用クローン
        let repeat_input = input_repeat.clone();
        let interval_input = input_interval.clone();
        let block_input = input_block.clone();
        let timeout_input = input_timeout.clone();
        let ttl_input = input_ttl.clone();
        scan_btn.set_callback(move |_| {
            flag.store(true, Ordering::SeqCst);
            // ヘッダー行：Result 列を追加
            let header = format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info");
            buf_clone.set_text(&header);
            // デバッグ: ボタンクリック検知
            buf_clone.append("[Debug] ScanList clicked\n");
            let lines: Vec<String> = inp.value()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if lines.is_empty() {
                buf_clone.append("[Error] IPアドレスが入力されていません\n");
                flag.store(false, Ordering::SeqCst);
                return;
            }
            // 同期スキャン: 列挙してバッファに追加
            for ip_str in lines {
                if !flag.load(Ordering::SeqCst) { break }
                if let Ok(addr) = ip_str.parse::<Ipv4Addr>() {
                    // Env設定から値をパース
                    let repeat = repeat_input.value().parse::<u32>().unwrap_or(1);
                    let interval = interval_input.value().parse::<u64>().unwrap_or(1000);
                    let block_size = block_input.value().parse::<u32>().unwrap_or(64);
                    let timeout = timeout_input.value().parse::<u32>().unwrap_or(1000);
                    let ttl = ttl_input.value().parse::<u32>().unwrap_or(128);
                    let alive = is_alive(&addr, repeat, timeout, block_size, ttl);
                    // Envで指定したインターバルを待機
                    std::thread::sleep(Duration::from_millis(interval));
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
            flag.store(false, Ordering::SeqCst);
        });
    }
    // 停止処理
    {
        let flag = running.clone();
        stop_btn.set_callback(move |_| {
            flag.store(false, Ordering::SeqCst)
        });
    }
    list_group.end();
    (running, buff, input, scan_btn, stop_btn, clear_btn, display)
}
