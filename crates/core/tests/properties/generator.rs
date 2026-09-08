//! RFC-066 §3.2 / §4: a Markdown document generator biased toward the shapes
//! that broke `structural.rs`, not toward average documents.
//!
//! Uniform random Markdown almost never produces these shapes by chance —
//! each one requires several independent choices (no trailing newline *and*
//! the last section has an empty body *and* it is a heading, not a
//! paragraph) to land together. So every shape below is generated with a
//! **directly elevated probability**, not left to combinatorial luck:
//!
//! - **bare trailing heading, no body, no trailing newline** — AUDIT-0170-002's
//!   exact shape (`# One\nbody\n\n# Last`, no final `\n`). `GenDoc::sections`'
//!   strategy forces the *last* top-level section to `body: ""`, `children: []`
//!   with probability [`BARE_TRAILING_HEADING_PROB`], independently of the
//!   general `trailing_newline` coin flip (see below), so the two conditions
//!   that must co-occur to hit AUDIT-0170-002 (empty last body **and** no
//!   trailing newline) are each individually likely rather than both needing
//!   to fall out of independent low-probability draws.
//! - **no trailing newline generally** — `trailing_newline` is a plain coin
//!   flip (`any::<bool>()`), so roughly half of all generated documents lack
//!   one, covering AUDIT-0170-003's precondition on its own, without the
//!   heading needing to be the very last line.
//! - **heading levels 1–6 including skips** — each section's level is drawn
//!   uniformly from 1..=6 independent of its siblings' levels, so `#` next to
//!   `###` (skipping `##`) is as likely as any other adjacent pair.
//! - **empty bodies** — `body_strategy()` returns `""` with weight 2 of 8
//!   (`prop_oneof!`), separately from the bare-trailing-heading case above
//!   (an empty body can land on *any* section, not only the last).
//! - **CRLF, occasionally mixed** — `newline_strategy()` picks `"\r\n"` with
//!   probability 0.5; `render_doc` uses one document-wide newline sequence
//!   (a genuinely mixed-per-line document is not a realistic save-path
//!   output and would only test the generator's own plumbing, not
//!   `structural.rs`), but LF and CRLF are each exercised on roughly half of
//!   all cases across a `proptest` run.
//! - **occasional setext headings** — `HeadingStyle::Setext` is only legal
//!   for level 1/2 (CommonMark), so it is drawn with probability
//!   [`SETEXT_PROB`] *conditioned on* the section's level already being 1 or
//!   2; other levels always render ATX.
//! - **inline markup and link URLs in titles** — `title_strategy()` unions
//!   plain-word titles with a pool of markup fragments (`**bold**`,
//!   `` `code span` ``, `*emphasis*`, `[text](https://example.com/path?q=1)`,
//!   raw inline `<b>` HTML) at roughly equal weight, so AUDIT-0170-004's
//!   destroyed-link-URL shape is reachable directly through `merge_with_prev_sibling`
//!   on a section titled with a live link.
//! - **root-parented sections with following siblings** — `sections_strategy()`
//!   generates 2..=5 top-level sections with meaningful probability (see
//!   `TOP_LEVEL_COUNT_RANGE`); since every top-level section's parent is the
//!   synthetic root, "2 or more top-level sections" *is* "a root-parented
//!   section with a following sibling," so this falls out of the top-level
//!   count distribution rather than needing separate machinery.
//!
//! `generator_reaches_required_shapes` (in this module's `#[cfg(test)]`
//! block) samples the strategy and asserts each shape above is actually
//! observed within a bounded number of draws — the executable form of this
//! doc comment's claims, run as part of `cargo test` like any other test.

use proptest::prelude::*;

/// Nesting is capped at one level (top-level sections may have children;
/// those children may not). The audit's five defects are all about
/// top-level/sibling relationships (moving a top-level section, promoting a
/// root-parented section, joining top-level siblings); one level of nesting
/// gives `move`/`delete`/`promote` a non-trivial subtree to carry without the
/// combinatorial cost of unbounded recursive generation.
const MAX_CHILDREN_PER_SECTION: usize = 2;
const TOP_LEVEL_COUNT_RANGE: std::ops::RangeInclusive<usize> = 1..=5;

const BARE_TRAILING_HEADING_PROB: u32 = 35; // out of 100
const SETEXT_PROB: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingStyle {
    Atx,
    Setext,
}

#[derive(Debug, Clone)]
pub struct GenSection {
    pub level: u8, // 1..=6
    pub style: HeadingStyle,
    pub title: String,
    pub body: String,
    /// A blank line separates this section's heading from whatever
    /// immediately precedes it. Sometimes false, deliberately: the audit's
    /// own repro shapes (`"# A\nalpha\n# B\nbeta"`) have no blank-line
    /// cushion between sections at all.
    pub blank_before: bool,
    pub children: Vec<GenSection>,
}

