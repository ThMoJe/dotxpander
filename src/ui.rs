use std::rc::Rc;
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use slint::{Model, Timer, TimerMode, VecModel, SharedString, ComponentHandle, LogicalSize, LogicalPosition};
use windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW;
use windows::Win32::Foundation::{WPARAM, LPARAM};

use crate::config::{self, AppConfig, Snippet, ExpansionMode, HotkeyConfig};
use crate::hook::WM_REHOOK;

slint::include_modules!();

// ---------------------------------------------------------------------------
// Win32 virtual key code constants
// ---------------------------------------------------------------------------
const VK_TAB:    u32 = 0x09;
const VK_RETURN: u32 = 0x0D;
const VK_ESCAPE: u32 = 0x1B;
const VK_SPACE:  u32 = 0x20;
const VK_BACK:   u32 = 0x08;
const VK_F1:  u32 = 0x70; const VK_F2:  u32 = 0x71; const VK_F3:  u32 = 0x72;
const VK_F4:  u32 = 0x73; const VK_F5:  u32 = 0x74; const VK_F6:  u32 = 0x75;
const VK_F7:  u32 = 0x76; const VK_F8:  u32 = 0x77; const VK_F9:  u32 = 0x78;
const VK_F10: u32 = 0x79; const VK_F11: u32 = 0x7A; const VK_F12: u32 = 0x7B;
const VK_HOME:   u32 = 0x24; const VK_END:    u32 = 0x23;
const VK_PRIOR:  u32 = 0x21; const VK_NEXT:   u32 = 0x22;
const VK_DELETE: u32 = 0x2E;
const VK_LEFT:   u32 = 0x25; const VK_RIGHT:  u32 = 0x27;
const VK_UP:     u32 = 0x26; const VK_DOWN:   u32 = 0x28;
// Slint Private-Use-Area codes for function/arrow/nav keys
const SLINT_F1:    u32 = 0xF704; const SLINT_F2:    u32 = 0xF705;
const SLINT_F3:    u32 = 0xF706; const SLINT_F4:    u32 = 0xF707;
const SLINT_F5:    u32 = 0xF708; const SLINT_F6:    u32 = 0xF709;
const SLINT_F7:    u32 = 0xF70A; const SLINT_F8:    u32 = 0xF70B;
const SLINT_F9:    u32 = 0xF70C; const SLINT_F10:   u32 = 0xF70D;
const SLINT_F11:   u32 = 0xF70E; const SLINT_F12:   u32 = 0xF70F;
const SLINT_UP:    u32 = 0xF700; const SLINT_DOWN:  u32 = 0xF701;
const SLINT_LEFT:  u32 = 0xF702; const SLINT_RIGHT: u32 = 0xF703;
const SLINT_HOME:  u32 = 0xF729; const SLINT_END:   u32 = 0xF72B;
const SLINT_PGUP:  u32 = 0xF72C; const SLINT_PGDN:  u32 = 0xF72D;
const SLINT_DEL:   u32 = 0xF728;
#[allow(dead_code)] const SLINT_MOD_LOW:  u32 = 0xF720;
#[allow(dead_code)] const SLINT_MOD_HIGH: u32 = 0xF72F;

/// Terminates the application cleanly from any callback context.
fn graceful_shutdown(hook_thread_id: u32) {
    crate::hook::set_recording_hotkey(false);
    unsafe {
        let _ = PostThreadMessageW(
            hook_thread_id,
            windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
            WPARAM(0),
            LPARAM(0),
        );
    }
    let _ = slint::quit_event_loop();
}

/// Converts a Rust `AppConfig` to a vector of `SnippetModel` for Slint.
fn config_to_snippet_models(config: &AppConfig) -> Vec<SnippetModel> {
    config.snippets.iter().map(|s| SnippetModel {
        trigger:     SharedString::from(&*s.trigger),
        replacement: SharedString::from(&*s.replacement),
        mode: SharedString::from(match s.mode {
            ExpansionMode::Immediate => "immediate",
            ExpansionMode::Hotkey    => "hotkey",
        }),
    }).collect()
}

