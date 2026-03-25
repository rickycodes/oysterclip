# Clipboard Viewer

A Dioxus Desktop app for browsing the shared clipboard history written by `clipboard-watcher`.

## Run

Pass the clipboard history database path as the first argument:

```bash
cargo run -- /path/to/.clipboard_history.db
```

The viewer currently requires that positional database path. Raw JSON input is also supported for read-only inspection.

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
