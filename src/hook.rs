/// The Windows low-level keyboard hook manager.
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;

use arc_swap::ArcSwap;
use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyboardState, RegisterHotKey, ToUnicode, UnregisterHotKey,
    VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_LCONTROL, VK_RCONTROL,
    VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_LMENU, VK_LSHIFT,
    VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SHIFT,
    VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, PostThreadMessageW,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, MSG, WH_KEYBOARD_LL,
    WM_HOTKEY, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

use crate::buffer::KeyBuffer;
use crate::config::{AppConfig, ExpansionMode, HotkeyConfig};
use crate::replacer::Replacer;

const EXPANDER_TAG: usize = 0x5245_5850; // ASCII 'REXP'
const HOTKEY_ID: i32 = 1;
pub const WM_REHOOK: u32 = 0x0400 + 100; // WM_USER + 100 - signal to re-register hotkey

/// Zero-allocation debug logging macro for the keyboard hook hot path.
///
/// Unlike calling `crate::config::log_debug(&format!(...))` directly, this
/// macro checks `is_debug_logging_enabled()` — a single relaxed `AtomicBool`
/// read — **before** evaluating the `format!()` expression. When debug logging
/// is disabled (the default in release builds without `DOTXPANDER_LOG`),
/// the format string is never evaluated and no heap allocation occurs.
///
/// # Example
/// ```ignore
/// debug_log!("Hook: push '{}' (U+{:04X})", c, c as u32);
/// ```
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if crate::config::is_debug_logging_enabled() {
            crate::config::log_debug(&format!($($arg)*));
        }
    };
}

static RECORDING_HOTKEY: AtomicBool = AtomicBool::new(false);
static PAUSED: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Lock-free callback statics (C4 fix)
//
// `Arc<dyn Fn + Send + Sync>` is a sized fat pointer, so we can use
// `ArcSwap<Option<Arc<dyn Fn + Send + Sync>>>` for wait-free reads inside
// the LL-keyboard-hook callback. The previous `Mutex<Option<Box<dyn Fn>>>`
// could block the hook thread if the UI thread was simultaneously writing —
// a latency spike that Win32 counts against the hook timeout.
//
// We wrap each `ArcSwap` in a `OnceLock` for safe `static` initialization
// (ArcSwap requires a runtime-constructed initial `Arc::new(None)` value).
//
// Write path (UI thread, startup-only):  `store()` — wait-free, no spin.
// Read path  (hook thread, every key):   `load()` — wait-free, no contention.
// ---------------------------------------------------------------------------
type HotkeyCaptureCallback = Arc<dyn Fn(HotkeyConfig) + Send + Sync + 'static>;
type ModDisplayCallback    = Arc<dyn Fn(String)       + Send + Sync + 'static>;

static CAPTURE_CALLBACK:     OnceLock<ArcSwap<Option<HotkeyCaptureCallback>>> = OnceLock::new();
static MOD_DISPLAY_CALLBACK: OnceLock<ArcSwap<Option<ModDisplayCallback>>>    = OnceLock::new();

/// Returns the `ArcSwap` guard for the hotkey-captured callback, initialising
/// it on first use.
fn capture_callback() -> &'static ArcSwap<Option<HotkeyCaptureCallback>> {
    CAPTURE_CALLBACK.get_or_init(|| ArcSwap::from(Arc::new(None)))
}

/// Returns the `ArcSwap` guard for the modifier-display callback, initialising
/// it on first use.
fn mod_display_callback() -> &'static ArcSwap<Option<ModDisplayCallback>> {
    MOD_DISPLAY_CALLBACK.get_or_init(|| ArcSwap::from(Arc::new(None)))
}

/// Registers a wait-free callback invoked when a hotkey is fully recorded.
///
/// Safe to call from any thread. The hook thread reads via a lock-free
/// `ArcSwap::load()` so there is no contention with this store.
pub fn set_on_hotkey_captured<F: Fn(HotkeyConfig) + Send + Sync + 'static>(f: F) {
    capture_callback().store(Arc::new(Some(Arc::new(f) as HotkeyCaptureCallback)));
}

