//! Small text-processing utilities used by the replacer and elsewhere.
//!
//! Lives in the library crate so the functions can be unit-tested by `cargo test`.

use unicode_segmentation::UnicodeSegmentation;

/// Normalises line endings in a string to CRLF (`\r\n`), as required by the
/// Win32 `CF_UNICODETEXT` clipboard format specification.
///
/// Any existing `\r\n` sequences are first collapsed to `\n` to prevent
/// double-conversion, then every remaining `\n` is replaced with `\r\n`.
#[must_use]
pub fn normalise_to_crlf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

/// Converts all characters in `text` to uppercase.
#[must_use]
pub fn to_uppercase(text: &str) -> String {
    text.to_uppercase()
}

/// Converts all characters in `text` to lowercase.
#[must_use]
pub fn to_lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// Capitalises the first letter of every whitespace-separated word.
///
/// Existing capitalisation inside words is preserved.
#[must_use]
pub fn to_title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalise_next = true;

    for ch in text.chars() {
        if ch.is_whitespace() {
            result.push(ch);
            capitalise_next = true;
        } else if capitalise_next {
            for upper in ch.to_uppercase() {
                result.push(upper);
            }
            capitalise_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Capitalises the first letter of each sentence.
///
/// Sentence boundaries are detected after `.`, `!`, and `?` (matching the
/// original AHK behaviour). Capitalises the very first letter of the string.
#[must_use]
pub fn to_sentence_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalise_next = true;
    let mut prev_was_terminator = false;

    for ch in text.chars() {
        if ch == '.' || ch == '!' || ch == '?' {
            result.push(ch);
            prev_was_terminator = true;
        } else if prev_was_terminator && ch.is_whitespace() {
            result.push(ch);
            capitalise_next = true;
            prev_was_terminator = false;
        } else if capitalise_next && !ch.is_whitespace() {
            for upper in ch.to_uppercase() {
                result.push(upper);
            }
            capitalise_next = false;
            prev_was_terminator = false;
        } else {
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_was_terminator = false;
        }
    }
    result
}

/// Normalises all line endings to CRLF (delegates to [`normalise_to_crlf`]).
///
/// Exposed as a named case-changer option so the menu can offer it uniformly
/// alongside the other transformations.
#[must_use]
pub fn fix_linebreaks(text: &str) -> String {
    normalise_to_crlf(text)
}

/// Removes all spaces and horizontal whitespace (spaces, tabs, non-breaking spaces) from `text`,
/// while preserving line breaks.
#[must_use]
pub fn remove_spaces(text: &str) -> String {
    text.chars()
        .filter(|&c| !(c.is_whitespace() && c != '\r' && c != '\n'))
        .collect()
}

/// Replaces all spaces and horizontal whitespace (spaces, tabs, non-breaking spaces) with `_`,
/// while preserving line breaks.
#[must_use]
pub fn replace_spaces_with_underscore(text: &str) -> String {
    replace_spaces_with_char(text, '_')
}

/// Replaces all spaces and horizontal whitespace (spaces, tabs, non-breaking spaces) with `-`,
/// while preserving line breaks.
#[must_use]
pub fn replace_spaces_with_dash(text: &str) -> String {
    replace_spaces_with_char(text, '-')
}

fn replace_spaces_with_char(text: &str, replacement: char) -> String {
    text.chars()
        .map(|c| {
            if c.is_whitespace() && c != '\r' && c != '\n' {
                replacement
            } else {
                c
            }
        })
        .collect()
}

/// Converts `text` to `lowerCamelCase`.
///
/// Words are split on whitespace, underscores, hyphens, and camelCase / `PascalCase`
/// boundaries (lowercase→uppercase transitions). The first word is all-lowercase;
/// subsequent words have their first grapheme uppercased.
#[must_use]
pub fn to_lower_camel_case(text: &str) -> String {
    let words = split_into_words(text);
    let mut result = String::new();
    for (i, word) in words.iter().enumerate() {
        if word.is_empty() {
            continue;
        }
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                for upper in first.to_uppercase() {
                    result.push(upper);
                }
                result.push_str(&chars.as_str().to_lowercase());
            }
        }
    }
    result
}

