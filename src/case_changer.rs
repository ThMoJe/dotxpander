//! Case Changer: Win32 popup menu for transforming selected text.
//!
//! Triggered by `Ctrl+CapsLock` via the low-level keyboard hook.
//! The menu is shown from the hook message loop (NOT from inside the hook
//! callback) so that `TrackPopupMenu` can run its internal message pump
//! without violating the LL-hook 300 ms response requirement.

use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use windows::Win32::Foundation::{HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, RegisterClassExW, SetForegroundWindow, TrackPopupMenu,
    HMENU, MF_POPUP, MF_SEPARATOR, MF_STRING, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::PCWSTR;

use crate::config::AppConfig;
use crate::replacer::Replacer;

/// Custom window message posted from the keyboard hook proc to the hook
/// message loop, asking it to show the case-changer menu. Posted (not sent)
/// so it arrives after the hook callback returns and the message pump is idle.
pub const WM_SHOW_CASE_MENU: u32 = windows::Win32::UI::WindowsAndMessaging::WM_USER + 101;

/// `CF_UNICODETEXT` clipboard format ID (Win32 predefined format 13).
const CF_UNICODETEXT: u32 = 13;

/// Menu item command IDs returned by `TrackPopupMenu`.
const IDM_UPPERCASE: u32 = 1;
const IDM_LOWERCASE: u32 = 2;
const IDM_TITLE_CASE: u32 = 3;
const IDM_SENTENCE_CASE: u32 = 4;
const IDM_FIX_LINEBREAKS: u32 = 5;
const IDM_REMOVE_SPACES: u32 = 6;
const IDM_REPLACE_SPACE_UNDERSCORE: u32 = 7;
const IDM_REPLACE_SPACE_DASH: u32 = 8;
const IDM_LOWER_CAMEL: u32 = 9;
const IDM_PASCAL_CASE: u32 = 10;

/// Shows the case-changer popup menu and applies the selected transformation.
///
/// **Must be called from the hook message loop thread**, NOT from inside the
/// low-level keyboard hook callback. `TrackPopupMenu` runs an internal message
/// pump; calling it from the hook proc would starve the LL hook's response
/// timer and trigger Windows' automatic hook removal.
///
/// # Flow
/// 1. Inhibit hook re-entrancy
/// 2. Release held modifiers (Ctrl is still physically pressed from `Ctrl+CapsLock`)
/// 3. Backup all clipboard formats
/// 4. Send synthetic `Ctrl+C` to copy selected text
/// 5. Wait for the target app to write to the clipboard
/// 6. Read `CF_UNICODETEXT` from the clipboard
/// 7. Re-enable hook (needed during `TrackPopupMenu`'s message pump)
/// 8. Abort silently if no text was selected
/// 9. Show the menu at the cursor
/// 10. Apply the selected transformation via clipboard paste
pub fn show_case_menu(config: &Arc<ArcSwap<AppConfig>>, restore_delay_ms: u64) {
    crate::config::log_debug("CaseChanger: show_case_menu called");

    // 1. Inhibit the hook so our synthetic Ctrl+C is not treated as user input.
    crate::hook::set_inhibit(true);

    // 2. Release held modifiers (Ctrl is still down from Ctrl+CapsLock).
    Replacer::release_modifiers();
    std::thread::sleep(Duration::from_millis(10));

    // 3. Backup original clipboard contents.
    let saved = Replacer::backup_all_clipboard_formats();

    // 4. Synthetic Ctrl+C — copy selected text.
    Replacer::send_ctrl_c();

    // 5. Give the target app time to write the selection to the clipboard.
    std::thread::sleep(Duration::from_millis(restore_delay_ms));

    // 6. Read the selected text from the clipboard.
    let selected_text = get_clipboard_text();

    // 7. Re-enable the hook before showing the menu.
    //    TrackPopupMenu runs its own message pump; LL hooks must respond within
    //    ~300 ms or Windows removes them. The hook must be active during that pump.
    crate::hook::set_inhibit(false);

    // 8. Abort silently if nothing was selected.
    let text = match selected_text {
        Some(t) if !t.is_empty() => t,
        _ => {
            crate::config::log_debug("CaseChanger: aborting — no text selected (clipboard empty after Ctrl+C)");
            Replacer::restore_clipboard_formats(saved);
            return;
        }
    };
    crate::config::log_debug(&format!("CaseChanger: showing menu for {} chars of selected text", text.len()));

    // 9. Build and show the menu at the current cursor position.
    let lang = {
        let conf = config.load();
        conf.language.clone()
    };

    let hmenu = if let Some(h) = build_case_menu(&lang) { h } else {
        crate::config::log_debug("CaseChanger: build_case_menu failed — aborting");
        Replacer::restore_clipboard_formats(saved);
        return;
    };
    let cmd = show_menu_at_cursor(hmenu);
    unsafe { let _ = DestroyMenu(hmenu); }

    // User dismissed the menu (Escape or click outside).
    if cmd == 0 {
        Replacer::restore_clipboard_formats(saved);
        return;
    }

    // 10. Apply transformation.
    let transformed = match cmd {
        IDM_UPPERCASE => crate::text_utils::to_uppercase(&text),
        IDM_LOWERCASE => crate::text_utils::to_lowercase(&text),
        IDM_TITLE_CASE => crate::text_utils::to_title_case(&text),
        IDM_SENTENCE_CASE => crate::text_utils::to_sentence_case(&text),
        IDM_FIX_LINEBREAKS => crate::text_utils::fix_linebreaks(&text),
        IDM_REMOVE_SPACES => crate::text_utils::remove_spaces(&text),
        IDM_REPLACE_SPACE_UNDERSCORE => crate::text_utils::replace_spaces_with_underscore(&text),
        IDM_REPLACE_SPACE_DASH => crate::text_utils::replace_spaces_with_dash(&text),
        IDM_LOWER_CAMEL => crate::text_utils::to_lower_camel_case(&text),
        IDM_PASCAL_CASE => crate::text_utils::to_pascal_case(&text),
        _ => {
            Replacer::restore_clipboard_formats(saved);
            return;
        }
    };

    // Paste the transformed text, then restore the original clipboard.
    crate::hook::set_inhibit(true);

    if Replacer::set_clipboard_text(&transformed) {
        std::thread::sleep(Duration::from_millis(10));
        Replacer::send_ctrl_v();
        std::thread::sleep(Duration::from_millis(restore_delay_ms));
    }

    Replacer::restore_clipboard_formats(saved);
    crate::hook::set_inhibit(false);
}

/// Reads the current `CF_UNICODETEXT` clipboard contents as a `String`.
///
/// Returns `None` if the clipboard cannot be opened or contains no text.
fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }

        let result = (|| -> Option<String> {
            let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
            let hglobal = HGLOBAL(handle.0);
            let size = GlobalSize(hglobal);
            if size < 2 {
                return None;
            }

            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                return None;
            }

            // CF_UNICODETEXT is a null-terminated UTF-16 string.
            let num_u16 = size / 2;
            let slice = std::slice::from_raw_parts(ptr as *const u16, num_u16);

            // Find the null terminator.
            let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
            let text = String::from_utf16_lossy(&slice[..len]);

            let _ = GlobalUnlock(hglobal);
            Some(text)
        })();

        let _ = CloseClipboard();
        result
    }
}

