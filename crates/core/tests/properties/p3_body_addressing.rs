//! RFC-066 §3.1 — Property 3: body addressing.
//!
//! ```text
//! P3  For any document source S, any node N, and any INERT body B:
//!     after replacing N's body with B, reading N's body back yields B
//!     followed only by line-break characters, and no node's title has
//!     changed.
//!
//!     Inert means B opens no block construct: no line of B begins with
//!     a heading marker, an HTML-block opener, a setext underline, a
//!     fence, a list marker, or a blockquote marker.
//! ```
//!
//! Added after review of the original P1/P2 slice found that P1 — a
//! *reversibility* property — does not fail on AUDIT-0170-002, because
//! commit-then-undo is byte-mechanical and reverses whatever range was
//! actually touched, regardless of whether that range correctly addressed
//! the body. P3 states the invariant AUDIT-0170-002 actually violates:
//! `body_range` must faithfully address the body, so writing `B` and
//! reading it back must yield `B` (allowing only the trailing line breaks
//! explained below) — not whatever the heading line's title parser happens
//! to see instead.
//!
//! For the bare-trailing-heading shape (a section that is the last line of
//! the document, empty body, no trailing newline — AUDIT-0170-002's exact
//! precondition), `body_range` is empty and sits *inside* the heading line.
//! Committing `B` writes it directly after the title text, so the
//! newly-parsed heading's title absorbs `B` and its (freshly re-derived,
//! now-empty-again) body no longer contains `B` at all. Reading the body
//! back yields `""`, not `B` (nor `B` plus any number of trailing line
//! breaks) — the read-back clause fails directly, without needing to
//! reason about titles at all. `"typed text"` (AUDIT-0170-002's own
//! example) is inert, so this shape is exactly what the restriction below
//! still catches.
//!
//! ## The trailing-line-break clause (added after RFC-065 landed)
//!
//! RFC-065 fixes AUDIT-0170-002 by having `replace_section_body` insert the
//! minimum separator needed to keep the document well-formed at a body
//! boundary (RFC-004 §Whitespace Policy's 2026-09-01 amendment). When a
//! *right-edge* separator is genuinely required — `N`'s new body doesn't
//! already end in a line break, and an existing following heading needs
//! protecting from being welded onto it — core writes that separator
//! *inside* the replaced range, because there is nowhere else for it to
//! go. `body_range` is then recomputed as "everything up to the next
//! heading" (RFC-006), so the separator becomes part of the body and reads
//! back with it: there is no third bucket for bytes belonging to neither
//! section.
//!
//! Exact equality is therefore unsatisfiable in that one sub-case, and this
//! clause is the minimum relaxation that admits it — B, plus zero or more
//! trailing `\n`/`\r` characters, and nothing else. It does not blunt the
//! property: AUDIT-0170-002's own defect reads back `""` against a written
//! `"typed text"`, which is not "B followed by line breaks" under any
//! reading, so this property still catches it. What the clause permits is
//! precisely the bytes `replace_section_body` is now *required* to add —
//! no reflow, no trimming, no reordering, and never a *prefix* (only a
//! trailing addition is structurally possible for this specific splice
//! site — see `preflight.rs`'s `heading_line_terminator` doc comment for
//! why the left edge never needs this).
//!
//! ## The inert-body restriction (added after RFC-065's follow-up) — not a defect, a correction to this RFC
//!
//! Re-testing against the unrestricted read-back clause above surfaced what
//! looked like a new defect: writing `B = "<!--"` into a section immediately
//! followed by another heading made that heading disappear from the
//! outline. Traced directly, and it is **not** a defect:
//!
//! ```text
//! source  "# One\nbody\n\n# Two\nmore\n"     write "<!--" into One's body
//! after   "# One\n<!--\n# Two\nmore\n"
//!         "# Two" STILL LITERALLY PRESENT     undo byte-exact: true
//!         outline ["One"]
//! ```
//!
//! **The bytes are untouched** — `# Two` is exactly where the user left it.
//! What changed is that CommonMark no longer *interprets* those bytes as a
//! heading, because `"<!--"` opens an HTML comment block, and such a block
//! runs to end of input (or its own closing `-->`) regardless of blank
//! lines. RFC-065's separator fix is unconditionally correct for
//! *paragraph*-shaped content; an HTML-block opener is not paragraph-shaped
//! content, and no separator closes it, because that is what the format
//! itself says an unclosed comment means.
//!
//! The mirror case makes it unambiguous: writing `B = "# Injected"` into a
//! body **adds** a node (`outline` gains `"Injected"`) just as readily as
//! `"<!--"` can remove one. A body edit changing the outline is not
//! inherently corruption — body text *is* Markdown, and typing structural
//! Markdown into a body changing the structure is the format working, not
//! omriss malfunctioning. `RELEASE_CHECKLIST.md` blocks on bytes being
//! corrupted; none are. Source preservation holds; undo is byte-exact.
//!
//! P3's title clause originally assumed a body edit can never change the
//! outline. It can, legitimately, in both directions — so the clause is
//! restricted to **inert** `B` (defined above): the question this property
//! must isolate is "did *omriss's splice* destroy structure the user did
//! not touch," not "did the *user's own text* change what their document
//! means." Only the first is a defect, and only inert bodies isolate it —
//! `inert_body_strategy()` below constrains the generator accordingly,
//! rather than filtering (which would reject a large fraction of arbitrary
//! strings and slow the suite for no benefit): each generated line that
//! would open a block construct is neutralized by prepending a harmless
//! character, keeping the rest of the string's randomness intact.
//!
//! The user-surprise half of this (an edit visibly changing the outline,
//! with no warning) is real and is RFC-004's own long-deferred "UI may
//! later warn" promise — tracked by RFC-071, not this property, and not a
//! `core` change: `omriss-core` must not refuse to store valid Markdown a
//! user deliberately wrote.

