# Squirrel

Squirrel is a clipboard manager.

https://github.com/user-attachments/assets/f7e38954-1004-441e-b94b-679466037e30

## About

Squirrel is intended to be a cross-platform\* clipboard manager that is simple and useful.
It is made using Tauri with a Rust backend and Svelte for its frontend.

Squirrel uses the [`clipboard-rs`](https://docs.rs/clipboard-rs/latest/clipboard_rs/) crate to
interface with the operating system's clipboard in order to listen to and capture clipboard events.

<small>\*work in progress</small>

## Features

- **Global hotkey**: Press Ctrl+Shift+V to bring up Squirrel.
- **Quick paste**: Selecting a clipboard item and pressing Return instantly pastes it into the last focused window.
  <img width="800" height="450" alt="paste" src="https://github.com/user-attachments/assets/a08b4930-2d22-4dc8-8fbd-61f6bfa66b77" />

- **Item search**: Instant exact string search.
  <img width="800" height="450" alt="search" src="https://github.com/user-attachments/assets/3fd53219-55e1-4f71-81db-9015e2fb4396" />
- **Dark/light theme**: Squirrel uses the theme of your system.
  <img width="800" alt="darklight" src="https://github.com/user-attachments/assets/50d20dd9-e3f4-43d2-a1e3-6c0b74663f0b" />

- **_Technically_ unlimited history**:
  There isn't a defined limit to the amount of entries that can be stored. Store as much as you want.
  Though practically, the program might load the history a bit slowly when there's a very large amount of items.

## Installation

The latest build is available in Releases. Squirrel is distributed as both a single file executable and a bundled
installer. If the portable version does not work, it's recommended to install Squirrel using the appropriate installer.

Currently, there are only Windows builds. Linux releases will be available soon.

## Building from source

You can also build Squirrel from source.

### Requirements:

- Rust >=1.95
- npm >=11.13.0

Older versions might work, though are untested.

### Instructions

1. Clone the repository

```
git clone https://github.com/Koei32/squirrel && cd squirrel
```

2. Install dependencies

```
npm install
```

3. Build the Tauri app

```
npm run tauri build
```

The executable is outputted in `src-tauri/target/release/`, and the bundles in `src-tauri/target/release/bundle`.

## Credits

All icons except the Squirrel logo are Copyright (c) 2026 Lucide Icons and Contributors (https://lucide.dev/)

#### TODO

- [ ] linux support & testing
- [ ] pinning entries
- [ ] image & file copy support
- [ ] lazy loading history
