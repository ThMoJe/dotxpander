# dotXPANDER

A high-performance, native Windows text expander and productivity utility written in **Rust** and **Slint UI**, supporting both **Windows 11 ARM64** and **Windows 10/11 x86_64 (Intel/AMD)**.

Designed from the ground up for speed, minimal resource consumption, and rock-solid compatibility with modern Windows 11 apps (including WinUI 3, Windows 11 Notepad, and Chromium-based browsers).

---

## 🚀 Key Features

- ⚡ **Ultra-Fast Snippet Expansion**:
  - **Immediate Mode**: Expands automatically the instant you finish typing a trigger string (e.g. `:email` → `user@example.com`).
  - **Hotkey Mode**: Expands on demand when your configured shortcut is pressed (e.g. typing a short abbreviation followed by `Alt + Shift + X`).
  - **Zero-Allocation Ring Buffer**: Microsecond-level matching (~1.12 ns / check, ~2.77 ns push, 360M+ ops/sec) with zero heap allocation during active typing.
- 🔤 **Universal Case Changer (`Ctrl + CapsLock`)**:
  - Select text anywhere in Windows and hit `Ctrl + CapsLock` to bring up a lightning-fast transformation menu:
    - **UPPERCASE** & **lowercase**
    - **Title Case** & **Sentence case**
    - **lowerCamelCase** & **PascalCase**
    - **Whitespace Helpers**: Remove all spaces, Replace spaces with underscores (`_`), or Replace spaces with hyphens/dashes (`-`)
    - **Fix Linebreaks**: Normalizes any selection to standard Windows CRLF (`\r\n`)
  - Grapheme-aware Unicode word splitting via `unicode-segmentation`.
- 📁 **Quick Switch (File Dialog & Explorer Sync)**:
  - Automatically synchronizes standard Windows Open/Save file dialogs (`#32770`) to the folder currently open in your active Windows Explorer window or tab.
  - Windows 11 multi-tab Explorer support via COM `IShellWindows` extraction.
  - 100% out-of-process event monitoring (`SetWinEventHook`) — no invasive DLL injection or hooking crashes.
- 🪟 **Modern Windows 11 / WinUI 3 Compatibility**:
  - Dedicated clipboard injection engine that avoids dropped or duplicated characters in asynchronous XAML/WinUI 3 applications.
  - Automatically manages modifier key state (Ctrl, Shift, Alt) during expansion.
- 🍃 **Near-Zero Resource Footprint**:
  - Uses only **~5.8 MB private memory** and **0.0% idle CPU** (100% event-driven Win32 hooks with zero polling).
- 🖥️ **Lightweight Slint UI & System Tray**:
  - Clean settings window and tray integration with software rendering (`renderer-software`) for GPU-independent stability across Qualcomm Snapdragon ARM64 chips and x86 GPUs.
  - Quick **Pause / Resume** toggle right from the UI.
- 🌐 **Multilingual Support (i18n)**: Native English (`en`) and Danish (`da`) UI translations.
- 🔒 **100% Offline & Private**: Zero network dependencies, zero telemetry, local configuration (`config.toml`).

---

## ⚡ Performance & Resource Profile

### 1. Minimal Resource Footprint (Background & Idle)

Written in 100% native Rust without Chromium, Node.js, or Electron runtimes, its memory and CPU footprint are orders of magnitude smaller than typical text expanders:

| Metric | dotXPANDER (Native) | Electron / Web-based Expanders | Efficiency Advantage |
| :--- | :--- | :--- | :--- |
| **Idle / Background CPU** | **0.0%** (0 ms CPU time) | 0.5% – 3.0% (Timer polling / GC) | **100% Event-Driven (Zero Polling)** |
| **Private Committed Memory** | **~5.8 MB** | 120 MB – 250 MB | **~96% Less Memory** |
| **Working Set (RAM)** | **~31 MB** *(incl. OS DLLs)* | 180 MB – 350+ MB | **~90% Smaller RAM Footprint** |
| **Thread Count** | **2 threads** *(UI + Hook)* | 12 – 24+ threads | **Minimal OS context switching** |
| **Cold Startup Time** | **< 50 ms** | 1,500 – 4,000 ms | **Instant launch** |

> [!TIP]
> **Why 0.0% CPU?**
> dotXPANDER relies entirely on native Win32 event hooks (`WH_KEYBOARD_LL` and `SetWinEventHook`) dispatched through `GetMessageW`. The OS wakes the thread only when an actual event occurs — zero polling loops, zero background timers, zero idle power draw on battery.

