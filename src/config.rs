use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};


/// The main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    /// UI language ("en" or "da")
    #[serde(default = "default_language")]
    pub language: String,
    /// Hotkey configuration for triggering expansions.
    pub hotkey: HotkeyConfig,
    /// Size of the typing buffer (clamped to 1..=256 at load time).
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// How long (in milliseconds) to wait after simulating Ctrl+V before restoring
    /// the original clipboard content. `WinUI` 3 / XAML apps have asynchronous paste
    /// pipelines that need time to read the clipboard before we overwrite it.
    /// Increase this on slow machines or under Prism emulation if clipboard restore
    /// happens too early and cuts off the pasted text.
    #[serde(default = "default_clipboard_restore_delay_ms")]
    pub clipboard_restore_delay_ms: u64,
    /// When `true`, the snippet hotkey triggers text expansion.
    /// Defaults to `true`. Persisted so the setting survives restarts.
    #[serde(default = "default_true")]
    pub snippet_hotkey_enabled: bool,
    /// When `true`, a background thread monitors focus changes and automatically
    /// navigates Open/Save file dialogs to the folder of the last focused
    /// Explorer window. Defaults to `true` (opt-in, now on by default).
    #[serde(default = "default_true")]
    pub quick_switch_enabled: bool,
    /// When `true`, the `Ctrl+CapsLock` shortcut shows a popup menu for
    /// transforming selected text (uppercase, lowercase, camelCase, etc.).
    /// Defaults to `true` (enabled by default — opt-out feature).
    #[serde(default = "default_case_changer_enabled")]
    pub case_changer_enabled: bool,
    /// Hotkey for the Case Changer popup menu. Defaults to `Ctrl+CapsLock`
    /// (modifiers=2, `virtual_key=0x14`). Stored separately from the snippet
    /// hotkey so users can remap it independently.
    #[serde(default = "default_case_changer_hotkey")]
    pub case_changer_hotkey: HotkeyConfig,
    /// List of user-defined snippets.
    #[serde(default)]
    pub snippets: Vec<Snippet>,
}

const fn default_true() -> bool { true }

fn default_language() -> String {
    "en".to_string()
}

const fn default_buffer_size() -> usize {
    10
}

const fn default_clipboard_restore_delay_ms() -> u64 {
    150
}


const fn default_case_changer_enabled() -> bool {
    true // Enabled by default (opt-out feature)
}

/// Default Case Changer hotkey: Ctrl+CapsLock
/// `MOD_CONTROL=2`, `VK_CAPITAL=0x14`
const fn default_case_changer_hotkey() -> HotkeyConfig {
    HotkeyConfig { modifiers: 2, virtual_key: 0x14 }
}

/// Hotkey configuration for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    /// Win32 modifier flags (`MOD_ALT=1`, `MOD_CONTROL=2`, `MOD_SHIFT=4`, `MOD_WIN=8`)
    pub modifiers: u32,
    /// Win32 virtual key code (e.g., 0x58 for 'X')
    pub virtual_key: u32,
}

/// A snippet definition containing a trigger, replacement, and expansion mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snippet {
    /// The string that triggers the expansion.
    pub trigger: String,
    /// The replacement string.
    pub replacement: String,
    /// The mode of expansion.
    pub mode: ExpansionMode,
}

/// Defines when a snippet should be expanded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExpansionMode {
    /// Expands immediately upon typing the trigger.
    Immediate,
    /// Expands only when the hotkey is pressed.
    Hotkey,
}

/// Errors that can occur during configuration operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Standard I/O error.
    Io(io::Error),
    /// Error deserializing TOML.
    Toml(toml::de::Error),
    /// Error serializing TOML.
    Serialize(toml::ser::Error),
    /// Win32 registry operation error (error code).
    Registry(u32),
    /// Path contains invalid characters.
    InvalidPath(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Toml(err) => write!(f, "TOML parsing error: {err}"),
            Self::Serialize(err) => write!(f, "TOML serialization error: {err}"),
            Self::Registry(code) => write!(f, "Registry error: Win32 error code {code:#010x}"),
            Self::InvalidPath(s) => write!(f, "Invalid path: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Serialize(err)
    }
}