/// Creates a Win32 popup menu with translated labels.
///
/// The returned `HMENU` must be destroyed by the caller with `DestroyMenu`.
/// Returns `None` if `CreatePopupMenu` fails (e.g. under extreme memory pressure).
/// Per the `unsafe-guard` skill: do not `.expect()` or `.unwrap()` on FFI calls.
fn build_case_menu(lang: &str) -> Option<HMENU> {
    let s = crate::i18n::get_strings(lang);

    unsafe {
        // Do NOT .expect() — CreatePopupMenu failure must not panic the hook thread.
        let hmenu = match CreatePopupMenu() {
            Ok(h) => h,
            Err(e) => {
                crate::config::log_debug(&format!("CaseChanger: CreatePopupMenu failed: {e}"));
                return None;
            }
        };

        append_string_item(hmenu, IDM_UPPERCASE, s.case_menu_uppercase);
        append_string_item(hmenu, IDM_LOWERCASE, s.case_menu_lowercase);
        append_string_item(hmenu, IDM_TITLE_CASE, s.case_menu_title_case);
        append_string_item(hmenu, IDM_SENTENCE_CASE, s.case_menu_sentence_case);
        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
        append_string_item(hmenu, IDM_FIX_LINEBREAKS, s.case_menu_fix_linebreaks);

        // Submenu: Spaces
        let spaces_submenu = match CreatePopupMenu() {
            Ok(h) => h,
            Err(e) => {
                crate::config::log_debug(&format!("CaseChanger: CreatePopupMenu for spaces failed: {e}"));
                let _ = DestroyMenu(hmenu);
                return None;
            }
        };
        append_string_item(spaces_submenu, IDM_REMOVE_SPACES, s.case_menu_remove_spaces);
        append_string_item(spaces_submenu, IDM_REPLACE_SPACE_UNDERSCORE, s.case_menu_space_to_underscore);
        append_string_item(spaces_submenu, IDM_REPLACE_SPACE_DASH, s.case_menu_space_to_dash);

        // Attach spaces submenu to parent menu
        append_popup_item(hmenu, spaces_submenu, s.case_menu_spaces_submenu);

        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
        append_string_item(hmenu, IDM_LOWER_CAMEL, s.case_menu_lower_camel);
        append_string_item(hmenu, IDM_PASCAL_CASE, s.case_menu_pascal_case);

        Some(hmenu)
    }
}

