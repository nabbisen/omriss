//! Startup locale detection (RFC-043; Task 013).
//!
//! `detect_locale()` composes two pieces that stay deliberately separate:
//! `os_locale_tag()`, a thin, platform-specific, untested-by-design shim
//! that asks the OS for a single best-guess BCP 47 tag, and
//! `locale_from_tag_or_default()`, the actual mapping/fallback logic, which
//! has no platform dependency at all and is tested unconditionally — on
//! every host this crate builds on, not just the one it happens to run on.

use omriss_ui::i18n::Locale;

/// Detects the startup locale: the OS's preferred UI language if it maps to
/// a locale this build ships, else English. There is no persisted explicit
/// user locale choice today (`AppSettings` carries no such field) for this
/// to take precedence over — RFC-043's "explicit setting if present" step
/// has nothing to read yet.
pub(crate) fn detect_locale() -> Locale {
    locale_from_tag_or_default(os_locale_tag())
}

fn locale_from_tag_or_default(tag: Option<String>) -> Locale {
    tag.and_then(|t| Locale::from_tag(&t)).unwrap_or_default()
}

/// The OS's preferred UI language as a BCP 47 tag (e.g. `"ja-JP"`), if one
/// can be determined. `None` is a normal outcome (`detect_locale` falls
/// back to English), not an error to report.
#[cfg(windows)]
fn os_locale_tag() -> Option<String> {
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
    use windows::core::PWSTR;

    let mut num_languages: u32 = 0;
    let mut buffer_len: u32 = 0;
    // SAFETY: passing no buffer just asks Windows how large one needs to
    // be, in `buffer_len`; no pointer from this call is ever dereferenced.
    unsafe {
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut num_languages, None, &mut buffer_len)
            .ok()?;
    }
    if buffer_len == 0 {
        return None;
    }

    let mut buffer = vec![0u16; buffer_len as usize];
    // SAFETY: `buffer` is sized to exactly `buffer_len`, the size Windows
    // itself just reported for this same call shape; the `PWSTR` is valid
    // for the duration of this call only, and the call fills at most
    // `buffer_len` code units into it.
    unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut num_languages,
            Some(PWSTR(buffer.as_mut_ptr())),
            &mut buffer_len,
        )
        .ok()?;
    }

    // The buffer is a MULTI_SZ: each language name is NUL-terminated, and an
    // extra NUL terminates the whole list. Only the first (most preferred)
    // tag is wanted here.
    let first_len = buffer.iter().position(|&c| c == 0)?;
    if first_len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..first_len]))
}

/// Detects the OS locale from environment variables (unchanged from the
/// pre-Task-013 `detect_locale`, moved rather than rewritten): `LC_ALL`,
/// then `LC_MESSAGES`, then `LANG`, in priority order, skipping any that
/// don't map to a locale this build ships rather than stopping at the first
/// one merely *set*.
#[cfg(not(windows))]
fn os_locale_tag() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|var| std::env::var(var).ok())
        .find(|tag| Locale::from_tag(tag).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_shaped_tags_map_to_their_locale() {
        assert_eq!(
            locale_from_tag_or_default(Some("ja-JP".to_string())),
            Locale::Ja
        );
        assert_eq!(
            locale_from_tag_or_default(Some("en-US".to_string())),
            Locale::En
        );
    }

    #[test]
    fn an_unsupported_language_falls_back_to_english() {
        assert_eq!(
            locale_from_tag_or_default(Some("fr-FR".to_string())),
            Locale::En
        );
    }

    #[test]
    fn no_tag_falls_back_to_english() {
        assert_eq!(locale_from_tag_or_default(None), Locale::En);
    }
}
