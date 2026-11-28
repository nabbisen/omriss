//! RFC-066 §3.1b — Property 4: rename and join touch only their own heading.
//!
//! ```text
//! P4  For any document S and a rename or join of node N, every byte of S
//!     outside N's own heading is unchanged.
//! ```
//!
//! P2 cannot say this. Rename and join exist to change a title, so P2 exempts
//! both from "titles are unchanged" and checks only the node count; a rename that
//! reached into the user's text would pass it. P4 pins everything *except* what
//! the operation is for (RFC-066 §3.1b: an exemption is a place where nothing is
//! watching).
//!
//! # What each operation changes, exactly
//!
//! - **rename**: N's title, and only that: for a setext heading every title
//!   line (RFC-070 §10), up to but not including the underline. Everything
//!   before the heading and everything after the title (its line break, the
//!   underline, the body, every other node) is byte-identical, and the changed
//!   part holds no line break.
//! - **join**: the heading is replaced by its own first line, minus ATX markers.
//!   Nothing is written at the seam: `merge_with_prev_sibling` does not call
//!   `joining_separator`, so there is no separator to state. Everything outside
//!   `heading_range` is byte-identical.
//!
//! Both clauses above are about *where* a change lands. They cannot tell a
//! title that was rewritten wrongly *inside* its own line, so each operation also
//! gets the clause that states what the line becomes (see the tests): rename
//! yields the title it was asked for; join of a setext heading keeps its title
//! line verbatim, and neither clause has an exemption. **Measured:** with RFC-065 §2.8's guards reverted, the two
//! clauses about *where* the change lands pass 2,000 cases each — the damage is
//! inside the heading's own line — so the in-line clauses are what catch it.

use omriss_core::{Document, NodeId};
use proptest::prelude::*;

use crate::generator::inert_markdown_source_strategy;

/// The end of `s`'s first line, excluding its terminator, and the terminator
/// (`""`, `"\n"`, `"\r\n"` or `"\r"`).
fn first_line(s: &str) -> (usize, &str) {
    match s.find(['\n', '\r']) {
        None => (s.len(), ""),
        Some(i) if s[i..].starts_with("\r\n") => (i, "\r\n"),
        Some(i) => (i, &s[i..i + 1]),
    }
}

/// Where the title ends in a heading's source: the end of its last title line,
/// before that line's break. An ATX heading is one line; a setext heading is its
/// title line(s) and then the underline, which is not title.
fn title_end(heading: &str) -> usize {
    let mut lines = Vec::new();
    let mut rest = heading;
    let mut offset = 0;
    while !rest.is_empty() {
        let (len, term) = first_line(rest);
        lines.push((offset, len));
        offset += len + term.len();
        rest = &rest[len + term.len()..];
    }
    match lines.as_slice() {
        [] => 0,
        [(start, len)] => start + len,
        [.., (start, len), _underline] => start + len,
    }
}

fn candidates(doc: &Document) -> Vec<NodeId> {
    doc.outline()
        .iter()
        .filter(|n| !n.is_root())
        .map(|n| n.id)
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Rename changes only the title text on N's own heading line.
    #[test]
    fn rename_changes_only_its_own_title(
        source in inert_markdown_source_strategy(),
        node_pick in any::<usize>(),
        new_title in "[A-Za-z0-9][A-Za-z0-9 ]{0,23}",
    ) {
        let Ok(mut doc) = Document::parse(source.clone()) else { return Ok(()); };
        let ids = candidates(&doc);
        prop_assume!(!ids.is_empty());
        let id = ids[node_pick % ids.len()];
        let heading = doc.outline().node(id).unwrap().heading_range;
        let rev = doc.revision();

        if doc.rename_section(id, &new_title, rev).is_err() {
            return Ok(());
        }

        let before = &source;
        let after = doc.source();
        let line_end = heading.start + title_end(&before[heading.start..heading.end]);
        let tail = &before[line_end..];

        prop_assert!(
            after.starts_with(&before[..heading.start]),
            "rename changed bytes before N's heading: {before:?} -> {after:?}"
        );
        prop_assert!(
            after.ends_with(tail),
            "rename changed bytes after N's title: {before:?} -> {after:?}"
        );
        let changed = &after[heading.start..after.len() - tail.len()];
        prop_assert!(
            !changed.contains(['\n', '\r']),
            "rename's change spans a line break: {changed:?} in {after:?}"
        );

        // Inside its own heading: the node that heads there is titled exactly what
        // was asked, whatever the heading's shape and however many lines its title
        // spans. (Fails when a setext title that starts with `#` is renamed as if it
        // were an ATX heading, and when a multi-line setext title keeps its later
        // lines: RFC-070 §10.)
        let renamed = doc
            .outline()
            .iter()
            .find(|n| !n.is_root() && n.heading_range.start == heading.start);
        let wanted = new_title.trim();
        prop_assert!(
            renamed.is_some_and(|n| n.title == wanted),
            "renamed to {wanted:?}, the heading reads {:?}: {before:?} -> {after:?}",
            renamed.map(|n| n.title.clone())
        );
    }

    /// Join changes only N's own heading: nothing is written anywhere else.
    #[test]
    fn join_changes_only_its_own_heading(
        source in inert_markdown_source_strategy(),
        node_pick in any::<usize>(),
    ) {
        let Ok(mut doc) = Document::parse(source.clone()) else { return Ok(()); };
        let ids = candidates(&doc);
        prop_assume!(!ids.is_empty());
        let id = ids[node_pick % ids.len()];
        let heading = doc.outline().node(id).unwrap().heading_range;
        let rev = doc.revision();

        if doc.merge_with_prev_sibling(id, rev).is_err() {
            return Ok(());
        }

        let before = &source;
        let after = doc.source();
        let tail = &before[heading.end..];
        prop_assert!(
            after.starts_with(&before[..heading.start]),
            "join changed bytes before N's heading: {before:?} -> {after:?}"
        );
        prop_assert!(
            after.ends_with(tail),
            "join changed bytes after N's heading: {before:?} -> {after:?}"
        );

        // Inside its own heading. A setext heading is two lines (title, underline);
        // joining it drops the underline and keeps the title line byte for byte,
        // whatever the title starts with. (Fails when a setext title that starts
        // with `#` is read as an ATX heading and its `#` is deleted.)
        let old_heading = &before[heading.start..heading.end];
        let (title_len, terminator) = first_line(old_heading);
        let is_setext = !old_heading[title_len + terminator.len()..].trim().is_empty();
        if is_setext {
            let written = &after[heading.start..after.len() - tail.len()];
            prop_assert!(
                written == &old_heading[..title_len + terminator.len()],
                "joining a setext heading changed its title line: {:?} became {written:?}",
                &old_heading[..title_len + terminator.len()]
            );
        }
    }
}
