# dotXPANDER Roadmap

Project roadmap and milestone tracking for **dotXPANDER** by **aiVOLUTION**.

---

## ✅ Completed Milestones (v0.2.0 Release)

### 1. Inno Setup Install Wizard & Dual-Mode Uninstall ✅
- Full Inno Setup script (`installer/setup.iss`) with per-user installation (`%LOCALAPPDATA%\Programs\aiVOLUTION\dotXPANDER`), no UAC requirement.
- Branded wizard artwork (`wizard_small.bmp` and `wizard_large.bmp`).
- Custom config location page with cloud sync preservation.
- **Smart Dual-Mode Uninstall**: Detects `unins000.exe` when installed to invoke official uninstaller (cleaning up Start Menu shortcuts, Autostart keys, and registry entries), while retaining delayed `del /f /q` self-deletion for portable mode.
- Downgrade prevention guard, running-instance detection (`AppMutex`), and silent install flags (`/VERYSILENT`).

### 2. Windows Filename-Safe Text Transformations ✅
- Added 3 Win32 filesystem-safe text transformations to the `Ctrl + CapsLock` Case & Space Changer:
  - **Clean Windows Filename**: Strips illegal characters (`< > : " / \ | ? *`), control codes, trims leading/trailing spaces and dots, and clamps length to 255 bytes while preserving international Unicode characters (Danish, etc.).
  - **Clean Windows Filename (Underscores)**: Sanitizes illegal characters and replaces whitespace sequences with `_`.
  - **Clean Windows Filename (Dashes)**: Sanitizes illegal characters and replaces whitespace sequences with `-`.

### 3. Rebranding: dotXPANDER & aiVOLUTION ✅
- Transitioned identity to **dotXPANDER** by **aiVOLUTION** across the codebase, Slint UI, VERSIONINFO string table, application manifest, and CI/CD workflows.

### 4. About Tab in Settings Window ✅
- Integrated dedicated About tab in `ui/main.slint` displaying live version, CPU target architecture (ARM64 vs. x64), author credits, and interactive links to repository and license.

### 5. High-DPI Cursor Centering & Window Stability ✅
- Implemented cursor-aware monitor work-area centering for multi-monitor setups.
- Stabilized Slint layout constraint envelopes (`min-width`, `preferred-width`) to guarantee rock-solid window dimensions during tab navigation.

---

## 🚀 Upcoming Milestones (Post-v0.2.0)

### Phase 2: Winget Package Distribution
- Submit package manifest to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) for one-command installation:
  ```powershell
  winget install dotXPANDER
  ```
- Automate future winget release submissions via GitHub Actions (`winget-releaser`).

### Phase 3: Code Signing (Authenticode)
- Acquire and integrate an OV/EV Code Signing Certificate into `.github/workflows/release.yml` to sign binaries and installers, eliminating Windows Defender SmartScreen warnings.

### Phase 4: Microsoft Store (MSIX Packaging)
- Configure MSIX packaging via Windows SDK / Partner Center.
- Package for Microsoft Store distribution under the Productivity & Accessibility categories.

### Phase 5: Feature Enhancements
- **Dynamic Snippet Variables**: Support date/time macros (e.g. `{date}`, `{time}`, `{clipboard}`) and cursor repositioning tokens (`{cursor}`) in snippet expansion text.
- **Snippet Search & Filtering**: Real-time search/filter bar in the Slint Snippets tab for large snippet collections.
- **Import / Export**: JSON/CSV snippet library import and export functionality in the Settings UI.
