//! RFC-066 P2 slice — Property 1: replace-then-undo round trip.
//!
//! ```text
//! P1  For any document source S and any node N in its outline:
//!     replacing N's body with any string B, then undoing,
//!     reproduces S byte-for-byte.
//! ```
//!
//! Targets AUDIT-0170-002 / RFC-065 §2.1: `replace_section_body` splices into
//! a `body_range` that, for a section which is the last line in the document
//! with no body and no trailing newline, starts *inside the heading line*
//! rather than after it. The property does not know that shape by name — it
//! only knows the round-trip invariant — which is the point: the generator's
//! bias toward that shape (see `generator.rs`) is what gives this property a
//! chance of finding it.
//!
//! ## Escalation: this property does not actually fail on AUDIT-0170-002
//!
//! RFC-066 §3.1 states "P1 catches AUDIT-0170-002." Empirically, against
//! current `main`, it does not — 256 generated cases, with the generator
//! landing the bare-trailing-heading shape roughly a third of the time (see
//! `BARE_TRAILING_HEADING_PROB` in `generator.rs`), produced zero failures.
//! A manual trace of the audit's own exact repro shows why:
//!
//! ```text
//! source before:            "# One\nbody\n\n# Last"
//! commit "typed text" into "Last"'s body:
//!   source after commit:    "# One\nbody\n\n# Lasttyped text"
//!   "Last"'s title, read right after the commit: "Lasttyped text"  <- the real defect
//!   node count after commit: 3 (unchanged)
//! undo():
//!   source after undo:      "# One\nbody\n\n# Last"   (byte-identical to before)
//! ```
//!
//! `undo` is byte-mechanical: it replays the exact `old_text`/`new_range` an
//! `EditRecord` captured at commit time, regardless of whether that range
//! happened to fall inside what a human would call "the heading" or "the
//! body." So the commit-then-undo pair really is reversible at the byte
//! level even when the commit itself corrupted the section's title while it
//! was live. AUDIT-0170-002 is real and severe (the outline row visibly
//! renames itself for as long as the user hasn't undone), but it violates a
//! *different* invariant than the one P1's literal English states — closer
//! to "a body-only edit must not change the node's own title" than to
//! "replace-then-undo is byte-reversible."
//!
//! Per the handoff's own §8 escalation trigger ("a property passes against
//! current `main`... shipping it would be worse than shipping nothing"),
//! this was reported rather than silently shipped as a pass, and rather
//! than unilaterally redefining P1 to something stronger than its RFC text
//! said. P1 is implemented below exactly as RFC-066 §3.1 specifies it —
//! unchanged by the resolution below, deliberately.
//!
//! ## Resolution
//!
//! The architect confirmed the trace, corrected RFC-066 §3.1's original
//! claim, and added **P3** (`p3_body_addressing.rs`) to state the
//! invariant AUDIT-0170-002 actually violates: write `B` into `N`'s body,
//! read `N`'s body back, and it must equal `B`. P1 was kept exactly as
//! specified rather than widened to cover for the wrong claim — it guards
//! a real, separate invariant (the AUDIT-0170-018 history-desync failure)
//! that a redefinition would have diluted.

use omriss_core::{Document, ReplaceSectionBody};
use proptest::prelude::*;

use crate::generator::markdown_source_strategy;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn replace_body_then_undo_reproduces_source_byte_for_byte(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
        new_body in any::<String>(),
    ) {
        let Ok(mut doc) = Document::parse(source.clone()) else {
            // The generator's own parse-guarantee is checked directly by
            // `generator::tests::generated_documents_parse`; a failure here
            // would be a duplicate signal, not new information.
            return Ok(());
        };

        let node_ids: Vec<_> = doc.outline().iter().map(|n| n.id).collect();
        prop_assume!(!node_ids.is_empty());
        let id = node_ids[node_pick % node_ids.len()];

        let original = doc.source().to_string();
        let base_revision = doc.revision();

        let commit = doc.replace_section_body(ReplaceSectionBody {
            node_id: id,
            base_revision,
            new_body: new_body.clone(),
        });
        // `id` was read from this same, freshly-parsed `doc` a moment ago,
        // so the revision is current and the node cannot be stale — any
        // `Err` here is a surprise worth seeing, not a case to discard.
        prop_assert!(
            commit.is_ok(),
            "replace_section_body failed on a freshly-parsed document: {:?}",
            commit.err()
        );

        doc.undo()
            .expect("undo must succeed immediately after a successful commit");

        prop_assert_eq!(
            doc.source(),
            original.as_str(),
            "replace-then-undo did not reproduce the original source byte-for-byte"
        );
    }
}
