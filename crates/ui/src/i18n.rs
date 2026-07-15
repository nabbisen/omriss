//! GUI internationalization (RFC-043).
//!
//! Catalogs are static key→string tables compiled into the binary. Lookup
//! falls back in this order: requested locale → English → the key itself.
//! `omriss-core` stays locale-free; only GUI strings live here.

mod en;
mod ja;

/// Locales shipped with the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    /// English (fallback locale).
    #[default]
    En,
    /// Japanese.
    Ja,
}

impl Locale {
    /// All locales the GUI can render, in menu order.
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Ja];

    /// BCP 47 language tag.
    pub fn tag(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ja => "ja",
        }
    }

    /// Native display name, used by the language switcher.
    pub fn native_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Ja => "日本語",
        }
    }

    /// Parses a language tag (or its primary subtag) into a supported locale.
    pub fn from_tag(tag: &str) -> Option<Locale> {
        let primary = tag.split(['-', '_']).next().unwrap_or(tag);
        match primary.to_ascii_lowercase().as_str() {
            "en" => Some(Locale::En),
            "ja" => Some(Locale::Ja),
            _ => None,
        }
    }

    fn catalog(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Locale::En => en::CATALOG,
            Locale::Ja => ja::CATALOG,
        }
    }
}

fn lookup(catalog: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    catalog
        .binary_search_by(|(entry, _)| entry.cmp(&key))
        .ok()
        .map(|found| catalog[found].1)
}

/// Translates `key` for `locale`.
///
/// Missing keys fall back to English; keys absent there too are returned
/// verbatim so the GUI never renders an empty label (RFC-043 §4).
pub fn t<'key>(locale: Locale, key: &'key str) -> &'key str
where
    'static: 'key,
{
    lookup(locale.catalog(), key)
        .or_else(|| lookup(Locale::En.catalog(), key))
        .unwrap_or(key)
}

#[cfg(test)]
pub(crate) fn catalogs() -> &'static [(Locale, &'static [(&'static str, &'static str)])] {
    static ALL: [(Locale, &[(&str, &str)]); 2] =
        [(Locale::En, en::CATALOG), (Locale::Ja, ja::CATALOG)];
    &ALL
}