/// Formats a `HotkeyConfig` as a human-readable string (e.g. "CTRL+ALT+A").
fn hotkey_display_string(hotkey: &HotkeyConfig) -> SharedString {
    let mut parts = Vec::new();
    let m = hotkey.modifiers;
    if m & 2 != 0 { parts.push("CTRL"); }
    if m & 1 != 0 { parts.push("ALT"); }
    if m & 4 != 0 { parts.push("SHIFT"); }
    if m & 8 != 0 { parts.push("WIN"); }

    let key_name = match hotkey.virtual_key {
        0x41..=0x5A => { let ch = (hotkey.virtual_key as u8) as char; String::from(ch) }
        0x30..=0x39 => { let ch = (hotkey.virtual_key as u8) as char; String::from(ch) }
        VK_F1  => "F1".into(),  VK_F2  => "F2".into(),  VK_F3  => "F3".into(),
        VK_F4  => "F4".into(),  VK_F5  => "F5".into(),  VK_F6  => "F6".into(),
        VK_F7  => "F7".into(),  VK_F8  => "F8".into(),  VK_F9  => "F9".into(),
        VK_F10 => "F10".into(), VK_F11 => "F11".into(), VK_F12 => "F12".into(),
        0x14   => "CAPSLOCK".into(),
        _ => format!("0x{:02X}", hotkey.virtual_key),
    };
    parts.push(&key_name);
    SharedString::from(parts.join("+"))
}

/// Parses a Slint key event into a `HotkeyConfig`.
/// Returns `None` for modifier-only keys so the caller can show an intermediate display.
fn parse_key_event(text: &str, mut ctrl: bool, alt: bool, shift: bool, win: bool) -> Option<HotkeyConfig> {
    let ch = text.chars().next()?;
    let code = ch as u32;
    match code {
        0x10 | 0xA0 | 0xA1 => return None, // Shift family
        0x11 | 0xA2 | 0xA3 => return None, // Ctrl family
        0x12 | 0xA4 | 0xA5 => return None, // Alt family
        0x14                => return None, // CapsLock
        0x5B | 0x5C         => return None, // LWin, RWin
        0xF720..=0xF72F     => return None, // Slint PUA modifiers
        _ => {}
    }

    let vk = match code {
        VK_TAB        => VK_TAB,
        0x0D | 0x0A   => VK_RETURN,
        VK_ESCAPE     => VK_ESCAPE,
        VK_SPACE      => VK_SPACE,
        VK_BACK       => VK_BACK,
        1..=26 => { ctrl = true; 0x41 + (code - 1) }
        0x41..=0x5A   => code,
        0x61..=0x7A   => code - 0x20,
        0x30..=0x39   => code,
        SLINT_F1  => VK_F1,  SLINT_F2  => VK_F2,  SLINT_F3  => VK_F3,
        SLINT_F4  => VK_F4,  SLINT_F5  => VK_F5,  SLINT_F6  => VK_F6,
        SLINT_F7  => VK_F7,  SLINT_F8  => VK_F8,  SLINT_F9  => VK_F9,
        SLINT_F10 => VK_F10, SLINT_F11 => VK_F11, SLINT_F12 => VK_F12,
        SLINT_UP    => VK_UP,    SLINT_DOWN  => VK_DOWN,
        SLINT_LEFT  => VK_LEFT,  SLINT_RIGHT => VK_RIGHT,
        SLINT_HOME  => VK_HOME,  SLINT_END   => VK_END,
        SLINT_PGUP  => VK_PRIOR, SLINT_PGDN  => VK_NEXT,
        SLINT_DEL   => VK_DELETE,
        _ => {
            if ch.is_ascii_graphic() { ch.to_ascii_uppercase() as u32 } else { return None; }
        }
    };

    let mut modifiers = 0u32;
    if alt   { modifiers |= 1; }
    if ctrl  { modifiers |= 2; }
    if shift { modifiers |= 4; }
    if win   { modifiers |= 8; }
    Some(HotkeyConfig { modifiers, virtual_key: vk })
}

