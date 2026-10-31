use std::process::Command;
use super::ScanResult;

pub fn scan_gc() -> ScanResult {
    // Run nix-store --gc --print-dead in background
    let mut dead_count = 0usize;
    let mut dead_bytes = 0u64;

    if let Ok(output) = Command::new("nix-store")
        .arg("--gc")
        .arg("--print-dead")
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("/nix/store/") {
                    dead_count += 1;
                    // Read file or symlink metadata if possible
                    if let Ok(meta) = std::fs::symlink_metadata(trimmed) {
                        dead_bytes += meta.len();
                    }
                }
            }
        }
    }

    // Since many dead paths are directories or trees, approximate 3 MiB per dead path
    let estimated_bytes = if dead_bytes > 0 {
        dead_bytes.max((dead_count as u64) * 3 * 1024 * 1024)
    } else {
        (dead_count as u64) * 3 * 1024 * 1024
    };

    let details = if dead_count > 0 {
        format!("{} unreferenced store path(s)", dead_count)
    } else {
        "No unreferenced paths in store".to_string()
    };

    ScanResult {
        id: "store_gc".to_string(),
        title: "Unreferenced Nix Store Paths".to_string(),
        details,
        bytes: estimated_bytes,
        count: dead_count,
        needs_root: false,
        default_selected: dead_count > 0,
    }
}

#[allow(dead_code)]
pub fn scan_optimise() -> ScanResult {
    ScanResult {
        id: "store_optimise".to_string(),
        title: "Nix Store Deduplication".to_string(),
        details: "Hardlink identical store objects (nix-store --optimise)".to_string(),
        bytes: 0,
        count: 0,
        needs_root: true,
        default_selected: false, // Optional, intensive I/O
    }
}
