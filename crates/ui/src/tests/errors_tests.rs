//! RFC-053 §11 / RFC-054 J4: `StructureErrorKind` -> friendly message.

use omriss_core::StructureErrorKind;

use crate::StructureErrorKindCatalogKey;
use crate::i18n::{Locale, t};

fn all_kinds() -> [StructureErrorKind; 5] {
    [
        StructureErrorKind::InvalidSyntax,
        StructureErrorKind::UnsupportedFeature,
        StructureErrorKind::UnsafeRange,
        StructureErrorKind::TooLarge,
        StructureErrorKind::InternalInvariantFailed,
    ]
}

#[test]
fn every_structure_error_kind_has_a_non_empty_catalog_key() {
    for kind in all_kinds() {
        let key = kind.catalog_key();
        assert!(
            !key.is_empty(),
            "StructureErrorKind::{kind:?} has an empty catalog key"
        );
        assert!(
            key.starts_with("structure_error."),
            "expected 'structure_error.' prefix, got '{key}'"
        );
    }
}

#[test]
fn every_structure_error_kind_resolves_to_a_real_catalog_entry_in_both_locales() {
    // A key present in the trait but absent from a catalog would silently
    // fall back to the raw key string (`t`'s documented behavior for an
    // unknown key) rather than failing the build -- exactly what RFC-054
    // J4 requires the mapping to not do. Assert resolution actually
    // produces translated text, not the key echoed back.
    for kind in all_kinds() {
        let key = kind.catalog_key();
        for locale in Locale::ALL {
            let resolved = t(*locale, key);
            assert_ne!(
                resolved, key,
                "{locale:?}: StructureErrorKind::{kind:?}'s key {key:?} has no catalog entry"
            );
        }
    }
}

#[test]
fn distinct_error_kinds_get_distinct_messages() {
    // Catches a copy-paste mapping (two kinds accidentally sharing a key),
    // which the exhaustive-match compile check alone would not catch.
    let mut keys: Vec<&'static str> = all_kinds().iter().map(|k| k.catalog_key()).collect();
    let before = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(
        keys.len(),
        before,
        "two StructureErrorKind variants share a catalog key"
    );
}