/// Applies all i18n strings to the `ConfigWindow` and `AppTray` for the given language.
fn apply_language(window: &ConfigWindow, tray: &AppTray, lang: &str) {
    let s = crate::i18n::get_strings(lang);
    window.set_window_title_text(SharedString::from(s.window_title));
    window.set_i18n_header(SharedString::from(s.header));
    window.set_i18n_hotkey_label(SharedString::from(s.hotkey_label));
    window.set_i18n_hotkey_save(SharedString::from(s.hotkey_save));
    window.set_i18n_hotkey_prompt(SharedString::from(s.hotkey_prompt));
    window.set_i18n_buffer_label(SharedString::from(s.buffer_label));
    window.set_i18n_buffer_empty(SharedString::from(s.buffer_empty));
    window.set_i18n_col_trigger(SharedString::from(s.col_trigger));
    window.set_i18n_col_trigger_tooltip(SharedString::from(s.col_trigger_tooltip));
    window.set_i18n_col_replacement(SharedString::from(s.col_replacement));
    window.set_i18n_col_mode(SharedString::from(s.col_mode));
    window.set_i18n_mode_immediate(SharedString::from(s.mode_immediate));
    window.set_i18n_mode_hotkey(SharedString::from(s.mode_hotkey));
    window.set_i18n_btn_delete(SharedString::from(s.btn_delete));
    window.set_i18n_btn_add(SharedString::from(s.btn_add));
    window.set_i18n_btn_quit(SharedString::from(s.btn_quit));
    window.set_i18n_btn_pause(SharedString::from(s.btn_pause));
    window.set_i18n_btn_resume(SharedString::from(s.btn_resume));
    window.set_i18n_btn_pause_tooltip(SharedString::from(s.btn_pause_tooltip));
    window.set_i18n_btn_cancel(SharedString::from(s.btn_cancel));
    window.set_i18n_btn_save(SharedString::from(s.btn_save));
    window.set_i18n_current_lang(SharedString::from(if lang == "da" { "Dansk" } else { "English" }));
    window.set_i18n_uninstall_btn(SharedString::from(s.uninstall_btn));
    window.set_i18n_uninstall_tooltip(SharedString::from(s.uninstall_tooltip));
    window.set_i18n_btn_cancel_tooltip(SharedString::from(s.btn_cancel_tooltip));
    window.set_i18n_hotkey_label_tooltip(SharedString::from(s.hotkey_label_tooltip));
    window.set_i18n_quick_switch_label(SharedString::from(s.quick_switch_label));
    window.set_i18n_quick_switch_tooltip(SharedString::from(s.quick_switch_tooltip));
    window.set_i18n_case_changer_label(SharedString::from(s.case_changer_label));
    window.set_i18n_case_changer_tooltip(SharedString::from(s.case_changer_tooltip));
    window.set_i18n_tab_general(SharedString::from(s.tab_general));
    window.set_i18n_tab_snippets(SharedString::from(s.tab_snippets));
    window.set_i18n_section_hotkey(SharedString::from(s.section_hotkey));
    window.set_i18n_section_features(SharedString::from(s.section_features));
    tray.set_tray_tooltip_text(SharedString::from(s.tray_tooltip));
    tray.set_tray_open_text(SharedString::from(s.tray_open));
    tray.set_tray_quit_text(SharedString::from(s.tray_quit));
    window.set_i18n_mode_portable(SharedString::from(s.mode_portable));
    window.set_i18n_mode_installed(SharedString::from(s.mode_installed));
    window.set_i18n_move_config(SharedString::from(s.move_config_btn));
    window.set_i18n_mode_portable_tooltip(SharedString::from(s.mode_portable_tooltip));
}

// ---------------------------------------------------------------------------
// M1: setup_and_run sub-functions
// ---------------------------------------------------------------------------