/// Converts `text` to `PascalCase`.
///
/// Words are split using the same rules as [`to_lower_camel_case`]. Every word
/// has its first grapheme uppercased.
#[must_use]
pub fn to_pascal_case(text: &str) -> String {
    let words = split_into_words(text);
    let mut result = String::new();
    for word in &words {
        if word.is_empty() {
            continue;
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            for upper in first.to_uppercase() {
                result.push(upper);
            }
            result.push_str(&chars.as_str().to_lowercase());
        }
    }
    result
}

/// Splits `text` into words by whitespace, underscores, hyphens, and
/// camelCase / `PascalCase` transitions (lowercase-to-uppercase boundary).
///
/// Uses `unicode-segmentation` for grapheme-cluster-aware boundary detection.
fn split_into_words(text: &str) -> Vec<&str> {
    // First pass: split on explicit delimiters (whitespace, _, -)
    let delimiter_words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == '_' || c == '-')
        .filter(|s| !s.is_empty())
        .collect();

    // Second pass: split each chunk on camelCase boundaries
    let mut words = Vec::new();
    for chunk in &delimiter_words {
        let graphemes: Vec<&str> = chunk.graphemes(true).collect();
        let mut start = 0;

        for i in 1..graphemes.len() {
            let prev_last = graphemes[i - 1].chars().last().unwrap_or(' ');
            let curr_first = graphemes[i].chars().next().unwrap_or(' ');

            // camelCase boundary: transition from lowercase to uppercase
            if prev_last.is_lowercase() && curr_first.is_uppercase() {
                // Reconstruct the byte slice covering graphemes[start..i]
                let start_byte = chunk
                    .grapheme_indices(true)
                    .nth(start)
                    .map_or(0, |(b, _)| b);
                let end_byte = chunk
                    .grapheme_indices(true)
                    .nth(i)
                    .map_or(chunk.len(), |(b, _)| b);
                let sub = &chunk[start_byte..end_byte];
                if !sub.is_empty() {
                    words.push(sub);
                }
                start = i;
            }
        }

        // Push the remainder
        let start_byte = chunk
            .grapheme_indices(true)
            .nth(start)
            .map_or(0, |(b, _)| b);
        let sub = &chunk[start_byte..];
        if !sub.is_empty() {
            words.push(sub);
        }
    }

    words
}

/// Transforms `text` to be safe for use as a Windows filename.
///
/// Rules:
/// - Windows-forbidden characters (`<`, `>`, `:`, `"`, `/`, `\`, `|`, `?`, `*`,
///   and ASCII control characters 0–31) are stripped completely.
/// - International and special characters across all languages (e.g. Danish `æ`,
///   `ø`, `å`) are preserved.
/// - Spaces are preserved. Consecutive whitespace and newlines (`\r`, `\n`) are
///   collapsed to a single space.
/// - Leading and trailing spaces and dots (`.`) are trimmed.
/// - Original letter casing is preserved.
/// - Truncated to 255 characters (Windows maximum filename component limit),
///   re-trimming any trailing dots or spaces at the boundary.
/// - If all characters are invalid or trimmed away, returns the localized prefix
///   (`"All chars are invalid: "` in English, `"Alle tegn er ugyldige: "` in Danish)
///   followed by `text`.
#[must_use]
pub fn to_windows_filename(text: &str, lang: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut filtered = String::with_capacity(text.len());
    let mut in_whitespace = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                filtered.push(' ');
                in_whitespace = true;
            }
        } else if is_windows_forbidden_char(ch) {
            // Strip forbidden character without altering whitespace state
        } else {
            filtered.push(ch);
            in_whitespace = false;
        }
    }

    let trimmed = filtered.trim_matches([' ', '.']);

    if trimmed.is_empty() {
        let prefix = crate::i18n::get_strings(lang).case_menu_all_chars_invalid;
        return format!("{prefix}{text}");
    }

    let mut result = if trimmed.chars().count() > 255 {
        let truncated: String = trimmed.chars().take(255).collect();
        truncated.trim_end_matches([' ', '.']).to_string()
    } else {
        trimmed.to_string()
    };

    if result.is_empty() {
        let prefix = crate::i18n::get_strings(lang).case_menu_all_chars_invalid;
        result = format!("{prefix}{text}");
    }

    result
}

