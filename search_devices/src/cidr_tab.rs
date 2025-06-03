use fltk::{
    prelude::*,
    group::Group,
    frame::Frame,
    input::Input,
    button::Button,
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{net::{Ipv4Addr, IpAddr}, process::Command, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use ipnetwork::Ipv4Network;
use dns_lookup::lookup_addr;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// CIDRタブを構築し、実行フラグと結果バッファを返します
pub fn build_cidr_tab(
    sender: app::Sender<(Ipv4Addr, bool, String)>,
    input_repeat: fltk::input::IntInput,
    input_interval: fltk::input::IntInput,
    input_block: fltk::input::IntInput,
    input_timeout: fltk::input::IntInput,
    input_ttl: fltk::input::IntInput,
) -> (Arc<AtomicBool>, TextBuffer) {
    let cidr_group = Group::new(0, 25, 500, 375, "CIDR");
    cidr_group.begin();
    Frame::new(10, 30, 480, 30, "CIDR形式で入力 (例: 192.168.1.0/24)");
    let mut input = Input::new(10, 70, 200, 30, "");
    input.set_value("192.168.1.0/24");
    let mut scan_btn = Button::new(320, 70, 80, 30, "Scan");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");
    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut display = TextDisplay::new(10, 110, 480, 260, "");
    let buff = TextBuffer::default();
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
        let flag = running.clone();
        let mut buf_clone = buff.clone();
        scan_btn.set_callback(move |_| {
            flag.store(true, Ordering::SeqCst);
            buf_clone.set_text(&format!("{:<15} {:<7} {:<12} {}\n",
                "IP Address", "Result", "Status", "Host Info"));
            let seg = inp.value();
            let rep = input_repeat.clone();
            let to = input_timeout.clone();
            let bs = input_block.clone();
            let ttl = input_ttl.clone();
            let iv = input_interval.clone();
            let thread_flag = flag.clone();
            let sender_inner = s.clone();
            std::thread::spawn(move || {
                if let Ok(net) = seg.parse::<Ipv4Network>() {
                    for ip in net.iter() {
                        if !thread_flag.load(Ordering::SeqCst) { break }
                        if ip == net.network() || ip == net.broadcast() { continue }
                        // 最新のEnv設定を読み込む
                        let repeat = rep.value().parse::<u32>().unwrap_or(1);
                        let timeout = to.value().parse::<u32>().unwrap_or(1000);
                        let block_size = bs.value().parse::<u32>().unwrap_or(64);
                        let time_to_live = ttl.value().parse::<u32>().unwrap_or(128);
                        let interval_ms = iv.value().parse::<u64>().unwrap_or(1000);
                        // ping実行
                        let alive = {
                            let mut cmd = Command::new("ping");
                            cmd.creation_flags(CREATE_NO_WINDOW);
                            cmd.args(&[
                                "-n", &repeat.to_string(),
                                "-w", &timeout.to_string(),
                                "-l", &block_size.to_string(),
                                "-i", &time_to_live.to_string(),
                                &ip.to_string(),
                            ]).
                            output().map(|o| o.status.success()).unwrap_or(false)
                        };
                        // Envで指定した間隔待機
                        std::thread::sleep(Duration::from_millis(interval_ms));
                        let host_info = lookup_addr(&IpAddr::V4(ip)).unwrap_or_default();
                        sender_inner.send((ip, alive, host_info));
                    }
                }
                thread_flag.store(false, Ordering::SeqCst);
            });
        });
    }
    // 停止: フラグを折る
    {
        let flag = running.clone();
        stop_btn.set_callback(move |_| {
            flag.store(false, Ordering::SeqCst)
        });
    }
    cidr_group.end();
    (running, buff)
}
