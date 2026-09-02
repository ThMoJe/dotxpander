//! Quick Switch — automatically navigate Open/Save dialogs to the last Explorer folder.
//!
//! When `quick_switch_enabled` is `true` in the config, a dedicated background
//! thread installs a `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` listener.
//!
//! **State machine (strict two-step sequence)**:
//! 1. Explorer window (`CabinetWClass`) gains focus — record its path.
//! 2. File dialog (`#32770`) gains focus immediately after — inject the path.
//!    Any window appearing *between* step 1 and step 2 resets the state so that
//!    only a direct Explorer -> dialog transition triggers navigation.
//!
//! **Path extraction** uses `IShellWindows` COM enumeration (ARM64-safe,
//! no DLL injection, works with multi-tab Windows 11 Explorer).
//!
//! **Dialog navigation** tries two strategies in order:
//! 1. Primary: `FindWindowExW` to locate the filename `Edit` control,
//!    then `WM_SETTEXT` + `SendInput(Enter)`.
//! 2. Fallback: `Ctrl+L` + clipboard paste + `Enter`.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use arc_swap::ArcSwap;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL,
    IServiceProvider, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetForegroundWindow, GetMessageW, GetWindow, GW_CHILD, GW_HWNDNEXT,
    PostThreadMessageW, SendMessageW, SetForegroundWindow, MSG, WM_QUIT, WM_SETTEXT,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
};
use windows::core::PCWSTR;

use crate::config::{self, AppConfig};
use crate::replacer::Replacer;
#[allow(clippy::cast_possible_wrap)]
    const SIZEOF_INPUT: i32 = std::mem::size_of::<INPUT>() as i32;

fn send_inputs(inputs: &[INPUT]) {
    let inserted = unsafe { SendInput(inputs, SIZEOF_INPUT) };
    if inserted != inputs.len() as u32 {
        crate::config::log_debug(&format!("SendInput failed: inserted {}/{}", inserted, inputs.len()));
    }
}


// ---------------------------------------------------------------------------
// Win32 constants
// ---------------------------------------------------------------------------

const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const CABINET_CLASS: &str = "CabinetWClass";
const DIALOG_CLASS: &str = "#32770";

/// Window classes that emit transient `EVENT_SYSTEM_FOREGROUND` events during
/// normal window switching but do NOT represent intentional user navigation
/// away from the Explorer → dialog flow.
///
/// When the user clicks the taskbar to switch from Explorer to an open dialog,
/// Windows fires `EVENT_SYSTEM_FOREGROUND` for `Shell_TrayWnd` *before* firing
/// it for the dialog. Without this list, the taskbar event would clear
/// `last_explorer_hwnd` and the subsequent dialog event would find nothing to do.
///
/// All classes here are pure system/shell chrome — they never host user content.
const PASSTHROUGH_CLASSES: &[&str] = &[
    "Shell_TrayWnd",                // Primary taskbar
    "Shell_SecondaryTrayWnd",       // Per-monitor taskbar (multi-monitor)
    "Progman",                      // Desktop
    "WorkerW",                      // Desktop worker window
    "XamlExplorerHostIslandWindow", // Win11 snap layout popup, Start menu island
    "TaskListThumbnailWnd",         // Taskbar thumbnail preview popup
    "MultitaskingViewFrame",        // Alt+Tab / Task View overlay
    "ForegroundStaging",            // Win11 focus-change staging window
];

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

/// Focus state machine for the Quick Switch feature.
///
/// We intentionally store only the **HWND** of the last Explorer window, not
/// its path. This is the key correctness property: the path is read via COM at
/// the moment the file dialog appears, so it always reflects what folder
/// Explorer was displaying *when the user left it* — even if they navigated
/// several folders after Explorer last received `EVENT_SYSTEM_FOREGROUND`.
/// Storing the path eagerly (at Explorer-focus time) produced a "one step
/// behind" bug because no new event fires while the user browses within Explorer.
struct FocusState {
    /// HWND of the last Explorer window that received foreground focus.
    /// `None` if any non-Explorer, non-dialog window has intervened since.
    last_explorer_hwnd: Option<HWND>,
}

impl FocusState {
    const fn new() -> Self {
        Self { last_explorer_hwnd: None }
    }
}

use std::cell::RefCell;