---

### 2. Microsecond-Level Latency & Instant Expansion

Any action under **100 ms** feels instantaneous to a human (Miller / Nielsen HCI threshold). dotXPANDER detects triggers in **nanoseconds** and injects text in **microseconds** — completing replacements in **0.36% of a single 144 Hz display frame**:

```
Display & Perception Budgets vs dotXPANDER Execution Time:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Human "Instant" Perception Threshold: 100,000 µs (100 ms)                   │
├──────────────────────────────────────────────────────┬──────────────────────┤
│ 60 Hz Display Frame Budget: 16,667 µs (16.7 ms)      │                      │
├───────────────────────────────────┬──────────────────┴──────────────────────┤
│ 144 Hz Display Frame: 6,944 µs    │                                         │
├───────────────────────────────────┴─────────────────────────────────────────┤
│ █ dotXPANDER Keystroke Overhead: 0.010 µs (~10 ns)                       │
│ ██ dotXPANDER Total Injection Setup: ~25 - 300 µs (0.025 - 0.30 ms)      │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Benchmark Suite Results (`cargo bench --bench buffer_benchmark`)

| Operation | Measured Latency | Throughput / Notes |
| :--- | :--- | :--- |
| **Keystroke Ring Buffer Ingestion** (`KeyBuffer::push`) | **2.77 ns** | **360+ Million keystrokes/sec** (Zero heap allocation) |
| **Lock-Free Config Lookup** (`ArcSwap::load`) | **6.26 ns** | Wait-free synchronization, 0 mutex contention |
| **Trigger Scan (1 Snippet)** | **1.12 ns** | Single-cycle early exit on first mismatched char |
| **Trigger Scan (10 Snippets)** | **9.34 ns** | Evaluated on 99.99% of normal typing keystrokes |
| **Trigger Scan (50 Snippets)** | **46.67 ns** | 0.046 µs overhead per keystroke |
| **Trigger Scan (100 Snippets)** | **97.61 ns** | < 0.1 µs overhead even with large snippet collections |
| **Exact Match Verification** (`:my_email_address`) | **8.46 ns** | Instant trigger confirmation |
| **Win32 CRLF Normalization (Single-line, 20 chars)** | **57.47 ns** | In-memory UTF-16 byte buffer formatting |
| **Win32 CRLF Normalization (Template, 110 chars)** | **206.41 ns** | Multi-line template transformation |
| **Full Clipboard Snapshot** *(All Active Formats)* | **34.14 µs** | Non-destructive multi-format preservation |
| **Win32 Clipboard Text Set** (`CF_UNICODETEXT`) | **~309 µs** | Global moveable memory allocation & transfer |

---

## 🛠️ Technical Architecture & Design

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           dotXPANDER Architecture                      │
└───────────────────────────────────────────────────────────────────────────┘
                                     │
         ┌───────────────────────────┴───────────────────────────┐
         ▼                                                       ▼
 ┌───────────────────────────────┐               ┌───────────────────────────────┐
 │       Win32 Event Thread      │               │        Slint UI Thread        │
 │  - WH_KEYBOARD_LL Hook        │               │  - Software Renderer          │
 │  - Zero-Alloc KeyBuffer Ring  │ ◄───ArcSwap─► │  - System Tray Integration    │
 │  - Replacer & Clipboard Guard │    (Lock-free)│  - Snippet Manager & Config   │
 │  - Case Changer Menu Pump     │               │  - i18n Translation Engine    │
 │  - Quick Switch COM Watcher   │               │                               │
 └───────────────────────────────┘               └───────────────────────────────┘
```

### 1. Zero-Allocation Reverse Buffer Matching
The keyboard hook maintains a fixed-size circular ring buffer (`KeyBuffer`) tracking recent keystrokes without heap allocation. When evaluating snippet triggers, matching iterates backwards using a double-ended iterator directly on UTF-8 streams. This achieves immediate early-exit on the first mismatched character with zero `malloc`/heap overhead during active typing.

### 2. WinUI 3 Reliable Text Injection Engine
Per-character `SendInput` (`KEYEVENTF_UNICODE`) frequently drops or duplicates characters in asynchronous XAML/WinUI 3 applications. dotXPANDER employs a dedicated clipboard injection technique:
1. Releases active modifier keys (Ctrl, Shift, Alt) to prevent unwanted shortcuts.
2. Backspaces the trigger length.
3. Temporarily injects the replacement text via the Win32 Clipboard (`CF_UNICODETEXT`) and simulates `Ctrl + V`.
4. Asynchronously restores the original clipboard content after an intentional 150 ms window (preserving all previous clipboard formats).