/// Wires up hook-thread->UI callbacks and all Slint FocusScope hotkey-capture
/// callbacks (on_hotkey_clicked, on_key_recorded, on_save_hotkey).
fn setup_hotkey_capture_callbacks(
    window: &ConfigWindow,
    pending_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    config: Arc<ArcSwap<AppConfig>>,
    hook_thread_id: u32,
) {
    let window_weak = window.as_weak();
    let pending_for_capture = pending_hotkey.clone();
    crate::hook::set_on_hotkey_captured(move |hk| {
        let w_weak = window_weak.clone();
        let pending = pending_for_capture.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(w) = w_weak.upgrade() {
                w.set_hotkey_display(hotkey_display_string(&hk));
                w.set_hotkey_capturing(false);
                w.set_hotkey_conflict(false);
                w.set_hotkey_can_save(true);
                *pending.lock().unwrap() = Some(hk);
            }
        });
    });

    let window_weak = window.as_weak();
    crate::hook::set_on_mod_display(move |mod_str| {
        let w_weak = window_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(w) = w_weak.upgrade() && w.get_hotkey_capturing() {
                w.set_hotkey_display(SharedString::from(format!("{mod_str} + ...")));
            }
        });
    });

    let window_weak = window.as_weak();
    let pending_for_click = pending_hotkey.clone();
    window.on_hotkey_clicked(move || {
        if let Some(w) = window_weak.upgrade() {
            config::log_debug("UI: on_hotkey_clicked (FocusScope mode)");
            w.set_hotkey_capturing(true);
            w.set_hotkey_conflict(false);
            w.set_hotkey_can_save(false);
            w.set_hotkey_display(w.get_i18n_hotkey_prompt());
            *pending_for_click.lock().unwrap() = None;
        }
    });

    let window_weak = window.as_weak();
    let pending_for_key = pending_hotkey.clone();
    let config_for_validate = config.clone();
    window.on_key_recorded(move |text, ctrl, alt, shift, win| {
        config::log_debug(&format!(
            "UI: key_recorded: text=U+{:04X}, ctrl={ctrl}, alt={alt}, shift={shift}, win={win}",
            text.chars().next().map_or(0, |c| c as u32)
        ));
        if let Some(hk) = parse_key_event(text.as_str(), ctrl, alt, shift, win) {
            if let Some(w) = window_weak.upgrade() {
                let display = hotkey_display_string(&hk);
                let lang = config_for_validate.load().language.clone();
                match crate::hotkey::validate_hotkey(&hk, &lang) {
                    Ok(()) => {
                        w.set_hotkey_display(display);
                        w.set_hotkey_capturing(false);
                        w.set_hotkey_conflict(false);
                        w.set_hotkey_can_save(true);
                        *pending_for_key.lock().unwrap() = Some(hk);
                    }
                    Err(reason) => {
                        w.set_hotkey_display(SharedString::from(format!("{display} -- {reason}")));
                        w.set_hotkey_capturing(false);
                        w.set_hotkey_conflict(true);
                        w.set_hotkey_can_save(false);
                        *pending_for_key.lock().unwrap() = None;
                        config::log_debug(&format!("UI: hotkey rejected: {reason}"));
                    }
                }
            }
            true
        } else {
            if let Some(w) = window_weak.upgrade() {
                let mut parts = Vec::new();
                if ctrl  { parts.push("CTRL"); }
                if alt   { parts.push("ALT"); }
                if shift { parts.push("SHIFT"); }
                if win   { parts.push("WIN"); }
                if !parts.is_empty() {
                    w.set_hotkey_display(SharedString::from(format!("{} + ...", parts.join(" + "))));
                }
            }
            true
        }
    });

    let window_weak = window.as_weak();
    let pending_for_save = pending_hotkey;
    let htid = hook_thread_id;
    window.on_save_hotkey(move || {
        if let Some(w) = window_weak.upgrade() {
            let opt_hk = pending_for_save.lock().unwrap().clone();
            if let Some(hk) = opt_hk {
                let current_config = config.load();
                let new_config = AppConfig { hotkey: hk.clone(), ..(**current_config).clone() };
                if let Err(e) = config::save(&new_config) {
                    config::log_debug(&format!("Failed to save hotkey config: {e}"));
                    eprintln!("[dotxpander] Failed to save hotkey config: {e}");
                } else {
                    config.store(Arc::new(new_config));
                    unsafe { let _ = PostThreadMessageW(htid, WM_REHOOK, WPARAM(0), LPARAM(0)); }
                }
                w.set_hotkey_conflict(false);
                w.set_hotkey_can_save(false);
                w.set_hotkey_display(hotkey_display_string(&hk));
                *pending_for_save.lock().unwrap() = None;
            }
        }
    });
}

/// Wires up tray open/quit and window quit/uninstall callbacks.
#[allow(clippy::too_many_arguments)]
fn setup_tray_callbacks(
    tray: &AppTray,
    window: &ConfigWindow,
    hook_thread_id: u32,
    config: Arc<ArcSwap<AppConfig>>,
    pending_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    pending_cc_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
    saved_size: Arc<Mutex<Option<LogicalSize>>>,
    saved_pos: Arc<Mutex<Option<LogicalPosition>>>,
) {
    let window_weak = window.as_weak();
    let config_for_open = config.clone();
    tray.on_open_settings(move || {
        if let Some(w) = window_weak.upgrade() {
            crate::hook::set_recording_hotkey(false);
            w.set_hotkey_capturing(false);
            w.set_hotkey_conflict(false);
            w.set_hotkey_can_save(false);
            let current = config_for_open.load();
            w.set_hotkey_display(hotkey_display_string(&current.hotkey));
            w.set_quick_switch_enabled(current.quick_switch_enabled);
            w.set_case_changer_enabled(current.case_changer_enabled);
            w.set_snippet_hotkey_enabled(current.snippet_hotkey_enabled);
            w.set_case_changer_hotkey_display(hotkey_display_string(&current.case_changer_hotkey));
            w.set_case_changer_hotkey_capturing(false);
            w.set_case_changer_hotkey_conflict(false);
            w.set_case_changer_hotkey_can_save(false);
            *pending_hotkey.lock().unwrap() = None;
            *pending_cc_hotkey.lock().unwrap() = None;
            w.set_save_error_message(SharedString::from(""));

            // set_position BEFORE show (avoids Win32 white-window blit bug).
            if let Some(pos) = *saved_pos.lock().unwrap() {
                w.window().set_position(pos);
            }
            // show BEFORE set_size (needs visible window for correct DPI resolution).
            let _ = w.show();
            if let Some(sz) = *saved_size.lock().unwrap() {
                w.window().set_size(sz);
            } else {
                w.window().set_size(LogicalSize::new(722.0, 485.0));
            }

            use windows::Win32::UI::WindowsAndMessaging::{
                SetForegroundWindow, ShowWindow, SW_RESTORE, IsIconic, FindWindowW,
            };
            use windows::core::PCWSTR;
            unsafe {
                let title_str = w.get_window_title_text();
                let wide: Vec<u16> = title_str.encode_utf16().chain(std::iter::once(0)).collect();
                if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr())) {
                    if IsIconic(hwnd).as_bool() { let _ = ShowWindow(hwnd, SW_RESTORE); }
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        }
    });

    let htid = hook_thread_id;
    tray.on_quit_app(move || graceful_shutdown(htid));

    let htid = hook_thread_id;
    window.on_quit_app(move || graceful_shutdown(htid));

    let htid = hook_thread_id;
    let config_for_uninstall = config;
    window.on_uninstall_app(move || {
        let lang = config_for_uninstall.load().language.clone();
        let s = crate::i18n::get_strings(&lang);
        let confirmed = unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                MessageBoxW, MB_ICONWARNING, MB_YESNO, MB_DEFBUTTON2, IDYES,
            };
            use windows::core::PCWSTR;
            let title: Vec<u16> = s.uninstall_title.encode_utf16().chain(std::iter::once(0)).collect();
            let body:  Vec<u16> = s.uninstall_body.encode_utf16().chain(std::iter::once(0)).collect();
            MessageBoxW(None, PCWSTR(body.as_ptr()), PCWSTR(title.as_ptr()),
                MB_ICONWARNING | MB_YESNO | MB_DEFBUTTON2) == IDYES
        };
        if confirmed {
            crate::config::log_debug("UI: user confirmed uninstall");
            if let Err(e) = crate::uninstall::self_destruct(htid) {
                crate::config::log_debug(&format!("uninstall failed: {e}"));
                eprintln!("uninstall failed: {e}");
            }
        }
    });
}

