//! Detecting a change to the document's outline that nobody asked for
//! (RFC-071).
//!
//! Text a user commits, or a section they move, can change whether *other*
//! sections are headings at all: an unterminated `<!--` or `<?` turns every
//! following heading into HTML-block content, and a typed `# Foo` creates one.
//! Both are correct Markdown and every byte survives, so this is feedback, not
//! a refusal — `omriss-core` never rejects the edit.
//!
//! The rule is **expected versus actual**, not before versus after. A
//! `delete_section` removes nodes on purpose; a detector that merely compared
//! outlines would fire on every delete, split and promote, and a warning that
//! fires when nothing is wrong is ignored on the one occasion it matters.
//!
//! [`expected_change`] is the table of what each operation intends and
//! [`detect`] is the comparator. They are the *single* expression of the rule
//! in `omriss-ui`. RFC-066's P2 properties
//! (`crates/core/tests/properties/p2_structural_invariants.rs`) state the same
//! deltas operation by operation; they live in `omriss-core`, which cannot
//! depend on this crate, so they are a second reading of one rule. If the two
//! ever disagree, one of them is wrong — fix that, do not adjust a test to fit.
//!
//! | Operation | Intended change | Warn when |
//! |---|---|---|
//! | body commit | none | anything differs |
//! | move, promote, demote | none | anything differs |
//! | split / add section | exactly one node gained | anything else differs |
//! | rename | the old title lost, one node gained | anything else differs |
//! | delete | the subtree's titles lost | anything else differs |
//! | merge (join) | the merged section's title lost | anything else differs |

use std::collections::BTreeMap;

use omriss_core::{NodeId, Outline};

use crate::i18n::{Locale, t, t_args};

/// The multiset of non-root section titles in an outline.
///
/// Compared as a multiset rather than by `NodeId`: a node's id is a hash of
/// its ordinal path, so it changes whenever an earlier heading appears or
/// disappears (RFC-066 P2's header explains why identity is the wrong tool).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutlineSummary {
    titles: BTreeMap<String, usize>,
}

impl OutlineSummary {
    /// Summarizes every non-root node of `outline`.
    pub fn of(outline: &Outline) -> Self {
        let mut titles = BTreeMap::new();
        for node in outline.iter().filter(|n| !n.is_root()) {
            *titles.entry(node.title.clone()).or_insert(0) += 1;
        }
        Self { titles }
    }

    /// Total number of sections.
    pub fn len(&self) -> usize {
        self.titles.values().sum()
    }

    /// Whether the document has no sections.
    pub fn is_empty(&self) -> bool {
        self.titles.is_empty()
    }
}

/// What an operation means to do to the outline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpectedChange {
    /// Titles that are meant to disappear, one entry per node.
    removed: Vec<String>,
    /// How many nodes are meant to appear. Their titles are not predicted: a
    /// new heading's parsed title can differ from the text typed into it.
    added: usize,
}

/// The operations that rewrite the source, as far as the rule is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// A body commit.
    BodyCommit,
    /// Moving a section; promote and demote share this expectation.
    Reshape,
    /// Splitting a body, or adding a section.
    Add,
    /// Renaming a section.
    Rename(NodeId),
    /// Deleting a section and its subtree.
    Delete(NodeId),
    /// Joining a section into its previous sibling.
    Merge(NodeId),
}

/// What `op` intends to do, read from `outline` as it is *before* the
/// operation runs.
pub fn expected_change(op: Operation, outline: &Outline) -> ExpectedChange {
    let title_of = |id: NodeId| outline.node(id).map(|n| n.title.clone());
    match op {
        Operation::BodyCommit | Operation::Reshape => ExpectedChange::default(),
        Operation::Add => ExpectedChange {
            removed: Vec::new(),
            added: 1,
        },
        Operation::Rename(id) => ExpectedChange {
            removed: title_of(id).into_iter().collect(),
            added: 1,
        },
        Operation::Delete(id) => ExpectedChange {
            removed: subtree_titles(outline, id),
            added: 0,
        },
        Operation::Merge(id) => ExpectedChange {
            removed: title_of(id).into_iter().collect(),
            added: 0,
        },
    }
}

