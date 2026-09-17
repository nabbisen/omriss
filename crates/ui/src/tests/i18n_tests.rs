use crate::i18n::{Locale, catalogs, t};

fn keys(catalog: &'static [(&'static str, &'static str)]) -> Vec<&'static str> {
    catalog.iter().map(|(key, _)| *key).collect()
}

#[test]
fn catalogs_are_sorted_and_unique_so_binary_search_is_valid() {
    for (locale, catalog) in catalogs() {
        let keys = keys(catalog);
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "{locale:?} catalog must be sorted and unique");
    }
}

#[test]
fn every_japanese_key_exists_in_english_fallback() {
    let english = keys(catalogs()[0].1);
    for (locale, catalog) in catalogs() {
        for (key, _) in *catalog {
            assert!(
                english.binary_search(key).is_ok(),
                "{locale:?} key {key:?} is missing from the English fallback catalog"
            );
        }
    }
}

#[test]
fn catalogs_cover_identical_key_sets_for_full_parity() {
    // RFC-043 allows partial catalogs, but the shipped MVP keeps en/ja in
    // lockstep so reviewers notice when a string is added on one side only.
    let english = keys(catalogs()[0].1);
    for (locale, catalog) in catalogs() {
        assert_eq!(
            keys(catalog),
            english,
            "{locale:?} catalog diverged from English key set"
        );
    }
}

#[test]
fn no_catalog_entry_is_empty() {
    for (locale, catalog) in catalogs() {
        for (key, value) in *catalog {
            assert!(
                !value.trim().is_empty(),
                "{locale:?} entry {key:?} renders as an empty label"
            );
        }
    }
}

#[test]
fn lookup_resolves_in_the_requested_locale() {
    assert_eq!(t(Locale::En, "toolbar.undo"), "Undo");
    assert_eq!(t(Locale::Ja, "toolbar.undo"), "元に戻す");
}

#[test]
fn unknown_keys_fall_back_to_the_key_itself() {
    assert_eq!(t(Locale::Ja, "missing.key"), "missing.key");
    assert_eq!(t(Locale::En, "missing.key"), "missing.key");
}

#[test]
fn locale_tags_round_trip_and_tolerate_regions() {
    assert_eq!(Locale::from_tag("ja"), Some(Locale::Ja));
    assert_eq!(Locale::from_tag("ja-JP"), Some(Locale::Ja));
    assert_eq!(Locale::from_tag("en_US"), Some(Locale::En));
    assert_eq!(Locale::from_tag("fr"), None);
    for locale in Locale::ALL {
        assert_eq!(Locale::from_tag(locale.tag()), Some(*locale));
    }
}

#[test]
fn locale_tags_in_the_shape_windows_reports_map_correctly() {
    // Task 013: GetUserPreferredUILanguages reports tags like "ja-JP" and
    // "en-US"; an unsupported language (e.g. French) must fall back to
    // English rather than `from_tag` returning `None` and being mishandled.
    assert_eq!(Locale::from_tag("ja-JP"), Some(Locale::Ja));
    assert_eq!(Locale::from_tag("en-US"), Some(Locale::En));
    assert_eq!(Locale::from_tag("fr-FR"), None);
}