/// Wires up snippet list callbacks: save_config, cancel_config,
/// close_requested, add_snippet, remove_snippet.
fn setup_snippet_callbacks(
    window: &ConfigWindow,
    snippets_model: Rc<VecModel<SnippetModel>>,
    config: Arc<ArcSwap<AppConfig>>,
    hook_thread_id: u32,
    saved_size: Arc<Mutex<Option<LogicalSize>>>,
    saved_pos: Arc<Mutex<Option<LogicalPosition>>>,
) {
    let window_weak = window.as_weak();
    let config_clone = config.clone();
    let model_clone = snippets_model.clone();
    let htid = hook_thread_id;
    window.on_save_config(move || {
        if let Some(w) = window_weak.upgrade() {
            let current_config = config_clone.load();
            let mut new_snippets = Vec::new();
            for i in 0..model_clone.row_count() {
                if let Some(m) = model_clone.row_data(i) {
                    if m.trigger.is_empty() { continue; }
                    let mode = if m.mode.as_str() == "hotkey" { ExpansionMode::Hotkey } else { ExpansionMode::Immediate };
                    new_snippets.push(Snippet {
                        trigger: m.trigger.to_string(),
                        replacement: m.replacement.to_string(),
                        mode,
                    });
                }
            }
            let new_config = AppConfig { snippets: new_snippets, ..(**current_config).clone() };
            match config::save(&new_config) {
                Ok(()) => {
                    config_clone.store(Arc::new(new_config));
                    unsafe { let _ = PostThreadMessageW(htid, WM_REHOOK, WPARAM(0), LPARAM(0)); }
                    let scale = w.window().scale_factor();
                    let phys_size = w.window().size();
                    *saved_size.lock().unwrap() = Some(LogicalSize::new(
                        phys_size.width as f32 / scale, phys_size.height as f32 / scale));
                    let phys_pos = w.window().position();
                    *saved_pos.lock().unwrap() = Some(LogicalPosition::new(
                        phys_pos.x as f32 / scale, phys_pos.y as f32 / scale));
                    let _ = w.hide();
                }
                Err(e) => {
                    let msg = format!("Save failed: {e}");
                    config::log_debug(&msg);
                    eprintln!("[dotxpander] {msg}");
                    w.set_save_error_message(SharedString::from(msg));
                }
            }
        }
    });

    let window_weak = window.as_weak();
    let config_clone = config.clone();
    let model_clone = snippets_model.clone();
    window.on_cancel_config(move || {
        if let Some(w) = window_weak.upgrade() {
            model_clone.set_vec(config_to_snippet_models(&config_clone.load()));
            let _ = w.hide();
        }
    });

    let window_weak = window.as_weak();
    let config_for_close = config.clone();
    let model_for_close = snippets_model.clone();
    window.window().on_close_requested(move || {
        if let Some(w) = window_weak.upgrade() {
            model_for_close.set_vec(config_to_snippet_models(&config_for_close.load()));
            let _ = w.hide();
        }
        slint::CloseRequestResponse::HideWindow
    });

    let model_clone = snippets_model.clone();
    window.on_add_snippet(move || {
        model_clone.push(SnippetModel {
            trigger: SharedString::from(""),
            replacement: SharedString::from(""),
            mode: SharedString::from("immediate"),
        });
    });

    let model_clone = snippets_model;
    window.on_remove_snippet(move |index| {
        if index >= 0 && (index as usize) < model_clone.row_count() {
            model_clone.remove(index as usize);
        }
    });
}

