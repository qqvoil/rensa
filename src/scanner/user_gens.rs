use std::fs;
use std::path::PathBuf;
use super::ScanResult;

pub fn scan() -> ScanResult {
    let mut to_delete = 0;
    let mut estimated_bytes = 0;

    let mut profile_dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        profile_dirs.push(PathBuf::from(&home).join(".local/state/nix/profiles"));
    }
    if let Ok(user) = std::env::var("USER") {
        profile_dirs.push(PathBuf::from("/nix/var/nix/profiles/per-user").join(user));
    }

    for dir in profile_dirs {
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut gens: Vec<u32> = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("profile-") && name_str.ends_with("-link") {
                    let parts: Vec<&str> = name_str.split('-').collect();
                    if parts.len() >= 3 {
                        if let Ok(num) = parts[1].parse::<u32>() {
                            gens.push(num);
                        }
                    }
                }
            }
            if gens.len() > 1 {
                let count = gens.len() - 1;
                to_delete += count;
                estimated_bytes += (count as u64) * 200 * 1024 * 1024;
            }
        }
    }

    let details = if to_delete > 0 {
        format!("{} old profile generation(s)", to_delete)
    } else {
        "Clean (only active profile)".to_string()
    };

    ScanResult {
        id: "user_generations".to_string(),
        title: "User Profile Generations".to_string(),
        details,
        bytes: estimated_bytes,
        count: to_delete,
        needs_root: false,
        default_selected: to_delete > 0,
    }
}
