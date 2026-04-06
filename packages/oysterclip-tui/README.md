# OysterClip TUI

A terminal user interface for browsing clipboard history.

## Features

- **List View**: Browse your clipboard history (up to 100 recent entries)
- **Detail View**: View full content of selected entry
- **Simple Navigation**: Arrow keys to move, `q` to quit

## Usage

```bash
cargo run -p oysterclip-tui
```

### Keybindings

- `↑` / `↓` - Navigate history
- `q` / `Esc` - Quit

## Architecture

The TUI reads directly from the encrypted clipboard history database, just like the GUI viewer:

- **Data Source**: Reads from `~/.local/share/OysterClip/clipboard.db` (configurable)
- **Display**: Text entries shown with newlines replaced by `↵` for clarity
- **Independent**: Doesn't require the watcher daemon to be running

## Future Enhancements

- Copy selected entry to clipboard
- Delete entries
- Search/filter functionality
- Authentication for sensitive entries
- Image preview