thread_local! {
    static FOCUS_STATE: RefCell<FocusState> = const { RefCell::new(FocusState::new()) };
    static SHARED_CONFIG: RefCell<Option<Arc<ArcSwap<AppConfig>>>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// Public manager
// ---------------------------------------------------------------------------

pub struct QuickSwitchManager {
    thread_id: u32,
    handle: Option<thread::JoinHandle<()>>,
}

impl QuickSwitchManager {
    pub fn start(config: Arc<ArcSwap<AppConfig>>) -> Self {
        let (id_tx, id_rx) = std::sync::mpsc::channel::<u32>();
        let handle = thread::spawn(move || {
            run_quick_switch_thread(config, id_tx);
        });
        let thread_id = id_rx.recv().unwrap_or(0);
        config::log_debug(&format!("QuickSwitch: thread started (tid={thread_id})"));
        Self { thread_id, handle: Some(handle) }
    }

    pub fn stop(&mut self) {
        if self.thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
            self.thread_id = 0;
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        config::log_debug("QuickSwitch: thread stopped.");
    }
}

impl Drop for QuickSwitchManager {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Thread entry point
// ---------------------------------------------------------------------------

fn run_quick_switch_thread(
    config: Arc<ArcSwap<AppConfig>>,
    id_tx: std::sync::mpsc::Sender<u32>,
) {
    // SAFETY: must be called before any COM usage on this thread.
    // CoInitializeEx returns an HRESULT (not Result<>):
    //   S_OK    (0x00000000) — successfully initialised as STA.
    //   S_FALSE (0x00000001) — already initialised as STA; still usable.
    //   Any negative HRESULT — fatal; COM unusable on this thread.
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if hr.is_err() {
        config::log_debug(&format!("QuickSwitch: CoInitializeEx failed (hr=0x{:08X}) — thread aborting", hr.0));
        let _ = id_tx.send(0);
        return;
    }

    SHARED_CONFIG.with(|cell| { *cell.borrow_mut() = Some(config); });

    // WINEVENT_OUTOFCONTEXT (0x0000): callback runs on this thread via GetMessage —
    //   no DLL injection required, ARM64-safe.
    // WINEVENT_SKIPOWNPROCESS (0x0002): do not fire when our own windows gain focus
    //   (e.g. the settings ConfigWindow), avoiding pointless COM enumeration.
    const WINEVENT_OUTOFCONTEXT:   u32 = 0x0000;
    const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

    let hook: HWINEVENTHOOK = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0, 0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };

    if hook.is_invalid() {
        config::log_debug("QuickSwitch: SetWinEventHook failed — thread aborting");
        let _ = id_tx.send(0);
        SHARED_CONFIG.with(|cell| { *cell.borrow_mut() = None; });
        unsafe { CoUninitialize(); }
        return;
    }

    let tid = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
    let _ = id_tx.send(tid);

    let mut msg = MSG::default();
    loop {
        let ret = unsafe { GetMessageW(&raw mut msg, None, 0, 0) };
        if ret.0 <= 0 { break; }
    }

    if !hook.is_invalid() { unsafe { let _ = UnhookWinEvent(hook); } }
    SHARED_CONFIG.with(|cell| { *cell.borrow_mut() = None; });
    unsafe { CoUninitialize(); }
    config::log_debug("QuickSwitch: message loop exited.");
}

// ---------------------------------------------------------------------------
// WinEvent callback
// ---------------------------------------------------------------------------

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _event_time: u32,
) {
    let enabled = SHARED_CONFIG.with(|cell| {
        cell.borrow().as_ref().is_some_and(|c| c.load().quick_switch_enabled)
    });
    if !enabled {
        FOCUS_STATE.with(|s| s.borrow_mut().last_explorer_hwnd = None);
        return;
    }

    let fg = unsafe { GetForegroundWindow() };
    if fg.0.is_null() { return; }

    let class = get_class_name(fg);

    if class == CABINET_CLASS {
        // Store the HWND — do NOT read the path yet. The user may still be
        // navigating inside Explorer; the path will be read lazily when a
        // dialog actually appears.
        config::log_debug(&format!("QuickSwitch: Explorer focused (hwnd={fg:?})"));
        FOCUS_STATE.with(|s| { s.borrow_mut().last_explorer_hwnd = Some(fg); });
    } else if class == DIALOG_CLASS {
        // Read the Explorer path NOW — at the last possible moment, so it
        // always reflects what was displayed when the user switched away.
        let maybe_hwnd = FOCUS_STATE.with(|s| s.borrow().last_explorer_hwnd);
        if let Some(explorer_hwnd) = maybe_hwnd {
            // Clear state immediately so a second dialog focus doesn't re-trigger.
            FOCUS_STATE.with(|s| s.borrow_mut().last_explorer_hwnd = None);
            match get_explorer_path(explorer_hwnd) {
                Some(path) => {
                    config::log_debug(&format!(
                        "QuickSwitch: dialog detected, navigating to '{path}'"
                    ));
                    navigate_dialog(fg, &path);
                }
                None => {
                    config::log_debug(
                        "QuickSwitch: dialog detected but path extraction failed — skipping"
                    );
                }
            }
        }
    } else if is_passthrough_class(&class) {
        // Transient system/shell chrome — ignore, do not break the sequence.
        config::log_debug(&format!("QuickSwitch: passthrough window '{class}' — state preserved"));
    } else {
        // Genuine user application window — reset state (strict sequence).
        config::log_debug(&format!("QuickSwitch: intervening window '{class}' — state cleared"));
        FOCUS_STATE.with(|s| s.borrow_mut().last_explorer_hwnd = None);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut buf) };
    if len <= 0 { return String::new(); }
    String::from_utf16_lossy(&buf[..len as usize])
}