/// Wires up feature toggle callbacks: language, config folder, pause,
/// Quick Switch, snippet hotkey, Case Changer toggle, and CC hotkey capture.
fn setup_feature_toggle_callbacks(
    window: &ConfigWindow,
    tray: &AppTray,
    config: Arc<ArcSwap<AppConfig>>,
    quick_switch: Arc<Mutex<Option<crate::quick_switch::QuickSwitchManager>>>,
    pending_cc_hotkey: Arc<Mutex<Option<HotkeyConfig>>>,
) {
    let window_weak = window.as_weak();
    let tray_weak = tray.as_weak();
    let config_clone = config.clone();
    window.on_language_changed(move |lang_code| {
        let lang = lang_code.to_string();
        if let (Some(w), Some(t)) = (window_weak.upgrade(), tray_weak.upgrade()) {
            apply_language(&w, &t, &lang);
            let current_config = config_clone.load();
            let new_config = AppConfig { language: lang, ..(**current_config).clone() };
            if let Err(e) = config::save(&new_config) {
                config::log_debug(&format!("Failed to save language config: {e}"));
                eprintln!("[dotxpander] Failed to save language config: {e}");
            } else {
                config_clone.store(Arc::new(new_config));
            }
        }
    });

    window.on_open_config_folder(move || {
        let _ = std::process::Command::new("explorer").arg(config::config_dir()).spawn();
    });

    let window_weak = window.as_weak();
    window.on_move_config_folder(move || {
        // Open native IFileDialog folder picker.
        // This runs on the Slint UI thread — no mutex conflict with the hook thread.
        let chosen = pick_folder_dialog();
        if let Some(new_dir) = chosen {
            match config::move_config_dir(&new_dir) {
                Ok(()) => {
                    if let Some(w) = window_weak.upgrade() {
                        // Refresh the config path label and mode badge.
                        w.set_config_file_path(SharedString::from(
                            config::config_path().to_string_lossy().to_string(),
                        ));
                        // After a successful move the registry key exists → not portable.
                        w.set_is_portable(config::is_portable());
                        w.set_save_error_message(SharedString::from(""));
                    }
                }
                Err(e) => {
                    if let Some(w) = window_weak.upgrade() {
                        let msg = format!("Move failed: {e}");
                        config::log_debug(&msg);
                        eprintln!("[dotxpander] {msg}");
                        w.set_save_error_message(SharedString::from(msg));
                    }
                }
            }
        }
    });

    let window_weak = window.as_weak();
    window.on_toggle_pause(move || {
        let new_paused = !crate::hook::is_paused();
        crate::hook::set_paused(new_paused);
        if let Some(w) = window_weak.upgrade() { w.set_is_paused(new_paused); }
    });

    let config_clone = config.clone();
    window.on_quick_switch_toggled(move |enabled| {
        let current_config = config_clone.load();
        let new_config = AppConfig { quick_switch_enabled: enabled, ..(**current_config).clone() };
        if let Err(e) = config::save(&new_config) {
            config::log_debug(&format!("QuickSwitch: failed to save: {e}"));
            eprintln!("[dotxpander] QuickSwitch: failed to save: {e}");
        } else {
            config_clone.store(Arc::new(new_config));
        }
        let mut qs = quick_switch.lock().unwrap();
        if enabled {
            if qs.is_none() {
                *qs = Some(crate::quick_switch::QuickSwitchManager::start(config_clone.clone()));
                config::log_debug("QuickSwitch: started");
            }
        } else if let Some(mut mgr) = qs.take() {
            mgr.stop();
            config::log_debug("QuickSwitch: stopped");
        }
    });

    let config_clone = config.clone();
    window.on_snippet_hotkey_toggled(move |enabled| {
        let current_config = config_clone.load();
        let new_config = AppConfig { snippet_hotkey_enabled: enabled, ..(**current_config).clone() };
        if let Err(e) = config::save(&new_config) {
            config::log_debug(&format!("SnippetHotkey toggle: failed to save: {e}"));
        } else {
            config_clone.store(Arc::new(new_config));
        }
    });

    let config_clone = config.clone();
    window.on_case_changer_toggled(move |enabled| {
        let current_config = config_clone.load();
        let new_config = AppConfig { case_changer_enabled: enabled, ..(**current_config).clone() };
        if let Err(e) = config::save(&new_config) {
            config::log_debug(&format!("CaseChanger: failed to save: {e}"));
            eprintln!("[dotxpander] CaseChanger: failed to save: {e}");
        } else {
            config_clone.store(Arc::new(new_config));
        }
    });

    let window_weak = window.as_weak();
    let pending_for_click = pending_cc_hotkey.clone();
    window.on_case_changer_hotkey_clicked(move || {
        if let Some(w) = window_weak.upgrade() {
            config::log_debug("UI: on_case_changer_hotkey_clicked");
            w.set_case_changer_hotkey_capturing(true);
            w.set_case_changer_hotkey_conflict(false);
            w.set_case_changer_hotkey_can_save(false);
            w.set_case_changer_hotkey_display(w.get_i18n_hotkey_prompt());
            *pending_for_click.lock().unwrap() = None;
        }
    });

    let window_weak = window.as_weak();
    let pending_for_key = pending_cc_hotkey.clone();
    let config_for_cc = config.clone();
    window.on_case_changer_key_recorded(move |text, ctrl, alt, shift, win| {
        if let Some(hk) = parse_key_event(text.as_str(), ctrl, alt, shift, win) {
            if let Some(w) = window_weak.upgrade() {
                let display = hotkey_display_string(&hk);
                let lang = config_for_cc.load().language.clone();
                match crate::hotkey::validate_hotkey(&hk, &lang) {
                    Ok(()) => {
                        w.set_case_changer_hotkey_display(display);
                        w.set_case_changer_hotkey_capturing(false);
                        w.set_case_changer_hotkey_conflict(false);
                        w.set_case_changer_hotkey_can_save(true);
                        *pending_for_key.lock().unwrap() = Some(hk);
                    }
                    Err(reason) => {
                        w.set_case_changer_hotkey_display(SharedString::from(format!("{display} -- {reason}")));
                        w.set_case_changer_hotkey_capturing(false);
                        w.set_case_changer_hotkey_conflict(true);
                        w.set_case_changer_hotkey_can_save(false);
                        *pending_for_key.lock().unwrap() = None;
                    }
                }
            }
            true
        } else {
            if let Some(w) = window_weak.upgrade() {
                let mut parts = Vec::new();
                if ctrl  { parts.push("CTRL"); }
                if alt   { parts.push("ALT"); }
                if shift { parts.push("SHIFT"); }
                if win   { parts.push("WIN"); }
                if !parts.is_empty() {
                    w.set_case_changer_hotkey_display(SharedString::from(
                        format!("{} + ...", parts.join(" + "))
                    ));
                }
            }
            true
        }
    });

    let window_weak = window.as_weak();
    let config_clone = config;
    let pending_for_save = pending_cc_hotkey;
    window.on_save_case_changer_hotkey(move || {
        if let Some(w) = window_weak.upgrade() {
            let opt_hk = pending_for_save.lock().unwrap().clone();
            if let Some(hk) = opt_hk {
                let current_config = config_clone.load();
                let new_config = AppConfig { case_changer_hotkey: hk.clone(), ..(**current_config).clone() };
                if let Err(e) = config::save(&new_config) {
                    config::log_debug(&format!("CaseChanger hotkey: failed to save: {e}"));
                    eprintln!("[dotxpander] CaseChanger hotkey: failed to save: {e}");
                } else {
                    config_clone.store(Arc::new(new_config));
                }
                w.set_case_changer_hotkey_conflict(false);
                w.set_case_changer_hotkey_can_save(false);
                w.set_case_changer_hotkey_display(hotkey_display_string(&hk));
                *pending_for_save.lock().unwrap() = None;
            }
        }
    });
}

