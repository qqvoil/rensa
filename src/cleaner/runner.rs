use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use super::CleanEvent;

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn run_clean(selected_ids: Vec<String>, tx: Sender<CleanEvent>) {
    std::thread::spawn(move || {
        let total = selected_ids.len();
        let mut success_count = 0;

        for id in &selected_ids {
            std::thread::sleep(std::time::Duration::from_millis(350));
            match id.as_str() {
                "system_generations" => {
                    let title = "Pruning NixOS System Generations".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    let ok = run_cmd(
                        &tx,
                        true,
                        "nix-env",
                        &["-p", "/nix/var/nix/profiles/system", "--delete-generations", "+3"],
                    );
                    if ok {
                        success_count += 1;
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: true,
                            message: "Kept last 3 generations".to_string(),
                        });
                    } else {
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: false,
                            message: "Failed to prune system generations".to_string(),
                        });
                    }
                }
                "user_generations" => {
                    let title = "Cleaning User Profile Generations".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    let ok = run_cmd(&tx, false, "nix-env", &["--delete-generations", "old"]);
                    if ok {
                        success_count += 1;
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: true,
                            message: "Deleted old user generations".to_string(),
                        });
                    } else {
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: false,
                            message: "Failed to delete user profile generations".to_string(),
                        });
                    }
                }
                "gcroots" => {
                    let title = "Removing Dead GC Roots & Result Links".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    clean_gcroots(&tx);
                    success_count += 1;
                    let _ = tx.send(CleanEvent::StepDone {
                        id: id.clone(),
                        success: true,
                        message: "Cleaned broken roots and result links".to_string(),
                    });
                }
                "store_gc" => {
                    let title = "Collecting Unreferenced Store Garbage".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    let ok = run_cmd(&tx, is_root(), "nix-collect-garbage", &[]);
                    if ok {
                        success_count += 1;
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: true,
                            message: "Store garbage collected".to_string(),
                        });
                    } else {
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: false,
                            message: "nix-collect-garbage failed".to_string(),
                        });
                    }
                }
                "journal" => {
                    let title = "Vacuuming Systemd Journal Logs".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    let ok = run_cmd(&tx, true, "journalctl", &["--vacuum-size=100M"]);
                    if ok {
                        success_count += 1;
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: true,
                            message: "Vacuumed journal logs to 100M".to_string(),
                        });
                    } else {
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: false,
                            message: "Failed to vacuum journal".to_string(),
                        });
                    }
                }
                "cache" => {
                    let title = "Purging User Caches".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    clean_cache(&tx);
                    success_count += 1;
                    let _ = tx.send(CleanEvent::StepDone {
                        id: id.clone(),
                        success: true,
                        message: "User cache cleaned".to_string(),
                    });
                }
                "trash" => {
                    let title = "Emptying User Trash".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    clean_trash(&tx);
                    success_count += 1;
                    let _ = tx.send(CleanEvent::StepDone {
                        id: id.clone(),
                        success: true,
                        message: "Trash emptied".to_string(),
                    });
                }
                "store_optimise" => {
                    let title = "Deduplicating Nix Store (Optimise)".to_string();
                    let _ = tx.send(CleanEvent::StepStart { id: id.clone(), title });
                    let ok = run_cmd(&tx, true, "nix-store", &["--optimise"]);
                    if ok {
                        success_count += 1;
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: true,
                            message: "Store deduplicated".to_string(),
                        });
                    } else {
                        let _ = tx.send(CleanEvent::StepDone {
                            id: id.clone(),
                            success: false,
                            message: "nix-store --optimise failed".to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        let _ = tx.send(CleanEvent::Finished {
            total_steps: total,
            successful_steps: success_count,
        });
    });
}

fn run_cmd(tx: &Sender<CleanEvent>, needs_root: bool, prog: &str, args: &[&str]) -> bool {
    let (cmd_bin, cmd_args): (String, Vec<String>) = if needs_root && !is_root() {
        let mut a = vec![prog.to_string()];
        a.extend(args.iter().map(|s| s.to_string()));
        ("sudo".to_string(), a)
    } else {
        (prog.to_string(), args.iter().map(|s| s.to_string()).collect())
    };

    let _ = tx.send(CleanEvent::LogLine(format!("$ {} {}", cmd_bin, cmd_args.join(" "))));

    let mut child = match Command::new(&cmd_bin)
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(CleanEvent::LogLine(format!("Error spawning {}: {}", cmd_bin, e)));
            return false;
        }
    };

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let _ = tx.send(CleanEvent::LogLine(line));
        }
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let _ = tx.send(CleanEvent::LogLine(line));
        }
    }

    match child.wait() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn clean_gcroots(tx: &Sender<CleanEvent>) {
    // 1. Delete dead auto gcroots
    let auto_gcroots = Path::new("/nix/var/nix/gcroots/auto");
    if let Ok(entries) = fs::read_dir(auto_gcroots) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(target) = fs::read_link(&p) {
                if !target.exists() {
                    let _ = tx.send(CleanEvent::LogLine(format!("Removing dead root: {:?}", p)));
                    let _ = fs::remove_file(&p);
                }
            }
        }
    }

    // 2. Delete result symlinks in user's home
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        if let Ok(entries) = fs::read_dir(&home_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("result") && p.is_symlink() {
                    let _ = tx.send(CleanEvent::LogLine(format!("Removing result symlink: {:?}", p)));
                    let _ = fs::remove_file(&p);
                }
            }
        }
    }
}

fn clean_cache(tx: &Sender<CleanEvent>) {
    if let Ok(home) = std::env::var("HOME") {
        let home_p = PathBuf::from(home);
        for sub in &[".cache/thumbnails", ".cache/nix"] {
            let p = home_p.join(sub);
            if p.exists() {
                let _ = tx.send(CleanEvent::LogLine(format!("Purging cache dir: {:?}", p)));
                let _ = fs::remove_dir_all(&p);
                let _ = fs::create_dir_all(&p);
            }
        }
    }
}

fn clean_trash(tx: &Sender<CleanEvent>) {
    if let Ok(home) = std::env::var("HOME") {
        let trash_dir = PathBuf::from(home).join(".local/share/Trash");
        for sub in &["files", "info"] {
            let p = trash_dir.join(sub);
            if p.exists() {
                if let Ok(entries) = fs::read_dir(&p) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let _ = tx.send(CleanEvent::LogLine(format!("Purging trash item: {:?}", path.file_name().unwrap_or_default())));
                        let _ = if path.is_dir() {
                            fs::remove_dir_all(&path)
                        } else {
                            fs::remove_file(&path)
                        };
                    }
                }
            }
        }
    }
}