/// Registers a wait-free callback invoked when modifier keys are pressed
/// during hotkey recording, allowing the UI to show an intermediate display.
pub fn set_on_mod_display<F: Fn(String) + Send + Sync + 'static>(f: F) {
    mod_display_callback().store(Arc::new(Some(Arc::new(f) as ModDisplayCallback)));
}

/// Enables or disables global hotkey capture mode.
pub fn set_recording_hotkey(val: bool) {
    RECORDING_HOTKEY.store(val, Ordering::SeqCst);
}

/// Returns true if hotkey recording mode is active.
#[allow(dead_code)]
pub fn is_recording_hotkey() -> bool {
    RECORDING_HOTKEY.load(Ordering::SeqCst)
}

/// Sets whether text expansion is paused.
pub fn set_paused(val: bool) {
    PAUSED.store(val, Ordering::SeqCst);
    crate::config::log_debug(&format!("Hook: pause state changed to {val}"));
}

/// Returns true if text expansion is currently paused.
pub fn is_paused() -> bool {
    PAUSED.load(Ordering::Relaxed)
}

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };

    // MUST-3 / NEXT-2: Re-entrancy guard explanation
    //
    // When we call SendInput() from inside the keyboard hook callback to inject
    // backspaces or Ctrl+V, the Win32 low-level keyboard hook (WH_KEYBOARD_LL)
    // is SYNCHRONOUSLY re-entered on the SAME THREAD before SendInput returns.
    // This is documented Win32 behaviour: low-level hooks are called inline in the
    // thread that installed them, regardless of whether that thread is already
    // inside a hook callback.
    //
    // A thread-local AtomicBool is the correct tool here:
    //   - Zero overhead: no heap allocation, no cross-thread synchronisation.
    //   - No false sharing: each thread has its own independent copy.
    //   - Accessible from the hook proc without any pointer indirection.
    //
    // An Arc<Mutex<bool>> would be WRONG: the hook thread already holds execution
    // context in the hook proc, so a Mutex::lock() on the same thread would either
    // deadlock (non-reentrant mutex) or silently succeed and double-expand
    // (reentrant mutex). The thread_local pattern avoids both failure modes.
    static INHIBIT_HOOK: AtomicBool = const { AtomicBool::new(false) };
}

/// Called by Replacer before/after `SendInput` to suppress hook re-entrancy.
pub fn set_inhibit(val: bool) {
    INHIBIT_HOOK.with(|f| f.store(val, Ordering::Relaxed));
}

/// Returns whether the hook is currently inhibited on this thread.
#[allow(dead_code)]
#[must_use]
pub fn is_inhibited() -> bool {
    INHIBIT_HOOK.with(|f| f.load(Ordering::Relaxed))
}

struct HookState {
    buffer: KeyBuffer,
    config: Arc<ArcSwap<AppConfig>>,
    buffer_debug: Arc<Mutex<String>>,
}

/// Manages the low-level keyboard hook thread.
pub struct HookManager {
    thread_id: u32,
    join_handle: Option<std::thread::JoinHandle<()>>,
    buffer_debug: Arc<Mutex<String>>,
}