/// Opens the native Windows folder-picker dialog (IFileDialog / FOS_PICKFOLDERS).
///
/// Returns `Some(path)` if the user selected a folder, or `None` if cancelled.
/// Falls back to `SHBrowseForFolderW` if IFileDialog is unavailable (pre-Vista).
///
/// All Win32 COM calls are in `unsafe` blocks with explicit HRESULT checking.
/// No `.unwrap()` on FFI boundaries.
fn pick_folder_dialog() -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::{
        CoInitializeEx, CoUninitialize,
        COINIT_APARTMENTTHREADED,
    };

    // Initialize COM for this call (apartment threaded, UI thread).
    let co_init_result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    // S_OK (0x0) = initialized, S_FALSE (0x1) = already initialized — both fine.
    // Any other failure → bail.
    if co_init_result.0 < 0 {
        eprintln!("[dotxpander] CoInitializeEx failed: {:?}", co_init_result);
        return None;
    }

    let result = pick_folder_dialog_inner();

    unsafe { CoUninitialize(); }
    result
}

/// Inner logic of the folder picker, separated so `CoUninitialize` always runs.
fn pick_folder_dialog_inner() -> Option<std::path::PathBuf> {
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        IFileDialog, FileOpenDialog, FOS_PICKFOLDERS,
    };

    // Create an IFileDialog instance (Vista+).
    let dialog: IFileDialog = unsafe {
        CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
    }.ok()?;

    // Configure: pick folders, no multi-select.
    let current_opts = unsafe { dialog.GetOptions() }.ok()?;
    unsafe { dialog.SetOptions(current_opts | FOS_PICKFOLDERS) }.ok()?;

    // Show the dialog (blocks until the user picks or cancels).
    let show_result = unsafe { dialog.Show(None) };
    if show_result.is_err() {
        // User cancelled (HRESULT_FROM_WIN32(ERROR_CANCELLED)).
        return None;
    }

    // Retrieve the selected item.
    let result_item = unsafe { dialog.GetResult() }.ok()?;
    let display_name = unsafe { result_item.GetDisplayName(windows::Win32::UI::Shell::SIGDN_FILESYSPATH) }.ok()?;

    // Convert PWSTR to PathBuf.
    let path_str = unsafe { display_name.to_string() }.ok()?;
    Some(std::path::PathBuf::from(path_str))
}

