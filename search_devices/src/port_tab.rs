use fltk::{
    prelude::*,
    frame::Frame,
    input::{Input, IntInput},
    button::Button,
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex},
    time::Duration,
    thread,
};
use dns_lookup::lookup_host;
use crate::utils::parse_ports;

/// Representative common TCP ports to scan.
const DEFAULT_PORTS: &[u16] = &[
    20, 21, 22, 23, 25, 53, 67, 68, 80, 110, 139, 143, 161, 389,
    443, 445, 587, 636, 993, 995, 1433, 1521, 1723, 3306, 3389,
    5900, 8080, 8443,
];

fn is_tcp_open(ip: Ipv4Addr, port: u16, timeout_ms: u64) -> bool {
    let addr = SocketAddr::from((ip, port));
    let timeout = Duration::from_millis(timeout_ms.max(1));
    TcpStream::connect_timeout(&addr, timeout).is_ok()
}

/// Build the Ports tab UI.
pub fn build_port_tab(
    sender: app::Sender<(String, Ipv4Addr, bool, String)>,
) -> (Arc<AtomicBool>, TextBuffer, Arc<Mutex<TextDisplay>>) {
    // Widen labels to avoid text clipping on some platforms
    Frame::new(10, 30, 480, 25, "Target (host or IPv4)");
    let mut target_inp = Input::new(10, 70, 200, 30, "");
    target_inp.set_value("127.0.0.1");

    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut scan_common_btn = Button::new(320, 70, 80, 30, "Common");
    let mut scan_custom_btn = Button::new(410, 70, 80, 30, "Custom");

    let _ports_label = Frame::new(10, 110, 480, 25, "Ports (e.g. 22,80,443 or 8000-8010)");
    let mut ports_inp = Input::new(10, 140, 260, 25, "");
    ports_inp.set_value("22,80,443");

    let _to_label = Frame::new(280, 140, 100, 25, "Timeout (ms)");
    let mut to_inp = IntInput::new(380, 140, 110, 25, "");
    to_inp.set_value("800");

    let mut display = TextDisplay::new(10, 175, 480, 195, "");
    let buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    let display_ref = Arc::new(Mutex::new(display));

    let running = Arc::new(AtomicBool::new(false));

    // Clear
    {
        let mut b = buff.clone();
        clear_btn.set_callback(move |_| b.set_text(""));
    }

    // Scan common ports
    {
        let s = sender.clone();
        let flag = running.clone();
        let mut b = buff.clone();
        let target_inp = target_inp.clone();
        let to_inp = to_inp.clone();
        let display_ref = display_ref.clone();
        scan_common_btn.set_callback(move |_| {
            if flag.load(Ordering::SeqCst) { return; }
            let target = target_inp.value();
            if target.trim().is_empty() {
                b.append("[Error] Target is empty.\n");
                return;
            }
            let timeout_ms: u64 = to_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(800) as u64;

            // Resolve target to an IPv4 address
            let ipv4 = resolve_target_ipv4(&target);
            let ip = match ipv4 {
                Some(ip) => ip,
                None => {
                    b.append("[Error] Failed to resolve target to IPv4.\n");
                    return;
                }
            };

            // Header
            b.set_text(&format!("{:<15} {:<7} {:<12} {}\n", "Target", "Result", "Status", "Info"));

            flag.store(true, Ordering::SeqCst);
            let flag_th = flag.clone();
            let sender = s.clone();
            let display_ref = display_ref.clone();
            thread::spawn(move || {
                for &port in DEFAULT_PORTS {
                    if !flag_th.load(Ordering::SeqCst) { break; }
                    let open = is_tcp_open(ip, port, timeout_ms);
                    sender.send(("PORTS".to_string(), ip, open, format!("{}/tcp", port)));
                }
                if let Ok(mut display) = display_ref.lock() { display.redraw(); }
                flag_th.store(false, Ordering::SeqCst);
            });
        });
    }

    // Scan custom ports
    {
        let s = sender.clone();
        let flag = running.clone();
        let mut b = buff.clone();
        let target_inp = target_inp.clone();
        let ports_inp = ports_inp.clone();
        let to_inp = to_inp.clone();
        let display_ref = display_ref.clone();
        scan_custom_btn.set_callback(move |_| {
            if flag.load(Ordering::SeqCst) { return; }
            let target = target_inp.value();
            if target.trim().is_empty() {
                b.append("[Error] Target is empty.\n");
                return;
            }
            let ports_str = ports_inp.value();
            let ports = match parse_ports(&ports_str) {
                Ok(v) if !v.is_empty() => v,
                Ok(_) => {
                    b.append("[Error] No ports specified.\n");
                    return;
                }
                Err(e) => {
                    b.append(&format!("[Error] {}\n", e));
                    return;
                }
            };
            let timeout_ms: u64 = to_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(800) as u64;

            let ipv4 = resolve_target_ipv4(&target);
            let ip = match ipv4 {
                Some(ip) => ip,
                None => {
                    b.append("[Error] Failed to resolve target to IPv4.\n");
                    return;
                }
            };

            // Header
            b.set_text(&format!("{:<15} {:<7} {:<12} {}\n", "Target", "Result", "Status", "Info"));

            flag.store(true, Ordering::SeqCst);
            let flag_th = flag.clone();
            let sender = s.clone();
            let display_ref = display_ref.clone();
            thread::spawn(move || {
                for port in ports {
                    if !flag_th.load(Ordering::SeqCst) { break; }
                    let open = is_tcp_open(ip, port, timeout_ms);
                    sender.send(("PORTS".to_string(), ip, open, format!("{}/tcp", port)));
                }
                if let Ok(mut display) = display_ref.lock() { display.redraw(); }
                flag_th.store(false, Ordering::SeqCst);
            });
        });
    }

    (running, buff, display_ref)
}

fn resolve_target_ipv4(target: &str) -> Option<Ipv4Addr> {
    if let Ok(ip) = target.parse::<Ipv4Addr>() { return Some(ip); }
    if let Ok(addrs) = lookup_host(target) {
        for a in addrs {
            if let std::net::IpAddr::V4(v4) = a { return Some(v4); }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_ipv4_local() {
        assert!(resolve_target_ipv4("127.0.0.1").is_some());
    }
}