fn get_explorer_path(target_hwnd: HWND) -> Option<String> {
    use windows::Win32::UI::Shell::{
        IShellWindows, ShellWindows, IWebBrowserApp,
        IShellBrowser, IFolderView, IPersistFolder2, SHGetPathFromIDListW,
    };
    use windows::Win32::UI::WindowsAndMessaging::IsChild;
    use windows::core::{Interface, VARIANT, GUID};

    // SID_STopLevelBrowser — service ID used with IServiceProvider::QueryService
    // to retrieve the IShellBrowser for a given IShellWindows entry.
    // GUID: {4C96BE40-915C-11CF-99D3-00AA004AE837}
    const SID_STOP_LEVEL_BROWSER: GUID = GUID::from_values(
        0x4C96_BE40, 0x915C, 0x11CF,
        [0x99, 0xD3, 0x00, 0xAA, 0x00, 0x4A, 0xE8, 0x37],
    );

    unsafe {
        // IShellWindows is a properly marshaled COM server — cross-process safe.
        // Each tab in Windows 11 tabbed Explorer appears as a separate entry,
        // all sharing the same top-level CabinetWClass HWND.
        let shell_windows: IShellWindows =
            CoCreateInstance(&ShellWindows, None, CLSCTX_ALL).ok()?;
        let count = shell_windows.Count().unwrap_or(0);
        config::log_debug(&format!("QuickSwitch: IShellWindows count={count}"));

        // Windows 11 tabbed Explorer: the active tab's ShellTabWindowClass
        // is at the top of the Z-order among its siblings.  find_child_class
        // returns the first Z-order match, i.e. the active tab.
        // On Windows 10 / non-tabbed Explorer this returns None (no tabs).
        let active_tab_hwnd = find_child_class(target_hwnd, "ShellTabWindowClass");
        config::log_debug(&format!("QuickSwitch: active_tab_hwnd={active_tab_hwnd:?}"));

        for i in 0..count {
            let idx = VARIANT::from(i);
            let disp = match shell_windows.Item(&idx) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let browser: IWebBrowserApp = match disp.cast() {
                Ok(b) => b,
                Err(_) => continue,
            };

            // Filter to entries that belong to our target Explorer window.
            let item_hwnd_raw = match browser.HWND() {
                Ok(h) => h,
                Err(_) => continue,
            };
            let item_hwnd = HWND(item_hwnd_raw.0 as *mut _);
            if item_hwnd != target_hwnd { continue; }

            // Identify the active tab via IServiceProvider → IShellBrowser.
            // QI: IWebBrowserApp → IServiceProvider → QueryService → IShellBrowser
            // This is the documented cross-process approach; COM handles marshaling.
            let shell_browser: IShellBrowser = match browser
                .cast::<IServiceProvider>()
                .and_then(|sp| sp.QueryService(&SID_STOP_LEVEL_BROWSER))
            {
                Ok(sb) => sb,
                Err(e) => {
                    config::log_debug(&format!(
                        "QuickSwitch: IServiceProvider QI failed for entry {i}: {e}"
                    ));
                    continue;
                }
            };

            // QueryActiveShellView gives the view for the currently active tab.
            let shell_view = match shell_browser.QueryActiveShellView() {
                Ok(sv) => sv,
                Err(_) => continue,
            };

            // Get the view's HWND (SHELLDLL_DefView or similar).
            let view_hwnd = match shell_view.GetWindow() {
                Ok(h) => h,
                Err(_) => continue,
            };

            // Windows 11 tab filtering: each tab's SHELLDLL_DefView is a
            // descendant of that tab's ShellTabWindowClass.  IsChild returns
            // true only when view_hwnd lives inside the active tab's subtree.
            if let Some(active_tab) = active_tab_hwnd
                && !IsChild(active_tab, view_hwnd).as_bool() {
                    config::log_debug(&format!(
                        "QuickSwitch: entry {i} view not in active tab — background tab"
                    ));
                    continue;
                }

            // Active tab confirmed. Extract path via IFolderView → PIDL.
            // Using PIDL + SHGetPathFromIDListW is locale-agnostic and handles
            // all path types (including non-ASCII), unlike URL percent-decoding.
            let folder_view: IFolderView = match shell_view.cast() {
                Ok(fv) => fv,
                Err(e) => {
                    config::log_debug(&format!("QuickSwitch: IShellView→IFolderView QI failed: {e}"));
                    continue;
                }
            };

            let persist: IPersistFolder2 = match folder_view.GetFolder() {
                Ok(p) => p,
                Err(e) => {
                    config::log_debug(&format!("QuickSwitch: GetFolder failed: {e}"));
                    continue;
                }
            };

            let pidl = match persist.GetCurFolder() {
                Ok(p) => p,
                Err(e) => {
                    config::log_debug(&format!("QuickSwitch: GetCurFolder failed: {e}"));
                    continue;
                }
            };

            // SHGetPathFromIDListW requires exactly &mut [u16; 260].
            let mut path_buf = [0u16; 260];
            if !SHGetPathFromIDListW(pidl, &mut path_buf).as_bool() {
                config::log_debug("QuickSwitch: SHGetPathFromIDListW failed (virtual folder?)");
                continue;
            }

            let len = path_buf.iter().position(|&c| c == 0).unwrap_or(260);
            let raw_path = String::from_utf16_lossy(&path_buf[..len]);
            let path = ensure_trailing_backslash(raw_path);
            config::log_debug(&format!("QuickSwitch: active tab path = '{path}'"));
            return Some(path);
        }

        config::log_debug("QuickSwitch: no active Explorer tab found for target hwnd");
        None
    }
}