// ---------------------------------------------------------------------------
// Registry constants
// ---------------------------------------------------------------------------

const REGISTRY_SUBKEY: &str = "Software\\aiVOLUTION\\dotXPANDER";
const REGISTRY_VALUE_NAME: &str = "ConfigPath";

/// Default config directory: %APPDATA%\aiVOLUTION\dotXPANDER
fn default_appdata_dir() -> PathBuf {
    let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(appdata);
    p.push("aiVOLUTION\\dotXPANDER");
    p
}

// ---------------------------------------------------------------------------
// Config directory resolution cache
// ---------------------------------------------------------------------------
//
// Stores (directory, is_portable). Uses Mutex<Option<...>> (not OnceLock)
// because the cache must be invalidatable when the user moves the config.

static CONFIG_DIR_CACHE: Mutex<Option<(PathBuf, bool)>> = Mutex::new(None);

/// Resolves the config directory and portability flag.
///
/// Resolution order:
///   1. Check cache.
///   2. Read `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath` from the registry.
///      If the key exists → installed mode (is_portable = false).
///   3. If the key is absent → portable mode: config lives next to the `.exe`.
///
/// The result is cached in `CONFIG_DIR_CACHE`. The cache is only invalidated
/// by `move_config_dir()` after a successful move.
///
/// Returns `(directory, is_portable)`.
pub fn resolve_config_dir() -> (PathBuf, bool) {
    // Fast path: return cached value
    {
        let guard = CONFIG_DIR_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(ref cached) = *guard {
            return cached.clone();
        }
    }

    // Slow path: resolve from registry / filesystem
    let result = resolve_config_dir_uncached();

    // Store in cache
    {
        let mut guard = CONFIG_DIR_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(result.clone());
    }

    result
}

/// Performs the actual registry read without touching the cache.
fn resolve_config_dir_uncached() -> (PathBuf, bool) {
    match read_registry_config_path() {
        Ok(Some(dir)) => {
            // Registry key present → installed mode
            (dir, false)
        }
        Ok(None) => {
            // No registry key → portable mode: config next to the exe
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            (exe_dir, true)
        }
        Err(e) => {
            // Registry read failed — fall back to default appdata location
            eprintln!("[dotxpander] WARNING: Could not read registry ({e}); falling back to %APPDATA%");
            (default_appdata_dir(), false)
        }
    }
}

/// Reads `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath` (REG_SZ).
///
/// Returns:
///   - `Ok(Some(path))` — key exists and contains a valid path string.
///   - `Ok(None)`       — key does not exist (ERROR_FILE_NOT_FOUND / ERROR_PATH_NOT_FOUND).
///   - `Err(_)`         — unexpected Win32 error.
///
/// All Win32 registry calls are in `unsafe` blocks with explicit error checking.
/// No `.unwrap()` on FFI boundaries.
fn read_registry_config_path() -> Result<Option<PathBuf>, ConfigError> {
    use windows::Win32::System::Registry::{
        RegOpenKeyExW, RegQueryValueExW, RegCloseKey,
        HKEY_CURRENT_USER, KEY_READ, REG_SZ, HKEY,
    };
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, WIN32_ERROR};
    use windows::core::PCWSTR;

    // Encode subkey as null-terminated UTF-16
    let subkey_wide: Vec<u16> = REGISTRY_SUBKEY
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut hkey = HKEY::default();

    // Open the registry key (read-only). ERROR_FILE_NOT_FOUND = key absent (portable mode).
    let open_result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if open_result != WIN32_ERROR(0) {
        // Close handle if somehow partially opened (defensive; unlikely here)
        if !hkey.is_invalid() {
            unsafe { let _ = RegCloseKey(hkey); }
        }
        if open_result == ERROR_FILE_NOT_FOUND || open_result == ERROR_PATH_NOT_FOUND {
            return Ok(None); // Key absent → portable mode
        }
        return Err(ConfigError::Registry(open_result.0));
    }

    let value_name_wide: Vec<u16> = REGISTRY_VALUE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // Query the required buffer size first (pass null data pointer).
    let mut data_type = REG_SZ; // initialise to REG_SZ; overwritten by the API
    let mut buf_size: u32 = 0;
    let query_size_result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut buf_size),
        )
    };

    if query_size_result != WIN32_ERROR(0) {
        unsafe { let _ = RegCloseKey(hkey); }
        if query_size_result == ERROR_FILE_NOT_FOUND {
            return Ok(None); // Value absent → portable mode
        }
        return Err(ConfigError::Registry(query_size_result.0));
    }

    // Allocate buffer (size is in bytes; each UTF-16 char = 2 bytes).
    let char_count = (buf_size / 2) as usize;
    let mut buf: Vec<u16> = vec![0u16; char_count];
    let mut actual_size = buf_size;

    let query_result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut actual_size),
        )
    };

    unsafe { let _ = RegCloseKey(hkey); }

    if query_result != WIN32_ERROR(0) {
        return Err(ConfigError::Registry(query_result.0));
    }

    // Verify it is a string type.
    if data_type != REG_SZ {
        return Err(ConfigError::Registry(0xDEAD_0001)); // unexpected type sentinel
    }

    // Strip trailing null terminators and decode UTF-16.
    let s: String = String::from_utf16_lossy(
        buf.iter().copied().take_while(|&c| c != 0).collect::<Vec<u16>>().as_slice(),
    );

    if s.is_empty() {
        return Ok(None);
    }

    Ok(Some(PathBuf::from(s)))
}

