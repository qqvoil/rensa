pub mod cache;
pub mod gcroots;
pub mod journal;
pub mod store;
pub mod system_gens;
pub mod trash;
pub mod user_gens;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub id: String,
    pub title: String,
    pub details: String,
    pub bytes: u64,
    pub count: usize,
    pub needs_root: bool,
    pub default_selected: bool,
}

pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;

    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn get_target_homes() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut homes = Vec::new();

    // 1. If invoked via sudo, detect the invoking user's home
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if sudo_user != "root" {
            if let Ok(sudo_home) = std::env::var("SUDO_HOME") {
                let p = PathBuf::from(sudo_home);
                if p.exists() && !homes.contains(&p) {
                    homes.push(p);
                }
            } else {
                let p = PathBuf::from(format!("/home/{}", sudo_user));
                if p.exists() && !homes.contains(&p) {
                    homes.push(p);
                }
            }
        }
    }

    // 2. Add current process $HOME
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home);
        if p.exists() && !homes.contains(&p) {
            homes.push(p);
        }
    }

    // 3. Fallback: discover any user directories in /home
    if let Ok(entries) = std::fs::read_dir("/home") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && !homes.contains(&p) {
                homes.push(p);
            }
        }
    }

    homes
}
