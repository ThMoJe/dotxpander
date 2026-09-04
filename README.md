![dotXPANDER](assets/design/original_logo_text.png)

**A lightweight, high-performance Windows text expander and productivity utility.**  
Native Rust and Slint UI for Windows 11 ARM64 and Windows 10/11 x86_64.

![Latest Release](https://img.shields.io/github/v/release/ThMoJe/dotxpander?style=flat-square)![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20ARM64%20%26%20x64-informational?style=flat-square)![Rust Edition](https://img.shields.io/badge/rust-2024%20edition-orange?style=flat-square)

---

dotXPANDER is an open-source productivity tool designed for speed, low resource usage, and seamless compatibility with modern Windows 11 applications (including WinUI 3, Windows Terminal, and Notepad).

Unlike electron-based expanders, dotXPANDER runs as a single, highly optimized native binary that requires no background runtime, uses **~5.8 MB of RAM**, and draws **0.0% CPU** while idle.

---

## Features

### Snippet Expansion

- **Immediate Mode**: Expands trigger strings instantly as you type (e.g. `.sig` → email signature).
- **Hotkey Mode**: Expands triggers on demand using a configurable keyboard shortcut (e.g. `Ctrl + Shift + T`).
- **Zero-Allocation Ring Buffer**: Keystroke matching runs against an in-memory ring buffer with zero heap allocations during active typing (~2.7 ns push latency).
- **Configurable Memory Buffer**: Customize how many keystrokes to retain in memory (2 to 25 characters) with an interactive stepper in Settings. The internal ring buffer resizes dynamically on configuration save without restarting the application.
- **WinUI 3 & Modern App Compatibility**: Uses an asynchronous clipboard injection pipeline with automatic modifier-key handling, avoiding dropped or duplicated characters in XAML and Chromium apps.

### Case & Space Changer (`Ctrl + CapsLock`)

Select text in any application and press `Ctrl + CapsLock` to transform the selection:

- **Case conversion**: UPPERCASE, lowercase, Title Case, Sentence case, lowerCamelCase, PascalCase.
- **Whitespace utilities**: Remove extra spaces, convert spaces to dashes (`kebab-case`), or convert spaces to underscores (`snake_case`).
- **Windows Filename Sanitization**: Clean selected text for safe Win32 filenames (stripping forbidden `< > : " / \ | ? *` characters and control codes, trimming dots/spaces, and clamping length to 255 bytes while preserving international Unicode characters):
  - Clean characters only
  - Clean + Underscores
  - Clean + Dashes
- **Linebreak normalization**: Convert mixed line endings to standard Windows CRLF (`\r\n`).
- Uses `unicode-segmentation` for grapheme-aware boundary detection.

### Quick Switch (File Dialog Navigation)

- Automatically synchronizes standard Windows Open/Save file dialogs (`#32770`) to the folder currently open in your active File Explorer window when you switch back and forth using ALT+Tab.
- Fully supports Windows 11 multi-tab Explorer windows via out-of-process COM (`IShellWindows`) inspection.
- Operates 100% out-of-process via `SetWinEventHook` — no DLL injection or system modification.

### Flexible Configuration & Cloud Sync

- **Config Location Wizard**: Choose where your `config.toml` lives during installation — default AppData or a cloud-synced folder (OneDrive, Dropbox, Google Drive) to automatically share snippets across multiple computers.
- **Portable Mode**: When run without an installed registry key, dotXPANDER automatically loads and saves `config.toml` directly alongside the executable.
- **In-App Management & Cloud-Sync Hints**: View your active mode ("Installed" vs. "Portable") and relocate your configuration file at any time from the Settings window, complete with helpful cloud-sync guidance tooltips.

### Privacy & Resource Efficiency

- **100% Offline**: Zero networking dependencies, zero telemetry, and zero data collection.
- **Event-Driven**: Built on native Win32 message queues. The application thread sleeps until an event arrives — zero polling loops and zero background battery drain.
- **Safe Clipboard Restore**: Original clipboard contents (including rich text and images) are captured and restored automatically after snippet insertion.
- **Multilingual UI**: Native English (`en`) and Danish (`da`) interface with runtime language selection.

---

## Resource Profile & Benchmarks

### Resource Usage Comparison

| Metric                       | dotXPANDER (Native Rust)               | Typical Web/Electron Expanders |
| ---------------------------- | -------------------------------------- | ------------------------------ |
| **Idle CPU Usage**           | **0.0%** (event-driven, 0 ms idle CPU) | 0.5% – 3.0% (timer loops / GC) |
| **Private Committed Memory** | **~5.8 MB**                            | 120 MB – 250 MB                |
| **Working Set (RAM)**        | **~31 MB** *(including OS DLLs)*       | 180 MB – 350+ MB               |
| **Thread Count**             | **2 threads** (Win32 Hook + Slint UI)  | 12 – 24+ threads               |
| **Cold Startup Time**        | **< 50 ms**                            | 1,500 – 4,000 ms               |

### Micro-Benchmark Latencies (`cargo bench`)

Measured on Windows 11 using Criterion:

| Operation                       | Latency      | Description                                           |
| ------------------------------- | ------------ | ----------------------------------------------------- |
| `KeyBuffer::push`               | **2.77 ns**  | Circular ring buffer insertion (zero heap allocation) |
| Config lookup (`ArcSwap`)       | **6.26 ns**  | Lock-free, wait-free configuration read               |
| Trigger Scan (10 snippets)      | **9.34 ns**  | Evaluated on active typing keystrokes                 |
| Exact Match Verification        | **8.46 ns**  | Direct trigger confirmation                           |
| Win32 CRLF Normalization        | **57.47 ns** | UTF-16 byte buffer formatting                         |
| Multi-Format Clipboard Snapshot | **34.14 µs** | Preserves existing clipboard formats prior to paste   |

---

## Installation

### Option A: Windows Installer (Recommended)

Download `dotXPANDER-x64-Setup.exe` (or `dotXPANDER-arm64-Setup.exe`) from [Releases](https://github.com/ThMoJe/dotxpander/releases).

The setup wizard:

- Installs to `%LOCALAPPDATA%\Programs\aiVOLUTION\dotXPANDER` (no Administrator/UAC prompt required).
- Lets you select where your `config.toml` is stored (local AppData or a cloud-synced folder).
- Creates Start Menu and optional Desktop shortcuts.
- Offers optional autostart at Windows sign-in.
- Supports clean uninstallation from Windows Settings, preserving cloud-synced snippet files.
- Displays the MIT license agreement during installation.
- Detects a running instance and offers to close it before upgrading — no manual steps needed.
- Guards against accidental downgrading over a newer version.
- Requires Windows 10 (build 17763 / 1809) or later.

### Option B: Portable ZIP

Download `dotXPANDER-x64-v0.2.0.zip` (or `dotXPANDER-arm64-v0.2.0.zip`) and extract the executable anywhere (such as a USB drive or local directory).

- When launched without an installer registry key, dotXPANDER runs in **Portable Mode**.
- `config.toml` is read from and saved to the same directory as `dotxpander.exe`.
- Nothing is written to the Windows Registry.

### Option C: Silent / Enterprise Installation

For automated deployment (Winget, MDM, Group Policy, or CI scripts), the installer supports standard Inno Setup silent flags:

```powershell
# Fully silent — no UI, no reboot prompt
dotXPANDER-x64-Setup.exe /VERYSILENT /NORESTART /ALLUSERS=0

# Silent with progress bar visible
dotXPANDER-x64-Setup.exe /SILENT /NORESTART

# Silent uninstall (locates uninstaller automatically)
"%LOCALAPPDATA%\Programs\aiVOLUTION\dotXPANDER\unins000.exe" /VERYSILENT /NORESTART
```

> **Note:** `/ALLUSERS=0` installs per-user (to `%LOCALAPPDATA%`), requiring no UAC prompt. This is the only supported mode — system-wide installation is not available.

---

## Configuration

Settings and snippet definitions are stored in plain TOML format in `config.toml`.

### File Location

| Mode                           | Location                                                            |
| ------------------------------ | ------------------------------------------------------------------- |
| **Installed** (Default)        | `%APPDATA%\aiVOLUTION\dotXPANDER\config.toml`                       |
| **Installed** (Custom / Cloud) | Path registered at `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath` |
| **Portable**                   | Same directory as `dotxpander.exe`                                  |

You can open or relocate the active config folder directly from the **General** tab in the Settings window.

### Example `config.toml`

```toml
language = "en" # "en" or "da"
buffer_size = 10
clipboard_restore_delay_ms = 150
snippet_hotkey_enabled = true
quick_switch_enabled = true # Auto-navigate file dialogs to active Explorer tab
case_changer_enabled = true

[hotkey]
modifiers = 6 # Ctrl (2) + Shift (4)
virtual_key = 84 # 'T' key (0x54)

[case_changer_hotkey]
modifiers = 2 # Ctrl (2)
virtual_key = 20 # CapsLock (0x14)

[[snippets]]
trigger = ".jd"
replacement = "john.doe@example.com"
mode = "immediate"

[[snippets]]
trigger = "jd"
replacement = "John Doe"
mode = "immediate"

[[snippets]]
trigger = ".sig"
replacement = """
Regards,
John Doe
101 Undisclosed Avenue
Nowhere, XX 99999
(212) 555-0123"""
mode = "immediate"

[[snippets]]
trigger = ",bio"
replacement = "John Doe is the nameless man with no known past, living somewhere unknown and carrying a phone that never got a real number, whose entire story is that nobody yet knows who he really is."
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

# Run full test suite (155 tests)
cargo test

# Run micro-benchmarks
cargo bench --bench buffer_benchmark

# Build release binary (native architecture)
cargo build --release

# Build optimized release using build-std & UPX
.\scripts\build-release.ps1
```

---

## Changelog

### v0.2.0

- **Windows Setup Installer & Smart Uninstall**: Per-user Inno Setup installer with custom branded wizard images, Start Menu / Desktop shortcuts, autostart configuration, and dual-mode uninstallation (delegating to `unins000.exe` when installed, and delayed self-deletion in portable mode).
- **Custom Config Location Wizard**: Interactive installer step to choose between default AppData and custom/cloud-synced folders (OneDrive, Dropbox, etc.) with automatic snippet preservation on upgrade.
- **Runtime Portable Mode**: Automatic fallback to local-directory config when no installation registry key is detected.
- **UI Mode Badge & About Tab**: Added mode indicator ("📦 Portable" / "💿 Installed"), About tab with live version / architecture badges, and "Move Config File…" relocation action in the Settings window with cloud-sync guidance tooltip.
- **Configurable Keystroke Memory Buffer**: Added numeric input with `[-] [ 10 ] [+]` horizontal stepper controls in Settings to configure buffer capacity (2 to 25 characters), with on-the-fly zero-allocation ring buffer resizing (`KeyBuffer::resize`) and real-time typed memory synchronization.
- **High-DPI Centering & Stable Window Layout**: Cursor-aware monitor work-area auto-centering, HiDPI scaling, and locked tab-switching constraints preventing layout jumps.
- **Win32 Executable Metadata**: Embedded VERSIONINFO block (`ProductName`, `FileDescription`, `CompanyName`, `LegalCopyright`, `FileVersion`) — visible in File Explorer → Properties → Details and Windows Task Manager.
- **Application Icon in Binary**: App icon (`ui/icon.ico`) embedded directly into `dotxpander.exe` — shown in File Explorer, Alt+Tab switcher, and taskbar without relying solely on the installer.
- **Application Manifest**: Embedded `app.manifest` declaring PerMonitorV2 DPI awareness (sharp rendering on HiDPI / multi-monitor setups), `asInvoker` UAC level, Windows 10/11 `supportedOS` GUIDs, long-path awareness, UTF-8 active code page, and Segment Heap.
- **Installer Hardening**: Added `MinVersion=10.0.17763` (blocks install on unsupported OS), `AppMutex` (detects running instance and offers to close it before upgrade), `CloseApplications` / `RestartApplications` (graceful upgrade flow), `LicenseFile` (MIT license shown in wizard), and a Pascal downgrade guard (`InitializeSetup()`) that warns before installing an older version over a newer one.
- **Silent / Enterprise Install**: Documented Inno Setup silent flags (`/VERYSILENT /SILENT /NORESTART /ALLUSERS=0`) in README for Winget, MDM, and sysadmin deployment.
- **Case & Space Changer**: Global `Ctrl + CapsLock` menu supporting 10+ text transformations (uppercase, lowercase, title case, camelCase, PascalCase, hyphenate, underscore, line-break normalization, and 3 Windows Filename sanitization modes).
- **Quick Switch File Dialog Sync**: Auto-navigation of Windows file dialogs to the active File Explorer folder, including Windows 11 multi-tab Explorer support via COM.
- **Release Optimization Pipeline**: Integrated Rust Nightly `build-std`, dead-code stripping MSVC linker flags, and UPX compression (~58% size reduction on x64).
- **Rebranding**: Complete identity update to **dotXPANDER** by **aiVOLUTION**.
- **Test Suite**: Expanded automated unit test suite to **155 unit tests** across ring buffer resizing, geometry, filename transformations, configuration, internationalization, and uninstall flows.

### v0.1.0

- Initial dual-architecture release supporting ARM64 and x86_64.
- Zero-allocation ring buffer matching engine.
- Immediate and hotkey snippet expansion modes.
- Slint UI settings interface with system tray support.
- English and Danish internationalization.

---

## License

This project is licensed under the [MIT License](LICENSE).