/// Returns the configuration directory, creating it if necessary.
///
/// Delegates to `resolve_config_dir()` for the path, then `create_dir_all`.
#[must_use]
pub fn config_dir() -> PathBuf {
    let (dir, _) = resolve_config_dir();

    if let Err(e) = fs::create_dir_all(&dir) {
        if !dir.exists() {
            eprintln!("[dotxpander] WARNING: Could not create config dir {}: {e}", dir.display());
        }
    }

    dir
}

/// Returns `true` when running in portable mode (no registry key present).
///
/// This is a cheap read from the cache (populated by the first `config_dir()` call).
#[must_use]
pub fn is_portable() -> bool {
    resolve_config_dir().1
}

/// Returns the path to the configuration file.
#[must_use]
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Returns the path to the backup configuration file.
fn backup_path() -> PathBuf {
    config_dir().join("config.toml.bak")
}

// ---------------------------------------------------------------------------
// Registry write helpers
// ---------------------------------------------------------------------------

/// Writes `new_dir` to `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath`.
///
/// Creates the registry key if it does not already exist.
/// Also invalidates the in-process cache so subsequent calls see the new value.
///
/// All Win32 calls are in `unsafe` blocks with explicit error checking.
/// No `.unwrap()` on FFI boundaries.
pub fn update_config_registry(new_dir: &std::path::Path) -> Result<(), ConfigError> {
    use windows::Win32::System::Registry::{
        RegCreateKeyExW, RegSetValueExW, RegCloseKey,
        HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, HKEY,
        REG_CREATE_KEY_DISPOSITION,
    };
    use windows::Win32::Foundation::WIN32_ERROR;
    use windows::core::PCWSTR;

    let dir_str = new_dir.to_str().ok_or_else(|| {
        ConfigError::InvalidPath("Config path contains non-UTF-8 characters".to_string())
    })?;

    let subkey_wide: Vec<u16> = REGISTRY_SUBKEY
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut hkey = HKEY::default();
    let mut disposition = REG_CREATE_KEY_DISPOSITION(0);

    let create_result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            Some(&mut disposition),
        )
    };

    if create_result != WIN32_ERROR(0) {
        return Err(ConfigError::Registry(create_result.0));
    }

    let value_name_wide: Vec<u16> = REGISTRY_VALUE_NAME
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // Encode value as null-terminated UTF-16 (RegSetValueExW expects byte slice).
    let value_data_wide: Vec<u16> = dir_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let byte_len = (value_data_wide.len() * 2) as u32;

    let set_result = unsafe {
        RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            0,
            REG_SZ,
            Some(
                std::slice::from_raw_parts(value_data_wide.as_ptr() as *const u8, byte_len as usize)
            ),
        )
    };

    unsafe { let _ = RegCloseKey(hkey); }

    if set_result != WIN32_ERROR(0) {
        return Err(ConfigError::Registry(set_result.0));
    }

    // Invalidate cache so the new path is picked up on the next call.
    invalidate_config_dir_cache();

    Ok(())
}

