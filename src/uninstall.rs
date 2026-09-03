//! Self-destruct / uninstall support.
//!
//! Handles both:
//! - **Installed mode (Inno Setup / Winget)**: Launches the official uninstaller
//!   `unins000.exe` as a detached process and cleanly exits the application so
//!   Windows can remove files, shortcuts, autostart keys, and registry entries.
//! - **Portable mode**: Spawns a hidden, detached `cmd.exe` process that sleeps
//!   briefly to allow the process to exit, then deletes the standalone `.exe` and
//!   wipes portable configuration files.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use windows::Win32::System::Threading::{CREATE_NO_WINDOW, DETACHED_PROCESS};

/// Checks if an Inno Setup uninstaller (`unins000.exe`) exists alongside the executable.
#[must_use]
pub fn find_uninstaller(exe_path: &Path) -> Option<PathBuf> {
    let parent = exe_path.parent()?;
    let uninstaller = parent.join("unins000.exe");
    if uninstaller.is_file() {
        Some(uninstaller)
    } else {
        None
    }
}

/// Executes the full self-destruct or uninstaller sequence:
///
/// 1. Resolves the current `.exe` path.
/// 2. If `unins000.exe` exists in the application directory (Installed Mode):
///    - Spawns `unins000.exe` as a detached process.
///    - Shuts down the hook thread and terminates the Slint event loop.
///    - Exits immediately so the uninstaller is not blocked by a running process.
/// 3. If `unins000.exe` does not exist (Portable Mode):
///    - Deletes the entire config directory (settings, log, etc.).
///    - Spawns a hidden, detached `cmd.exe` process that waits ~3 s then
///      deletes the `.exe` file.
///    - Shuts down the hook thread and terminates the Slint event loop.
///
/// Returns `Err(String)` if the `.exe` path cannot be resolved or if spawning
/// the delete/uninstall process fails.
pub fn self_destruct(hook_thread_id: u32) -> Result<(), String> {
    // --- Step 1: Resolve own exe path ---
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Could not determine exe path: {e}"))?;

    // --- Step 2: Check for Inno Setup / Winget uninstaller ---
    if let Some(uninstaller) = find_uninstaller(&exe_path) {
        // Installed mode: launch official unins000.exe
        std::process::Command::new(&uninstaller)
            .creation_flags(DETACHED_PROCESS.0)
            .spawn()
            .map_err(|e| format!("Failed to spawn uninstaller {}: {e}", uninstaller.display()))?;
    } else {
        // Portable mode: wipe config and run delayed file deletion
        let exe_str = exe_path
            .to_str()
            .ok_or_else(|| "Exe path contains non-UTF-8 characters".to_string())?
            .to_owned();

        // Wipe config directory (best-effort)
        crate::config::delete_config_dir();

        // Spawn hidden delayed-delete process
        let delete_cmd = format!("ping -n 4 127.0.0.1 >nul & del /f /q \"{exe_str}\"");
        let creation_flags = CREATE_NO_WINDOW.0 | DETACHED_PROCESS.0;

        std::process::Command::new("cmd")
            .args(["/c", &delete_cmd])
            .creation_flags(creation_flags)
            .spawn()
            .map_err(|e| format!("Failed to spawn delete process: {e}"))?;
    }

    // --- Step 3: Shut down the hook thread ---
    unsafe {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
        let _ = PostThreadMessageW(hook_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
    }

    // --- Step 4: Terminate the Slint event loop ---
    let _ = slint::quit_event_loop();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_find_uninstaller_none_when_nonexistent() {
        let fake_path = Path::new("C:\\fake\\path\\dotxpander.exe");
        assert_eq!(find_uninstaller(fake_path), None);
    }
}
