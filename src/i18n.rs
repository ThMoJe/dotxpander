//! Internationalization (i18n) support.
//! Currently supports English ("en") and Danish ("da").

/// All translatable UI strings for the application.
#[derive(Debug, Clone)]
pub struct Strings {
    // Window
    pub window_title: &'static str,
    pub header: &'static str,

    // Hotkey section
    pub hotkey_label: &'static str,
    pub hotkey_save: &'static str,
    pub hotkey_prompt: &'static str,

    // Buffer
    pub buffer_label: &'static str,
    pub buffer_empty: &'static str,

    // Snippet table
    pub col_trigger: &'static str,
    pub col_trigger_tooltip: &'static str,
    pub col_replacement: &'static str,
    pub col_mode: &'static str,
    pub mode_immediate: &'static str,
    pub mode_hotkey: &'static str,
    pub btn_delete: &'static str,
    pub btn_add: &'static str,

    // Bottom buttons
    pub btn_quit: &'static str,
    pub btn_pause: &'static str,
    pub btn_resume: &'static str,
    pub btn_pause_tooltip: &'static str,
    pub btn_cancel: &'static str,
    pub btn_save: &'static str,

    // Tray
    pub tray_tooltip: &'static str,
    pub tray_open: &'static str,
    pub tray_quit: &'static str,

    // Validation errors
    pub err_needs_mod: &'static str,
    pub err_ctrl_reserved: &'static str,
    pub err_sys_reserved: &'static str,
    pub err_win_reserved: &'static str,
    pub err_conflict: &'static str,

    // Uninstall confirmation dialog
    pub uninstall_btn: &'static str,
    pub uninstall_tooltip: &'static str,
    pub uninstall_title: &'static str,
    pub uninstall_body: &'static str,

    // Cancel button tooltip
    pub btn_cancel_tooltip: &'static str,

    // Snippet hotkey tooltip
    pub hotkey_label_tooltip: &'static str,
    pub hotkey_reset_tooltip: &'static str,
    pub case_changer_reset_tooltip: &'static str,

    // Quick Switch
    pub quick_switch_label: &'static str,
    pub quick_switch_tooltip: &'static str,

    // Case Changer menu items
    pub case_menu_uppercase: &'static str,
    pub case_menu_lowercase: &'static str,
    pub case_menu_title_case: &'static str,
    pub case_menu_sentence_case: &'static str,
    pub case_menu_fix_linebreaks: &'static str,
    pub case_menu_spaces_submenu: &'static str,
    pub case_menu_remove_spaces: &'static str,
    pub case_menu_space_to_underscore: &'static str,
    pub case_menu_space_to_dash: &'static str,
    pub case_menu_lower_camel: &'static str,
    pub case_menu_pascal_case: &'static str,
    pub case_menu_windows_filename: &'static str,
    pub case_menu_all_chars_invalid: &'static str,

    // Case Changer settings UI
    pub case_changer_label: &'static str,
    pub case_changer_tooltip: &'static str,

    // Settings tab labels
    pub tab_general: &'static str,
    pub tab_snippets: &'static str,

    // General tab section headers
    pub section_hotkey: &'static str,
    pub section_features: &'static str,

    // Config location mode badge and move button
    pub mode_portable: &'static str,
    pub mode_installed: &'static str,
    pub move_config_btn: &'static str,
    #[allow(dead_code)]
    pub move_config_tooltip: &'static str,
    pub mode_portable_tooltip: &'static str,

    // About tab
    pub tab_about: &'static str,
    pub about_tagline: &'static str,
    pub about_developed_by: &'static str,
    pub about_btn_website: &'static str,
    pub about_btn_github: &'static str,
    pub about_license: &'static str,
    pub about_disclaimer_title: &'static str,
    pub about_disclaimer_text: &'static str,
}

/// Returns the UI strings for the given language code.
/// Falls back to English for unknown languages.
#[must_use]
pub fn get_strings(lang: &str) -> Strings {
    match lang {
        "da" => STRINGS_DA,
        _ => STRINGS_EN,
    }
}