#[derive(Debug, Clone)]
pub struct GenDoc {
    /// Root-level content before the first heading. Usually empty.
    pub preamble: String,
    pub sections: Vec<GenSection>,
    pub newline: &'static str,
    pub trailing_newline: bool,
}

fn title_fragment_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[A-Za-z][A-Za-z0-9 ]{0,20}",
        Just("**bold**".to_string()),
        Just("*emphasis*".to_string()),
        Just("`code span`".to_string()),
        Just("[a link](https://example.com/path?q=1)".to_string()),
        Just("plain and <b>raw html</b> mixed".to_string()),
        Just("日本語 heading".to_string()),
    ]
}

/// `title_fragment_strategy`'s `prop_oneof!` already mixes one plain-word
/// branch against six markup/link/HTML/non-ASCII branches, so roughly 6 in 7
/// generated titles carry markup without a separate probability knob.
fn title_strategy() -> impl Strategy<Value = String> {
    title_fragment_strategy()
}

/// `proptest`'s `String` shrinker on fully arbitrary Unicode input reduces
/// slowly (many independent multi-byte characters, each shrunk
/// individually), which produced multi-hundred-byte "minimal" counterexamples
/// in early runs of this suite — technically correct, but not the small
/// reproducer `TESTING.md`'s eventual fixture-catalog entry needs. Weighting
/// this toward short ASCII words, with occasional plain arbitrary text for
/// stress, keeps most failures shrinking down to a handful of bytes while
/// still exercising non-ASCII/control-character bodies some of the time.
fn body_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        2 => Just(String::new()),
        5 => "[A-Za-z0-9 ]{0,16}",
        1 => any::<String>().prop_map(|s| s.chars().filter(|c| *c != '\0').take(60).collect()),
    ]
}

fn leaf_section_strategy(level: u8) -> impl Strategy<Value = GenSection> {
    (title_strategy(), body_strategy(), any::<bool>(), 0..100u32).prop_map(
        move |(title, body, blank_before, setext_roll)| {
            let style = if (level == 1 || level == 2) && setext_roll < SETEXT_PROB {
                HeadingStyle::Setext
            } else {
                HeadingStyle::Atx
            };
            GenSection {
                level,
                style,
                title,
                body,
                blank_before,
                children: Vec::new(),
            }
        },
    )
}

fn section_with_children_strategy(level: u8) -> impl Strategy<Value = GenSection> {
    (
        title_strategy(),
        body_strategy(),
        any::<bool>(),
        0..100u32,
        proptest::collection::vec(1..=6u8, 0..=MAX_CHILDREN_PER_SECTION).prop_flat_map(
            |child_levels| {
                child_levels
                    .into_iter()
                    .map(leaf_section_strategy)
                    .collect::<Vec<_>>()
            },
        ),
    )
        .prop_map(move |(title, body, blank_before, setext_roll, children)| {
            let style = if (level == 1 || level == 2) && setext_roll < SETEXT_PROB {
                HeadingStyle::Setext
            } else {
                HeadingStyle::Atx
            };
            GenSection {
                level,
                style,
                title,
                body,
                blank_before,
                children,
            }
        })
}

fn sections_strategy() -> impl Strategy<Value = Vec<GenSection>> {
    proptest::collection::vec(1..=6u8, TOP_LEVEL_COUNT_RANGE).prop_flat_map(|levels| {
        levels
            .into_iter()
            .map(section_with_children_strategy)
            .collect::<Vec<_>>()
    })
}

/// The top-level `GenDoc` strategy: RFC-066 §4's document generator.
pub fn doc_strategy() -> impl Strategy<Value = GenDoc> {
    (
        sections_strategy(),
        prop_oneof![Just("\n"), Just("\r\n")],
        any::<bool>(),
        0..100u32,
    )
        .prop_map(|(mut sections, newline, trailing_newline, bare_roll)| {
            if bare_roll < BARE_TRAILING_HEADING_PROB
                && let Some(last) = sections.last_mut()
            {
                last.body.clear();
                last.children.clear();
            }
            GenDoc {
                preamble: String::new(),
                sections,
                newline,
                trailing_newline,
            }
        })
}

fn render_section(sec: &GenSection, newline: &str, out: &mut String) {
    if sec.blank_before && !out.is_empty() {
        out.push_str(newline);
    }
    match sec.style {
        HeadingStyle::Atx => {
            out.push_str(&"#".repeat(sec.level as usize));
            out.push(' ');
            out.push_str(&sec.title);
            out.push_str(newline);
        }
        HeadingStyle::Setext => {
            out.push_str(&sec.title);
            out.push_str(newline);
            let underline_char = if sec.level == 1 { '=' } else { '-' };
            out.push_str(&underline_char.to_string().repeat(3));
            out.push_str(newline);
        }
    }
    if !sec.body.is_empty() {
        out.push_str(&sec.body);
        out.push_str(newline);
    }
    for child in &sec.children {
        render_section(child, newline, out);
    }
}

