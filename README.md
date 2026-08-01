# rust-file-explorer (`fe`)

A simple, fast TUI file explorer built with Rust and Ratatui.

![Demo](demo.gif)

## Layout

```text
┌── Directory: /home/user/projects ──────────────────────────────────┐
│ >> 📁  src/                                                        │
│    📄  Cargo.toml                          [320 B]                 │
│    📄  README.md                           [1.1 KB]                │
├── Search ──────────────────────────────────────────────────────────┤
│ rust                                                               │
└────────────────────────────────────────────────────────────────────┘
```

## Features

* **Type to filter:** Just start typing anywhere to instantly search the current folder.
* **Folder priority:** Directories always stay pinned to the top.
* **Zero clutter:** No bloat—just quick directory navigation from your shell.

## Keybindings

| Key | Action |
| --- | --- |
| `j` / `k` or `↑` / `↓` | Navigate files |
| `l` / `Enter` or `→` | Open directory |
| `h` or `←` | Go up one directory |
| `Type any text` | Filter current listing |
| `Backspace` | Erase filter character |
| `Esc` | Clear active filter |
| `Ctrl + C` | Quit |

## Installation

```bash
git clone https://github.com/GrybosKamil/rust-file-explorer.git
cd rust-file-explorer
cargo install --path .
```

Make sure `~/.cargo/bin` is in your `$PATH`, then launch it anywhere with:

```bash
fe
```

## License

MIT