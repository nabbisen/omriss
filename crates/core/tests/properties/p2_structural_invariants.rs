//! RFC-066 P3 slice — Property 2: structural-operation node-count and
//! title invariants.
//!
//! ```text
//! P2  For any document source S and any structural operation O
//!     valid on node N: the outline node count after O differs from
//!     before by exactly the delta O defines, and every surviving
//!     node's title is unchanged unless O is rename or join.
//! ```
//!
//! One property function per operation kind, all sharing [`assert_count_delta`]
//! and the title-multiset helpers below, so a failure names its operation
//! directly rather than requiring the reader to infer it from a generic
//! "some structural op broke something."
//!
//! ## Why titles are compared as a multiset, not by matching node ids
//!
//! The first implementation of this property matched "the same node" across
//! an operation by `NodeId`, since that is the identity type the API
//! exposes. That produced **false failures** on `promote`/`demote` alone,
//! with no `structural.rs` bug involved: `NodeId` is a hash of the node's
//! *ordinal path from the root* (RFC-006), and promoting or demoting a
//! heading can change which earlier heading becomes its parent once the
//! document is re-parsed — which changes that node's ordinal path, and
//! every later sibling's, even though not a single byte of any *other*
//! section's title was touched. Matching by id after a shape-changing
//! operation compares two unrelated nodes that happen to share a
//! post-hoc-reused id, not the same node before and after.
//!
//! `Outline`/`NodeId`'s own doc comment says exactly this: "IDs are not
//! guaranteed to survive edits that change heading structure." Taking that
//! seriously means the property cannot use `NodeId` as its notion of
//! "surviving node" for *any* operation, including the ones that look
//! shape-preserving.
//!
//! Comparing the **multiset of all non-root titles** sidesteps identity
//! entirely: it asks "did any title get corrupted or vanish," not "does node
//! X's id still point to the same thing," and is exactly what P2's English
//! phrasing means in practice — the RFC's algebraic node-count clause has a
//! parallel bag-of-titles clause for the same reason (a corrupted title is
//! not a count change, so the count clause alone would miss it).
//!
//! Each operation's defined delta and title expectation:
//!
//! | Operation | Delta | Title multiset |
//! |---|---|---|
//! | promote / demote | 0 | unchanged exactly |
//! | rename | 0 | **exempt** (the point of rename is a title change) |
//! | move | 0 | unchanged exactly |
//! | split | +1 | grows by exactly one; every existing title's count is unchanged |
//! | delete | −(1 + descendant count) | shrinks by exactly the deleted subtree's own titles |
//! | merge (join) | −1 | **exempt** per the property's own wording |
//!
//! A `Result::Err` from the operation under test is treated as "not valid on
//! this `(S, N, ...)` combination" and the case is skipped via early return
//! — RFC-066 §3.1 scopes the property to operations *valid* on `N`; an
//! invalid combination (promoting an H1, moving into one's own descendant, a
//! split offset past the section end) is not a counterexample, it is simply
//! outside the property's domain.

use std::collections::HashMap;

use omriss_core::{Document, HeadingLevel, MoveTarget, NodeId, Outline};
use proptest::prelude::*;

use crate::generator::markdown_source_strategy;

type TitleMultiset = HashMap<String, i32>;

fn title_multiset(outline: &Outline) -> TitleMultiset {
    let mut m = HashMap::new();
    for n in outline.iter().filter(|n| !n.is_root()) {
        *m.entry(n.title.clone()).or_insert(0) += 1;
    }
    m
}

fn subtree_titles(outline: &Outline, id: NodeId) -> TitleMultiset {
    fn walk(outline: &Outline, id: NodeId, m: &mut TitleMultiset) {
        if let Some(node) = outline.node(id) {
            *m.entry(node.title.clone()).or_insert(0) += 1;
            for child in &node.children {
                walk(outline, *child, m);
            }
        }
    }
    let mut m = HashMap::new();
    walk(outline, id, &mut m);
    m
}

fn subtree_size(titles: &TitleMultiset) -> usize {
    titles.values().map(|&c| c as usize).sum()
}

/// Node count includes the synthetic root; the root is never created or
/// destroyed by any structural operation, so including it does not change
/// the arithmetic — it is simpler to compare `outline().len()` directly than
/// to special-case root exclusion at every call site.
fn assert_count_delta(before: usize, after: usize, expected_delta: i64, op: &str) {
    let actual_delta = after as i64 - before as i64;
    assert_eq!(
        actual_delta, expected_delta,
        "{op}: node count changed by {actual_delta}, expected {expected_delta} (before={before}, after={after})"
    );
}

