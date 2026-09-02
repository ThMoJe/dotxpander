<p align="center">
  <img src="assets/design/original_logo_text.png" alt="dotXPANDER" width="280">
</p>

<p align="center">
  <strong>A lightweight, high-performance Windows text expander and productivity utility.</strong><br>
  Native Rust and Slint UI for Windows 11 ARM64 and Windows 10/11 x86_64.
</p>

<p align="center">
  <a href="https://github.com/ThMoJe/dotxpander/releases"><img src="https://img.shields.io/github/v/release/ThMoJe/dotxpander?style=flat-square" alt="Latest Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20ARM64%20%26%20x64-informational?style=flat-square" alt="Platforms">
  <img src="https://img.shields.io/badge/rust-2024%20edition-orange?style=flat-square" alt="Rust Edition">
</p>

---

dotXPANDER is an open-source productivity tool designed for speed, low resource usage, and seamless compatibility with modern Windows 11 applications (including WinUI 3, Windows Terminal, and Notepad).

Unlike electron-based expanders, dotXPANDER runs as a single, highly optimized native binary that requires no background runtime, uses **~5.8 MB of RAM**, and draws **0.0% CPU** while idle.

---

## Features

### Snippet Expansion
- **Immediate Mode**: Expands trigger strings instantly as you type (e.g. `.sig` → email signature).
- **Hotkey Mode**: Expands triggers on demand using a configurable keyboard shortcut (e.g. `Ctrl + Shift + T`).
- **Zero-Allocation Ring Buffer**: Keystroke matching runs against an in-memory ring buffer with zero heap allocations during active typing (~2.7 ns push latency).
- **WinUI 3 & Modern App Compatibility**: Uses an asynchronous clipboard injection pipeline with automatic modifier-key handling, avoiding dropped or duplicated characters in XAML and Chromium apps.

### Case & Space Changer (`Ctrl + CapsLock`)
Select text in any application and press `Ctrl + CapsLock` to transform the selection:
- **Case conversion**: UPPERCASE, lowercase, Title Case, Sentence case, lowerCamelCase, PascalCase.
- **Whitespace utilities**: Remove extra spaces, convert spaces to dashes (`kebab-case`), or convert spaces to underscores (`snake_case`).
- **Linebreak normalization**: Convert mixed line endings to standard Windows CRLF (`\r\n`).
- Uses `unicode-segmentation` for grapheme-aware boundary detection.

### Quick Switch (File Dialog Navigation)
- Automatically synchronizes standard Windows Open/Save file dialogs (`#32770`) to the folder currently open in your active File Explorer window.
- Fully supports Windows 11 multi-tab Explorer windows via out-of-process COM (`IShellWindows`) inspection.
- Operates 100% out-of-process via `SetWinEventHook` — no DLL injection or system modification.

### Flexible Configuration & Cloud Sync
- **Config Location Wizard**: Choose where your `config.toml` lives during installation — default AppData or a cloud-synced folder (OneDrive, Dropbox, Google Drive) to automatically share snippets across multiple computers.
- **Portable Mode**: When run without an installed registry key, dotXPANDER automatically loads and saves `config.toml` directly alongside the executable.
- **In-App Management**: View your active mode ("Installed" vs. "Portable") and relocate your configuration file at any time from the Settings window.

### Privacy & Resource Efficiency
- **100% Offline**: Zero networking dependencies, zero telemetry, and zero data collection.
- **Event-Driven**: Built on native Win32 message queues. The application thread sleeps until an event arrives — zero polling loops and zero background battery drain.
- **Safe Clipboard Restore**: Original clipboard contents (including rich text and images) are captured and restored automatically after snippet insertion.
- **Multilingual UI**: Native English (`en`) and Danish (`da`) interface with runtime language selection.

---

## Resource Profile & Benchmarks

### Resource Usage Comparison

| Metric | dotXPANDER (Native Rust) | Typical Web/Electron Expanders |
| :--- | :--- | :--- |
| **Idle CPU Usage** | **0.0%** (event-driven, 0 ms idle CPU) | 0.5% – 3.0% (timer loops / GC) |
| **Private Committed Memory** | **~5.8 MB** | 120 MB – 250 MB |
| **Working Set (RAM)** | **~31 MB** *(including OS DLLs)* | 180 MB – 350+ MB |
| **Thread Count** | **2 threads** (Win32 Hook + Slint UI) | 12 – 24+ threads |
| **Cold Startup Time** | **< 50 ms** | 1,500 – 4,000 ms |

### Micro-Benchmark Latencies (`cargo bench`)

Measured on Windows 11 using Criterion:

| Operation | Latency | Description |
| :--- | :--- | :--- |
| `KeyBuffer::push` | **2.77 ns** | Circular ring buffer insertion (zero heap allocation) |
| Config lookup (`ArcSwap`) | **6.26 ns** | Lock-free, wait-free configuration read |
| Trigger Scan (10 snippets) | **9.34 ns** | Evaluated on active typing keystrokes |
| Exact Match Verification | **8.46 ns** | Direct trigger confirmation |
| Win32 CRLF Normalization | **57.47 ns** | UTF-16 byte buffer formatting |
| Multi-Format Clipboard Snapshot | **34.14 µs** | Preserves existing clipboard formats prior to paste |

---

## Installation

### Option A: Windows Installer (Recommended)

