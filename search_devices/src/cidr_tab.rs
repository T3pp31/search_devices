use fltk::{
    prelude::*,
    frame::Frame,
    input::{Input, IntInput},
    button::Button,
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{net::{Ipv4Addr, IpAddr}, process::Command, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use ipnetwork::Ipv4Network;
use dns_lookup::lookup_addr;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::utils::{ms_to_secs_ceil, ping_args_unix, ping_args_windows};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// CIDRタブを構築し、実行フラグと結果バッファを返します
pub fn build_cidr_tab(sender: app::Sender<(String, Ipv4Addr, bool, String)>) -> (Arc<AtomicBool>, TextBuffer) {
    Frame::new(10, 30, 480, 30, "CIDR形式で入力 (例: 192.168.1.0/24)");
    let mut input = Input::new(10, 70, 200, 30, "");
    input.set_value("192.168.1.0/24");
    let mut scan_btn = Button::new(320, 70, 80, 30, "Scan");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");

    // Ping設定（Count / Timeout）
    let _count_label = Frame::new(10, 110, 60, 25, "Count");
    let mut count_inp = IntInput::new(70, 110, 60, 25, "");
    count_inp.set_value("1");
    let _timeout_label = Frame::new(140, 110, 100, 25, "Timeout(ms)");
    let mut timeout_inp = IntInput::new(240, 110, 80, 25, "");
    timeout_inp.set_value("1000");

    let mut display = TextDisplay::new(10, 140, 480, 230, "");
    let buff = TextBuffer::default();
    println!("[Debug] CIDR buffer created: {:p}", &buff);
    display.set_buffer(buff.clone());
    // クリア
    {
        let mut b = buff.clone();
        clear_btn.set_callback(move |_| {
            b.set_text("")
        });
    }
    let running = Arc::new(AtomicBool::new(false));
    // スキャン開始
    {
        let inp = input.clone();
        let s = sender.clone();
        println!("[Debug] CIDR: Using sender channel: {:p}", &s);
        let flag = running.clone();
        let mut buf_clone = buff.clone();
        let cnt_inp = count_inp.clone();
        let to_inp = timeout_inp.clone();
        scan_btn.set_callback(move |_| {
            // 実行フラグを立てる
            flag.store(true, Ordering::SeqCst);
            // ヘッダーを表示
            buf_clone.set_text(&format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info"));
            let seg = inp.value();
            let thread_flag = flag.clone();
            let sender_inner = s.clone();
            // 設定値の取得
            let count: u32 = cnt_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(1);
            let timeout_ms: u32 = to_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(1000);
            std::thread::spawn(move || {
                if let Ok(net) = seg.parse::<Ipv4Network>() {
                    for ip in net.iter() {
                        if !thread_flag.load(Ordering::SeqCst) { break }
                        if ip == net.network() || ip == net.broadcast() { continue }
                        // ping 実行
                        let alive = {
                            let mut cmd = Command::new("ping");

                            #[cfg(windows)]
                            {
                                cmd.creation_flags(CREATE_NO_WINDOW);
                                let args = ping_args_windows(count, timeout_ms, &ip.to_string());
                                cmd.args(&args);
                            }

                            #[cfg(not(windows))]
                            {
                                // Linuxの-Wは秒。ミリ秒→切り上げ秒へ変換
                                let args = ping_args_unix(count, timeout_ms, &ip.to_string());
                                cmd.args(&args);
                            }

                            cmd.output()
                                .map(|o| o.status.success())
                                .unwrap_or(false)
                        };
                        let host_info = lookup_addr(&IpAddr::V4(ip)).unwrap_or_default();
                        sender_inner.send(("CIDR".to_string(), ip, alive, host_info));
                    }
                }
                // 実行完了フラグを倒す
                thread_flag.store(false, Ordering::SeqCst);
            });
        });
    }
    // 停止
    {
        let flag = running.clone();
        stop_btn.set_callback(move |_| {
            flag.store(false, Ordering::SeqCst)
        });
    }
    (running, buff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_ping_args_for_cidr_tab() {
        let a = ping_args_windows(1, 1000, "192.168.1.10");
        assert_eq!(a, vec!["-n","1","-w","1000","192.168.1.10"]);
        let b = ping_args_unix(3, 2500, "10.0.0.5");
        assert_eq!(b, vec!["-c","3","-W","3","10.0.0.5"]);
    }

    #[test]
    fn test_ms_to_secs_in_cidr() {
        assert_eq!(ms_to_secs_ceil(1), 1);
        assert_eq!(ms_to_secs_ceil(1000), 1);
        assert_eq!(ms_to_secs_ceil(1001), 2);
    }
}