impl HookManager {
    /// Starts the hook thread and installs the low-level keyboard hook.
    ///
    /// Returns an error string if the hook could not be installed. This happens most
    /// commonly when an overly aggressive antivirus blocks `SetWindowsHookExW`, or
    /// when the process lacks the required privilege level.
    pub fn start(config: Arc<ArcSwap<AppConfig>>) -> Result<Self, String> {
        let buffer_debug = Arc::new(Mutex::new(String::new()));
        let buffer_debug_clone = buffer_debug.clone();
        // This channel carries either the hook thread ID (success) or an error string
        // (failure) from the spawned thread back to the caller.
        let (tx, rx) = mpsc::channel::<Result<u32, String>>();

        let join_handle = thread::spawn(move || {
            let thread_id = unsafe { GetCurrentThreadId() };

            unsafe {
                let hook_result = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(low_level_keyboard_proc),
                    HMODULE::default(),
                    0,
                );

                // MUST-3: Instead of panicking with .expect(), we send the error back
                // to the caller via the channel so it can show a proper UI error dialog.
                let hook_handle = match hook_result {
                    Ok(h) => h,
                    Err(e) => {
                        let _ = tx.send(Err(format!(
                            "SetWindowsHookExW failed: {e}. Your antivirus may be blocking keyboard hooks."
                        )));
                        return;
                    }
                };

                // Signal success: send the hook thread ID back to the caller.
                let _ = tx.send(Ok(thread_id));
                
                // Read initial config for buffer size and hotkey
                let initial_config = config.load();
                let buffer_size = initial_config.buffer_size;
                
                HOOK_STATE.with(|state| {
                    *state.borrow_mut() = Some(HookState {
                        buffer: KeyBuffer::new(buffer_size),
                        config: config.clone(),
                        buffer_debug: buffer_debug_clone,
                    });
                });
                
                // Register initial hotkey
                let hk = &initial_config.hotkey;
                let modifiers = windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS(hk.modifiers);
                match RegisterHotKey(None, HOTKEY_ID, modifiers, hk.virtual_key) {
                    Ok(()) => debug_log!(
                        "Hook: initial RegisterHotKey OK (mods={}, vk=0x{:X})",
                        hk.modifiers, hk.virtual_key
                    ),
                    Err(e) => debug_log!(
                        "Hook: initial RegisterHotKey FAILED (mods={}, vk=0x{:X}): {}",
                        hk.modifiers, hk.virtual_key, e
                    ),
                }
                
                let mut msg = MSG::default();
                while GetMessageW(&raw mut msg, None, 0, 0).into() {
                    if msg.message == WM_QUIT {
                        break;
                    } else if msg.message == WM_REHOOK {
                        // Re-register hotkey from updated ArcSwap config
                        let _ = UnregisterHotKey(None, HOTKEY_ID);
                        let new_config = config.load();
                        let hk = &new_config.hotkey;
                        let modifiers = windows::Win32::UI::Input::KeyboardAndMouse::HOT_KEY_MODIFIERS(hk.modifiers);
                        match RegisterHotKey(None, HOTKEY_ID, modifiers, hk.virtual_key) {
                            Ok(()) => debug_log!(
                                "Hook: rehook RegisterHotKey OK (mods={}, vk=0x{:X})",
                                hk.modifiers, hk.virtual_key
                            ),
                            Err(e) => debug_log!(
                                "Hook: rehook RegisterHotKey FAILED (mods={}, vk=0x{:X}): {}",
                                hk.modifiers, hk.virtual_key, e
                            ),
                        }

                        // Resize in-memory buffer if buffer_size changed
                        HOOK_STATE.with(|state| {
                            if let Some(st) = state.borrow_mut().as_mut() {
                                if st.buffer.capacity() != new_config.buffer_size {
                                    debug_log!(
                                        "Hook: resizing buffer from {} to {}",
                                        st.buffer.capacity(),
                                        new_config.buffer_size
                                    );
                                    st.buffer.resize(new_config.buffer_size);
                                }
                            }
                        });
                    } else if msg.message == crate::case_changer::WM_SHOW_CASE_MENU {
                        // Show case-changer menu from the message loop (NOT from inside the hook
                        // callback). TrackPopupMenu runs an internal message pump, so calling it
                        // from the hook proc would starve the LL-hook timer.
                        HOOK_STATE.with(|state| {
                            if let Some(st) = state.borrow().as_ref() {
                                let conf = st.config.load();
                                let delay = conf.clipboard_restore_delay_ms;
                                crate::case_changer::show_case_menu(&st.config, delay);
                            }
                        });
                    } else if msg.message == WM_HOTKEY {
                        if is_paused() {
                            continue;
                        }
                        let mut hotkey_replacement: Option<(usize, String)> = None;
                        HOOK_STATE.with(|state| {
                            if let Some(st) = state.borrow_mut().as_mut() {
                                let conf = st.config.load();
                                let buf_content = st.buffer.content();
                                debug_log!(
                                    "Hook WM_HOTKEY: buffer='{}', checking {} snippets",
                                    buf_content, conf.snippets.len()
                                );
                                for snippet in &conf.snippets {
                                    if snippet.mode == ExpansionMode::Hotkey {
                                        let matches = st.buffer.ends_with(&snippet.trigger);
                                        debug_log!(
                                            "  trigger='{}', len={}, ends_with={}",
                                            snippet.trigger, snippet.trigger.len(), matches
                                        );
                                        if matches {
                                            hotkey_replacement = Some((
                                                snippet.trigger.chars().count(),
                                                snippet.replacement.clone(),
                                            ));
                                            st.buffer.clear();
                                            update_buffer_debug(st);
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                        if let Some((trigger_len, replacement)) = hotkey_replacement {
                            debug_log!(
                                "Hook: replacing trigger_len={} with '{}'", trigger_len, replacement
                            );
                            let delay_ms = {
                                let conf = config.load();
                                conf.clipboard_restore_delay_ms
                            };
                            Replacer::replace_hotkey(trigger_len, &replacement, delay_ms);
                        }
                    }
                    let _ = TranslateMessage(&raw const msg);
                    DispatchMessageW(&raw const msg);
                }
                
                // Do NOT .expect() here — a failed unhook is non-fatal and must
                // not panic the hook thread during shutdown. Log and continue.
                if let Err(e) = UnhookWindowsHookEx(hook_handle) {
                    crate::config::log_debug(&format!("UnhookWindowsHookEx failed: {e}"));
                }
                let _ = UnregisterHotKey(None, HOTKEY_ID);
            }
        });

        let thread_id = rx.recv().expect("Failed to get hook thread id")?;

        Ok(HookManager {
            thread_id,
            join_handle: Some(join_handle),
            buffer_debug,
        })
    }

    /// Returns the thread ID for IPC (`PostThreadMessageW`).
    #[must_use]
    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }

    /// Returns a clone of the buffer debug string for UI display.
    #[must_use]
    pub fn buffer_debug(&self) -> Arc<Mutex<String>> {
        self.buffer_debug.clone()
    }


    /// Stops the hook thread gracefully.
    pub fn stop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Updates the shared debug string with current buffer contents.
///
/// Uses `write_content_to` to reuse the existing `String`'s capacity,
/// avoiding a heap allocation on every keystroke (C6 perf fix).
fn update_buffer_debug(st: &HookState) {
    if let Ok(mut dbg) = st.buffer_debug.try_lock() {
        st.buffer.write_content_to(&mut dbg);
    }
}

// ---------------------------------------------------------------------------
// M2: low_level_keyboard_proc handler sub-functions
//
// The main proc is decomposed into three focused handlers to keep the overall
// function readable. Each handler returns:
//   Some(LRESULT) — immediately return that result from the proc
//   None          — fall through to the next handler
// ---------------------------------------------------------------------------

/// Handles RECORDING MODE: a prior call to `set_recording_hotkey(true)` puts the
/// hook into capture mode. All keystrokes are intercepted until a non-modifier key
/// is pressed (which completes the capture) or Escape (which cancels it).
///
/// Returns `Some(LRESULT(1))` to consume the key, or `None` to fall through.
#[allow(clippy::fn_params_excessive_bools)]
unsafe fn handle_hotkey_recording(
    kbd: &KBDLLHOOKSTRUCT,
    ctrl: bool,
    alt: bool,
    shift: bool,
    win: bool,
    current_modifiers: u32,
) -> Option<LRESULT> {
    if !RECORDING_HOTKEY.load(Ordering::Relaxed) {
        return None;
    }

    let is_modifier_key = matches!(
        kbd.vkCode,
        0x10 | 0xA0 | 0xA1  // VK_SHIFT, VK_LSHIFT, VK_RSHIFT
        | 0x11 | 0xA2 | 0xA3  // VK_CONTROL, VK_LCONTROL, VK_RCONTROL
        | 0x12 | 0xA4 | 0xA5  // VK_MENU, VK_LMENU, VK_RMENU
        | 0x5B | 0x5C          // VK_LWIN, VK_RWIN
    );

    debug_log!(
        "Hook recording: vk=0x{:X}, mods={}, is_mod={}",
        kbd.vkCode, current_modifiers, is_modifier_key
    );

    if is_modifier_key {
        // Show intermediate modifier display (e.g. "CTRL + ALT + ...").
        // C4 fix: wait-free load() instead of Mutex::lock().
        let mut parts = Vec::new();
        if ctrl  { parts.push("CTRL"); }
        if alt   { parts.push("ALT"); }
        if shift { parts.push("SHIFT"); }
        if win   { parts.push("WIN"); }
        if !parts.is_empty() {
            let mod_str = parts.join(" + ");
            let guard = mod_display_callback().load();
            if let Some(cb) = guard.as_ref() {
                cb(mod_str);
            }
        }
        // Consume modifier press so Alt doesn't activate the window menu.
        return Some(LRESULT(1));
    }

    if kbd.vkCode == 0x1B {
        // Escape: cancel recording without capturing anything.
        set_recording_hotkey(false);
        return Some(LRESULT(1));
    }

    // Non-modifier key pressed — capture the complete hotkey combination.
    let recorded = HotkeyConfig {
        modifiers: current_modifiers,
        virtual_key: kbd.vkCode,
    };
    debug_log!("Hook captured hotkey: {:?}", recorded);
    RECORDING_HOTKEY.store(false, Ordering::SeqCst);

    // C4 fix: wait-free load() instead of Mutex::lock().
    let guard = capture_callback().load();
    if let Some(cb) = guard.as_ref() {
        cb(recorded);
    }

    Some(LRESULT(1)) // Consume the key event
}

/// Handles CONFIGURED SNIPPET HOTKEY EXPANSION: checks whether the current key
/// press matches the user-configured hotkey and, if so, scans the buffer for a
/// matching `Hotkey`-mode snippet.
///
/// Returns `Some(LRESULT(1))` if the hotkey fired (consume), or `None` to fall
/// through to normal keystroke handling.
unsafe fn handle_hotkey_expansion(
    kbd: &KBDLLHOOKSTRUCT,
    current_modifiers: u32,
) -> Option<LRESULT> {
    let mut is_hotkey_match = false;
    let mut hotkey_replacement: Option<(usize, String)> = None;

    HOOK_STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        if let Some(st) = state_ref.as_mut() {
            let conf = st.config.load();
            let hk = &conf.hotkey;
            if conf.snippet_hotkey_enabled
                && current_modifiers == hk.modifiers
                && kbd.vkCode == hk.virtual_key
            {
                is_hotkey_match = true;
                for snippet in &conf.snippets {
                    if snippet.mode == ExpansionMode::Hotkey
                        && st.buffer.ends_with(&snippet.trigger)
                    {
                        hotkey_replacement = Some((
                            snippet.trigger.chars().count(),
                            snippet.replacement.clone(),
                        ));
                        st.buffer.clear();
                        update_buffer_debug(st);
                        break;
                    }
                }
            }
        }
    });

    if !is_hotkey_match {
        return None;
    }

    if let Some((trigger_len, replacement)) = hotkey_replacement {
        let delay_ms = HOOK_STATE.with(|state| {
            state.borrow().as_ref()
                .map_or(150, |st| st.config.load().clipboard_restore_delay_ms)
        });
        Replacer::replace_hotkey(trigger_len, &replacement, delay_ms);
    }
    // Consume the hotkey keystroke so it doesn't leak into the target app.
    Some(LRESULT(1))
}

/// Handles NORMAL KEYSTROKE PROCESSING: pushes printable characters into the
/// buffer, handles Backspace/navigation/Ctrl-key buffer resets, and checks for
/// `Immediate`-mode snippet triggers after every push.
///
/// Returns `LRESULT(1)` if a trigger fired and we must consume the keystroke,
/// otherwise delegates to `CallNextHookEx`.
#[allow(clippy::fn_params_excessive_bools)]
unsafe fn handle_normal_keystroke(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
    kbd: &KBDLLHOOKSTRUCT,
    ctrl: bool,
    alt: bool,
) -> LRESULT {
    let mut consume = false;
    let mut immediate_replacement: Option<(usize, String)> = None;

    HOOK_STATE.with(|state| {
        let mut state_ref = state.borrow_mut();
        if let Some(st) = state_ref.as_mut() {
            let vk = VIRTUAL_KEY(kbd.vkCode as u16);

            // Skip modifier-only keys entirely — they shouldn't affect the buffer.
            // This prevents the buffer from being cleared when the user presses Ctrl
            // as the first key of a hotkey combination like Ctrl+Alt+Shift+O.
            match vk {
                VK_SHIFT | VK_CONTROL | VK_MENU | VK_LSHIFT | VK_RSHIFT
                | VK_LCONTROL | VK_RCONTROL | VK_LMENU | VK_RMENU | VK_CAPITAL => {
                    return; // Modifier-only: ignore, don't touch the buffer.
                }
                _ => {}
            }

            // If Ctrl is held (with a non-modifier key), clear the buffer for editing
            // commands like Ctrl+A, Ctrl+C, Ctrl+Z etc.
            // But NOT if Alt is also held — that combination might be a configured hotkey.
            if ctrl && !alt {
                debug_log!("Hook: Ctrl+key clears buffer. vk=0x{:X}", kbd.vkCode);
                st.buffer.clear();
                update_buffer_debug(st);
                return;
            }

            match vk {
                VK_BACK => {
                    st.buffer.pop();
                    update_buffer_debug(st);
                }
                VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END
                | VK_DELETE | VK_ESCAPE | VK_TAB | VK_RETURN | VK_PRIOR | VK_NEXT => {
                    st.buffer.clear();
                    update_buffer_debug(st);
                }
                _ => {
                    let mut keyboard_state = [0u8; 256];
                    unsafe { let _ = GetKeyboardState(&mut keyboard_state); }

                    let mut char_buf = [0u16; 4];
                    // NEXT-4: ToUnicode return values:
                    //   > 0  : UTF-16 code units written; 1 for BMP, 2 for surrogates.
                    //   = 0  : no character produced (dead key consumed).
                    //   < 0  : dead key + base character — treat as > 0.
                    let result = unsafe {
                        ToUnicode(
                            kbd.vkCode,
                            kbd.scanCode,
                            Some(&keyboard_state),
                            &mut char_buf,
                            4, // TOUNICODE_FLAG_MENU: don't modify dead-key state
                        )
                    };

                    // MUST-2: Handle result > 0 (not just == 1) to correctly track
                    // emoji and supplementary-plane characters (surrogate pair = 2).
                    if result != 0 {
                        let units_written = result.unsigned_abs() as usize;
                        let slice = &char_buf[..units_written.min(char_buf.len())];
                        for c in char::decode_utf16(slice.iter().copied()).flatten() {
                            if !c.is_control() {
                                // C5 fix: debug_log! checks is_debug_logging_enabled()
                                // before evaluating format! — zero allocation when off.
                                debug_log!(
                                    "Hook: push '{}' (U+{:04X}), buffer='{}'",
                                    c, c as u32, st.buffer.content()
                                );
                                st.buffer.push(c);
                                update_buffer_debug(st);

                                let conf = st.config.load();
                                for snippet in &conf.snippets {
                                    if snippet.mode == ExpansionMode::Immediate
                                        && st.buffer.ends_with(&snippet.trigger)
                                    {
                                        immediate_replacement = Some((
                                            snippet.trigger.chars().count(),
                                            snippet.replacement.clone(),
                                        ));
                                        st.buffer.clear();
                                        update_buffer_debug(st);
                                        consume = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    if let Some((trigger_len, replacement)) = immediate_replacement {
        let delay_ms = HOOK_STATE.with(|state| {
            state.borrow().as_ref()
                .map_or(150, |st| st.config.load().clipboard_restore_delay_ms)
        });
        Replacer::replace_immediate(trigger_len, &replacement, delay_ms);
    }

    if consume {
        return LRESULT(1);
    }

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

/// The Win32 low-level keyboard hook procedure.
///
/// Dispatches each keystroke through three sequential handlers:
/// 1. [`handle_hotkey_recording`] — intercepts all keys while in capture mode.
/// 2. [`handle_hotkey_expansion`] — checks for the configured snippet hotkey.
/// 3. [`handle_normal_keystroke`] — buffer management and immediate-trigger check.
///
/// The function returns early as soon as a handler produces a result.
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // nCode < 0 means we must pass the hook without processing.
    if n_code < 0 {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    // PRIMARY re-entrancy guard: our own SendInput calls.
    if INHIBIT_HOOK.with(|f| f.load(Ordering::Relaxed)) {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    // SAFETY: l_param points to a valid KBDLLHOOKSTRUCT when nCode >= 0.
    let kbd = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };

    // SECONDARY guard: dwExtraInfo tag (belt-and-suspenders).
    // Our synthetic keystrokes are tagged — let them pass through to the app.
    if kbd.dwExtraInfo == EXPANDER_TAG {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    let message = w_param.0 as u32;
    if message != WM_KEYDOWN && message != WM_SYSKEYDOWN {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    // Determine the current modifier state, including the key being pressed now.
    let mut ctrl  = (unsafe { GetAsyncKeyState(i32::from(VK_CONTROL.0)) } as u16 & 0x8000) != 0;
    let mut alt   = (unsafe { GetAsyncKeyState(i32::from(VK_MENU.0)) } as u16 & 0x8000) != 0;
    let mut shift = (unsafe { GetAsyncKeyState(i32::from(VK_SHIFT.0)) } as u16 & 0x8000) != 0;
    let mut win   = (unsafe { GetAsyncKeyState(0x5B) } as u16 & 0x8000 != 0)
                 || (unsafe { GetAsyncKeyState(0x5C) } as u16 & 0x8000 != 0);

    // Account for the vkCode itself when the pressed key IS a modifier.
    match kbd.vkCode {
        0x11 | 0xA2 | 0xA3 => ctrl  = true,
        0x12 | 0xA4 | 0xA5 => alt   = true,
        0x10 | 0xA0 | 0xA1 => shift = true,
        0x5B | 0x5C         => win   = true,
        _ => {}
    }

    let mut current_modifiers = 0u32;
    if alt   { current_modifiers |= 1; }
    if ctrl  { current_modifiers |= 2; }
    if shift { current_modifiers |= 4; }
    if win   { current_modifiers |= 8; }

    // --- Handler 1: hotkey recording mode -----------------------------------
    if let Some(result) = unsafe {
        handle_hotkey_recording(&kbd, ctrl, alt, shift, win, current_modifiers)
    } {
        return result;
    }

    // --- Pause guard: pass through without recording or expanding -----------
    if is_paused() {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    // --- Case changer: configurable hotkey (default Ctrl+CapsLock) ----------
    // Read both enabled flag AND hotkey in one ArcSwap load to avoid TOCTOU.
    // Must be checked BEFORE the modifier skip and BEFORE the buffer-clear path.
    let (case_changer_enabled, cc_mods, cc_vk) = HOOK_STATE.with(|state| {
        state.borrow().as_ref()
            .map_or((false, 2, 0x14), |st| {
                let conf = st.config.load();
                let hk = &conf.case_changer_hotkey;
                (conf.case_changer_enabled, hk.modifiers, hk.virtual_key)
            })
    });
    if case_changer_enabled && current_modifiers == cc_mods && kbd.vkCode == cc_vk {
        crate::case_changer::post_show_case_menu();
        return LRESULT(1); // Consume to suppress any key toggle side-effects.
    }

    // --- Handler 2: configured snippet hotkey expansion ---------------------
    if let Some(result) = unsafe { handle_hotkey_expansion(&kbd, current_modifiers) } {
        return result;
    }

    // --- Handler 3: normal keystroke (buffer + immediate trigger check) -----
    unsafe { handle_normal_keystroke(n_code, w_param, l_param, &kbd, ctrl, alt) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_state_toggling() {
        set_paused(false);
        assert!(!is_paused());

        set_paused(true);
        assert!(is_paused());

        set_paused(false);
        assert!(!is_paused());
    }

    #[test]
    fn test_recording_hotkey_state_toggling() {
        set_recording_hotkey(false);
        assert!(!is_recording_hotkey());

        set_recording_hotkey(true);
        assert!(is_recording_hotkey());

        set_recording_hotkey(false);
        assert!(!is_recording_hotkey());
    }

    #[test]
    fn test_inhibit_state_toggling() {
        set_inhibit(false);
        assert!(!is_inhibited());

        set_inhibit(true);
        assert!(is_inhibited());

        set_inhibit(false);
        assert!(!is_inhibited());
    }
}


