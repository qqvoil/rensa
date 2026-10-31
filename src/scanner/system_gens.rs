use std::fs;
use std::path::Path;
use super::ScanResult;

pub fn scan(keep_generations: usize) -> ScanResult {
    let profiles_dir = Path::new("/nix/var/nix/profiles");
    let mut gens: Vec<u32> = Vec::new();

    if let Ok(entries) = fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("system-") && name_str.ends_with("-link") {
                let parts: Vec<&str> = name_str.split('-').collect();
                if parts.len() >= 3 {
                    if let Ok(num) = parts[1].parse::<u32>() {
                        gens.push(num);
                    }
                }
            }
        }
    }

    gens.sort_unstable();
    let total = gens.len();
    let to_delete = if total > keep_generations {
        total - keep_generations
    } else {
        0
    };

    // Rough conservative estimate of freed space per old generation (approx 800 MiB unique diff on average)
    let estimated_bytes = (to_delete as u64) * 850 * 1024 * 1024;

    let details = if to_delete > 0 {
        format!("{} to purge (keeping last {})", to_delete, keep_generations)
    } else {
        format!("All {} generations kept", total)
    };

    ScanResult {
        id: "system_generations".to_string(),
        title: "NixOS System Generations".to_string(),
        details,
        bytes: estimated_bytes,
        count: to_delete,
        needs_root: true,
        default_selected: to_delete > 0,
    }
}
