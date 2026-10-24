//! The definition of an **inert** body, shared by P2 and P3 (RFC-066 §3.1,
//! §3.1a).
//!
//! A body is inert when it cannot change block structure: no line opens a
//! heading, an HTML block, a setext underline, a fence, a list or a
//! blockquote. Body text *is* Markdown, so a non-inert body legitimately
//! changes what the document's outline is — the bytes survive and CommonMark
//! re-reads them (RFC-071 §2.1). A property that isolates *omriss's splice*
//! from *Markdown's own semantics* must therefore generate only inert bodies.
//!
//! P3 applies it to the body being written; P2 applies it to every body in the
//! generated source, because a structural operation moves those bodies around
//! and an unterminated HTML block (`<?`, `<!--`) swallows whatever follows it
//! at its new position. This is the single definition; do not write a second.

use proptest::prelude::*;

/// Characters that open a block construct when they start a line: a
/// heading marker (`#`), an HTML-block opener (`<`), a setext underline
/// (`=`/`-`), a fence (`` ` ``/`~`), a list marker (`-`/`*`/`+`, or a
/// digit for an ordered-list marker), or a blockquote marker (`>`).
/// Deliberately conservative — e.g. every digit is treated as a possible
/// ordered-list marker regardless of what follows it — since the cost of
/// over-excluding here is only a slightly less varied generator, while
/// under-excluding would let a non-inert body back into a property meant
/// to isolate omriss's own splice from Markdown's own semantics.
const BLOCK_OPENER_STARTS: &[char] = &[
    '#', '<', '=', '-', '*', '+', '>', '`', '~', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// CommonMark allows up to three spaces of indentation before a block
/// construct still counts as one (four or more makes it an indented code
/// block instead, which *is* inert). Checking `BLOCK_OPENER_STARTS`
/// against the raw line therefore missed `" # foo"`, `"  ## bar"`, and
/// `"   # baz"` — each classified inert while actually opening a heading.
/// Stripping *all* leading spaces before the check, rather than at most
/// three, over-excludes 4-space-indented lines (genuinely inert code-block
/// content gets neutralized too) — the same conservative direction already
/// chosen for digits and `<` above, and for the same reason: over-excluding
/// only narrows the generator, under-excluding lets a non-inert body back
/// into a property built to keep them out.
fn opens_a_block(line: &str) -> bool {
    line.trim_start_matches(' ')
        .starts_with(BLOCK_OPENER_STARTS)
}

pub fn is_inert(body: &str) -> bool {
    body.lines().all(|line| !opens_a_block(line))
}

/// Neutralizes every line of `body` that would open a block construct, by
/// prepending a harmless letter — chosen over filtering so the strategy
/// never rejects a generated case (rejection would both slow the suite and
/// bias the corpus toward whatever happens to survive rejection). Line
/// endings are normalized to bare `\n` in the process (`str::lines` does
/// not distinguish `\r\n` from `\n`, and CRLF variation is exercised
/// elsewhere in this suite, not lost by narrowing this one property).
pub fn make_inert(body: String) -> String {
    body.lines()
        .map(|line| {
            if opens_a_block(line) {
                format!("x{line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn inert_body_strategy() -> impl Strategy<Value = String> {
    any::<String>().prop_map(make_inert)
}

#[cfg(test)]
mod inert_tests {
    use super::*;

    #[test]
    fn plain_text_is_inert() {
        assert!(is_inert("typed text"));
        assert!(is_inert("line one\nline two"));
        assert!(is_inert(""));
    }

    #[test]
    fn block_openers_are_not_inert() {
        assert!(!is_inert("# Injected"));
        assert!(!is_inert("<!--"));
        assert!(!is_inert("===\n"));
        assert!(!is_inert("---"));
        assert!(!is_inert("```rust"));
        assert!(!is_inert("~~~"));
        assert!(!is_inert("- item"));
        assert!(!is_inert("* item"));
        assert!(!is_inert("+ item"));
        assert!(!is_inert("1. item"));
        assert!(!is_inert("> quote"));
    }

    #[test]
    fn a_later_line_opening_a_block_is_also_not_inert() {
        assert!(!is_inert("fine so far\n# but not this line"));
    }

    /// Pins the fix for the flake found in review: CommonMark allows up
    /// to three spaces of indentation before a block construct still
    /// counts as one, so a leading-space-indented opener must be
    /// classified non-inert exactly like an unindented one.
    ///
    /// A four-space indent turns a `#`-line into an indented *code block*
    /// instead — genuinely inert per CommonMark — but this check strips
    /// *all* leading spaces before comparing, not just up to three, so it
    /// is (over-cautiously, deliberately) excluded too: the same
    /// conservative direction already chosen for digits and `<` above.
    #[test]
    fn a_block_opener_indented_by_up_to_three_spaces_is_still_not_inert() {
        assert!(!is_inert(" # foo"));
        assert!(!is_inert("  ## bar"));
        assert!(!is_inert("   # baz"));
        assert!(!is_inert(
            "    # over-excluded: this is really an inert code block"
        ));
    }

    #[test]
    fn make_inert_neutralizes_every_offending_line() {
        let made = make_inert("# H\nfine\n<!--\n> q".to_string());
        assert!(
            is_inert(&made),
            "make_inert did not produce an inert string: {made:?}"
        );
    }
}