/// Renders a `GenDoc` to Markdown source text. Every internal line boundary
/// uses `doc.newline` and is always terminated; `trailing_newline` is
/// implemented as a single trim of the *final* newline sequence rather than
/// conditionally omitting it during construction, so it applies uniformly
/// regardless of what the last-rendered content happens to be (a heading
/// line, a body line, or the preamble) — this is what makes the
/// bare-trailing-heading shape reachable: force the last section's body
/// empty (above) and let this trim remove the heading line's own newline.
pub fn render_doc(doc: &GenDoc) -> String {
    let mut out = String::new();
    if !doc.preamble.is_empty() {
        out.push_str(&doc.preamble);
        out.push_str(doc.newline);
    }
    for sec in &doc.sections {
        render_section(sec, doc.newline, &mut out);
    }
    if !doc.trailing_newline && out.ends_with(doc.newline) {
        out.truncate(out.len() - doc.newline.len());
    }
    out
}

/// The full generator: `GenDoc` plus its rendered source, as the string
/// property tests actually consume.
pub fn markdown_source_strategy() -> impl Strategy<Value = String> {
    doc_strategy().prop_map(|d| render_doc(&d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::{Config, TestRunner};

    /// Samples the strategy directly (not through `Document::parse`) and
    /// asserts each of this module's doc-comment claims is actually
    /// observed within a bounded number of draws. This is `generator
    /// produces the shapes in §4, demonstrated` — P1's own completion
    /// criterion — made executable rather than asserted in prose.
    #[test]
    fn generator_reaches_required_shapes() {
        let mut runner = TestRunner::new(Config {
            cases: 2000,
            ..Config::default()
        });
        let strategy = doc_strategy();

        let mut saw_bare_trailing_heading = false;
        let mut saw_no_trailing_newline = false;
        let mut saw_level_skip = false;
        let mut saw_empty_body = false;
        let mut saw_crlf = false;
        let mut saw_setext = false;
        let mut saw_markup_title = false;
        let mut saw_root_with_following_sibling = false;

        for _ in 0..2000 {
            let tree = strategy.new_tree(&mut runner).unwrap();
            let doc = tree.current();
            let source = render_doc(&doc);

            if !doc.trailing_newline {
                saw_no_trailing_newline = true;
            }
            if let Some(last) = doc.sections.last()
                && last.body.is_empty()
                && last.children.is_empty()
                && !doc.trailing_newline
                // Exactly AUDIT-0170-002's shape: the very last thing in the
                // document is a heading line with nothing after it.
                && (source.trim_end_matches(['\n', '\r']).ends_with(&last.title)
                    || source.ends_with(&last.title))
            {
                saw_bare_trailing_heading = true;
            }
            if doc.newline == "\r\n" {
                saw_crlf = true;
            }
            if doc.sections.len() >= 2 {
                saw_root_with_following_sibling = true;
            }
            for pair in doc.sections.windows(2) {
                if pair[1].level as i16 - pair[0].level as i16 > 1
                    || pair[0].level as i16 - pair[1].level as i16 > 1
                {
                    saw_level_skip = true;
                }
            }
            for sec in doc
                .sections
                .iter()
                .chain(doc.sections.iter().flat_map(|s| s.children.iter()))
            {
                if sec.body.is_empty() {
                    saw_empty_body = true;
                }
                if sec.style == HeadingStyle::Setext {
                    saw_setext = true;
                }
                if sec.title.contains("**")
                    || sec.title.contains('`')
                    || sec.title.contains("](")
                    || sec.title.contains('<')
                {
                    saw_markup_title = true;
                }
            }
        }

        assert!(
            saw_bare_trailing_heading,
            "never generated a bare trailing heading with no body and no trailing newline"
        );
        assert!(
            saw_no_trailing_newline,
            "never generated a document with no trailing newline"
        );
        assert!(
            saw_level_skip,
            "never generated adjacent top-level sections with a level skip"
        );
        assert!(
            saw_empty_body,
            "never generated a section with an empty body"
        );
        assert!(saw_crlf, "never generated a CRLF document");
        assert!(saw_setext, "never generated a setext heading");
        assert!(
            saw_markup_title,
            "never generated a title with inline markup or a link URL"
        );
        assert!(
            saw_root_with_following_sibling,
            "never generated 2+ top-level (root-parented) sections"
        );
    }

    /// Sanity: the generator's own output must actually parse as a document,
    /// or every downstream property test would be vacuous.
    #[test]
    fn generated_documents_parse() {
        let mut runner = TestRunner::new(Config {
            cases: 500,
            ..Config::default()
        });
        let strategy = markdown_source_strategy();
        for _ in 0..500 {
            let tree = strategy.new_tree(&mut runner).unwrap();
            let source = tree.current();
            omriss_core::Document::parse(source.clone()).unwrap_or_else(|e| {
                panic!("generated document failed to parse: {e:?}\nsource:\n{source:?}")
            });
        }
    }
}