### 3. Out-of-Hook Case Changer Menu Loop
The `Ctrl + CapsLock` case changer popup is displayed using a custom window message (`WM_SHOW_CASE_MENU`) dispatched to the background message loop rather than inside the low-level hook callback. This prevents `TrackPopupMenu`'s modal message pump from exceeding Windows' ~300 ms low-level hook watchdog limit.

### 4. Windows 11 Tab-Aware COM Path Extraction (Quick Switch)
When a standard file dialog (`#32770`) gains focus:
- Monitors window focus via `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` with `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS`.
- Filters out transient shell chrome (`Shell_TrayWnd`, `MultitaskingViewFrame`, `XamlExplorerHostIslandWindow`).
- Resolves the active Explorer tab path via `IShellWindows` / `IServiceProvider` / `IShellBrowser` / `IFolderView` / `IPersistFolder2` COM interfaces.
- Navigates the dialog directly via `WM_SETTEXT` + simulated `Enter`, with address-bar fallback.

---

## 🚀 Quickstart

### Prerequisites
- Windows 11 or Windows 10 (ARM64 or x86_64)
- [Rust toolchain](https://rustup.rs/) (edition 2024+)

### Building and Running

```powershell
# Run in development mode (defaults to your machine's native architecture)
cargo run

# Build optimized release for ARM64 (Snapdragon X / Surface Pro)
cargo build --release --target aarch64-pc-windows-msvc

# Build optimized release for x86_64 (Intel / AMD)
cargo build --release --target x86_64-pc-windows-msvc

# Run unit test suite
cargo test

# Run micro-benchmark suite
cargo bench --bench buffer_benchmark
```

---

## 📦 Installation

dotXPANDER ships in two flavours. Choose whichever fits your workflow:

### Option A — Windows Installer (Recommended)

Download `dotXPANDER-Setup-x64.exe` (or `-arm64`) from the [Releases page](https://github.com/ThMoJe/dotXPANDER/releases) and run it.

The installer handles everything:
- Installs to `%LOCALAPPDATA%\Programs\dotXPANDER` — no UAC prompt required.
- Creates a Start Menu shortcut and optionally a Desktop shortcut.
- Optionally configures autostart at Windows login.
- Asks **where to store your settings and snippets** (`config.toml`):

| Choice | When to use |
|:-------|:------------|
| **Default** (`%APPDATA%\dotXPANDER`) | Single computer, no sync needed |
| **Cloud-synced folder** (e.g. `OneDrive\Apps\dotXPANDER`) | Share snippets across multiple PCs automatically |

> [!TIP]
> **Sync snippets across computers for free** — pick a folder inside your OneDrive, Dropbox, or other cloud drive during the installer wizard. Every machine running dotXPANDER pointing to the same folder will share snippets and settings without any account or server required.

The chosen directory is saved to `HKCU\Software\dotXPANDER\ConfigPath`. On uninstall, only the default `%APPDATA%` location is wiped — custom/cloud paths are always left intact.

---

### Option B — Portable ZIP (No Install)

Download `dotXPANDER-x64.zip` (or `-arm64`) and extract the `.exe` anywhere — a USB drive, a Dropbox folder, or a local directory.

When **no registry key is present**, dotXPANDER runs in **Portable Mode**:
- `config.toml` is created and read from the **same folder as the `.exe`**.
- Nothing is written to the registry or `%APPDATA%`.

To switch from Portable to Installed mode, run the installer at any time — it will detect and preserve your existing `config.toml`.

---


## ⚙️ Configuration

Settings and snippets are saved to `config.toml`. The location depends on your installation mode:

| Mode | Config location |
|:-----|:----------------|
| **Installed** (default) | `%APPDATA%\dotXPANDER\config.toml` |
| **Installed** (custom) | Path stored in `HKCU\Software\dotXPANDER\ConfigPath` |
| **Portable** | Same folder as `dotXPANDER.exe` |

Open the config folder at any time via the 📁 button on the **General** tab.

```toml
language = "en" # "en" or "da"
buffer_size = 10
clipboard_restore_delay_ms = 150
quick_switch_enabled = true # Auto-navigate file dialogs to active Explorer folder

[hotkey]
modifiers = 6 # Ctrl (2) + Shift (4)
virtual_key = 84 # 'T' key (0x54)

[[snippets]]
trigger = ".sig"
replacement = "Regards,\nJohn Doe"
mode = "immediate"

[[snippets]]
trigger = ".em"
replacement = "john.doe@example.com"
mode = "immediate"

[[snippets]]
trigger = "jd"
replacement = "With kind regards and sincerely - John Doe"
mode = "hotkey"
```

---

## 🔒 Security, Privacy & Offline Guarantee

Because dotXPANDER operates via a low-level Windows keyboard hook (`WH_KEYBOARD_LL`) to detect snippet triggers, privacy and transparency are paramount:

- 🔒 **100% Offline**: The binary contains **zero networking crates** (`reqwest`, `tokio`, sockets, etc.) and transmits **zero telemetry or analytics**.
- 🗄️ **Local Storage Only**: Snippet definitions, hotkeys, and preferences remain strictly on your local device in `config.toml`.
- 📋 **Safe Clipboard Restoration**: During expansion, the clipboard is temporarily replaced with the injected text and then restored to its original content (all formats preserved) within 150 ms.
- 🔍 **Auditable & Open Source**: The entire codebase is open source and can be inspected or compiled from source directly.

---

## ⚠️ Known Limitations & Technical Considerations

- **Elevated Windows / Administrator Mode (UAC)**: Due to Windows User Interface Privilege Isolation (UIPI), standard user-mode applications cannot capture keystrokes or inject text into windows running with elevated Administrator privileges (such as an Administrator Terminal or Task Manager). If you frequently type into elevated windows, run dotXPANDER as Administrator.
- **DirectInput / Exclusive Fullscreen Games**: Games or software that read raw hardware input directly bypassing standard Win32 message queues will not trigger expansions.
- **Password Fields**: When typing into sensitive fields, use hotkey-triggered expansion mode or pause active expansion if needed.

---

## 📜 Changelog

### v0.3.0 (In Progress)
- 💿 **Inno Setup Windows Installer**: Classic setup experience with per-user install (no UAC), Start Menu shortcut, optional Desktop shortcut, and optional autostart.
- 📍 **Config Location Wizard** *(installer)*: During setup, choose where to store `config.toml` — default `%APPDATA%\dotXPANDER` or any folder, including cloud-synced drives like OneDrive or Dropbox.
- ☁️ **Registry-Backed Config Path** *(installer)*: Chosen directory written to `HKCU\Software\dotXPANDER\ConfigPath`. Existing `config.toml` at the target is preserved (supports sharing snippets from another machine).
- 🧹 **Smart Uninstall** *(installer)*: Only wipes config files from the default `%APPDATA%` path — custom/cloud locations are always left intact.
- 📦 **Portable Mode** *(runtime — planned)*: Registry-aware config discovery in the Rust app. When no registry key is found, `config.toml` is read from the executable's folder.
- 🏷️ **Mode Badge** *(UI — planned)*: General tab will show a badge indicating Portable or Installed mode.
- 🔀 **Move Config File** *(UI — planned)*: "Move Config File…" button to relocate `config.toml` at any time from the settings window.


### v0.2.0
- 🔤 **Case Changer Popup (`Ctrl + CapsLock`)**: Transform selected text instantly across 10+ formats (UPPERCASE, lowercase, Title Case, Sentence case, lowerCamelCase, PascalCase, remove/replace spaces, fix linebreaks).
- 📁 **Quick Switch File Dialog Auto-Navigation**: Automatically synchronizes standard Windows Open/Save file dialogs (`#32770`) to the active Windows Explorer folder/tab.
- 🪟 **Windows 11 Tabbed Explorer Support**: Direct COM `IShellWindows` enumeration to reliably resolve active tab paths without DLL injection.
- ⏸️ **Pause / Resume Expansion**: Dedicated pause toggle in the Slint settings UI and system tray.
- ⚡ **Expanded Micro-Benchmark Suite**: Added comprehensive sub-microsecond benchmarks for buffer pushes, trigger scans, CRLF normalization, and clipboard snapshots.
- 🧪 **Comprehensive Unit Test Suite**: Over 40 unit tests covering edge cases, grapheme clusters, clipboard format backups, and configuration serialization.

### v0.1.0 — Initial Release
- Native ARM64 & x86_64 dual architecture support.
- Zero-allocation ring buffer matching engine.
- Immediate and Hotkey expansion modes.
- Slint UI with system tray integration and English / Danish i18n.
- WinUI 3 and Windows 11 Notepad compatible clipboard injection engine.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
