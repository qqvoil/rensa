use walkdir::WalkDir;
use super::{get_target_homes, ScanResult};

pub fn scan() -> ScanResult {
    let mut total_bytes = 0u64;
    let mut targets = Vec::new();

    for home in get_target_homes() {
        targets.push(home.join(".cache/thumbnails"));
        targets.push(home.join(".cache/nix"));
    }

    for dir in &targets {
        if dir.exists() {
            for entry in WalkDir::new(dir).into_iter().flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total_bytes += meta.len();
                    }
                }
            }
        }
    }

    let details = "Nix eval cache & thumbnails (~/.cache/nix, thumbnails)".to_string();

    ScanResult {
        id: "cache".to_string(),
        title: "User Caches".to_string(),
        details,
        bytes: total_bytes,
        count: if total_bytes > 0 { 1 } else { 0 },
        needs_root: false,
        default_selected: total_bytes > 0,
    }
}