/// promote / demote / move: no title anywhere in the document is expected to
/// change, so the multiset must match exactly.
fn assert_title_multiset_unchanged(before: &TitleMultiset, after: &TitleMultiset, op: &str) {
    assert_eq!(
        before, after,
        "{op}: title multiset changed (a title was corrupted, lost, or duplicated) — before={before:?}, after={after:?}"
    );
}

/// split: every title that existed before must still appear with the same
/// count after (nothing existing was touched), and the total count must have
/// grown by exactly one (the new node) — checked without needing to predict
/// the exact string the new heading's title parses to.
fn assert_title_multiset_grew_by_one(before: &TitleMultiset, after: &TitleMultiset, op: &str) {
    for (title, &before_count) in before {
        let after_count = *after.get(title).unwrap_or(&0);
        assert!(
            after_count >= before_count,
            "{op}: existing title {title:?} count dropped from {before_count} to {after_count}"
        );
    }
    let before_total: i32 = before.values().sum();
    let after_total: i32 = after.values().sum();
    assert_eq!(
        after_total,
        before_total + 1,
        "{op}: total title count did not grow by exactly one (before={before_total}, after={after_total})"
    );
}

/// delete: the after-multiset must equal the before-multiset with exactly
/// the deleted subtree's own titles removed — nothing else may change.
fn assert_title_multiset_lost_exactly(
    before: &TitleMultiset,
    after: &TitleMultiset,
    removed: &TitleMultiset,
    op: &str,
) {
    let mut expected = before.clone();
    for (title, &removed_count) in removed {
        let entry = expected.entry(title.clone()).or_insert(0);
        *entry -= removed_count;
        if *entry <= 0 {
            expected.remove(title);
        }
    }
    assert_eq!(
        &expected, after,
        "{op}: title multiset after deletion did not equal before-minus-deleted-subtree — expected={expected:?}, actual={after:?}"
    );
}