fn navigate_dialog(dialog_hwnd: HWND, path: &str) {
    if try_navigate_via_edit(dialog_hwnd, path) {
        config::log_debug("QuickSwitch: navigation via Edit control succeeded");
        return;
    }
    config::log_debug("QuickSwitch: Edit control not found, trying Ctrl+L fallback");
    navigate_via_ctrl_l(dialog_hwnd, path);
}

fn try_navigate_via_edit(dialog_hwnd: HWND, path: &str) -> bool {
    let edit_hwnd = match find_child_class(dialog_hwnd, "Edit") {
        Some(h) => h,
        None => return false,
    };

    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        SendMessageW(edit_hwnd, WM_SETTEXT, WPARAM(0), LPARAM(wide.as_ptr() as isize));
    }

    std::thread::sleep(Duration::from_millis(50));

    let inputs = [
        Replacer::make_vk_input(0x0D, KEYBD_EVENT_FLAGS(0)),
        Replacer::make_vk_input(0x0D, KEYEVENTF_KEYUP),
    ];
    send_inputs(&inputs);
    true
}

fn find_child_class(parent: HWND, class: &str) -> Option<HWND> {
    unsafe {
        let class_wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();

        // Direct child search (fast path).
        if let Ok(h) = FindWindowExW(parent, None, PCWSTR(class_wide.as_ptr()), PCWSTR::null())
            && !h.is_invalid() { return Some(h); }

        // DFS through child windows (recursive depth-first traversal).
        let first = match GetWindow(parent, GW_CHILD) {
            Ok(h) if !h.is_invalid() => h,
            _ => return None,
        };
        let mut current = first;
        loop {
            if let Some(found) = find_child_class(current, class) {
                return Some(found);
            }
            current = match GetWindow(current, GW_HWNDNEXT) {
                Ok(h) if !h.is_invalid() => h,
                _ => break,
            };
        }
        None
    }
}

