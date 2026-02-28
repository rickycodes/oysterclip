# Clipboard Viewer (Dioxus Desktop MVP)

This branch ports the UI/runtime path to pure Dioxus Desktop.
Legacy Tauri sources are archived at `archive/src-tauri-legacy`.

## Run

Pass the clipboard history path (or raw JSON) as the first argument:

```bash
cargo run -- /path/to/clipboard_history.json
```

## Notes

- The app polls for changes every 500ms and updates in place.
- Text entries can be copied back to the system clipboard.
- Image entries load local image files and render as data URLs.
