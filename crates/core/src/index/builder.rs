//! Outline construction from canonical Markdown source
//! (RFC-003 heading indexing, RFC-007 tree construction).
//!
//! Headings are detected through parser events, never line regexes, so that
//! heading-like text inside fenced code blocks is ignored. YAML (`---`) and
//! TOML (`+++`) front matter is recognized as a metadata block so that the
//! line after an opening `---` cannot be misread as a Setext heading; the
//! front matter bytes themselves remain untouched root-level source content.

use pulldown_cmark::{Event, Options, Parser, Tag};

use crate::error::IndexError;
use crate::index::outline::{HeadingLevel, NodeId, Outline, SectionNode};
use crate::range::ByteRange;

/// A heading occurrence collected from parser events, before tree assembly.
struct RawHeading {
    level: HeadingLevel,
    title: String,
    /// The full heading element (ATX line or Setext lines) including the
    /// trailing newline when present (verified pulldown-cmark behavior).
    heading_range: ByteRange,
}

fn parser_options() -> Options {
    Options::ENABLE_YAML_STYLE_METADATA_BLOCKS | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
}

fn convert_level(level: pulldown_cmark::HeadingLevel) -> HeadingLevel {
    use pulldown_cmark::HeadingLevel as P;
    match level {
        P::H1 => HeadingLevel::H1,
        P::H2 => HeadingLevel::H2,
        P::H3 => HeadingLevel::H3,
        P::H4 => HeadingLevel::H4,
        P::H5 => HeadingLevel::H5,
        P::H6 => HeadingLevel::H6,
    }
}

/// Collects headings in source order with their byte ranges and plain titles.
fn collect_headings(source: &str) -> Vec<RawHeading> {
    let mut headings = Vec::new();
    let mut current: Option<RawHeading> = None;

    for (event, range) in Parser::new_ext(source, parser_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some(RawHeading {
                    level: convert_level(level),
                    title: String::new(),
                    heading_range: ByteRange::from(range),
                });
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                if let Some(mut heading) = current.take() {
                    heading.title = heading.title.trim().to_string();
                    headings.push(heading);
                }
            }
            // Flatten inline markup into a plain title.
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = current.as_mut() {
                    heading.title.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = current.as_mut() {
                    heading.title.push(' ');
                }
            }
            _ => {}
        }
    }
    headings
}

/// Builds the derived outline for `source`.
///
/// The tree uses the RFC-003 stack algorithm: a heading's parent is the
/// nearest previous heading with a strictly lower level, else the synthetic
/// root. Skipped levels (`# A` followed by `### B`) are represented literally
/// with no synthetic intermediate nodes.
pub(crate) fn build_outline(source: &str) -> Result<Outline, IndexError> {
    let headings = collect_headings(source);
    let len = source.len();

    // Full range end per heading: start of the next heading with the same or
    // shallower level, else end of document.
    let mut full_ends = vec![len; headings.len()];
    for (i, heading) in headings.iter().enumerate() {
        for later in &headings[i + 1..] {
            if later.level <= heading.level {
                full_ends[i] = later.heading_range.start;
                break;
            }
        }
    }

    // Body end per heading: the next heading in source order ends the body if
    // it lies inside this heading's full range (it is then this section's
    // first descendant); otherwise the body runs to the full range end.
    let body_end = |i: usize| -> usize {
        match headings.get(i + 1) {
            Some(next) if next.heading_range.start < full_ends[i] => next.heading_range.start,
            _ => full_ends[i],
        }
    };

    // Synthetic root (RFC-007 root design).
    let first_heading_start = headings.first().map_or(len, |h| h.heading_range.start);
    let root_id = NodeId::from_ordinal_path(&[]);
    let mut nodes = vec![SectionNode {
        id: root_id,
        parent_id: None,
        level: None,
        title: String::new(),
        heading_range: ByteRange::empty_at(0),
        body_range: ByteRange {
            start: 0,
            end: first_heading_start,
        },
        full_range: ByteRange { start: 0, end: len },
        children: Vec::new(),
        ordinal: 0,
    }];

    // Stack of (node index in `nodes`, level as depth with root = 0,
    // ordinal path from root).
    let mut stack: Vec<(usize, u8, Vec<usize>)> = vec![(0, 0, Vec::new())];

    for (i, heading) in headings.iter().enumerate() {
        let depth = heading.level.as_u8();
        while stack.last().is_some_and(|(_, level, _)| *level >= depth) {
            stack.pop();
        }
        let (parent_idx, _, parent_path) = stack
            .last()
            .cloned()
            .ok_or(IndexError::InvariantViolation("indexing stack underflow"))?;

        let ordinal = nodes[parent_idx].children.len();
        let mut path = parent_path;
        path.push(ordinal);
        let id = NodeId::from_ordinal_path(&path);
        let parent_id = nodes[parent_idx].id;

        let node_idx = nodes.len();
        nodes.push(SectionNode {
            id,
            parent_id: Some(parent_id),
            level: Some(heading.level),
            title: heading.title.clone(),
            heading_range: heading.heading_range,
            body_range: ByteRange {
                start: heading.heading_range.end,
                end: body_end(i),
            },
            full_range: ByteRange {
                start: heading.heading_range.start,
                end: full_ends[i],
            },
            children: Vec::new(),
            ordinal,
        });
        nodes[parent_idx].children.push(id);
        stack.push((node_idx, depth, path));
    }

    let outline = Outline::from_nodes(nodes)?;
    outline.validate(source)?;
    Ok(outline)
}