use std::collections::HashMap;

use omriss_core::{Document, NodeId, Outline, ReplaceSectionBody};
use proptest::prelude::*;

use crate::generator::markdown_source_strategy;
use crate::inert::{inert_body_strategy, is_inert};

fn title_multiset(outline: &Outline) -> HashMap<String, i32> {
    let mut m = HashMap::new();
    for n in outline.iter().filter(|n| !n.is_root()) {
        *m.entry(n.title.clone()).or_insert(0) += 1;
    }
    m
}

/// Whether `actual` is `expected` followed by zero or more `\n`/`\r`
/// characters and nothing else — P3's read-back clause.
fn is_body_plus_only_trailing_line_breaks(actual: &str, expected: &str) -> bool {
    actual
        .strip_prefix(expected)
        .is_some_and(|rest| rest.chars().all(|c| c == '\n' || c == '\r'))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn replace_body_reads_back_exactly_and_preserves_every_title(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
        new_body in inert_body_strategy(),
    ) {
        prop_assert!(is_inert(&new_body), "test bug: generated body is not inert: {new_body:?}");

        let Ok(mut doc) = Document::parse(source.clone()) else {
            return Ok(());
        };

        let node_ids: Vec<NodeId> = doc.outline().iter().map(|n| n.id).collect();
        prop_assume!(!node_ids.is_empty());
        let id = node_ids[node_pick % node_ids.len()];

        let before_titles = title_multiset(doc.outline());
        let base_revision = doc.revision();

        let commit = doc.replace_section_body(ReplaceSectionBody {
            node_id: id,
            base_revision,
            new_body: new_body.clone(),
        });
        prop_assert!(
            commit.is_ok(),
            "replace_section_body failed on a freshly-parsed document: {:?}",
            commit.err()
        );

        let read_back = doc.section_body(id);
        prop_assert!(
            read_back.is_ok(),
            "node {id:?} no longer resolves immediately after replacing its own body: {:?}",
            read_back.err()
        );
        let read_back = read_back.unwrap();
        prop_assert!(
            is_body_plus_only_trailing_line_breaks(read_back, &new_body),
            "reading N's body back did not yield B (plus only trailing line breaks) — \
             body_range did not faithfully address the body: wrote {:?}, read back {:?}",
            new_body,
            read_back
        );

        // With B guaranteed inert, the title multiset must be *exactly*
        // unchanged: B can neither introduce nor (correctly) destroy any
        // heading, so unlike the pre-restriction property, no growth is
        // legitimate here either.
        prop_assert_eq!(
            title_multiset(doc.outline()),
            before_titles,
            "an inert body-only edit changed the title multiset — a heading was corrupted or destroyed by omriss's own splice"
        );
    }
}