/// Deletes `HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath` (and the `dotXPANDER` key if empty).
///
/// Returns `Ok(())` even if the key does not exist.
fn delete_config_registry() -> Result<(), ConfigError> {
    use windows::Win32::System::Registry::{
        RegDeleteKeyW, HKEY_CURRENT_USER,
    };
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
    use windows::core::PCWSTR;

    let subkey_wide: Vec<u16> = REGISTRY_SUBKEY
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        RegDeleteKeyW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
        )
    };

    if result != WIN32_ERROR(0) && result != ERROR_FILE_NOT_FOUND {
        return Err(ConfigError::Registry(result.0));
    }

    Ok(())
}

/// Clears the config directory cache so the next call re-resolves from the registry.
fn invalidate_config_dir_cache() {
    let mut guard = CONFIG_DIR_CACHE.lock().unwrap_or_else(|p| p.into_inner());
    *guard = None;
}

// ---------------------------------------------------------------------------
// Config file move
// ---------------------------------------------------------------------------

/// Moves the config directory to `new_dir`:
///
/// 1. Creates `new_dir` if it does not exist.
/// 2. Copies `config.toml` to `new_dir` — **skips** if one already exists there.
/// 3. Copies `config.toml.bak` if present (always overwrite).
/// 4. Copies `debug.log` if present (always overwrite).
/// 5. Writes `new_dir` to the registry (`HKCU\Software\aiVOLUTION\dotXPANDER\ConfigPath`).
/// 6. Deletes the old files from the old directory.
/// 7. Removes the old directory if it is now empty.
/// 8. Invalidates the cache so the new path is active immediately.
pub fn move_config_dir(new_dir: &std::path::Path) -> Result<(), ConfigError> {
    let old_dir = config_dir(); // triggers cache population

    // Resolve canonical paths to avoid treating same-dir as a move
    let old_canonical = fs::canonicalize(&old_dir).unwrap_or_else(|_| old_dir.clone());
    let new_canonical = fs::canonicalize(new_dir).unwrap_or_else(|_| new_dir.to_path_buf());
    if old_canonical == new_canonical {
        // Nothing to do — already in the right place
        return Ok(());
    }

    // 1. Ensure new directory exists
    fs::create_dir_all(new_dir)?;

    // 2. Copy config.toml (preserve existing at target)
    let old_config = old_dir.join("config.toml");
    let new_config = new_dir.join("config.toml");
    if old_config.exists() && !new_config.exists() {
        fs::copy(&old_config, &new_config)?;
    }

    // 3. Copy backup (always overwrite)
    let old_bak = old_dir.join("config.toml.bak");
    if old_bak.exists() {
        let _ = fs::copy(&old_bak, new_dir.join("config.toml.bak"));
    }

    // 4. Copy debug log (always overwrite)
    let old_log = old_dir.join("debug.log");
    if old_log.exists() {
        let _ = fs::copy(&old_log, new_dir.join("debug.log"));
    }

    // 5. Write new path to registry (also invalidates cache)
    update_config_registry(new_dir)?;

    // 6. Delete old files (best-effort)
    if old_config.exists() {
        let _ = fs::remove_file(&old_config);
    }
    if old_bak.exists() {
        let _ = fs::remove_file(&old_bak);
    }
    if old_log.exists() {
        let _ = fs::remove_file(&old_log);
    }

    // 7. Remove old directory if empty (best-effort)
    let _ = fs::remove_dir(&old_dir); // only succeeds if dir is empty

    Ok(())
}

// ---------------------------------------------------------------------------
// Uninstall helper
// ---------------------------------------------------------------------------