/// Appends a string menu item to `hmenu`.
///
/// The label is converted to a null-terminated UTF-16 wide string on the stack.
unsafe fn append_string_item(hmenu: HMENU, id: u32, label: &str) {
    let wide: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = AppendMenuW(
            hmenu,
            MF_STRING,
            id as usize,
            windows::core::PCWSTR(wide.as_ptr()),
        );
    }
}

/// Appends a submenu popup item to `hmenu`.
///
/// When `hmenu` is destroyed with `DestroyMenu`, Windows automatically destroys
/// all child submenus attached with `MF_POPUP`.
unsafe fn append_popup_item(hmenu: HMENU, submenu: HMENU, label: &str) {
    let wide: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = AppendMenuW(
            hmenu,
            MF_POPUP,
            submenu.0 as usize,
            windows::core::PCWSTR(wide.as_ptr()),
        );
    }
}

static REGISTER_CLASS_ONCE: std::sync::Once = std::sync::Once::new();
const OWNER_CLASS_NAME: PCWSTR = windows::core::w!("dotXPANDERCaseMenuOwner");

/// Custom window procedure for the helper owner window.
///
/// Handles `WM_MENUCHAR` so that pressing `_` (underscore) or `-` (dash)
/// executes "Replace space with _" (index 1) and "Replace space with -" (index 2)
/// in the Spaces submenu without requiring a visual underline under the symbol.
unsafe extern "system" fn case_menu_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    const WM_MENUCHAR: u32 = 0x0120;
    const MNC_EXECUTE: isize = 2;

    if msg == WM_MENUCHAR {
        let char_code = (wparam.0 & 0xFFFF) as u32;
        if let Some(ch) = char::from_u32(char_code) {
            if ch == '_' {
                // Item index 1 in Spaces submenu ("Replace space with _")
                return LRESULT((MNC_EXECUTE << 16) | 1);
            } else if ch == '-' {
                // Item index 2 in Spaces submenu ("Replace space with -")
                return LRESULT((MNC_EXECUTE << 16) | 2);
            }
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_owner_class_registered() {
    REGISTER_CLASS_ONCE.call_once(|| unsafe {
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(case_menu_wndproc),
            hInstance: hinstance.into(),
            lpszClassName: OWNER_CLASS_NAME,
            ..Default::default()
        };
        let _ = RegisterClassExW(&wc);
    });
}

/// Shows the popup menu at the current cursor position and returns the
/// selected command ID, or `0` if the user dismissed without selecting.
///
/// A temporary invisible popup window is created and made the foreground
/// window before calling `TrackPopupMenu`. This is required on Windows:
/// if the owning window is not the foreground window, the menu is dismissed
/// immediately after it opens. The helper window is destroyed when done.
fn show_menu_at_cursor(hmenu: HMENU) -> u32 {
    unsafe {
        let mut pt = windows::Win32::Foundation::POINT::default();
        let _ = GetCursorPos(&raw mut pt);

        ensure_owner_class_registered();

        // Create a temporary invisible popup window to own the menu.
        // TrackPopupMenu requires its owner to be the foreground window;
        // without this the menu is dismissed the instant it appears.
        // WS_EX_TOOLWINDOW keeps it off the taskbar.
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let hwnd_owner = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            OWNER_CLASS_NAME,
            PCWSTR::null(),
            WS_POPUP,
            pt.x, pt.y, 1, 1,
            None, None,
            hinstance,
            None,
        );

        let owner = match hwnd_owner {
            Ok(h) => h,
            Err(e) => {
                crate::config::log_debug(&format!("CaseChanger: CreateWindowExW failed: {e}"));
                windows::Win32::Foundation::HWND::default()
            }
        };

        // Bring our helper window to the foreground so TrackPopupMenu stays open.
        if !owner.is_invalid() {
            let _ = SetForegroundWindow(owner);
        }

        // TPM_RETURNCMD: return the menu item ID directly instead of posting WM_COMMAND.
        // TPM_NONOTIFY:  suppress WM_MENUSELECT / WM_INITMENUPOPUP notifications.
        // TPM_RIGHTBUTTON: allow selection with both left and right mouse buttons.
        let cmd = TrackPopupMenu(
            hmenu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            pt.x,
            pt.y,
            0,
            owner,
            None,
        );

        // Clean up the temporary owner window.
        if !owner.is_invalid() {
            let _ = DestroyWindow(owner);
        }

        // Cast to u32: TPM_RETURNCMD makes TrackPopupMenu return the item ID directly.
        cmd.0 as u32
    }
}

/// Posts `WM_SHOW_CASE_MENU` to the hook thread's own message queue.
///
/// Called from inside `low_level_keyboard_proc` when `Ctrl+CapsLock` is detected.
/// Posting (not sending) ensures the menu is shown after the hook callback
/// returns, which is required for `TrackPopupMenu` to work correctly.
pub fn post_show_case_menu() {
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
            GetCurrentThreadId(),
            WM_SHOW_CASE_MENU,
            WPARAM(0),
            LPARAM(0),
        );
    }
}