Download `dotXPANDER-Setup-x64.exe` (or `-arm64`) from [Releases](https://github.com/ThMoJe/dotxpander/releases).

The setup wizard:
- Installs to `%LOCALAPPDATA%\Programs\aiVOLUTION\dotXPANDER` (no Administrator/UAC prompt required).
- Lets you select where your `config.toml` is stored (local AppData or a cloud-synced folder).
- Creates Start Menu and optional Desktop shortcuts.
- Offers optional autostart at Windows sign-in.
- Supports clean uninstallation from Windows Settings, preserving cloud-synced snippet files.

### Option B: Portable ZIP

Download `dotXPANDER-x64.zip` (or `-arm64`) and extract the executable anywhere (such as a USB drive or local directory).
- When launched without an installer registry key, dotXPANDER runs in **Portable Mode**.
- `config.toml` is read from and saved to the same directory as `dotxpander.exe`.
- Nothing is written to the Windows Registry.

---

## Configuration

Settings and snippet definitions are stored in plain TOML format in `config.toml`.

### File Location

| Mode | Location |
| :--- | :--- |
| **Installed** (Default) | `%APPDATA%\aiVOLUTION\dotXPANDER\config.toml` |
| **Installed** (Custom / Cloud) | Path registered at `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath` |
| **Portable** | Same directory as `dotxpander.exe` |

You can open or relocate the active config folder directly from the **General** tab in the Settings window.

### Example `config.toml`

```toml
language = "en" # "en" or "da"
buffer_size = 10
clipboard_restore_delay_ms = 150
quick_switch_enabled = true # Auto-navigate file dialogs to active Explorer tab

[hotkey]
modifiers = 6 # Ctrl (2) + Shift (4)
virtual_key = 84 # 'T' key (0x54)

[[snippets]]
trigger = ".sig"
replacement = "Kind regards,\nYour Name"
mode = "immediate"

[[snippets]]
trigger = ".em"
replacement = "your.email@example.com"
mode = "immediate"

[[snippets]]
trigger = "addr"
replacement = "123 Innovation Way, Suite 400\nTech City, TC 54321"
mode = "hotkey"
```

---

## Architecture Overview

```
                        dotXPANDER Architecture
                                   │
         ┌─────────────────────────┴─────────────────────────┐
         ▼                                                   ▼
 ┌─────────────────────────────┐             ┌─────────────────────────────┐
 │     Win32 Hook Thread       │             │       Slint UI Thread       │
 │ - WH_KEYBOARD_LL hook       │             │ - Software renderer         │
 │ - Zero-alloc ring buffer    │ ◄─ ArcSwap ─► - System tray integration   │
 │ - Clipboard replacer engine │  (lock-free)│ - Snippet management        │
 │ - Quick Switch COM listener │             │ - i18n localization engine │
 │ - Case Changer menu dispatch│             │ - Config migration helpers  │
 └─────────────────────────────┘             └─────────────────────────────┘
```

1. **Ring Buffer Matching**: Recent keystrokes are recorded in a fixed-size ring buffer. Matching iterates backwards directly on UTF-8 streams with immediate early exit upon mismatch.
2. **Clipboard Injection Pipeline**: Releases pressed modifier keys, backspaces the trigger sequence, writes replacement text via `CF_UNICODETEXT`, simulates `Ctrl + V`, and restores original clipboard contents after paste completion.
3. **Out-of-Hook Menu Pump**: The `Ctrl + CapsLock` case changer menu is dispatched to the background message queue via a custom window message (`WM_SHOW_CASE_MENU`), avoiding Windows low-level hook watchdog timeouts.
4. **Tab-Aware Shell Inspection**: For file dialog navigation, dotXPANDER queries the active Explorer window via `IShellWindows` and `IFolderView` COM interfaces to reliably extract active tab paths without invasive API hooking.

---

## Building from Source

### Prerequisites
- Windows 10 or Windows 11 (ARM64 or x86_64)
- [Rust toolchain](https://rustup.rs/) (2024 edition)

### Commands

```powershell
# Run in development mode
cargo run

# Run full test suite (130 tests)
cargo test

# Run micro-benchmarks
cargo bench --bench buffer_benchmark

# Build release binary (native architecture)
cargo build --release

# Build optimized release using build-std & UPX
.\scripts\build-release.ps1 -Arch x64
```

---

## Changelog

### v0.2.0
- **Windows Setup Installer**: Per-user Inno Setup installer with custom branded header, desktop/start menu shortcuts, autostart configuration, and post-install launch option.
- **Custom Config Location Wizard**: Interactive installer step to choose between default AppData and custom/cloud-synced folders (OneDrive, Dropbox, etc.) with automatic snippet preservation.
- **Runtime Portable Mode**: Automatic fallback to local directory configuration when no installation registry key is detected.
- **UI Mode Badge & Migration**: Added mode indicator ("📦 Portable" / "💿 Installed") and "Move Config File…" relocation action in the Settings window.
- **Case & Space Changer**: Global `Ctrl + CapsLock` menu supporting 10+ text transformations (uppercase, lowercase, title case, camelCase, PascalCase, hyphenate, underscore, line break normalization).
- **Quick Switch File Dialog Sync**: Auto-navigation of Windows file dialogs to the active File Explorer folder, including Windows 11 multi-tab Explorer support via COM.
- **Release Optimization Pipeline**: Integrated Rust Nightly `build-std`, dead-code stripping MSVC linker flags, and UPX compression.
- **Rebranding**: Complete identity update to **dotXPANDER** by **aiVOLUTION**.
- **Test Suite**: Expanded automated unit test suite to 130 tests.

### v0.1.0
- Initial dual-architecture release supporting ARM64 and x86_64.
- Zero-allocation ring buffer matching engine.
- Immediate and hotkey snippet expansion modes.
- Slint UI settings interface with system tray support.
- English and Danish internationalization.

---

## License

This project is licensed under the [MIT License](LICENSE).