/// Deletes config files during the self-destruct / uninstall flow.
///
/// Behaviour depends on the current mode:
///   - **Installed mode, default path** (`%APPDATA%\aiVOLUTION\dotXPANDER`):
///     Deletes the entire directory and removes the registry key.
///   - **Installed mode, custom path** (cloud-synced folder, etc.):
///     Only removes the registry key; leaves config files intact.
///   - **Portable mode**: No registry key to remove; leaves config files intact
///     (they live next to the exe which the cmd.exe script deletes separately).
pub fn delete_config_dir() {
    let (dir, portable) = resolve_config_dir();

    // Remove registry key (best-effort; ignore errors)
    if !portable {
        if let Err(e) = delete_config_registry() {
            eprintln!("[dotxpander] uninstall: failed to delete registry key: {e}");
        }
    }

    // Only wipe the config directory when it is the default %APPDATA% location.
    // Custom / cloud-synced paths are preserved.
    let default_dir = default_appdata_dir();
    if dir == default_dir && dir.exists() {
        if let Err(e) = fs::remove_dir_all(&dir) {
            eprintln!("[dotxpander] uninstall: failed to remove config dir {}: {e}", dir.display());
        }
    }
}

// ---------------------------------------------------------------------------
// Buffered debug logging
// ---------------------------------------------------------------------------
//
// Opens debug.log once and keeps it open for the lifetime of the process.
// This avoids the overhead of open/write/close on every keystroke, which was
// previously happening inside the WH_KEYBOARD_LL hook callback.
// The file is flushed on every write so log entries appear immediately.
//
// In release builds the function is a no-op unless the DOTXPANDER_LOG
// environment variable is set, keeping hot-path overhead at zero.

static LOG_FILE: OnceLock<Mutex<Option<fs::File>>> = OnceLock::new();

/// Cached debug-logging enabled state.
///
/// Set once on the first `log_debug` call that resolves the env-var check.
/// Allows the hook hot path to call `is_debug_logging_enabled()` — a single
/// relaxed atomic load — instead of allocating a `String` for `format!()`.
static DEBUG_LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Returns `true` if debug logging is currently active.
///
/// This is a **zero-cost, wait-free** predicate: a single relaxed `AtomicBool`
/// read with no heap allocation. Use it to guard `format!()` calls inside the
/// keyboard hook callback so they are skipped when logging is off.
///
/// In debug builds this always returns `true` after the first `log_debug` call.
/// In release builds it returns `true` only when `DOTXPANDER_LOG` (or legacy `RUST_EXPANDER_LOG`) is set.
#[inline]
pub fn is_debug_logging_enabled() -> bool {
    DEBUG_LOGGING_ENABLED.load(Ordering::Relaxed)
}

/// Writes a diagnostic message to debug.log in the config directory.
///
/// The file is opened lazily on first call and kept open. In release builds
/// this is a no-op unless the `DOTXPANDER_LOG` env-var is set.
///
/// **Do not call this directly from the keyboard hook hot path.** Use the
/// `debug_log!` macro in `hook.rs` which skips `format!` evaluation entirely
/// when logging is disabled, achieving true zero-allocation on the hot path.
pub fn log_debug(msg: &str) {
    // In release builds, skip logging unless explicitly enabled.
    // Cache the result in DEBUG_LOGGING_ENABLED so subsequent hot-path checks
    // are a single atomic load with no env-var string allocation.
    #[cfg(not(debug_assertions))]
    {
        use std::sync::atomic::AtomicU8;
        // 0 = uninitialised, 1 = enabled, 2 = disabled
        static RELEASE_STATE: AtomicU8 = AtomicU8::new(0);
        let state = RELEASE_STATE.load(Ordering::Relaxed);
        if state == 2 {
            return; // fast path: disabled, zero allocation
        }
        if state == 0 {
            let enabled = env::var("DOTXPANDER_LOG").is_ok() || env::var("RUST_EXPANDER_LOG").is_ok();
            RELEASE_STATE.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
            DEBUG_LOGGING_ENABLED.store(enabled, Ordering::Relaxed);
            if !enabled {
                return;
            }
        }
    }
    // In debug builds logging is always active — update the flag once so
    // is_debug_logging_enabled() returns true for the debug_log! macro.
    #[cfg(debug_assertions)]
    DEBUG_LOGGING_ENABLED.store(true, Ordering::Relaxed);

    let guard = LOG_FILE.get_or_init(|| {
        let path = config_dir().join("debug.log");
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
        Mutex::new(file)
    });

    if let Ok(mut lock) = guard.lock()
        && let Some(ref mut file) = *lock {
            let _ = writeln!(file, "{msg}");
            // Flush immediately so entries are visible even if the process dies.
            let _ = file.flush();
        }
}