fn navigate_via_ctrl_l(dialog_hwnd: HWND, path: &str) {
    unsafe { let _ = SetForegroundWindow(dialog_hwnd); }
    std::thread::sleep(Duration::from_millis(50));

    let saved = Replacer::backup_all_clipboard_formats();

    if !Replacer::set_clipboard_text(path) {
        config::log_debug("QuickSwitch: Ctrl+L fallback -- clipboard write failed");
        return;
    }

    let inputs = [
        Replacer::make_vk_input(0xA2, KEYBD_EVENT_FLAGS(0)), // VK_LCONTROL down
        Replacer::make_vk_input(0x4C, KEYBD_EVENT_FLAGS(0)), // L down
        Replacer::make_vk_input(0x4C, KEYEVENTF_KEYUP),      // L up
        Replacer::make_vk_input(0xA2, KEYEVENTF_KEYUP),      // VK_LCONTROL up
    ];
    send_inputs(&inputs);

    std::thread::sleep(Duration::from_millis(50));
    Replacer::send_ctrl_v();
    std::thread::sleep(Duration::from_millis(100));

    let inputs = [
        Replacer::make_vk_input(0x0D, KEYBD_EVENT_FLAGS(0)),
        Replacer::make_vk_input(0x0D, KEYEVENTF_KEYUP),
    ];
    send_inputs(&inputs);

    std::thread::sleep(Duration::from_millis(150));
    Replacer::restore_clipboard_formats(saved);
    config::log_debug("QuickSwitch: Ctrl+L fallback completed");
}

/// Checks if a window class belongs to the known transient shell chrome / passthrough list.
#[inline]
#[must_use]
pub fn is_passthrough_class(class: &str) -> bool {
    PASSTHROUGH_CLASSES.contains(&class)
}

/// Ensures a directory path string terminates with a trailing backslash for dialog injection.
#[inline]
#[must_use]
pub fn ensure_trailing_backslash(mut path: String) -> String {
    if !path.is_empty() && !path.ends_with('\\') {
        path.push('\\');
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_state_initial_is_none() {
        let state = FocusState::new();
        assert!(state.last_explorer_hwnd.is_none());
    }

    #[test]
    fn test_passthrough_classes_contains_transient_windows() {
        assert!(is_passthrough_class("Shell_TrayWnd"));
        assert!(is_passthrough_class("Shell_SecondaryTrayWnd"));
        assert!(is_passthrough_class("Progman"));
        assert!(is_passthrough_class("WorkerW"));
        assert!(is_passthrough_class("XamlExplorerHostIslandWindow"));
        assert!(is_passthrough_class("TaskListThumbnailWnd"));
        assert!(is_passthrough_class("MultitaskingViewFrame"));
        assert!(is_passthrough_class("ForegroundStaging"));
    }

    #[test]
    fn test_passthrough_classes_rejects_actual_apps() {
        assert!(!is_passthrough_class("CabinetWClass"));
        assert!(!is_passthrough_class("#32770"));
        assert!(!is_passthrough_class("Notepad"));
        assert!(!is_passthrough_class("Chrome_WidgetWin_1"));
        assert!(!is_passthrough_class(""));
    }

    #[test]
    fn test_ensure_trailing_backslash() {
        assert_eq!(ensure_trailing_backslash(r"C:\Users\User\Documents".to_string()), r"C:\Users\User\Documents\");
        assert_eq!(ensure_trailing_backslash(r"C:\Users\User\Documents\".to_string()), r"C:\Users\User\Documents\");
        assert_eq!(ensure_trailing_backslash(r"C:\Påske\Båd".to_string()), r"C:\Påske\Båd\");
        assert_eq!(ensure_trailing_backslash("".to_string()), "");
    }
}

