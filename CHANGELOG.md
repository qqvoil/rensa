# Changelog

All notable changes to **Rensa** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.1] - 2026-10-31

### Fixed
* **Pipe Buffer Deadlock**: Resolved child process freezing during extensive `nix-collect-garbage` runs when stdout/stderr exceeded the Linux 64 KB pipe buffer. `stderr` is now consumed concurrently in a background thread.
* **User Profile Permissions**: Excluded `/nix/var/nix/profiles/per-user/root` from user profile generations to eliminate `Permission denied` errors when running as an unprivileged user.
* **Accurate Profile Targeting**: Targeted `~/.local/state/nix/profiles/profile` explicitly for user generation deletion (`nix-env -p`).

---

## [0.2.0] - 2026-10-31

### Added
* **Swedish Status Badges**: Unified lifecycle status indicators:
  * ` Rensa skräp [Enter] ` (main menu action)
  * ` Rensar... ` (live execution in muted amber)
  * ` Rensning klar ` (completion in forest green)
* **Smart Sudo User Resolution**: Added `get_target_homes()` to discover invoking user directories (`$SUDO_USER`, `$SUDO_HOME`), correctly scanning and purging user Trash and caches even under `sudo`.
* **Itemized Live Output**: `clean_trash` now logs individual file paths during deletion for live streaming output.
* **UI Pacing**: Introduced a smooth 350 ms step pacing so spinners and status checks are clearly visible.

### Changed
* **Header Harmonization**: Unified top header across both views to display ` Rensa ` and version ` v0.2.1 ` without visual jumping.
* **Centered Keybinds**: Relocated navigation and action keybindings outside borders, centered at the bottom without redundant key indicators.
* **Table Column Synchronization**: Aligned the Execution Plan table columns (`STATUS: 8`, `TARGET: 32`, `REMAINDER: min 30`) with the Targets table for seamless layout transitions.

---

## [0.1.0] - 2026-10-31

### Added
* Initial release of **Rensa**.
* Multi-threaded asynchronous scanning engine for NixOS.
* Scanners for System Generations, User Profiles, Dead GC Roots & Result Links, Unreferenced Store Paths, Systemd Journal, Caches, and Trash.
* Real-time streaming command runner.
* Flake packaging with stripped minimal static binary (~1.4 MB).
