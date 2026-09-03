fn main() {
    // ── Slint UI compilation ──────────────────────────────────────────────
    slint_build::compile("ui/main.slint").expect("Failed to compile Slint UI");

    // ── Windows subsystem & entry point ──────────────────────────────────
    // Prevents a console window from appearing when the app is launched.
    // These are emitted as raw linker flags; they are overridden by the
    // #![windows_subsystem = "windows"] attribute on the binary crate, but
    // explicitly emitting them here keeps build.rs self-documenting and also
    // applies when building the lib target under test.
    println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg-bins=/ENTRY:mainCRTStartup");

    // ── Win32 resources (Windows only) ───────────────────────────────────
    // Embeds into the .exe:
    //   • VERSIONINFO block  — shows in Properties → Details tab
    //   • Application icon   — shown in File Explorer, Task Manager, Alt+Tab
    //   • Application manifest — DPI awareness, UAC level, OS targeting
    #[cfg(target_os = "windows")]
    {
        let version = env!("CARGO_PKG_VERSION");

        // Convert "MAJOR.MINOR.PATCH" → (MAJOR, MINOR, PATCH, 0) for FILEVERSION
        let parts: Vec<u64> = version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let (major, minor, patch) = (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        );

        let mut res = winresource::WindowsResource::new();

        // ── VERSIONINFO string table ──────────────────────────────────────
        res.set("FileDescription",  "dotXPANDER — Text Expander & Productivity Utility");
        res.set("ProductName",      "dotXPANDER");
        res.set("CompanyName",      "aiVOLUTION");
        res.set("LegalCopyright",   "Copyright \u{00a9} 2026 Thomas M\u{00f8}ller Jensen");
        res.set("FileVersion",      version);
        res.set("ProductVersion",   version);
        res.set("InternalName",     "dotxpander");
        res.set("OriginalFilename", "dotxpander.exe");
        res.set("Comments",         "https://github.com/ThMoJe/dotxpander");

        // ── Numeric version (for FILEVERSION / PRODUCTVERSION fields) ─────
        res.set_version_info(winresource::VersionInfo::FILEVERSION,    (major << 48) | (minor << 32) | (patch << 16));
        res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, (major << 48) | (minor << 32) | (patch << 16));

        // ── Embedded application icon ──────────────────────────────────────
        // Shows in File Explorer, taskbar, Alt+Tab switcher, and Task Manager.
        res.set_icon("ui/icon.ico");

        // ── Embedded application manifest ─────────────────────────────────
        // Declares: PerMonitorV2 DPI awareness, asInvoker UAC, Windows 10/11
        // supported OS, long-path awareness, UTF-8 code page.
        res.set_manifest_file("app.manifest");

        res.compile().expect("Failed to compile Windows resources");
    }
}
