# rensa

> **rensa** *(Swedish)* — to cleanse, clear away, purge.

A fast, minimalist TUI garbage collection and system maintenance helper for **NixOS**, written in Rust.

```text
┌─ rensa v0.1.0 ────────────────────────────────────────── [ NixOS Garbage Collection Helper ] ─┐
│                                                                                               │
│   STATUS  TARGET                       DETAILS                            SCOPE     EST. SIZE │
│                                                                                               │
│ >  [ x ]  NixOS System Generations     17 to purge (keeping last 3)       [root] [  14.11 GiB ]│
│    [ x ]  User Profile Generations     2 old profile generation(s)        [user] [ 400.00 MiB ]│
│    [ x ]  Dead GC Roots & Result Links 1 dead root(s), 1 result link(s)   [root] [   1.00 GiB ]│
│    [ x ]  Unreferenced Nix Store Paths 142 unreferenced store path(s)     [user] [ 426.00 MiB ]│
│    [ x ]  Systemd Journal Logs         Current: 1.9 GiB (vacuum to 100M)  [root] [   1.80 GiB ]│
│    [ x ]  User Application & Cache     Nix eval cache & thumbnails        [user] [ 508.00 MiB ]│
│    [ x ]  User Trash                   23 item(s) in ~/.local/share/Trash [user] [ 227.00 MiB ]│
│    [   ]  Nix Store Deduplication      Hardlink identical store objects   [root] [          - ]│
│                                                                                               │
├───────────────────────────────────────────────────────────────────────────────────────────────┤
│  Total selected to purge: 18.47 GiB      Rensa skräp [Enter]                                  │
│  [Space] Toggle   [a] All   [j/k / ↑/↓] Navigate   [r] Rescan   [q] Quit                      │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Highlights

* **Instant & Asynchronous**: Launches in sub-milliseconds. Each subsystem is scanned concurrently in background threads while displaying animated brackets `[ ⠋ ]`.
* **Complete System Coverage**: Cleans beyond just `/nix/store`:
  * **System Generations**: Prunes old generations while safely preserving the last 3.
  * **User Profiles**: Purges stale user profile generations.
  * **Dangling GC Roots**: Detects broken symlinks in `/nix/var/nix/gcroots/auto` and lingering `result` symlinks in `$HOME`.
  * **Store Garbage**: Runs garbage collection on unreferenced derivations and paths.
  * **Systemd Journal**: Vacuums bloated `/var/log/journal/` files down to 100 MiB.
  * **Caches & Trash**: Purges `~/.cache/nix`, thumbnail caches, and `~/.local/share/Trash`.
  * **Store Deduplication**: Optional `nix-store --optimise` hardlinking.
* **Live Execution**: Switches to an execution screen showing real-time command output and step progress.
* **Industrial Minimalist UI**: Built with [Ratatui](https://github.com/ratatui/ratatui). Clean ASCII brackets, monospace layouts, zero emojis.

---

## Quick Start

### Run directly with Nix Flakes
```bash
nix run github:qqvoil/rensa
```
*(Or with `sudo` if you want to purge system-level profiles and vacuum journald without prompting):*
```bash
sudo nix run github:qqvoil/rensa
```

### Install in NixOS Configuration
Add `rensa` to your flake inputs:
```nix
inputs.rensa.url = "github:qqvoil/rensa";
```
And add it to `environment.systemPackages`:
```nix
environment.systemPackages = [
  inputs.rensa.packages.${pkgs.system}.default
];
```

### Build from source
```bash
git clone https://github.com/qqvoil/rensa.git
cd rensa
cargo build --release
./target/release/rensa
```

---

## Keybindings

| Key | Action |
| :--- | :--- |
| <kbd>Space</kbd> | Toggle selected item |
| <kbd>a</kbd> | Toggle all items |
| <kbd>j</kbd> / <kbd>↓</kbd> | Move down |
| <kbd>k</kbd> / <kbd>↑</kbd> | Move up |
| <kbd>r</kbd> | Rescan targets |
| <kbd>Enter</kbd> | **Rensa skräp** (Execute cleanup) |
| <kbd>q</kbd> / <kbd>Esc</kbd> | Quit / Back to menu |
| <kbd>Ctrl+C</kbd> | Interrupt and exit |

---

## License

MIT © [qqvoil](https://github.com/qqvoil)
