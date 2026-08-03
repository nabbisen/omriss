//! JSON format adapter (RFC-054 J2/J5/J6).
//!
//! `build_structure` (J2): `scanner` parses strict RFC 8259 JSON, retaining
//! a byte range for every value (including duplicate object keys, which
//! are never merged — RFC-054 §15.2), and `projection` turns the result
//! into a `DocumentStructure` per RFC-054 §5 (structure model) and §6
//! (node identity).
//!
//! `focused_content`/`validate_focused_edit`/`apply_validated_edit` (J5,
//! extended J6): real for every `Value` node whose literal is not `null`
//! (RFC-054 §8's text/number/on-off editing, J5) and, as of J6, for
//! `Group`/`List` nodes too (RFC-054 §7.4/§8.5's raw container-text
//! editing — conservative: a replacement must parse as JSON and keep the
//! same container kind, object stays object, array stays array). `null`
//! values still refuse: type-changing a `null` is out of scope per
//! RFC-054 §15 question 3, permanently, not just until some later slice.
//! `structure_command` refuses unconditionally regardless of slice:
//! RFC-054 §13 Phase 4 is out of scope for this entire handoff.
//!
//! Per the RFC-054 J5 scope decision
//! (`.git-exclude/reviewed/008-rfc-054-j5-scope-question.md`, which also
//! governs J6 — "core-only for J5 AND J6"): this slice is `omriss-core`
//! only. Nothing in `crates/ui`/`crates/app` calls these methods yet — a
//! JSON-aware focus/commit path through `EditorSession` and a right-panel
//! editor component are J7's scope.

mod error_mapping;
mod projection;
mod scalar;
mod scanner;
mod string_literal;

use error_mapping::map_edit_error;
use projection::find_node;
use scalar::ScalarKind;
use scanner::JsonValue;

use crate::formats::edit::{AppliedEdit, EditDescription, StructureCommand, ValidatedEdit};
use crate::formats::error::{
    ApplyEditError, EditValidationError, FocusError, StructureCommandError, StructureError,
    StructureErrorKind,
};
use crate::formats::focused_content::FocusedContent;
use crate::formats::structure::{DocumentStructure, StructureNodeKind};
use crate::{Document, DocumentFormat, DocumentFormatAdapter, DocumentRevision, NodeId};

/// The JSON format adapter (RFC-054).
///
/// Owns no state, per RFC-053 §3.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonAdapter;