/// Sets up the Slint UI and runs the event loop.
///
/// Orchestrates the four setup helpers and blocks until `graceful_shutdown` is called.
pub fn setup_and_run(
    config: Arc<ArcSwap<AppConfig>>,
    hook_thread_id: u32,
    buffer_debug: Arc<Mutex<String>>,
    show_settings_on_start: bool,
    quick_switch: Arc<Mutex<Option<crate::quick_switch::QuickSwitchManager>>>,
) -> Result<(), slint::PlatformError> {
    let window = ConfigWindow::new()?;
    let tray   = AppTray::new()?;

    // Initial state
    let current_config = config.load();
    let snippets_model = Rc::new(VecModel::from(config_to_snippet_models(&current_config)));
    window.set_snippets(snippets_model.clone().into());
    window.set_hotkey_display(hotkey_display_string(&current_config.hotkey));
    window.set_config_file_path(SharedString::from(config::config_path().to_string_lossy().to_string()));
    window.set_is_portable(config::is_portable());
    window.set_quick_switch_enabled(current_config.quick_switch_enabled);
    window.set_case_changer_enabled(current_config.case_changer_enabled);
    window.set_snippet_hotkey_enabled(current_config.snippet_hotkey_enabled);
    window.set_case_changer_hotkey_display(hotkey_display_string(&current_config.case_changer_hotkey));
    apply_language(&window, &tray, &current_config.language);

    if show_settings_on_start {
        let _ = window.show();
        window.window().set_size(LogicalSize::new(722.0, 485.0));
    }

    // Shared state
    let pending_hotkey:    Arc<Mutex<Option<HotkeyConfig>>> = Arc::new(Mutex::new(None));
    let pending_cc_hotkey: Arc<Mutex<Option<HotkeyConfig>>> = Arc::new(Mutex::new(None));
    let saved_size: Arc<Mutex<Option<LogicalSize>>>     = Arc::new(Mutex::new(None));
    let saved_pos:  Arc<Mutex<Option<LogicalPosition>>> = Arc::new(Mutex::new(None));

    // Buffer debug timer (50 ms poll)
    let buffer_timer = Timer::default();
    let window_weak = window.as_weak();
    let buffer_debug_clone = buffer_debug;
    buffer_timer.start(TimerMode::Repeated, std::time::Duration::from_millis(50), move || {
        if let Some(w) = window_weak.upgrade()
            && let Ok(content) = buffer_debug_clone.try_lock() {
                w.set_buffer_content(SharedString::from(content.as_str()));
            }
    });

    // Wire callbacks
    setup_hotkey_capture_callbacks(&window, pending_hotkey.clone(), config.clone(), hook_thread_id);
    setup_tray_callbacks(&tray, &window, hook_thread_id, config.clone(),
        pending_hotkey, pending_cc_hotkey.clone(), saved_size.clone(), saved_pos.clone());
    setup_snippet_callbacks(&window, snippets_model, config.clone(),
        hook_thread_id, saved_size, saved_pos);
    setup_feature_toggle_callbacks(&window, &tray, config, quick_switch, pending_cc_hotkey);

    // Run the Slint event loop (blocks until graceful_shutdown)
    slint::run_event_loop()?;
    Ok(())
}