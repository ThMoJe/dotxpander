# Pre-Release Roadmap

Internal tracking document for features and improvements planned prior to the public release, after which public issue tracking and community feedback will move to GitHub.

---

## High Priority

### 1. Inno Setup Install Wizard Review & Polish
Review the current installer wizard flow (`installer/setup.iss`) to identify UX friction and technical gaps.
- Verify per-user and autostart registration behavior across clean Windows installs.
- Test custom config directory selection and ensure fallback paths gracefully handle non-standard setups.
- Streamline wizard text, step ordering, and default choices for first-time users.

### 2. Filename-Safe Text Transformations in Case/Space Changer
Extend `case_changer.rs` and `text_utils.rs` with menu actions that sanitize selected text into valid, filesystem-safe filenames (stripping Win32 reserved characters: `< > : " / \ | ? *` and control codes).
- **Clean Characters Only**: Remove illegal characters while keeping valid letters, numbers, spaces, and punctuation intact.
- **Clean + Underscore**: Remove illegal characters and convert all whitespace sequences into single underscores (`_`).
- **Clean + Hyphen**: Remove illegal characters and convert all whitespace sequences into single hyphens (`-`).

### 3. Code Signing (Authenticode)
Acquire an OV or EV Code Signing Certificate and integrate it into the GitHub Actions pipeline (`release.yml`) to sign the installer and executable, preventing Windows Defender SmartScreen warnings.

---

## Medium Priority

### 3. Rebranding: "dot eXpander" (.XPANDER) & aiVOLUTION
Transition project naming and branding across the repository, UI, installer, and deployment assets.
- **Assets & App Identity**: Update application titles, window headers, system tray tooltips, and iconography with new `.XPANDER` / "dot eXpander" visual assets.
- **Organization Branding**: Incorporate aiVOLUTION publisher branding into metadata, installer details, and documentation.
- **Repository & Distribution**: Rename GitHub repository paths and update references in `Cargo.toml`, CI/CD workflows (`.github/workflows/release.yml`), and binary output filenames.

### 4. About Tab in Settings Window
Add a minimal "About" tab to the Slint configuration window (`ui/main.slint` and `src/ui.rs`).
- Display current version, target architecture (ARM64 vs. x86_64), and build information.
- Provide direct, clickable external links to the GitHub repository, release notes, license, and upcoming Microsoft Store listing.
