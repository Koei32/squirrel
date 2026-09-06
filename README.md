# Squirrel

Squirrel is a clipboard manager.

https://github.com/user-attachments/assets/f7e38954-1004-441e-b94b-679466037e30

## About

Squirrel is a to be a clipboard manager for Windows and X11 Linux[*](#a-note-on-linux-support) that
is simple and useful. It is made using Tauri with a Rust backend and Svelte for its frontend.

Squirrel uses the [`clipboard-rs`](https://docs.rs/clipboard-rs/latest/clipboard_rs/) crate to
interface with the operating system's clipboard in order to listen to and capture clipboard events.

## Features

- **Global hotkey**: Press Ctrl+Shift+V to bring up Squirrel.

- **Pinning**: Pin important items to the top of the list.

<img width="600" height="338" alt="pin" src="https://github.com/user-attachments/assets/c3838359-eb32-4405-a5e8-2833d3d8bf86" />

- **Item search**: Instant exact string search.

<img width="600" height="338" alt="search" src="https://github.com/user-attachments/assets/3fd53219-55e1-4f71-81db-9015e2fb4396" />

- **Image Support**: In addition to text, Squirrel supports images of up to ~5 megabytes in size.

<img width="600" height="338" alt="images" src="https://github.com/user-attachments/assets/5b8fde3d-0b2e-4d07-b4d2-0d9ba9bcb4d0" />

- **Configuration**: Squirrel allows basic app configuration. Configuration file is stored at
  `%appdata%/Squirrel/squirrel.toml` for Windows and `~/.config/Squirrel/squirrel.toml` for Linux.
  Check `default.toml` for available options.

- **_Technically_ unlimited history**:
  There isn't a defined limit to the amount of entries that can be stored. Store as much as you
  want. Though practically, the program might load the history a bit slowly when there's a very
  large amount of items.

### Shortcuts

We love them

- `Ctrl` `Shift` `V`: show Squirrel
- `Esc`: hide Squirrel
- `←` `↑` `→` `↓`: navigation
- `/`: jump to searchbar
- `Ctrl` `C`: copy selected item
- `Ctrl` `P`: pin/unpin selected item
- `Return`: paste selected item into last focused window
- `Delete`: delete an item
- `Ctrl` `Shift` `Delete`: permanently clear _**all**_ history (!)

## Installation

The latest build is available in Releases. Squirrel is distributed as both a single file executable
and a bundled installer. If the portable version does not work, it's recommended to install
Squirrel using the appropriate installer.

### A note on Linux support

While builds for Linux are available, the functionality is severely limited on Wayland due to the
nature of the protocol. Global hotkey and quick paste are _non-functional_ on Wayland. It does work
but only on apps that still use the X11 backend on Wayland.

Global hotkey could be functional in the near future when the active
[PR tackling the issue](https://github.com/tauri-apps/global-hotkey/pull/172) on
[tauri-plugin-global-shortcut](https://github.com/tauri-apps/global-hotkey/) is merged.

Though not tested as thoroughly as on Windows, Squirrel works on X11 with close to zero problems.
Please open an issue if something doesn't behave as expected.

## Building from source

You can also build Squirrel from source for your operating system.

### Requirements:

- **Rust** >=1.95
- **npm** >=11.13.0

Older versions might work, though are untested.

### Instructions

1. Clone the repository

```bash
git clone https://github.com/Koei32/squirrel && cd squirrel
```

2. Install dependencies

```bash
npm install
```

3. Build the Tauri app

```bash
npm run tauri build
```

The executable is output in `src-tauri/target/release/`, and the bundles in `src-tauri/target/release/bundle`.

## Credits

All icons except the Squirrel logo are Copyright (c) 2026 [Lucide Icons and Contributors](https://lucide.dev/)
