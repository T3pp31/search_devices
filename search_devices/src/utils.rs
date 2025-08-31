// Utility helpers shared across UI modules

/// Convert milliseconds to seconds, rounding up, with a minimum of 1 second.
pub fn ms_to_secs_ceil(ms: u32) -> u32 {
    let secs = (ms + 999) / 1000;
    secs.max(1)
}

/// Trim a line and return None if it is empty after trimming.
pub fn sanitize_line(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

/// Build OS-appropriate ping arguments for Windows.
pub fn ping_args_windows(count: u32, timeout_ms: u32, ip: &str) -> Vec<String> {
    vec![
        "-n".into(), count.to_string(),
        "-w".into(), timeout_ms.to_string(),
        ip.into(),
    ]
}

/// Build OS-appropriate ping arguments for Unix-like systems.
pub fn ping_args_unix(count: u32, timeout_ms: u32, ip: &str) -> Vec<String> {
    let secs = ms_to_secs_ceil(timeout_ms);
    vec![
        "-c".into(), count.to_string(),
        "-W".into(), secs.to_string(),
        ip.into(),
    ]
}

/// Build Windows tracert arguments.
pub fn tracert_args_windows(max_hops: u32, timeout_ms: u32, resolve_dns: bool, target: &str) -> Vec<String> {
    let mut args = Vec::new();
    if !resolve_dns { args.push("-d".into()); }
    args.push("-h".into()); args.push(max_hops.to_string());
    args.push("-w".into()); args.push(timeout_ms.to_string());
    args.push(target.into());
    args
}

/// Build Unix traceroute arguments.
pub fn traceroute_args_unix(max_hops: u32, timeout_ms: u32, resolve_dns: bool, target: &str) -> Vec<String> {
    let mut args = Vec::new();
    if !resolve_dns { args.push("-n".into()); }
    args.push("-m".into()); args.push(max_hops.to_string());
    let secs = ms_to_secs_ceil(timeout_ms);
    args.push("-w".into()); args.push(secs.to_string());
    args.push(target.into());
    args
}

/// Parse a port list string like "22,80,443" or with ranges "8000-8010".
/// Returns a deduplicated list of ports in input order; errors on invalid tokens.
pub fn parse_ports(s: &str) -> Result<Vec<u16>, String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for raw in s.split(',') {
        let token = raw.trim();
        if token.is_empty() { continue; }
        if let Some((a, b)) = token.split_once('-') {
            let start: u16 = a.trim().parse().map_err(|_| format!("Invalid port: {}", token))?;
            let end: u16 = b.trim().parse().map_err(|_| format!("Invalid port: {}", token))?;
            if start > end { return Err(format!("Invalid range: {}", token)); }
            for p in start..=end {
                if seen.insert(p) { out.push(p); }
            }
        } else {
            let p: u16 = token.parse().map_err(|_| format!("Invalid port: {}", token))?;
            if seen.insert(p) { out.push(p); }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ms_to_secs_ceil_basic() {
        assert_eq!(ms_to_secs_ceil(0), 1);
        assert_eq!(ms_to_secs_ceil(1), 1);
        assert_eq!(ms_to_secs_ceil(999), 1);
        assert_eq!(ms_to_secs_ceil(1000), 1);
        assert_eq!(ms_to_secs_ceil(1001), 2);
        assert_eq!(ms_to_secs_ceil(1999), 2);
        assert_eq!(ms_to_secs_ceil(2000), 2);
    }

    #[test]
    fn test_sanitize_line() {
        assert_eq!(sanitize_line("").is_none(), true);
        assert_eq!(sanitize_line("   ").is_none(), true);
        assert_eq!(sanitize_line("  x  "), Some("x".to_string()));
        assert_eq!(sanitize_line("\tline\n"), Some("line".to_string()));
    }

    #[test]
    fn test_sanitize_multiline_filtering() {
        let sample = "\n\n  Tracing route to 8.8.8.8 (max 30 hops)  \n\n  1   3 ms   2 ms   2 ms  gw [192.168.0.1]  \n\n  2   *      *      *     \n\n";
        let lines: Vec<String> = sample
            .lines()
            .filter_map(sanitize_line)
            .collect();
        assert!(lines.len() >= 2, "expected at least 2 non-empty lines");
        assert!(lines[0].starts_with("Tracing route to"));
        assert!(lines[1].starts_with("1"));
    }

    #[test]
    fn test_ping_args_builders() {
        let w = ping_args_windows(3, 1500, "1.2.3.4");
        assert_eq!(w, vec!["-n","3","-w","1500","1.2.3.4"]);
        let u = ping_args_unix(2, 1, "10.0.0.1");
        assert_eq!(u, vec!["-c","2","-W","1","10.0.0.1"]);
        let u2 = ping_args_unix(2, 1999, "10.0.0.1");
        assert_eq!(u2, vec!["-c","2","-W","2","10.0.0.1"]);
    }

    #[test]
    fn test_traceroute_args_builders() {
        let t_win = tracert_args_windows(20, 500, false, "example.com");
        assert_eq!(t_win, vec!["-d","-h","20","-w","500","example.com"]);
        let t_win2 = tracert_args_windows(30, 1000, true, "8.8.8.8");
        assert_eq!(t_win2, vec!["-h","30","-w","1000","8.8.8.8"]);

        let t_unix = traceroute_args_unix(16, 1, false, "example.com");
        assert_eq!(t_unix, vec!["-n","-m","16","-w","1","example.com"]);
        let t_unix2 = traceroute_args_unix(32, 1501, true, "8.8.8.8");
        assert_eq!(t_unix2, vec!["-m","32","-w","2","8.8.8.8"]);
    }

    #[test]
    fn test_parse_ports_simple_and_range() {
        let v = parse_ports("22, 80,443").unwrap();
        assert_eq!(v, vec![22, 80, 443]);
        let v2 = parse_ports("8000-8003").unwrap();
        assert_eq!(v2, vec![8000, 8001, 8002, 8003]);
    }

    #[test]
    fn test_parse_ports_mixed_and_dedup() {
        let v = parse_ports("22,80,80,79-81,22").unwrap();
        // input order, dedup
        assert_eq!(v, vec![22, 80, 79, 81]);
    }

    #[test]
    fn test_parse_ports_invalid() {
        assert!(parse_ports("abc").is_err());
        assert!(parse_ports("10-5").is_err());
        assert!(parse_ports("65536").is_err());
    }
}
