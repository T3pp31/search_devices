use fltk::{
    prelude::*,
    frame::Frame,
    input::{MultilineInput, IntInput},
    button::Button,
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{net::{Ipv4Addr, IpAddr}, process::Command, sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex}, thread};
use dns_lookup::lookup_addr;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 指定した IP に ping を実行し、生存を判定します
fn is_alive(ip: &Ipv4Addr, count: u32, timeout_ms: u32) -> bool {
    let ip_str = ip.to_string();
    let mut cmd = Command::new("ping");
    
    #[cfg(windows)]
    {
        // Windows 用の引数
        let args = ["-n", &count.to_string(), "-w", &timeout_ms.to_string(), &ip_str];
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args(&args);
    }
    
    #[cfg(not(windows))]
    {
        // Linux/Unix 用の引数
        // -W は秒単位。ミリ秒→切り上げ秒へ変換
        let secs = std::cmp::max(1u32, (timeout_ms + 999) / 1000);
        let args = ["-c", &count.to_string(), "-W", &secs.to_string(), &ip_str];
        cmd.args(&args);
    }
    
    cmd.output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// IPリストタブを構築し、実行中フラグと結果バッファ、TextDisplayを返します
pub fn build_ip_list_tab(sender: app::Sender<(String, Ipv4Addr, bool, String)>) -> (Arc<AtomicBool>, TextBuffer, Arc<Mutex<TextDisplay>>) {
    Frame::new(10, 30, 480, 30, "Enter IP addresses (one per line)");
    let mut input = MultilineInput::new(10, 70, 200, 150, "");  // 高さを150に増加
    input.set_value("192.168.0.1\n192.168.0.2\n192.168.0.3");
    input.wrap();  // 自動改行を有効化
    let mut scan_btn = Button::new(320, 70, 80, 30, "Scan List");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    // Ping設定（Count / Timeout） - 同一行に整列
    let _count_label = Frame::new(240, 100, 60, 25, "Count");
    let mut count_inp = IntInput::new(300, 100, 60, 25, "");
    count_inp.set_value("1");
    let _timeout_label = Frame::new(370, 100, 80, 25, "Timeout (ms)");
    let mut timeout_inp = IntInput::new(450, 100, 50, 25, "");
    timeout_inp.set_value("1000");
    let mut display = TextDisplay::new(10, 230, 480, 150, "");  // Y位置を230に、高さを150に調整
    let buff = TextBuffer::default();
    println!("[Debug] IP List buffer created: {:p}", &buff);
    display.set_buffer(buff.clone());
    let display_ref = Arc::new(Mutex::new(display));
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
        let s = sender.clone();
        println!("[Debug] IP List: Using sender channel: {:p}", &s);
        scan_btn.set_callback(move |_| {
            // ヘッダー行：Result 列を追加
            let header = format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info");
            buf_clone.set_text(&header);
            buf_clone.append("[Debug] Scan started\n");

            let lines: Vec<String> = inp.value()
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if lines.is_empty() {
                buf_clone.append("[Error] IPアドレスが入力されていません\n");
                return;
            }

            // 設定値の取得
            let count: u32 = count_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(1);
            let timeout_ms: u32 = timeout_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(1000);

            flag.store(true, Ordering::SeqCst);
            let flag_clone = flag.clone();
            let sender = s.clone();

            // 別スレッドでスキャンを実行
            thread::spawn(move || {
                println!("[Debug] IP List: Thread started with {} IPs", lines.len());
                for ip_str in lines {
                    if !flag_clone.load(Ordering::SeqCst) { break }

                    if let Ok(addr) = ip_str.parse::<Ipv4Addr>() {
                        println!("[Debug] Checking IP: {}", addr);
                        let alive = is_alive(&addr, count, timeout_ms);
                        let host_info = lookup_addr(&IpAddr::V4(addr)).unwrap_or_default();
                        println!("[Debug] IP {} - alive: {}, host: {}", addr, alive, host_info);

                        // 結果をチャンネル経由で送信
                        println!("[Debug] IP List: About to send to channel {:p}", &sender);
                        sender.send(("IPLIST".to_string(), addr, alive, host_info));
                        println!("[Debug] IP List: Sent result for {}", addr);
                    } else {
                        // 無効なIPアドレスの場合
                        sender.send(("IPLIST".to_string(), Ipv4Addr::UNSPECIFIED, false, "Invalid IP".to_string()));
                    }
                }
                println!("[Debug] Thread finished");
                flag_clone.store(false, Ordering::SeqCst);
            });
        });
    }
    // 停止処理
    {
        let flag = running.clone();
        stop_btn.set_callback(move |_| {
            flag.store(false, Ordering::SeqCst)
        });
    }
    (running, buff, display_ref)
}