fn is_windows_forbidden_char(c: char) -> bool {
    matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0'..='\x1f')
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // normalise_to_crlf
    // -------------------------------------------------------------------------

    #[test]
    fn test_lf_converted_to_crlf() {
        assert_eq!(normalise_to_crlf("hello\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_crlf_unchanged() {
        assert_eq!(normalise_to_crlf("hello\r\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_mixed_endings_normalised() {
        assert_eq!(
            normalise_to_crlf("a\r\nb\nc"),
            "a\r\nb\r\nc"
        );
    }

    #[test]
    fn test_no_newlines_unchanged() {
        assert_eq!(normalise_to_crlf("hello world"), "hello world");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalise_to_crlf(""), "");
    }

    // -------------------------------------------------------------------------
    // to_uppercase
    // -------------------------------------------------------------------------

    #[test]
    fn test_uppercase_basic() {
        assert_eq!(to_uppercase("hello world"), "HELLO WORLD");
    }

    #[test]
    fn test_uppercase_already_upper() {
        assert_eq!(to_uppercase("HELLO"), "HELLO");
    }

    #[test]
    fn test_uppercase_empty() {
        assert_eq!(to_uppercase(""), "");
    }

    #[test]
    fn test_uppercase_danish() {
        assert_eq!(to_uppercase("æøå"), "ÆØÅ");
    }

    #[test]
    fn test_uppercase_mixed() {
        assert_eq!(to_uppercase("Hello World THIS IS a TEST"), "HELLO WORLD THIS IS A TEST");
    }

    // -------------------------------------------------------------------------
    // to_lowercase
    // -------------------------------------------------------------------------

    #[test]
    fn test_lowercase_basic() {
        assert_eq!(to_lowercase("HELLO WORLD"), "hello world");
    }

    #[test]
    fn test_lowercase_already_lower() {
        assert_eq!(to_lowercase("hello"), "hello");
    }

    #[test]
    fn test_lowercase_empty() {
        assert_eq!(to_lowercase(""), "");
    }

    #[test]
    fn test_lowercase_danish() {
        assert_eq!(to_lowercase("ÆØÅ"), "æøå");
    }

    #[test]
    fn test_lowercase_mixed() {
        assert_eq!(to_lowercase("Hello World THIS IS a TEST"), "hello world this is a test");
    }

    // -------------------------------------------------------------------------
    // to_title_case
    // -------------------------------------------------------------------------

    #[test]
    fn test_title_case_basic() {
        assert_eq!(to_title_case("hello world"), "Hello World");
    }

    #[test]
    fn test_title_case_already_title() {
        assert_eq!(to_title_case("Hello World"), "Hello World");
    }

    #[test]
    fn test_title_case_empty() {
        assert_eq!(to_title_case(""), "");
    }

    #[test]
    fn test_title_case_single_char() {
        assert_eq!(to_title_case("h"), "H");
    }

    #[test]
    fn test_title_case_danish() {
        assert_eq!(to_title_case("hej verden"), "Hej Verden");
    }

    #[test]
    fn test_title_case_all_caps() {
        // Only first letter of each word changes; rest preserved as-is
        assert_eq!(to_title_case("HELLO WORLD"), "HELLO WORLD");
    }

    // -------------------------------------------------------------------------
    // to_sentence_case
    // -------------------------------------------------------------------------

    #[test]
    fn test_sentence_case_basic() {
        assert_eq!(to_sentence_case("hello world"), "Hello world");
    }

    #[test]
    fn test_sentence_case_multiple_sentences() {
        assert_eq!(
            to_sentence_case("hello world. this is a test! another sentence? yes."),
            "Hello world. This is a test! Another sentence? Yes."
        );
    }

    #[test]
    fn test_sentence_case_empty() {
        assert_eq!(to_sentence_case(""), "");
    }

    #[test]
    fn test_sentence_case_already_correct() {
        assert_eq!(to_sentence_case("Hello world."), "Hello world.");
    }

    #[test]
    fn test_sentence_case_all_caps() {
        assert_eq!(to_sentence_case("HELLO WORLD"), "Hello world");
    }

    #[test]
    fn test_sentence_case_danish() {
        assert_eq!(to_sentence_case("hej verden"), "Hej verden");
    }

    // -------------------------------------------------------------------------
    // fix_linebreaks
    // -------------------------------------------------------------------------

    #[test]
    fn test_fix_linebreaks_lf_to_crlf() {
        assert_eq!(fix_linebreaks("hello\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_fix_linebreaks_crlf_unchanged() {
        assert_eq!(fix_linebreaks("hello\r\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_fix_linebreaks_empty() {
        assert_eq!(fix_linebreaks(""), "");
    }

    // -------------------------------------------------------------------------
    // remove_spaces
    // -------------------------------------------------------------------------

    #[test]
    fn test_remove_spaces_basic() {
        assert_eq!(remove_spaces("hello world"), "helloworld");
    }

    #[test]
    fn test_remove_spaces_multiple_runs() {
        assert_eq!(remove_spaces("a   b   c"), "abc");
    }

    #[test]
    fn test_remove_spaces_trim_ends() {
        assert_eq!(remove_spaces("  hello  "), "hello");
    }

    #[test]
    fn test_remove_spaces_single_word() {
        assert_eq!(remove_spaces("hello"), "hello");
    }

    #[test]
    fn test_remove_spaces_empty() {
        assert_eq!(remove_spaces(""), "");
    }

    #[test]
    fn test_remove_spaces_tabs() {
        assert_eq!(remove_spaces("hello\t\tworld"), "helloworld");
    }

    #[test]
    fn test_remove_spaces_preserves_linebreaks() {
        assert_eq!(remove_spaces("hello world\r\nfoo bar"), "helloworld\r\nfoobar");
    }

    #[test]
    fn test_remove_spaces_danish() {
        assert_eq!(remove_spaces("dette er en test"), "detteerentest");
    }

    // -------------------------------------------------------------------------
    // replace_spaces_with_underscore
    // -------------------------------------------------------------------------

    #[test]
    fn test_replace_spaces_with_underscore_basic() {
        assert_eq!(replace_spaces_with_underscore("hello world"), "hello_world");
    }

    #[test]
    fn test_replace_spaces_with_underscore_multiple() {
        assert_eq!(replace_spaces_with_underscore("a   b   c"), "a___b___c");
    }

    #[test]
    fn test_replace_spaces_with_underscore_tabs() {
        assert_eq!(replace_spaces_with_underscore("hello\tworld"), "hello_world");
    }

    #[test]
    fn test_replace_spaces_with_underscore_preserves_linebreaks() {
        assert_eq!(
            replace_spaces_with_underscore("hello world\r\nfoo bar"),
            "hello_world\r\nfoo_bar"
        );
    }

    #[test]
    fn test_replace_spaces_with_underscore_empty() {
        assert_eq!(replace_spaces_with_underscore(""), "");
    }

    #[test]
    fn test_replace_spaces_with_underscore_danish() {
        assert_eq!(replace_spaces_with_underscore("dette er en test"), "dette_er_en_test");
    }

    // -------------------------------------------------------------------------
    // replace_spaces_with_dash
    // -------------------------------------------------------------------------

    #[test]
    fn test_replace_spaces_with_dash_basic() {
        assert_eq!(replace_spaces_with_dash("hello world"), "hello-world");
    }

    #[test]
    fn test_replace_spaces_with_dash_multiple() {
        assert_eq!(replace_spaces_with_dash("a   b   c"), "a---b---c");
    }

    #[test]
    fn test_replace_spaces_with_dash_tabs() {
        assert_eq!(replace_spaces_with_dash("hello\tworld"), "hello-world");
    }

    #[test]
    fn test_replace_spaces_with_dash_preserves_linebreaks() {
        assert_eq!(
            replace_spaces_with_dash("hello world\r\nfoo bar"),
            "hello-world\r\nfoo-bar"
        );
    }

    #[test]
    fn test_replace_spaces_with_dash_empty() {
        assert_eq!(replace_spaces_with_dash(""), "");
    }

    #[test]
    fn test_replace_spaces_with_dash_danish() {
        assert_eq!(replace_spaces_with_dash("dette er en test"), "dette-er-en-test");
    }

    // -------------------------------------------------------------------------
    // to_lower_camel_case
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_camel_basic() {
        assert_eq!(to_lower_camel_case("hello world"), "helloWorld");
    }

    #[test]
    fn test_lower_camel_single_word() {
        assert_eq!(to_lower_camel_case("hello"), "hello");
    }

    #[test]
    fn test_lower_camel_empty() {
        assert_eq!(to_lower_camel_case(""), "");
    }

    #[test]
    fn test_lower_camel_from_pascal() {
        // "HelloWorld" → ["Hello", "World"] → "helloWorld"
        assert_eq!(to_lower_camel_case("HelloWorld"), "helloWorld");
    }

    #[test]
    fn test_lower_camel_from_snake() {
        assert_eq!(to_lower_camel_case("hello_world"), "helloWorld");
    }

    #[test]
    fn test_lower_camel_from_kebab() {
        assert_eq!(to_lower_camel_case("hello-world"), "helloWorld");
    }

    #[test]
    fn test_lower_camel_multi_word() {
        assert_eq!(to_lower_camel_case("hello world this is a test"), "helloWorldThisIsATest");
    }

    #[test]
    fn test_lower_camel_all_caps_words() {
        assert_eq!(to_lower_camel_case("HELLO WORLD"), "helloWorld");
    }

    // -------------------------------------------------------------------------
    // to_pascal_case
    // -------------------------------------------------------------------------

    #[test]
    fn test_pascal_basic() {
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
    }

    #[test]
    fn test_pascal_single_word() {
        assert_eq!(to_pascal_case("hello"), "Hello");
    }

    #[test]
    fn test_pascal_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_pascal_from_camel() {
        // "helloWorld" → ["hello", "World"] → "HelloWorld"
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
    }

    #[test]
    fn test_pascal_from_snake() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
    }

    #[test]
    fn test_pascal_from_kebab() {
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
    }

    #[test]
    fn test_pascal_multi_word() {
        assert_eq!(to_pascal_case("hello world this is a test"), "HelloWorldThisIsATest");
    }

    #[test]
    fn test_pascal_all_caps_words() {
        assert_eq!(to_pascal_case("HELLO WORLD"), "HelloWorld");
    }

    // -------------------------------------------------------------------------
    // to_windows_filename
    // -------------------------------------------------------------------------

    #[test]
    fn test_windows_filename_basic() {
        assert_eq!(to_windows_filename("hello world", "en"), "hello world");
    }

    #[test]
    fn test_windows_filename_strips_forbidden_chars() {
        assert_eq!(
            to_windows_filename("Report: 2026/Q1 *Final* <draft>? | \\ \"", "en"),
            "Report 2026Q1 Final draft"
        );
    }

    #[test]
    fn test_windows_filename_strips_control_chars() {
        assert_eq!(
            to_windows_filename("hello\x01\x1fworld", "en"),
            "helloworld"
        );
    }

    #[test]
    fn test_windows_filename_collapses_whitespace_and_newlines() {
        assert_eq!(
            to_windows_filename("hello \r\n\t  world\nfoo", "en"),
            "hello world foo"
        );
    }

    #[test]
    fn test_windows_filename_trims_leading_trailing_spaces_and_dots() {
        assert_eq!(
            to_windows_filename("  ...hello world...  ", "en"),
            "hello world"
        );
    }

    #[test]
    fn test_windows_filename_preserves_internal_dots() {
        assert_eq!(
            to_windows_filename("archive.tar.gz", "en"),
            "archive.tar.gz"
        );
    }

    #[test]
    fn test_windows_filename_preserves_danish_and_international() {
        assert_eq!(
            to_windows_filename("Danish æ, ø og å - Ä, ö, ü & 日本語", "en"),
            "Danish æ, ø og å - Ä, ö, ü & 日本語"
        );
    }

    #[test]
    fn test_windows_filename_preserves_casing() {
        assert_eq!(
            to_windows_filename("MyCamelCaseDocument", "en"),
            "MyCamelCaseDocument"
        );
    }

    #[test]
    fn test_windows_filename_all_invalid_en() {
        assert_eq!(
            to_windows_filename(":::*?<>", "en"),
            "All chars are invalid: :::*?<>"
        );
    }

    #[test]
    fn test_windows_filename_all_invalid_da() {
        assert_eq!(
            to_windows_filename(":::*?<>", "da"),
            "Alle tegn er ugyldige: :::*?<>"
        );
    }

    #[test]
    fn test_windows_filename_empty() {
        assert_eq!(to_windows_filename("", "en"), "");
    }

    #[test]
    fn test_windows_filename_truncates_to_255() {
        let long_name = "a".repeat(300);
        let result = to_windows_filename(&long_name, "en");
        assert_eq!(result.chars().count(), 255);
        assert_eq!(result, "a".repeat(255));
    }

    #[test]
    fn test_windows_filename_truncation_re_trims_dots_and_spaces() {
        let long_name = format!("{}. ...", "a".repeat(254));
        let result = to_windows_filename(&long_name, "en");
        assert_eq!(result, "a".repeat(254));
    }
}
