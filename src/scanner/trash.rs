use walkdir::WalkDir;
use super::{get_target_homes, ScanResult};

pub fn scan() -> ScanResult {
    let mut total_bytes = 0u64;
    let mut file_count = 0usize;

    for home in get_target_homes() {
        let trash_dir = home.join(".local/share/Trash");
        if trash_dir.exists() {
            for entry in WalkDir::new(&trash_dir).into_iter().flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total_bytes += meta.len();
                        file_count += 1;
                    }
                }
            }
        }
    }

    let details = if file_count > 0 {
        format!("{} item(s) across user trash", file_count)
    } else {
        "Empty".to_string()
    };

    ScanResult {
        id: "trash".to_string(),
        title: "User Trash".to_string(),
        details,
        bytes: total_bytes,
        count: file_count,
        needs_root: false,
        default_selected: total_bytes > 0,
    }
}