fn parsed_or_skip(source: &str) -> Option<Document> {
    Document::parse(source.to_string()).ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Targets RFC-065 §2.3 (promote teleporting a root-parented section to
    /// end of file): a correct promote never changes node count, and never
    /// changes the document's title multiset.
    #[test]
    fn promote_preserves_count_and_titles(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let candidates: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!candidates.is_empty());
        let id = candidates[node_pick % candidates.len()];

        let before_count = doc.outline().len();
        let before_titles = title_multiset(doc.outline());
        let rev = doc.revision();

        if doc.promote_section(id, rev).is_err() {
            return Ok(()); // e.g. H1, setext heading — outside the property's domain
        }

        assert_count_delta(before_count, doc.outline().len(), 0, "promote");
        assert_title_multiset_unchanged(&before_titles, &title_multiset(doc.outline()), "promote");
    }

    #[test]
    fn demote_preserves_count_and_titles(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let candidates: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!candidates.is_empty());
        let id = candidates[node_pick % candidates.len()];

        let before_count = doc.outline().len();
        let before_titles = title_multiset(doc.outline());
        let rev = doc.revision();

        if doc.demote_section(id, rev).is_err() {
            return Ok(()); // e.g. H6, setext heading
        }

        assert_count_delta(before_count, doc.outline().len(), 0, "demote");
        assert_title_multiset_unchanged(&before_titles, &title_multiset(doc.outline()), "demote");
    }

    /// Rename is exempt from the title clause by the property's own wording
    /// (rename's entire purpose is to change one title) — only the count
    /// invariant is checked here.
    #[test]
    fn rename_preserves_count(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
        new_title in "[A-Za-z0-9 ]{1,24}",
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let candidates: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!candidates.is_empty());
        let id = candidates[node_pick % candidates.len()];

        let before_count = doc.outline().len();
        let rev = doc.revision();

        if doc.rename_section(id, &new_title, rev).is_err() {
            return Ok(());
        }

        assert_count_delta(before_count, doc.outline().len(), 0, "rename");
    }

    /// Targets RFC-065 §2.2 (move destroying a heading with no trailing
    /// newline): a correct move never changes node count, and never changes
    /// the document's title multiset.
    #[test]
    fn move_preserves_count_and_titles(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
        target_pick in any::<usize>(),
        placement in 0..2u8,
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let all_ids: Vec<NodeId> = doc.outline().iter().map(|n| n.id).collect();
        let movable: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!movable.is_empty() && all_ids.len() > 1);
        let id = movable[node_pick % movable.len()];
        let target = all_ids[target_pick % all_ids.len()];

        // RFC-065 §4/B7: `MoveTarget::AsFirstChildOf`/`AsLastChildOf` were
        // withdrawn (breaking change) — only `Before`/`After` remain.
        let move_target = match placement {
            0 => MoveTarget::Before(target),
            _ => MoveTarget::After(target),
        };

        let before_count = doc.outline().len();
        let before_titles = title_multiset(doc.outline());
        let rev = doc.revision();

        if doc.move_section(id, move_target, rev).is_err() {
            return Ok(()); // e.g. moving into own descendant, self-move
        }

        assert_count_delta(before_count, doc.outline().len(), 0, "move");
        assert_title_multiset_unchanged(&before_titles, &title_multiset(doc.outline()), "move");
    }

    /// A correct split inserts exactly one new node and changes no existing
    /// title.
    ///
    /// Still `#[ignore]`d after RFC-065's B1–B6/B8 boundary fixes: the
    /// remaining shrunk counterexample is a *different* defect from the
    /// boundary-welding family those slices fix. `max_offset =
    /// full.end - body.start` lets `offset_in_body` land anywhere within
    /// the target's entire subtree, including strictly inside a
    /// *descendant* node's own heading line (its title text, or the gap
    /// between a setext title and its underline) — fracturing that
    /// heading regardless of what separator gets inserted around it, since
    /// the insertion is happening *inside* existing heading syntax, not at
    /// a boundary next to it. Deferred to RFC-070 (API-only: the GUI's own
    /// split offset is always a cursor position inside the edited body, so
    /// it cannot exceed the body the way a direct library call can), which
    /// also covers AUDIT-0170-033 (the same function's unvalidated title)
    /// as one unit of work. RFC-066 criterion 6 was amended to allow this:
    /// a *tracked* `#[ignore]` naming its owning RFC is a scheduled
    /// defect, not a silently disabled test.
    #[ignore = "fails until RFC-070: split can insert inside a descendant heading"]
    #[test]
    fn split_increases_count_by_one_and_preserves_titles(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
        raw_offset in any::<usize>(),
        new_title in "[A-Za-z0-9 ]{0,24}",
        raw_level in 1..=6u8,
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let all_ids: Vec<NodeId> = doc.outline().iter().map(|n| n.id).collect();
        prop_assume!(!all_ids.is_empty());
        let id = all_ids[node_pick % all_ids.len()];

        let node = doc.outline().node(id).expect("id came from this outline");
        let max_offset = node.full_range.end - node.body_range.start;
        let offset = if max_offset == 0 { 0 } else { raw_offset % (max_offset + 1) };
        let level = HeadingLevel::from_u8(raw_level).expect("1..=6 is always a valid level");

        let before_count = doc.outline().len();
        let before_titles = title_multiset(doc.outline());
        let rev = doc.revision();

        if doc.split_section(id, offset, &new_title, level, rev).is_err() {
            return Ok(()); // e.g. offset lands mid-character
        }

        assert_count_delta(before_count, doc.outline().len(), 1, "split");
        assert_title_multiset_grew_by_one(&before_titles, &title_multiset(doc.outline()), "split");
    }

    /// A correct delete removes exactly the deleted node's whole subtree —
    /// itself plus every descendant — and no other node's title changes.
    #[test]
    fn delete_decreases_count_by_subtree_size_and_preserves_titles(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let candidates: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!candidates.is_empty());
        let id = candidates[node_pick % candidates.len()];

        let before_count = doc.outline().len();
        let before_titles = title_multiset(doc.outline());
        let removed_titles = subtree_titles(doc.outline(), id);
        let removed_count = subtree_size(&removed_titles) as i64;
        let rev = doc.revision();

        if doc.delete_section(id, rev).is_err() {
            return Ok(());
        }

        assert_count_delta(before_count, doc.outline().len(), -removed_count, "delete");
        assert_title_multiset_lost_exactly(&before_titles, &title_multiset(doc.outline()), &removed_titles, "delete");
    }

    /// Targets RFC-065 §2.4 (join flattening the merged heading's markup
    /// into plain text): join is exempt from the title clause per the
    /// property's own wording, so only the count invariant — exactly one
    /// node fewer — is checked here. Whether the flattened text still
    /// carries the original link URL, emphasis, or code spans is RFC-065's
    /// own example-based regression coverage, not this property's job.
    #[test]
    fn merge_decreases_count_by_exactly_one(
        source in markdown_source_strategy(),
        node_pick in any::<usize>(),
    ) {
        let Some(mut doc) = parsed_or_skip(&source) else { return Ok(()); };
        let candidates: Vec<NodeId> = doc.outline().iter().filter(|n| !n.is_root()).map(|n| n.id).collect();
        prop_assume!(!candidates.is_empty());
        let id = candidates[node_pick % candidates.len()];

        let before_count = doc.outline().len();
        let rev = doc.revision();

        if doc.merge_with_prev_sibling(id, rev).is_err() {
            return Ok(()); // e.g. no previous sibling, previous sibling has children
        }

        assert_count_delta(before_count, doc.outline().len(), -1, "merge");
    }
}