impl DocumentFormatAdapter for JsonAdapter {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Json
    }

    /// Strict RFC 8259 only (RFC-052 §14.1, RFC-054 §0.5): comments,
    /// trailing commas, unquoted keys, and every other JSONC-style
    /// tolerance are rejected with `StructureErrorKind::InvalidSyntax`,
    /// never silently accepted.
    fn build_structure(
        &self,
        source: &str,
        revision: DocumentRevision,
    ) -> Result<DocumentStructure, StructureError> {
        let root = scanner::parse(source).map_err(|_| StructureErrorKind::InvalidSyntax)?;
        Ok(projection::build(&root, revision))
    }

    /// `Value` nodes report their `ValueKind` (RFC-054 §8) and both a
    /// display and an editable rendering of their literal; `Group`/`List`
    /// nodes report a child count and a raw-source preview. This method's
    /// own output is unchanged by J6 (it was already real, read-only, in
    /// J5) — J6 makes what it reports for `Group`/`List` actually
    /// editable via `validate_focused_edit`.
    fn focused_content(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
    ) -> Result<FocusedContent, FocusError> {
        let node = find_node(structure, node_id)?;
        match node.kind {
            StructureNodeKind::Value => {
                let range = node
                    .source_range
                    .ok_or(StructureErrorKind::InternalInvariantFailed)?;
                let literal = source
                    .get(range.as_range())
                    .ok_or(StructureErrorKind::InternalInvariantFailed)?;
                let kind = scalar::classify(source, range);
                let text = match kind {
                    ScalarKind::Text => scalar::decode_string(source, range),
                    ScalarKind::Number | ScalarKind::OnOff => literal.to_string(),
                    // "No value", not the literal "null" -- ValueKind::NoValue's
                    // own doc comment: distinct from an empty string, and
                    // there is nothing useful to show as editable text for it.
                    ScalarKind::NoValue => String::new(),
                };
                Ok(FocusedContent::StructuredValue {
                    title: node.title.clone(),
                    value_kind: kind.to_value_kind(),
                    display_text: text.clone(),
                    editable_text: text,
                })
            }
            StructureNodeKind::Group | StructureNodeKind::List => {
                let range = node
                    .source_range
                    .ok_or(StructureErrorKind::InternalInvariantFailed)?;
                let raw = source
                    .get(range.as_range())
                    .ok_or(StructureErrorKind::InternalInvariantFailed)?;
                Ok(FocusedContent::StructuredGroup {
                    title: node.title.clone(),
                    child_count: node.children.len(),
                    summary: raw_preview(raw),
                    raw_text_available: true,
                })
            }
            _ => Err(StructureErrorKind::InternalInvariantFailed.into()),
        }
    }

    /// Validates a draft per RFC-054 §8 (scalars, J5) and §8.5 (container
    /// raw text, J6). Refuses only for `null` (type-changing is out of
    /// scope, RFC-054 §15 question 3) — matching each node's own
    /// `can_edit_content` capability (`projection::json_node_capabilities`).
    fn validate_focused_edit(
        &self,
        source: &str,
        structure: &DocumentStructure,
        node_id: NodeId,
        draft: &str,
    ) -> Result<ValidatedEdit, EditValidationError> {
        let node = find_node(structure, node_id)?;
        let range = node
            .editable_range
            .ok_or(StructureErrorKind::InternalInvariantFailed)?;
        let replacement_text = match node.kind {
            StructureNodeKind::Value => match scalar::classify(source, range) {
                ScalarKind::NoValue => return Err(StructureErrorKind::UnsupportedFeature.into()),
                ScalarKind::Text => scalar::encode_string(draft),
                ScalarKind::Number => {
                    if scalar::is_valid_number(draft) {
                        draft.to_string()
                    } else {
                        return Err(StructureErrorKind::InvalidSyntax.into());
                    }
                }
                ScalarKind::OnOff => {
                    if scalar::is_valid_bool(draft) {
                        draft.to_string()
                    } else {
                        return Err(StructureErrorKind::InvalidSyntax.into());
                    }
                }
            },
            StructureNodeKind::Group | StructureNodeKind::List => {
                if is_valid_container_replacement(node.kind, draft) {
                    draft.to_string()
                } else {
                    return Err(StructureErrorKind::InvalidSyntax.into());
                }
            }
            _ => return Err(StructureErrorKind::UnsupportedFeature.into()),
        };
        Ok(ValidatedEdit {
            node_id,
            base_revision: structure.revision,
            replacement_range: range,
            replacement_text,
            description: EditDescription::FocusedContentReplacement,
        })
    }

    /// Wraps the shipped `Document::replace_range` (RFC-054 §0.1/J1) —
    /// the format-neutral counterpart to `MarkdownAdapter`'s
    /// `replace_section_body` wrap, routed through the same transactional
    /// path (RFC-053 §7.1).
    fn apply_validated_edit(
        &self,
        document: &mut Document,
        edit: ValidatedEdit,
    ) -> Result<AppliedEdit, ApplyEditError> {
        let result = document
            .replace_range(
                edit.replacement_range,
                edit.replacement_text,
                edit.base_revision,
            )
            .map_err(map_edit_error)?;
        Ok(AppliedEdit { result })
    }

    /// RFC-054 §13 Phase 4 (add/delete/rename/move) is out of scope for
    /// this entire handoff, not just this slice; refuses unconditionally.
    fn structure_command(
        &self,
        _document: &mut Document,
        _structure: &DocumentStructure,
        _command: StructureCommand,
    ) -> Result<AppliedEdit, StructureCommandError> {
        Err(StructureErrorKind::UnsupportedFeature.into())
    }
}

/// A single-line, length-capped preview of a `Group`/`List` node's own raw
/// source. Not prose: `omriss-core` must never name a catalog key
/// (RFC-001), and this is not one — it is the user's own file content,
/// quoted back at them, which needs no localization. Deliberately not the
/// RFC-054 §4.3 mockup's "This group contains 2 items." sentence: that
/// sentence is fully derivable by the UI from `child_count` alone, and
/// baking English prose into a core adapter would be exactly the
/// dependency-direction problem RFC-001/RFC-053 §11 exist to prevent.
fn raw_preview(raw: &str) -> String {
    const MAX_CHARS: usize = 80;
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > MAX_CHARS {
        let truncated: String = collapsed.chars().take(MAX_CHARS).collect();
        format!("{truncated}\u{2026}")
    } else {
        collapsed
    }
}

/// RFC-054 §8.5: a container raw-text replacement must parse as valid JSON
/// and be "appropriate to the selected node." Conservative first
/// implementation (§8.5's own wording): the container's kind is preserved,
/// not changeable — an object stays an object, an array stays an array —
/// mirroring §15 question 3's rule that a scalar `null` cannot be
/// type-changed either. `scanner::parse` rejects trailing content after
/// the value (the same grammar `build_structure` enforces), so this also
/// refuses a draft that is valid JSON followed by garbage.
fn is_valid_container_replacement(kind: StructureNodeKind, draft: &str) -> bool {
    matches!(
        (kind, scanner::parse(draft)),
        (StructureNodeKind::Group, Ok(JsonValue::Object { .. }))
            | (StructureNodeKind::List, Ok(JsonValue::Array { .. }))
    )
}
