use std::fs;
use std::path::PathBuf;
use super::{get_target_homes, ScanResult};

pub fn get_user_profile_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for home in get_target_homes() {
        let p = home.join(".local/state/nix/profiles");
        if p.exists() && !dirs.contains(&p) {
            dirs.push(p);
        }
    }
    if let Ok(entries) = fs::read_dir("/nix/var/nix/profiles/per-user") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }
    dirs
}

pub fn get_user_profile_links() -> Vec<PathBuf> {
    let mut links = Vec::new();
    for dir in get_user_profile_dirs() {
        let link = dir.join("profile");
        if link.exists() || link.is_symlink() {
            if !links.contains(&link) {
                links.push(link);
            }
        }
    }
    links
}

pub fn scan() -> ScanResult {
    let mut to_delete = 0;
    let mut estimated_bytes = 0;

    for dir in get_user_profile_dirs() {
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
