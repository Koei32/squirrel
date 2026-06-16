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

- Global hotkey: Press Ctrl+Shift+V to bring up Squirrel.
- Quick paste: Selecting a clipboard item and pressing Return instantly pastes it into the last focused window.
- Item search: Instant exact string search.
- Dark/light theme: Squirrel uses the theme of your system.
- _Technically_ unlimited history:
  There isn't a defined limit to the amount of entries that can be stored. Store as much as you want.
  Though practically, the program might be slow to load the history when there's a very large amount of items.

## Credits

All icons except the Squirrel logo are Copyright (c) 2026 Lucide Icons and Contributors (https://lucide.dev/)

#### TODO

- [ ] linux support & testing
- [ ] pinning entries
- [ ] image & file copy support
- [ ] lazy loading history