const STRINGS_EN: Strings = Strings {
    window_title: ".XPANDER - Settings",
    header: "Settings",
    hotkey_label: "Snippet hotkey:",
    hotkey_save: "Set",
    hotkey_prompt: "Press new key combination",
    buffer_label: "Buffer:",
    buffer_empty: "(empty)",
    col_trigger: "Trigger (max 10)",
    col_trigger_tooltip: "When you type any of these character sequences they will be replaced either immediately or when you press the set Hotkey",
    col_replacement: "Replacement (trailing space?)",
    col_mode: "Mode",
    mode_immediate: "⚡ Immediate",
    mode_hotkey: "⌨ Hotkey",
    btn_delete: "Delete",
    btn_add: "+ Add new",
    btn_quit: "Quit",
    btn_pause: "Pause",
    btn_resume: "Resume",
    btn_pause_tooltip: "Temporarily pause replacing text you type",
    btn_cancel: "Cancel",
    btn_save: "Save",
    tray_tooltip: ".XPANDER\nClick for settings",
    tray_open: "Open settings",
    tray_quit: "Quit",
    err_needs_mod: "Invalid: Requires Ctrl/Alt/Shift/Win",
    err_ctrl_reserved: "Invalid: Single Ctrl is reserved",
    err_sys_reserved: "Invalid: System-reserved shortcut",
    err_win_reserved: "Invalid: Windows-reserved shortcut",
    err_conflict: "Conflict: Already in use by another app",
    uninstall_btn: "\u{2620}",
    uninstall_tooltip: "Uninstall and delete app and its files",
    uninstall_title: "Uninstall .XPANDER",
    uninstall_body: "This will permanently delete:\r\n\r\n  \u{2022} All settings and snippets\r\n  \u{2022} The application .exe file\r\n  \u{2022} The debug log\r\n\r\nThe app closes immediately. The .exe is removed a moment later.\r\n\r\nThis cannot be undone. Proceed?",
    btn_cancel_tooltip: "Discard all changes since last save",
    hotkey_label_tooltip: "When enabled, pressing the hotkey expands non-immediate snippets in any app",
    hotkey_reset_tooltip: "Reset to default: CTRL+SHIFT+T",
    case_changer_reset_tooltip: "Reset to default: CTRL+CAPSLOCK",
    quick_switch_label: "Open dialog to follow Explorer path",
    quick_switch_tooltip: "When enabled and in an Open/Save dialog: switch to Explorer, select a folder or file, return to the dialog — it will automatically navigate to that folder.",
    case_menu_uppercase: "&UPPERCASE",
    case_menu_lowercase: "&lowercase",
    case_menu_title_case: "&Title Case",
    case_menu_sentence_case: "Se&ntence case",
    case_menu_fix_linebreaks: "&Fix Linebreaks",
    case_menu_spaces_submenu: "&Spaces",
    case_menu_remove_spaces: "&Remove spaces",
    case_menu_space_to_underscore: "Replace space with _",
    case_menu_space_to_dash: "Replace space with -",
    case_menu_lower_camel: "lower&CamelCase",
    case_menu_pascal_case: "&PascalCase",
    case_menu_windows_filename: "Windows &filename",
    case_menu_all_chars_invalid: "All chars are invalid: ",
    case_changer_label: "Case/Space Changer:",
    case_changer_tooltip: "When enabled the Hotkey will open a change menu on any selected text in any program",
    tab_general: "\u{2699} General",
    tab_snippets: "\u{1F4DD} Snippets",
    section_hotkey: "Hotkeys",
    section_features: "Features",
    mode_portable: "\u{1F4E6} Portable",
    mode_installed: "\u{1F4BF} Installed",
    move_config_btn: "Move Config File\u{2026}",
    move_config_tooltip: "Move config file to a new folder (e.g., a cloud-synced folder to share settings across computers)",
    mode_portable_tooltip: "The app is running in Portable mode and you cannot change the folder where the config file is stored. To store the config file in a cloud-synced folder, install the app with the installer.",
    tab_about: "\u{2139} About",
    about_tagline: "Lightweight, native Windows text expander and productivity utility",
    about_developed_by: "Developed by",
    about_btn_website: "User Guide & FAQ",
    about_btn_github: "GitHub",
    about_license: "MIT License",
    about_disclaimer_title: "Terms & Conditions / Disclaimer",
    about_disclaimer_text: "Provided 'as is' without warranty of any kind, express or implied. In no event shall aiVOLUTION or the authors be liable for any claim, damages, or other liability resulting from the use of this software.",
};

