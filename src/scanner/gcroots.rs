use std::fs;
use std::path::Path;
use super::{get_target_homes, ScanResult};

pub fn scan() -> ScanResult {
    let mut dead_roots = 0;
    let mut result_links = 0;

    // 1. Check auto gcroots for broken symlinks
    let auto_gcroots = Path::new("/nix/var/nix/gcroots/auto");
    if let Ok(entries) = fs::read_dir(auto_gcroots) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(target) = fs::read_link(&path) {
                // If target doesn't exist, it's a dead root
                if !target.exists() {
                    dead_roots += 1;
                }
            }
        }
    }

    // 2. Check for result symlinks across target user home directories
    for home in get_target_homes() {
        if let Ok(entries) = fs::read_dir(&home) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("result") && path.is_symlink() {
                    result_links += 1;
                }
            }
        }
    }

    let total = dead_roots + result_links;
    let details = if total > 0 {
        format!("{} dead root(s), {} result link(s)", dead_roots, result_links)
    } else {
        "No dangling roots found".to_string()
    };

    // Rough estimate: result symlinks and dead roots often pin closures of 500 MiB - 2 GiB
    let estimated_bytes = (total as u64) * 500 * 1024 * 1024;

    ScanResult {
        id: "gcroots".to_string(),
        title: "Dead GC Roots & Dangling Results".to_string(),
        details,
        bytes: estimated_bytes,
        count: total,
        needs_root: dead_roots > 0,
        default_selected: total > 0,
    }
}
