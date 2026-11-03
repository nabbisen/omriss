use crate::i18n::{Locale, catalogs, placeholders, t, t_args};

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

// ── Interpolation (RFC-073 §4.1) ────────────────────────────────────────────

#[test]
fn every_key_has_the_same_placeholder_set_in_every_locale() {
    // A `ja` value that omits `{count}` would silently drop the number, and no
    // test of key *names* can see that.
    let english = catalogs()[0].1;
    for (locale, catalog) in catalogs() {
        for (key, value) in *catalog {
            let (_, en_value) = english
                .iter()
                .find(|(k, _)| k == key)
                .unwrap_or_else(|| panic!("{key:?} missing from English"));
            let mut expected = placeholders(en_value);
            let mut actual = placeholders(value);
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(
                actual, expected,
                "{locale:?} {key:?} has placeholders {actual:?}, English has {expected:?}"
            );
        }
    }
}

#[test]
fn placeholders_are_named_identifiers() {
    // `{}` and `{0}` are positional: they would make a translation
    // order-dependent, which is exactly what named placeholders avoid.
    for (locale, catalog) in catalogs() {
        for (key, value) in *catalog {
            for name in placeholders(value) {
                assert!(
                    !name.is_empty()
                        && name.starts_with(|c: char| c.is_ascii_lowercase())
                        && name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                    "{locale:?} {key:?} has the non-named placeholder {{{name}}}"
                );
            }
        }
    }
}

#[test]
fn t_args_passes_a_placeholder_free_key_through_unchanged() {
    // A real key with no placeholder passes through unchanged, as `t` would.
    assert_eq!(t_args(Locale::En, "toolbar.undo", &[]), "Undo");
    assert_eq!(t_args(Locale::Ja, "toolbar.undo", &[]), "元に戻す");
}

#[test]
fn t_args_leaves_unknown_keys_as_the_key_itself() {
    assert_eq!(
        t_args(Locale::Ja, "missing.key", &[("n", "1")]),
        "missing.key"
    );
}

mod substitution {
    //! The substitution rules, exercised without depending on any catalog
    //! entry: the key *is* the template when it is missing from the catalogs.
    use super::*;

    fn fill(template: &str, args: &[(&str, &str)]) -> String {
        t_args(Locale::En, template, args)
    }

    #[test]
    fn replaces_a_placeholder() {
        assert_eq!(fill("{count} sections", &[("count", "3")]), "3 sections");
    }

    #[test]
    fn replaces_every_occurrence_and_is_order_independent() {
        assert_eq!(
            fill("{b} then {a} then {b}", &[("a", "1"), ("b", "2")]),
            "2 then 1 then 2"
        );
    }

    #[test]
    fn a_missing_argument_is_left_visible_not_dropped() {
        assert_eq!(fill("{count} sections", &[]), "{count} sections");
    }

    #[test]
    fn an_unused_argument_is_ignored() {
        assert_eq!(fill("plain", &[("count", "3")]), "plain");
    }

    #[test]
    fn a_value_containing_a_placeholder_is_not_expanded_again() {
        assert_eq!(
            fill("{a} {b}", &[("a", "{b}"), ("b", "x")]),
            "{b} x",
            "substitution must be a single pass"
        );
    }

    #[test]
    fn an_unclosed_brace_is_literal() {
        assert_eq!(fill("a { b", &[("b", "x")]), "a { b");
        assert_eq!(fill("{count", &[("count", "3")]), "{count");
    }

    #[test]
    fn non_ascii_text_around_placeholders_survives() {
        assert_eq!(fill("日本語 {n} 件", &[("n", "2")]), "日本語 2 件");
    }
}
