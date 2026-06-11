# Cuniform 💠

**Cuniform** is a hyper-lightweight, native Windows workspace orchestrator built with **Rust**, **Tauri v2**, and **Svelte**. 

It allows you to group disparate desktop applications into logical "Tiles" (workspace families), bringing order to chaotic desktop environments. Built directly on top of the native Win32 API, Cuniform operates entirely offline with microscopic CPU/RAM footprints.

---

## ✨ Features

- **Workspace Tiles**: Group any arbitrary native Windows applications (e.g., Chrome, VS Code, and Spotify) into a single "Tile".
- **Instant Z-Order Sync**: When you focus one window in a Tile, the entire family of windows is instantly brought to the foreground, preserving their Z-order.
- **Toggle Visibility**: Minimize or restore an entire Tile of applications with a single click.
- **Omnipresent UI**: A sleek, borderless, frosted-glass control center that hovers above your work. Summon or banish it instantly using the global shortcut `Alt + Shift + C`.
- **Zero-Delay Capture**: Switch to any app on your desktop, summon Cuniform, and click the `[+]` capture button to instantly snap that window into a Tile.
- **Dynamic Pruning**: Automatically detects when you close an application via the OS and cleanly removes it from its assigned Tile.
- **Aero Snap Protected**: Custom borderless behavior prevents accidental Windows 11 edge-snapping when moving the orchestrator.

## 🚀 Installation & Build

### Prerequisites
You will need [Node.js](https://nodejs.org/), [Rust](https://rustup.rs/), and the standard [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) installed on your system.

### Development Server
To run Cuniform in development mode with hot-module reloading:
```powershell
npm install
npm run tauri dev
```

### Building the Executable
To compile Cuniform into a highly optimized, standalone Windows installer (`.exe` and `.msi`):
```powershell
npm run tauri build
```
Once the build completes, the installers will be available in `src-tauri/target/release/bundle/`.

## 🛠️ Tech Stack
* **Backend**: Rust (Win32 API interop, memory tracking, thread management).
* **Framework**: Tauri v2
* **Frontend**: SvelteKit, TailwindCSS (Glassmorphism design system)
* **OS**: Windows 10/11 exclusive.