fn subtree_titles(outline: &Outline, id: NodeId) -> Vec<String> {
    let mut titles = Vec::new();
    let mut stack = vec![id];
    while let Some(next) = stack.pop() {
        if let Some(node) = outline.node(next) {
            titles.push(node.title.clone());
            stack.extend(node.children.iter().copied());
        }
    }
    titles
}

/// An outline change the operation did not intend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureChange {
    /// Sections that stopped being headings and should not have.
    pub lost: usize,
    /// Sections that appeared and should not have.
    pub gained: usize,
}

/// Which of RFC-071 §4.2's three message shapes applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureChangeKind {
    /// Headings disappeared.
    Disappeared,
    /// Headings appeared.
    Appeared,
    /// Both.
    Both,
}

impl StructureChange {
    /// The message shape for this change.
    pub fn kind(&self) -> StructureChangeKind {
        match (self.lost > 0, self.gained > 0) {
            (true, true) => StructureChangeKind::Both,
            (true, false) => StructureChangeKind::Disappeared,
            _ => StructureChangeKind::Appeared,
        }
    }
}

impl StructureChange {
    /// The message for the status surface, as sentences to be shown in order
    /// (RFC-071 §4.2–§4.3): what changed, then — only when headings
    /// disappeared — the class of cause, then how to undo.
    ///
    /// Three shapes, because one generic sentence would be useless. The count
    /// selects a `.one` / `.other` key, the convention RFC-073 §4.2 adopts for
    /// plurals, so `t_plural` can replace the selection here without touching a
    /// catalog. Returned as separate sentences, not one string, because the
    /// joiner is a typographic decision (Japanese has no inter-sentence space).
    pub fn sentences(&self, locale: Locale) -> Vec<String> {
        let counted = |stem: &str, n: usize| {
            let key = format!("{stem}.{}", if n == 1 { "one" } else { "other" });
            t_args(locale, &key, &[("count", &n.to_string())])
        };
        let mut sentences = Vec::with_capacity(3);
        sentences.push(match self.kind() {
            StructureChangeKind::Disappeared => {
                counted("status.structure_change.disappeared", self.lost)
            }
            StructureChangeKind::Appeared => {
                counted("status.structure_change.appeared", self.gained)
            }
            StructureChangeKind::Both => t(locale, "status.structure_change.both").to_string(),
        });
        if self.lost > 0 {
            sentences.push(t(locale, "status.structure_change.cause").to_string());
        }
        sentences.push(t(locale, "status.structure_change.undo").to_string());
        sentences
    }
}

/// Compares the outline `after` an operation with what the operation intended
/// to leave behind. `None` means the outline is exactly as intended — the
/// common case, and silent.
pub fn detect(
    before: &OutlineSummary,
    after: &OutlineSummary,
    expected: &ExpectedChange,
) -> Option<StructureChange> {
    // The titles that should still be there once the operation has done what it
    // meant to: everything before, minus what was meant to go.
    let mut intended = before.titles.clone();
    for title in &expected.removed {
        if let Some(count) = intended.get_mut(title) {
            *count -= 1;
            if *count == 0 {
                intended.remove(title);
            }
        }
    }

    let lost = surplus(&intended, &after.titles);
    let gained = surplus(&after.titles, &intended).saturating_sub(expected.added);
    (lost > 0 || gained > 0).then_some(StructureChange { lost, gained })
}

/// How many entries `a` has beyond what `b` has, counting duplicates.
fn surplus(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> usize {
    a.iter()
        .map(|(title, &n)| n.saturating_sub(b.get(title).copied().unwrap_or(0)))
        .sum()
}