const STRINGS_DA: Strings = Strings {
    window_title: ".XPANDER - Indstillinger",
    header: "Indstillinger",
    hotkey_label: "Tekstklip genvej:",
    hotkey_save: "Sæt",
    hotkey_prompt: "Tast ny taste-kombination",
    buffer_label: "Buffer:",
    buffer_empty: "(tom)",
    col_trigger: "Sekvens (max 10)",
    col_trigger_tooltip: "Når du taster en af disse tegnsekvenser, erstattes de enten med det samme eller når du trykker på den valgte genvejstast",
    col_replacement: "Erstatning (mellemrum til sidst?)",
    col_mode: "Mode",
    mode_immediate: "⚡ Omgående",
    mode_hotkey: "⌨ Genvej",
    btn_delete: "Slet",
    btn_add: "+ Tilføj ny",
    btn_quit: "Afslut",
    btn_pause: "Pause",
    btn_resume: "Genoptag",
    btn_pause_tooltip: "Sæt teksterstatning midlertidigt på pause",
    btn_cancel: "Annuller",
    btn_save: "Gem",
    tray_tooltip: ".XPANDER\nKlik for indstillinger",
    tray_open: "Åbn indstillinger",
    tray_quit: "Afslut",
    err_needs_mod: "Ugyldig: Kræver Ctrl/Alt/Shift/Win",
    err_ctrl_reserved: "Ugyldig: Enkelt Ctrl er reserveret",
    err_sys_reserved: "Ugyldig: System-reserveret genvej",
    err_win_reserved: "Ugyldig: Windows-reserveret genvej",
    err_conflict: "Konflikt: Allerede i brug af anden app",
    uninstall_btn: "\u{2620}",
    uninstall_tooltip: "Afinstaller og slet app og dens filer",
    uninstall_title: "Afinstaller .XPANDER",
    uninstall_body: "Dette vil permanent slette:\r\n\r\n  \u{2022} Alle indstillinger og genvejstekster\r\n  \u{2022} Applikationens .exe-fil\r\n  \u{2022} Debug-loggen\r\n\r\nAppen lukker med det samme. .exe-filen fjernes kort efter.\r\n\r\nDette kan ikke fortrydes. Forts\u{00E6}t?",
    btn_cancel_tooltip: "Fortryd alle ændringer siden sidste gem",
    hotkey_label_tooltip: "Når aktiveret indsætter genvejstast ikke-automatiske tekstklip i et hvilket som helst program",
    hotkey_reset_tooltip: "Nulstil til standard: CTRL+SHIFT+T",
    case_changer_reset_tooltip: "Nulstil til standard: CTRL+CAPSLOCK",
    quick_switch_label: "Åbn dialogboks følger sti i Stifinder",
    quick_switch_tooltip: "Når aktiveret og i en Åbn/Gem-dialog, skift til Stifinder, vælg en mappe, vend tilbage til åbn dialogboksen og den skifter automatisk til åbn i stifinder.",
    case_menu_uppercase: "&STORE BOGSTAVER",
    case_menu_lowercase: "s&må bogstaver",
    case_menu_title_case: "S&tort Forbogstav",
    case_menu_sentence_case: "Sæt&nings-format",
    case_menu_fix_linebreaks: "&Fix Linjeskift",
    case_menu_spaces_submenu: "&Mellemrum",
    case_menu_remove_spaces: "&Fjern mellemrum",
    case_menu_space_to_underscore: "Erstat mellemrum med _",
    case_menu_space_to_dash: "Erstat mellemrum med -",
    case_menu_lower_camel: "lower&CamelCase",
    case_menu_pascal_case: "&PascalCase",
    case_menu_windows_filename: "Windows &filnavn",
    case_menu_all_chars_invalid: "Alle tegn er ugyldige: ",
    case_changer_label: "Kapitalisering/Mellemrums Skifter:",
    case_changer_tooltip: "Når aktiv åbner genvejstaste en ændringsmenu på den markerede tekst i et hvilket som helst program.",
    tab_general: "\u{2699} Generelt",
    tab_snippets: "\u{1F4DD} Tekstklip",
    section_hotkey: "Genveje",
    section_features: "Funktioner",
    mode_portable: "\u{1F4E6} Portabel",
    mode_installed: "\u{1F4BF} Installeret",
    move_config_btn: "Flyt konfigurationsfil\u{2026}",
    move_config_tooltip: "Flyt konfigurationsfilen til en ny mappe (fx en cloud-synkroniseret mappe for at dele indstillinger p\u{00E5} tv\u{00E6}rs af computere)",
    mode_portable_tooltip: "Appen k\u{00F8}rer i Portabel tilstand og du kan ikke ændre mappen, hvor konfigurationsfilen gemmes. For at gemme konfigurationsfilen i en cloud-synkroniseret mappe skal du installere appen med installationsprogrammet.",
    tab_about: "\u{2139} Om",
    about_tagline: "Hurtig og let native Windows tekstudvider og produktivitetsv\u{00E6}rkt\u{00F8}j",
    about_developed_by: "Udviklet af",
    about_btn_website: "User Guide & FAQ",
    about_btn_github: "GitHub",
    about_license: "MIT-licens",
    about_disclaimer_title: "Vilk\u{00E5}r & Betingelser / Ansvarsfraskrivelse",
    about_disclaimer_text: "Leveres 'som den er og forefindes' uden nogen form for garanti, hverken udtrykkelig eller stiltiende. Under ingen omst\u{00E6}ndigheder kan aiVOLUTION eller forfatterne drages til ansvar for eventuelle krav, skader eller andet ansvar som f\u{00F8}lge af brugen af denne software.",
};

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_all_fields_non_empty(s: &Strings, lang: &str) {
        assert!(!s.window_title.is_empty(), "window_title empty for {lang}");
        assert!(!s.header.is_empty(), "header empty for {lang}");
        assert!(!s.hotkey_label.is_empty(), "hotkey_label empty for {lang}");
        assert!(!s.hotkey_save.is_empty(), "hotkey_save empty for {lang}");
        assert!(!s.hotkey_prompt.is_empty(), "hotkey_prompt empty for {lang}");
        assert!(!s.buffer_label.is_empty(), "buffer_label empty for {lang}");
        assert!(!s.buffer_empty.is_empty(), "buffer_empty empty for {lang}");
        assert!(!s.col_trigger.is_empty(), "col_trigger empty for {lang}");
        assert!(!s.col_trigger_tooltip.is_empty(), "col_trigger_tooltip empty for {lang}");
        assert!(!s.col_replacement.is_empty(), "col_replacement empty for {lang}");
        assert!(!s.col_mode.is_empty(), "col_mode empty for {lang}");
        assert!(!s.mode_immediate.is_empty(), "mode_immediate empty for {lang}");
        assert!(!s.mode_hotkey.is_empty(), "mode_hotkey empty for {lang}");
        assert!(!s.btn_delete.is_empty(), "btn_delete empty for {lang}");
        assert!(!s.btn_add.is_empty(), "btn_add empty for {lang}");
        assert!(!s.btn_quit.is_empty(), "btn_quit empty for {lang}");
        assert!(!s.btn_pause.is_empty(), "btn_pause empty for {lang}");
        assert!(!s.btn_resume.is_empty(), "btn_resume empty for {lang}");
        assert!(!s.btn_pause_tooltip.is_empty(), "btn_pause_tooltip empty for {lang}");
        assert!(!s.btn_cancel.is_empty(), "btn_cancel empty for {lang}");
        assert!(!s.btn_save.is_empty(), "btn_save empty for {lang}");
        assert!(!s.tray_tooltip.is_empty(), "tray_tooltip empty for {lang}");
        assert!(!s.tray_open.is_empty(), "tray_open empty for {lang}");
        assert!(!s.tray_quit.is_empty(), "tray_quit empty for {lang}");
        assert!(!s.err_needs_mod.is_empty(), "err_needs_mod empty for {lang}");
        assert!(!s.err_ctrl_reserved.is_empty(), "err_ctrl_reserved empty for {lang}");
        assert!(!s.err_sys_reserved.is_empty(), "err_sys_reserved empty for {lang}");
        assert!(!s.err_win_reserved.is_empty(), "err_win_reserved empty for {lang}");
        assert!(!s.err_conflict.is_empty(), "err_conflict empty for {lang}");
        assert!(!s.uninstall_btn.is_empty(), "uninstall_btn empty for {lang}");
        assert!(!s.uninstall_tooltip.is_empty(), "uninstall_tooltip empty for {lang}");
        assert!(!s.uninstall_title.is_empty(), "uninstall_title empty for {lang}");
        assert!(!s.uninstall_body.is_empty(), "uninstall_body empty for {lang}");
        assert!(!s.btn_cancel_tooltip.is_empty(), "btn_cancel_tooltip empty for {lang}");
        assert!(!s.hotkey_label_tooltip.is_empty(), "hotkey_label_tooltip empty for {lang}");
        assert!(!s.hotkey_reset_tooltip.is_empty(), "hotkey_reset_tooltip empty for {lang}");
        assert!(!s.case_changer_reset_tooltip.is_empty(), "case_changer_reset_tooltip empty for {lang}");
        assert!(!s.quick_switch_label.is_empty(), "quick_switch_label empty for {lang}");
        assert!(!s.quick_switch_tooltip.is_empty(), "quick_switch_tooltip empty for {lang}");
        assert!(!s.case_menu_uppercase.is_empty(), "case_menu_uppercase empty for {lang}");
        assert!(!s.case_menu_lowercase.is_empty(), "case_menu_lowercase empty for {lang}");
        assert!(!s.case_menu_title_case.is_empty(), "case_menu_title_case empty for {lang}");
        assert!(!s.case_menu_sentence_case.is_empty(), "case_menu_sentence_case empty for {lang}");
        assert!(!s.case_menu_fix_linebreaks.is_empty(), "case_menu_fix_linebreaks empty for {lang}");
        assert!(!s.case_menu_spaces_submenu.is_empty(), "case_menu_spaces_submenu empty for {lang}");
        assert!(!s.case_menu_remove_spaces.is_empty(), "case_menu_remove_spaces empty for {lang}");
        assert!(!s.case_menu_space_to_underscore.is_empty(), "case_menu_space_to_underscore empty for {lang}");
        assert!(!s.case_menu_space_to_dash.is_empty(), "case_menu_space_to_dash empty for {lang}");
        assert!(!s.case_menu_lower_camel.is_empty(), "case_menu_lower_camel empty for {lang}");
        assert!(!s.case_menu_pascal_case.is_empty(), "case_menu_pascal_case empty for {lang}");
        assert!(!s.case_menu_windows_filename.is_empty(), "case_menu_windows_filename empty for {lang}");
        assert!(!s.case_menu_all_chars_invalid.is_empty(), "case_menu_all_chars_invalid empty for {lang}");
        assert!(!s.case_changer_label.is_empty(), "case_changer_label empty for {lang}");
        assert!(!s.case_changer_tooltip.is_empty(), "case_changer_tooltip empty for {lang}");
        assert!(!s.tab_general.is_empty(), "tab_general empty for {lang}");
        assert!(!s.tab_snippets.is_empty(), "tab_snippets empty for {lang}");
        assert!(!s.section_hotkey.is_empty(), "section_hotkey empty for {lang}");
        assert!(!s.section_features.is_empty(), "section_features empty for {lang}");
        assert!(!s.mode_portable.is_empty(), "mode_portable empty for {lang}");
        assert!(!s.mode_installed.is_empty(), "mode_installed empty for {lang}");
        assert!(!s.move_config_btn.is_empty(), "move_config_btn empty for {lang}");
        assert!(!s.move_config_tooltip.is_empty(), "move_config_tooltip empty for {lang}");
        assert!(!s.mode_portable_tooltip.is_empty(), "mode_portable_tooltip empty for {lang}");
        assert!(!s.tab_about.is_empty(), "tab_about empty for {lang}");
        assert!(!s.about_tagline.is_empty(), "about_tagline empty for {lang}");
        assert!(!s.about_developed_by.is_empty(), "about_developed_by empty for {lang}");
        assert!(!s.about_btn_website.is_empty(), "about_btn_website empty for {lang}");
        assert!(!s.about_btn_github.is_empty(), "about_btn_github empty for {lang}");
        assert!(!s.about_license.is_empty(), "about_license empty for {lang}");
        assert!(!s.about_disclaimer_title.is_empty(), "about_disclaimer_title empty for {lang}");
        assert!(!s.about_disclaimer_text.is_empty(), "about_disclaimer_text empty for {lang}");
    }

    #[test]
    fn test_english_strings_fully_populated() {
        let strings = get_strings("en");
        assert_all_fields_non_empty(&strings, "en");
        assert_eq!(strings.window_title, ".XPANDER - Settings");
    }

    #[test]
    fn test_danish_strings_fully_populated() {
        let strings = get_strings("da");
        assert_all_fields_non_empty(&strings, "da");
        assert_eq!(strings.window_title, ".XPANDER - Indstillinger");
    }

    #[test]
    fn test_unknown_language_falls_back_to_english() {
        let fallback = get_strings("xyz_unknown");
        assert_eq!(fallback.window_title, STRINGS_EN.window_title);
        assert_eq!(fallback.header, STRINGS_EN.header);
        assert_eq!(fallback.quick_switch_label, STRINGS_EN.quick_switch_label);
    }

    #[test]
    fn test_danish_strings_contain_danish_characters() {
        let da = get_strings("da");
        // Verify genuine Danish characters (æ, ø, å / Æ, Ø, Å) are present
        assert!(da.tray_open.contains('Å'), "Expected Å in tray_open");
        assert!(da.btn_cancel_tooltip.contains('æ'), "Expected æ in btn_cancel_tooltip");
        assert!(da.btn_add.contains('ø'), "Expected ø in btn_add");
        assert!(da.quick_switch_tooltip.contains('Å'), "Expected Å in quick_switch_tooltip");
        assert!(da.about_tagline.contains('æ'), "Expected æ in about_tagline");
        assert!(da.about_disclaimer_title.contains('å'), "Expected å in about_disclaimer_title");
    }

    #[test]
    fn test_about_tab_strings_content() {
        let en = get_strings("en");
        assert!(en.tab_about.contains("About"));
        assert_eq!(en.about_btn_website, "User Guide & FAQ");
        assert_eq!(en.about_btn_github, "GitHub");
        assert_eq!(en.about_license, "MIT License");
        assert!(en.about_disclaimer_text.contains("aiVOLUTION"));
        assert!(en.about_disclaimer_text.contains("without warranty"));

        let da = get_strings("da");
        assert!(da.tab_about.contains("Om"));
        assert_eq!(da.about_btn_website, "User Guide & FAQ");
        assert_eq!(da.about_btn_github, "GitHub");
        assert_eq!(da.about_license, "MIT-licens");
        assert!(da.about_disclaimer_text.contains("aiVOLUTION"));
        assert!(da.about_disclaimer_text.contains("uden nogen form for garanti"));
    }
}