/// Returns the default application configuration.
#[must_use]
pub fn default_config() -> AppConfig {
    AppConfig {
        language: "en".to_string(),
        hotkey: HotkeyConfig {
            modifiers: 6, // CTRL(2) | SHIFT(4)
            virtual_key: 0x54, // 'T'
        },
        buffer_size: 10,
        clipboard_restore_delay_ms: default_clipboard_restore_delay_ms(),
        quick_switch_enabled: true,
        snippet_hotkey_enabled: true,
        case_changer_enabled: true,
        case_changer_hotkey: default_case_changer_hotkey(),
        snippets: vec![
            Snippet {
                trigger: ".sig".to_string(),
                replacement: "Regards,\nJohn Doe".to_string(),
                mode: ExpansionMode::Immediate,
            },
            Snippet {
                trigger: ".em".to_string(),
                replacement: "john.doe@example.com".to_string(),
                mode: ExpansionMode::Immediate,
            },
            Snippet {
                trigger: "jd".to_string(),
                replacement: "With kind regards and sincerely - John Doe".to_string(),
                mode: ExpansionMode::Hotkey,
            },
        ],
    }
}

/// Loads the configuration from the config file.
///
/// Recovery strategy:
/// 1. If `config.toml` does not exist, create it from defaults.
/// 2. If `config.toml` exists but is corrupt/invalid, automatically fall back
///    to `config.toml.bak` (written after every successful save).
/// 3. `buffer_size` is clamped to 1..=256 regardless of what is in the file.
pub fn load() -> Result<AppConfig, ConfigError> {
    let path = config_path();

    if !path.exists() {
        let config = default_config();
        save(&config)?;
        return Ok(config);
    }

    let contents = fs::read_to_string(&path)?;
    match toml::from_str::<AppConfig>(&contents) {
        Ok(mut config) => {
            // Clamp buffer_size to a safe range — prevents both the
            // buffer_size=0 crash (usize underflow in pop/ends_with) and
            // absurdly large allocations from hand-edited config files.
            config.buffer_size = config.buffer_size.clamp(1, 256);
            Ok(config)
        }
        Err(primary_err) => {
            // config.toml is corrupt — try the backup before giving up.
            let bak = backup_path();
            if bak.exists() {
                eprintln!(
                    "[dotxpander] WARNING: config.toml is corrupt ({primary_err}). \
                     Trying config.toml.bak."
                );
                let bak_contents = fs::read_to_string(&bak)?;
                match toml::from_str::<AppConfig>(&bak_contents) {
                    Ok(mut config) => {
                        config.buffer_size = config.buffer_size.clamp(1, 256);
                        eprintln!("[dotxpander] INFO: Recovered config from backup.");
                        Ok(config)
                    }
                    Err(_) => Err(ConfigError::Toml(primary_err)),
                }
            } else {
                Err(ConfigError::Toml(primary_err))
            }
        }
    }
}

