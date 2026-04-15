# OysterClip

[![CI Status](https://github.com/rickycodes/oysterclip/actions/workflows/ci.yml/badge.svg)](https://github.com/rickycodes/oysterclip/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE)
[![Rust 1.70+](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Platform: Linux | macOS](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS-green.svg)](#requirements)

![OysterClip](OysterClip.png)

> This is my clipboard manager. There are many like it, but this one is mine (and now yours).

**OysterClip** consists of a background watcher daemon that captures clipboard entries to an encrypted database, paired with a powerful desktop UI and terminal-based viewer for searching and managing your clipboard history.

## Quick Start

**Requirements:** Rust 1.70+, GTK 3.24+ (Linux), macOS 10.13+

```bash
# Clone and build
git clone https://github.com/rickycodes/oysterclip.git
cd oysterclip
cargo build --release --workspace

# Run the watcher daemon
./target/release/oysterclip-watcher &

# Launch the viewer
./target/release/oysterclip-viewer
```

## Table of Contents

- [Features](#features)
- [Components](#components)
  - [Watcher](#watcher-packagesoysterclip-watcher)
  - [Viewer](#viewer-packagesoysterclip-viewer)
  - [Terminal UI](#terminal-ui-packagesoysterclip-tui)
- [Storage](#storage)
- [Architecture](#architecture)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

✅ **Secure** — XChaCha20Poly1305 encryption at rest with OS keychain integration  
✅ **Lightweight** — Minimal resource usage with efficient polling  
✅ **Searchable** — Structured query syntax (type:image, kind:url, since:1h)  
✅ **Multi-Interface** — Desktop UI, terminal interface, and headless daemon  
✅ **Cross-Platform** — Linux and macOS support  
✅ **Theme Support** — Auto-detecting dark/light themes  

## Components

### Watcher (`packages/oysterclip-watcher`)

A daemon that monitors your system clipboard and persists entries to SQLite with encryption.

**Features:**
- Clipboard polling on 500ms interval
- XChaCha20Poly1305 encryption at rest
- Image storage as PNG blobs
- Automatic text deduplication
- Unix socket control API (pause/resume/status)
- OS keychain integration for secure key storage

```bash
cargo run -p oysterclip-watcher
cargo run -p oysterclip-watcher -- control status
```

See [`packages/oysterclip-watcher/README.md`](packages/oysterclip-watcher/README.md) for details.

### Viewer (`packages/oysterclip-viewer`)

A Dioxus Desktop app for browsing, searching, and managing clipboard history.

**Features:**
- Keyboard-driven navigation (↑↓ hjkl, Home/End)
- Structured search: `type:image`, `kind:url`, `since:1h`
- Copy/delete/clear history
- Dark/light theme
- Password masking with auth caching
- URL previews and clickable links
- Multi-select bulk operations

```bash
cargo run -p oysterclip-viewer
cargo run -p oysterclip-viewer -- --theme light
```

See [`packages/oysterclip-viewer/README.md`](packages/oysterclip-viewer/README.md) for details.

### Terminal UI (`packages/oysterclip-tui`)

A simple terminal interface for clipboard history on headless servers and SSH sessions.

**Features:**
- Browse clipboard history (last 100 entries)
- View full content of selected entries
- Arrow key navigation
- Integrated with the same database as viewer/watcher
- Independent operation (doesn't require watcher daemon)

```bash
cargo run -p oysterclip-tui
```

See [`packages/oysterclip-tui/README.md`](packages/oysterclip-tui/README.md) for details.

## Storage

All files live in the canonical per-user app-data directory (`~/.config/oysterclip` on Linux):

| File | Purpose |
|------|---------|
| `.oysterclip.db` | SQLite history (encrypted text, image blobs) |
| `.oysterclip.toml` | Watcher config (retention, image export) |
| `.oysterclip.sock` | Unix socket for control commands |
| `clipboard_images/` | Optional PNG image export directory |

## Architecture

```
System Clipboard
       ↓
Watcher (Daemon) ← 500ms polling
       ↓
SQLite DB (encrypted)
  • Encrypted text
  • Image PNG blobs
  • Deduplication
       ↓
Viewer (UI) ← Browse, search, copy
```

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for planned features and architecture improvements.

## Development

### Setup

**Prerequisites:**
- Rust 1.70+ ([install](https://rustup.rs/))
- GTK 3.24+ development files (Linux): `sudo apt-get install libgtk-3-dev libglib2.0-dev`
- macOS: Xcode Command Line Tools

**Build all packages:**
```bash
cargo build --workspace
```

**Run tests:**
```bash
cargo test --workspace
```

**Run with release optimizations:**
```bash
cargo build --release --workspace
```

**Hot reload (development):**
```bash
cargo install dioxus-cli --version 0.7.3 --locked
dx serve --platform desktop
```

### Project Structure

```
oysterclip/
├── packages/
│   ├── common/               # Shared encryption, paths, IPC
│   ├── oysterclip-watcher/   # Daemon that monitors clipboard
│   ├── oysterclip-viewer/    # Desktop UI (Dioxus)
│   └── oysterclip-tui/       # Terminal interface
├── Cargo.toml               # Workspace configuration
├── ROADMAP.md               # Feature roadmap
└── README.md                # This file
```

## Contributing

Contributions are welcome! Whether it's bug fixes, features, or documentation improvements.

**Getting Started:**
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes and add tests if applicable
4. Ensure tests pass: `cargo test --workspace`
5. Ensure no clippy warnings: `cargo clippy --workspace -- -D warnings`
6. Commit with clear messages
7. Push to your fork and open a pull request

**Development Workflow:**
- All PRs require passing CI (check, clippy, tests)
- Commits should be atomic and well-documented
- No external dependencies without justification

See [`ROADMAP.md`](ROADMAP.md) for high-level roadmap and areas that need work.

## License

This project is licensed under the GNU General Public License v3.0 — see [`LICENSE`](LICENSE) for details.
