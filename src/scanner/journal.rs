use std::process::Command;
use super::{format_bytes, ScanResult};

pub fn scan() -> ScanResult {
    let mut total_bytes = 0u64;
    let mut details = "No logs found".to_string();

    if let Ok(output) = Command::new("journalctl").arg("--disk-usage").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            // Example: "Archived and active journals take up 1.9G in the file system."
            if let Some(pos) = text.find("take up ") {
                let rest = &text[pos + 8..];
                if let Some(end) = rest.find(" in the") {
                    let size_str = rest[..end].trim();
                    total_bytes = parse_journal_size(size_str);
                    details = format!("Current: {} (vacuum to 100M)", format_bytes(total_bytes));
                }
            }
        }
    }

    let vacuum_limit = 100 * 1024 * 1024; // 100 MiB
    let bytes_to_free = if total_bytes > vacuum_limit {
        total_bytes - vacuum_limit
    } else {
        0
    };

    ScanResult {
        id: "journal".to_string(),
        title: "Systemd Journal Logs".to_string(),
        details,
        bytes: bytes_to_free,
        count: if bytes_to_free > 0 { 1 } else { 0 },
        needs_root: true,
        default_selected: bytes_to_free > 0,
    }
}

fn parse_journal_size(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    let last = s.chars().last().unwrap_or('B');
    let num_part = &s[..s.len().saturating_sub(1)].trim();
    let num: f64 = num_part.parse().unwrap_or(0.0);

    match last {
        'T' | 't' => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
        'G' | 'g' => (num * 1024.0 * 1024.0 * 1024.0) as u64,
        'M' | 'm' => (num * 1024.0 * 1024.0) as u64,
        'K' | 'k' => (num * 1024.0) as u64,
        _ => s.parse::<u64>().unwrap_or(0),
    }
}
