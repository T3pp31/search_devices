use fltk::{
    prelude::*,
    frame::Frame,
    input::{Input, IntInput},
    button::{Button, CheckButton},
    text::{TextDisplay, TextBuffer},
    app,
};
use std::{
    net::Ipv4Addr,
    process::{Command, Stdio, Child},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    io::{BufRead, BufReader},
    thread,
};
use crate::utils::{ms_to_secs_ceil, tracert_args_windows, traceroute_args_unix};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn build_tracert_tab(sender: app::Sender<(String, Ipv4Addr, bool, String)>) -> (Arc<AtomicBool>, TextBuffer, Arc<Mutex<TextDisplay>>) {
    Frame::new(10, 30, 200, 25, "Target (host or IPv4)");
    let mut input = Input::new(10, 70, 200, 30, "");
    input.set_value("8.8.8.8");

    let mut clear_btn = Button::new(240, 70, 80, 30, "Clear");
    let mut trace_btn = Button::new(320, 70, 80, 30, "Trace");
    let mut stop_btn = Button::new(410, 70, 80, 30, "Stop");

    // Options row
    let _max_label = Frame::new(10, 110, 70, 25, "Max Hops");
    let mut max_inp = IntInput::new(80, 110, 50, 25, "");
    max_inp.set_value("30");
    let _to_label = Frame::new(140, 110, 100, 25, "Timeout (ms)");
    let mut to_inp = IntInput::new(240, 110, 70, 25, "");
    to_inp.set_value("1000");
    let mut resolve_cb = CheckButton::new(320, 110, 120, 25, "Resolve DNS");
    resolve_cb.set_value(true);

    // Output area
    let mut display = TextDisplay::new(10, 140, 480, 230, "");
    let buff = TextBuffer::default();
    display.set_buffer(buff.clone());
    let display_ref = Arc::new(Mutex::new(display));

    let running = Arc::new(AtomicBool::new(false));
    let child_handle: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    // Clear
    {
        let mut b = buff.clone();
        clear_btn.set_callback(move |_| {
            b.set_text("");
        });
    }

    // Trace
    {
        let s = sender.clone();
        let flag = running.clone();
        let child_ref = child_handle.clone();
        let inp = input.clone();
        let max_inp = max_inp.clone();
        let to_inp = to_inp.clone();
        let res_cb = resolve_cb.clone();
        let mut b = buff.clone();
        trace_btn.set_callback(move |_| {
            if flag.load(Ordering::SeqCst) {
                return; // already running
            }
            let target = inp.value();
            if target.trim().is_empty() {
                b.append("[Error] Target is empty.\n");
                return;
            }

            // Read settings
            let max_hops: u32 = max_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(30);
            let timeout_ms: u32 = to_inp.value().parse().ok().filter(|v| *v >= 1).unwrap_or(1000);
            let resolve_dns = res_cb.value();

            // Header
            b.set_text(&format!(
                "Tracing route to {} (max {} hops, timeout {}ms, DNS {})\n",
                target,
                max_hops,
                timeout_ms,
                if resolve_dns {"on"} else {"off"}
            ));

            flag.store(true, Ordering::SeqCst);
            let flag_thread = flag.clone();
            let sender = s.clone();
            let child_ref = child_ref.clone();
            let target_clone = target.clone();

            thread::spawn(move || {
                // Build command per platform
                #[cfg(windows)]
                let mut cmd = {
                    let mut c = Command::new("tracert");
                    c.creation_flags(CREATE_NO_WINDOW);
                    let args = tracert_args_windows(max_hops, timeout_ms, resolve_dns, &target_clone);
                    c.args(&args);
                    c
                };

                #[cfg(not(windows))]
                let mut cmd = {
                    let mut c = Command::new("traceroute");
                    let args = traceroute_args_unix(max_hops, timeout_ms, resolve_dns, &target_clone);
                    c.args(&args);
                    c
                };

                // Spawn and stream output
                let child_res = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn();
                match child_res {
                    Ok(mut child) => {
                        let stdout = child.stdout.take();
                        let stderr = child.stderr.take();
                        {
                            let mut slot = child_ref.lock().unwrap();
                            *slot = Some(child);
                        }
                        if let Some(out) = stdout {
                            let reader = BufReader::new(out);
                            for line in reader.lines() {
                                if !flag_thread.load(Ordering::SeqCst) {
                                    // request stop: kill child via handle
                                    if let Ok(mut slot) = child_ref.lock() {
                                        if let Some(ref mut ch) = *slot {
                                            let _ = ch.kill();
                                        }
                                    }
                                    break;
                                }
                                let line = line.unwrap_or_default();
                                sender.send(("TRACERT".to_string(), Ipv4Addr::UNSPECIFIED, false, line));
                            }
                        }
                        if let Some(err) = stderr {
                            let reader = BufReader::new(err);
                            for line in reader.lines() {
                                if !flag_thread.load(Ordering::SeqCst) { break; }
                                let line = line.unwrap_or_default();
                                sender.send(("TRACERT".to_string(), Ipv4Addr::UNSPECIFIED, false, line));
                            }
                        }
                        // Ensure process is not lingering
                        if let Ok(mut slot) = child_ref.lock() {
                            if let Some(mut ch) = slot.take() {
                                let _ = ch.wait();
                            }
                        }
                    }
                    Err(e) => {
                        sender.send(("TRACERT".to_string(), Ipv4Addr::UNSPECIFIED, false, format!("[Error] Failed to start traceroute: {}", e)));
                    }
                }

                flag_thread.store(false, Ordering::SeqCst);
            });
        });
    }

    // Stop
    {
        let flag = running.clone();
        let child_ref = child_handle.clone();
        stop_btn.set_callback(move |_| {
            flag.store(false, Ordering::SeqCst);
            if let Ok(mut slot) = child_ref.lock() {
                if let Some(mut ch) = slot.take() {
                    let _ = ch.kill();
                }
            }
        });
    }

    (running, buff, display_ref)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_traceroute_args_for_tracert_tab() {
        let w = tracert_args_windows(30, 1000, true, "8.8.8.8");
        assert_eq!(w, vec!["-h","30","-w","1000","8.8.8.8"]);
        let w2 = tracert_args_windows(20, 500, false, "example.com");
        assert_eq!(w2, vec!["-d","-h","20","-w","500","example.com"]);

        let u = traceroute_args_unix(16, 1, false, "example.com");
        assert_eq!(u, vec!["-n","-m","16","-w","1","example.com"]);
        let u2 = traceroute_args_unix(32, 1500, true, "8.8.8.8");
        assert_eq!(u2, vec!["-m","32","-w","2","8.8.8.8"]);
    }
}