/// Saves the given configuration to the config file.
///
/// After a successful write, the file is also copied to `config.toml.bak`.
/// If config.toml ever becomes corrupt (power loss, disk full, etc.), `load()`
/// will automatically recover from the backup.
pub fn save(config: &AppConfig) -> Result<(), ConfigError> {
    let path = config_path();
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(&path, &toml_str)?;

    // Write backup — best-effort, never fails the save itself.
    let bak = backup_path();
    if let Err(e) = fs::copy(&path, &bak) {
        eprintln!("[dotxpander] WARNING: Could not write config backup: {e}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serialization_roundtrip() {
        let config = default_config();
        let toml_str = toml::to_string(&config).expect("Failed to serialize default config");
        let deserialized: AppConfig = toml::from_str(&toml_str).expect("Failed to deserialize default config");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_default_config_values() {
        let config = default_config();
        assert_eq!(config.language, "en");
        assert_eq!(config.hotkey.modifiers, 6);
        assert_eq!(config.hotkey.virtual_key, 0x54);
        assert_eq!(config.buffer_size, 10);
        assert_eq!(config.clipboard_restore_delay_ms, 150);
        assert!(config.quick_switch_enabled, "quick_switch_enabled should default to true");
        assert!(config.snippet_hotkey_enabled, "snippet_hotkey_enabled should default to true");
        assert_eq!(config.snippets.len(), 3);
        assert_eq!(config.snippets[0].trigger, ".sig");
        assert_eq!(config.snippets[0].mode, ExpansionMode::Immediate);
        assert_eq!(config.snippets[1].trigger, ".em");
        assert_eq!(config.snippets[1].mode, ExpansionMode::Immediate);
        assert_eq!(config.snippets[2].trigger, "jd");
        assert_eq!(config.snippets[2].mode, ExpansionMode::Hotkey);
    }

    // NEXT-5: Verify that completely invalid TOML is rejected cleanly.
    #[test]
    fn test_invalid_toml_is_rejected() {
        let bad_toml = "this is not [[valid] toml = {";
        let result: Result<AppConfig, _> = toml::from_str(bad_toml);
        assert!(result.is_err(), "Expected parse failure for invalid TOML");
    }

    // NEXT-5: Verify a minimal config (only mandatory fields) gets correct defaults
    // for all optional fields, confirming backward-compatibility with old config files
    // that predate newly added optional fields like clipboard_restore_delay_ms.
    #[test]
    fn test_minimal_config_applies_defaults() {
        let minimal = r#"
            buffer_size = 5
            [hotkey]
            modifiers = 5
            virtual_key = 88
        "#;
        let config: AppConfig = toml::from_str(minimal).expect("Minimal config should parse");
        assert_eq!(config.language, "en", "language should default to 'en'");
        assert_eq!(
            config.clipboard_restore_delay_ms, 150,
            "clipboard_restore_delay_ms should default to 150"
        );
        assert!(config.quick_switch_enabled, "quick_switch_enabled should default to true");
        assert!(config.snippet_hotkey_enabled, "snippet_hotkey_enabled should default to true");
        assert!(config.snippets.is_empty(), "snippets should default to empty");
    }

    // NEXT-5: Verify that invalid hotkey modifier values are stored as-is (they are u32
    // so the parser accepts any value; validation happens at the Win32 layer).
    #[test]
    fn test_hotkey_modifier_boundary_values() {
        let toml_str = r#"
            buffer_size = 64
            [hotkey]
            modifiers = 0
            virtual_key = 112
        "#;
        let config: AppConfig = toml::from_str(toml_str).expect("Zero-modifier config should parse");
        assert_eq!(config.hotkey.modifiers, 0);
        assert_eq!(config.hotkey.virtual_key, 112); // F1 VK code
    }

    // NEXT-5: Verify that a missing [hotkey] section fails gracefully (it is required).
    #[test]
    fn test_missing_hotkey_section_fails() {
        let toml_str = "buffer_size = 10";
        let result: Result<AppConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "Config without [hotkey] should fail");
    }

    // NEXT-5: Verify custom clipboard_restore_delay_ms survives a round-trip.
    #[test]
    fn test_custom_clipboard_delay_roundtrip() {
        let mut config = default_config();
        config.clipboard_restore_delay_ms = 300;
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let back: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert_eq!(back.clipboard_restore_delay_ms, 300);
    }

    // NEXT-5: Verify that an unknown expansion mode string causes a parse error.
    #[test]
    fn test_unknown_expansion_mode_is_rejected() {
        let toml_str = r#"
            buffer_size = 10
            [hotkey]
            modifiers = 5
            virtual_key = 88
            [[snippets]]
            trigger = ".test"
            replacement = "hello"
            mode = "turbo"
        "#;
        let result: Result<AppConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "Unknown expansion mode should fail to parse");
    }

    #[test]
    fn test_multiline_replacement_roundtrip() {
        let mut config = default_config();
        config.snippets = vec![Snippet {
            trigger: ".sig".to_string(),
            replacement: "Line one\nLine two\nLine three".to_string(),
            mode: ExpansionMode::Immediate,
        }];
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let back: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert_eq!(
            back.snippets[0].replacement,
            "Line one\nLine two\nLine three",
            "Newlines in replacement should survive TOML round-trip"
        );
    }

    #[test]
    fn test_buffer_size_clamp_zero() {
        // buffer_size=0 used to cause usize underflow in KeyBuffer::pop().
        // The load() function now clamps it to 1.
        let toml_str = r#"
            buffer_size = 0
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let mut config: AppConfig = toml::from_str(toml_str).expect("Should parse");
        config.buffer_size = config.buffer_size.clamp(1, 256);
        assert_eq!(config.buffer_size, 1, "buffer_size=0 should be clamped to 1");
    }

    #[test]
    fn test_buffer_size_clamp_huge() {
        let toml_str = r#"
            buffer_size = 99999
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let mut config: AppConfig = toml::from_str(toml_str).expect("Should parse");
        config.buffer_size = config.buffer_size.clamp(1, 256);
        assert_eq!(config.buffer_size, 256, "buffer_size=99999 should be clamped to 256");
    }

    #[test]
    fn test_buffer_size_default_when_missing() {
        // buffer_size is now optional with a default of 10.
        let toml_str = r#"
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let config: AppConfig = toml::from_str(toml_str).expect("Should parse without buffer_size");
        assert_eq!(config.buffer_size, 10, "buffer_size should default to 10");
    }

    #[test]
    fn test_quick_switch_enabled_roundtrip() {
        let mut config = default_config();
        config.quick_switch_enabled = true;
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let back: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert!(back.quick_switch_enabled, "quick_switch_enabled=true should survive TOML round-trip");
    }

    #[test]
    fn test_quick_switch_enabled_default_when_missing() {
        let toml_str = r#"
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let config: AppConfig = toml::from_str(toml_str).expect("Should parse without quick_switch_enabled");
        assert!(config.quick_switch_enabled, "quick_switch_enabled should default to true when omitted");
    }

    #[test]
    fn test_case_changer_enabled_roundtrip() {
        let mut config = default_config();
        config.case_changer_enabled = false;
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let back: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert!(!back.case_changer_enabled, "case_changer_enabled=false should survive TOML round-trip");
    }

    #[test]
    fn test_snippet_hotkey_enabled_default_when_missing() {
        let toml_str = r#"
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let config: AppConfig = toml::from_str(toml_str).expect("Should parse without snippet_hotkey_enabled");
        assert!(config.snippet_hotkey_enabled, "snippet_hotkey_enabled should default to true when omitted");
    }

    #[test]
    fn test_snippet_hotkey_enabled_roundtrip() {
        let mut config = default_config();
        config.snippet_hotkey_enabled = false;
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let back: AppConfig = toml::from_str(&toml_str).expect("Should deserialize");
        assert!(!back.snippet_hotkey_enabled, "snippet_hotkey_enabled=false should survive TOML round-trip");
    }

    #[test]
    fn test_case_changer_enabled_default_when_missing() {
        let toml_str = r#"
            [hotkey]
            modifiers = 6
            virtual_key = 84
        "#;
        let config: AppConfig = toml::from_str(toml_str).expect("Should parse without case_changer_enabled");
        assert!(config.case_changer_enabled, "case_changer_enabled should default to true when omitted");
    }

    #[test]
    fn test_config_error_display_registry() {
        let e = ConfigError::Registry(0x0000_0005);
        assert!(e.to_string().contains("Registry error"));
    }

    #[test]
    fn test_config_error_display_invalid_path() {
        let e = ConfigError::InvalidPath("bad path".to_string());
        assert!(e.to_string().contains("Invalid path"));
    }
}
