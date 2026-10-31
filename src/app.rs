use std::sync::mpsc::{channel, Receiver, Sender};
use crate::cleaner::{runner::run_clean, CleanEvent};
use crate::scanner::{self, ScanResult};

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    Select,
    Executing,
    Done,
}

pub struct ItemState {
    pub id: String,
    pub title: String,
    pub details: String,
    pub bytes: u64,
    pub count: usize,
    pub needs_root: bool,
    pub selected: bool,
    pub is_scanning: bool,
}

pub struct App {
    pub mode: AppMode,
    pub items: Vec<ItemState>,
    pub selected_index: usize,
    pub spinner_frame: usize,
    pub scan_rx: Option<Receiver<ScanResult>>,
    pub clean_rx: Option<Receiver<CleanEvent>>,
    pub logs: Vec<String>,
    pub current_executing_id: Option<String>,
    pub completed_steps: Vec<(String, bool, String)>,
    pub total_freed_bytes: u64,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            mode: AppMode::Select,
            items: Vec::new(),
            selected_index: 0,
            spinner_frame: 0,
            scan_rx: None,
            clean_rx: None,
            logs: Vec::new(),
            current_executing_id: None,
            completed_steps: Vec::new(),
            total_freed_bytes: 0,
            should_quit: false,
        };
        app.start_scan();
        app
    }

    pub fn start_scan(&mut self) {
        let (tx, rx) = channel();
        self.scan_rx = Some(rx);

        // Define the items with placeholder scanning states
        self.items = vec![
            ItemState {
                id: "system_generations".to_string(),
                title: "NixOS System Generations".to_string(),
                details: "Scanning profiles...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: true,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "user_generations".to_string(),
                title: "User Profile Generations".to_string(),
                details: "Scanning user profiles...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: false,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "gcroots".to_string(),
                title: "Dead GC Roots & Result Links".to_string(),
                details: "Scanning dangling symlinks...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: false,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "store_gc".to_string(),
                title: "Unreferenced Nix Store Paths".to_string(),
                details: "Analyzing store graph...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: false,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "journal".to_string(),
                title: "Systemd Journal Logs".to_string(),
                details: "Reading journalctl usage...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: true,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "cache".to_string(),
                title: "User Application & Eval Cache".to_string(),
                details: "Measuring ~/.cache/nix & thumbnails...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: false,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "trash".to_string(),
                title: "User Trash".to_string(),
                details: "Counting files in ~/.local/share/Trash...".to_string(),
                bytes: 0,
                count: 0,
                needs_root: false,
                selected: true,
                is_scanning: true,
            },
            ItemState {
                id: "store_optimise".to_string(),
                title: "Nix Store Deduplication".to_string(),
                details: "Hardlink identical store objects (nix-store --optimise)".to_string(),
                bytes: 0,
                count: 0,
                needs_root: true,
                selected: false, // Intensive disk I/O, opt-in
                is_scanning: false,
            },
        ];

        // Spawn parallel scanners
        let tx1 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::system_gens::scan(3);
            let _ = tx1.send(res);
        });

        let tx2 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::user_gens::scan();
            let _ = tx2.send(res);
        });

        let tx3 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::gcroots::scan();
            let _ = tx3.send(res);
        });

        let tx4 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::journal::scan();
            let _ = tx4.send(res);
        });

        let tx5 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::cache::scan();
            let _ = tx5.send(res);
        });

        let tx6 = tx.clone();
        std::thread::spawn(move || {
            let res = scanner::trash::scan();
            let _ = tx6.send(res);
        });

        let tx7 = tx;
        std::thread::spawn(move || {
            let res = scanner::store::scan_gc();
            let _ = tx7.send(res);
        });
    }

    pub fn tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;

        // Drain scanning messages
        let mut updates = Vec::new();
        if let Some(ref rx) = self.scan_rx {
            while let Ok(res) = rx.try_recv() {
                updates.push(res);
            }
        }

        for update in updates {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == update.id) {
                item.bytes = update.bytes;
                item.count = update.count;
                item.details = update.details;
                item.needs_root = update.needs_root;
                item.selected = update.default_selected;
                item.is_scanning = false;
            }
        }

        // Drain cleaning execution messages
        let mut clean_events = Vec::new();
        if let Some(ref rx) = self.clean_rx {
            while let Ok(ev) = rx.try_recv() {
                clean_events.push(ev);
            }
        }

        for ev in clean_events {
            match ev {
                CleanEvent::StepStart { id, title } => {
                    self.current_executing_id = Some(id);
                    self.logs.push(format!("▶ {}", title));
                }
                CleanEvent::LogLine(line) => {
                    self.logs.push(line);
                }
                CleanEvent::StepDone { id, success, message } => {
                    self.completed_steps.push((id, success, message));
                }
                CleanEvent::Finished { .. } => {
                    self.mode = AppMode::Done;
                    self.current_executing_id = None;
                }
            }
        }
    }

    pub fn total_selected_bytes(&self) -> u64 {
        self.items
            .iter()
            .filter(|i| i.selected && !i.is_scanning)
            .map(|i| i.bytes)
            .sum()
    }

    pub fn toggle_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_index) {
            item.selected = !item.selected;
        }
    }

    pub fn toggle_all(&mut self) {
        let any_unselected = self.items.iter().any(|i| !i.selected);
        for item in &mut self.items {
            item.selected = any_unselected;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.items.is_empty() {
            if self.selected_index + 1 < self.items.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn start_clean(&mut self) {
        let selected_ids: Vec<String> = self
            .items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.id.clone())
            .collect();

        if selected_ids.is_empty() {
            return;
        }

        self.total_freed_bytes = self.total_selected_bytes();
        self.mode = AppMode::Executing;
        self.logs.clear();
        self.completed_steps.clear();

        let (tx, rx): (Sender<CleanEvent>, Receiver<CleanEvent>) = channel();
        self.clean_rx = Some(rx);
        run_clean(selected_ids, tx);
    }
}
