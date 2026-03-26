# Clipboard Viewer

A Dioxus Desktop app for browsing the shared clipboard history written by `clipboard-watcher`.

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Previous entry |
| `↓` / `j` | Next entry |
| `Home` | Jump to first entry |
| `End` | Jump to last entry |

### Actions
| Key | Action |
|-----|--------|
| `Enter` / `y` | Copy selected entry to clipboard |
| `Delete` / `Backspace` / `d` | Delete selected entry |
| `Escape` | Close image overlay or clear search |

### Search
- Focus the search input and type to filter entries by content, type, or path
- `Escape` while in search clears the search and refocuses the list

## Run

The viewer resolves its history source in this order:
- first positional argument
- `CLIPBOARD_HISTORY_DB`
- the canonical per-user app-data history path for `clipboard-manager`

Examples:

```bash
cargo run -- /path/to/.clipboard_history.db
CLIPBOARD_HISTORY_DB=/path/to/.clipboard_history.db cargo run
cargo run
```

Raw JSON input is also supported as the first argument for read-only inspection.

## Hot Reload (Desktop)

Install Dioxus CLI:

```bash
cargo install dioxus-cli --version 0.7.3 --locked
```

Run with hot reload:

```bash
dx serve --platform desktop --args /path/to/.clipboard_history.db
```

## Notes

- The app polls for history changes every 500ms and watcher status every 1000ms.
- Text entries are decrypted from the shared SQLite history database using the OS keychain.
- Text entries can be copied back to the system clipboard.
- Image entries render from stored PNG blobs when available and fall back to local image paths.
- The UI supports search, keyboard navigation, delete, clear, clickable URLs, and watcher pause/resume on unix targets.
- Password-like text entries are masked by default in the detail pane and can be temporarily revealed after local authentication.
