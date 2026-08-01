//! RFC-053 S5: `PlainTextAdapter` and the recovery mechanism it supplies
//! (RFC-052 §5.2, RFC-053 §18).

use crate::{
    Document, DocumentFormat, DocumentFormatAdapter, FocusedContent, JsonAdapter, MarkdownAdapter,
    PlainTextAdapter, StructureErrorKind, StructureNodeKind,
};

#[test]
fn build_structure_always_succeeds_with_exactly_one_hidden_node() {
    let structure = PlainTextAdapter
        .build_structure("anything at all, { not [ valid ] json (either")
        .expect("PlainTextAdapter never fails");

    assert_eq!(structure.format, DocumentFormat::PlainText);
    assert_eq!(structure.nodes.len(), 1);
    let node = &structure.nodes[0];
    assert_eq!(node.id, structure.root_id);
    assert_eq!(node.kind, StructureNodeKind::RawRegion);
    assert!(node.children.is_empty());

    let caps = &node.capabilities;
    assert!(caps.can_select.is_hidden());
    assert!(caps.can_edit_content.is_hidden());
    assert!(caps.can_add_inside.is_hidden());
    assert!(caps.can_add_after.is_hidden());
    assert!(caps.can_rename.is_hidden());
    assert!(caps.can_move_up.is_hidden());
    assert!(caps.can_move_down.is_hidden());
    assert!(caps.can_move_inside_previous.is_hidden());
    assert!(caps.can_move_out_one_level.is_hidden());
    assert!(caps.can_join_with_previous.is_hidden());
    assert!(caps.can_delete.is_hidden());
    assert!(caps.can_show_plain_text.is_allowed());
}

#[test]
fn build_structure_on_empty_source_still_succeeds() {
    let structure = PlainTextAdapter.build_structure("").expect("empty is fine");
    assert_eq!(structure.nodes.len(), 1);
}

#[test]
fn focused_content_reports_unsupported_with_raw_text_available() {
    let structure = PlainTextAdapter.build_structure("hello").unwrap();
    let content = PlainTextAdapter
        .focused_content("hello", &structure, structure.root_id)
        .expect("viewing is allowed, editing is not");
    let FocusedContent::Unsupported {
        reason,
        raw_text_available,
        ..
    } = content
    else {
        panic!("expected FocusedContent::Unsupported");
    };
    assert_eq!(reason, StructureErrorKind::UnsupportedFeature);
    assert!(raw_text_available);
}

#[test]
fn editing_methods_all_refuse_rather_than_guess() {
    let structure = PlainTextAdapter.build_structure("hello").unwrap();

    let validate_err = PlainTextAdapter
        .validate_focused_edit("hello", &structure, structure.root_id, "changed")
        .expect_err("no editable content");
    assert_eq!(validate_err.kind, StructureErrorKind::UnsupportedFeature);

    let mut document = Document::parse(String::new()).unwrap();
    let command_err = PlainTextAdapter
        .structure_command(
            &mut document,
            &structure,
            crate::StructureCommand::Delete {
                target: structure.root_id,
            },
        )
        .expect_err("no structural action is available");
    assert_eq!(command_err.kind, StructureErrorKind::UnsupportedFeature);
}

/// The RFC-052 §5.2 mechanism this slice exists to prove: when a real
/// format adapter refuses (today, `JsonAdapter` refuses everything —
/// RFC-054 has not landed), `PlainTextAdapter` succeeds on the exact same
/// source, and the source itself was never touched — trivially true, since
/// `build_structure` takes `&str`, an immutable borrow, so no adapter can
/// mutate the caller's text regardless of whether it succeeds or fails.
#[test]
fn when_a_format_adapter_fails_plain_text_still_succeeds_on_the_same_source() {
    let source = r#"{"key": "value"}"#;

    let json_err = JsonAdapter
        .build_structure(source)
        .expect_err("JSON is not implemented yet (RFC-054)");
    assert_eq!(json_err.kind, StructureErrorKind::UnsupportedFeature);

    let plain_text_structure = PlainTextAdapter
        .build_structure(source)
        .expect("the fallback always succeeds");
    assert_eq!(plain_text_structure.nodes.len(), 1);

    // The source binding itself is untouched — build_structure never had a
    // mutable path to it in the first place.
    assert_eq!(source, r#"{"key": "value"}"#);
}

#[test]
fn active_adapter_plain_text_variant_dispatches_correctly() {
    let adapter = crate::ActiveAdapter::PlainText(PlainTextAdapter);
    let format = match &adapter {
        crate::ActiveAdapter::PlainText(a) => a.format(),
        _ => unreachable!(),
    };
    assert_eq!(format, DocumentFormat::PlainText);
}

/// RFC-053 §18: deeply nested input must not overflow the stack. Markdown
/// heading depth is inherently bounded (ATX headings only go to H6), so the
/// realistic risk is in body content the shipped `pulldown-cmark`-based
/// parser still has to tokenize even though it produces no headings —
/// deeply nested block quotes are the classic pathological case for
/// recursive-descent Markdown parsers. `PlainTextAdapter` itself parses
/// nothing, so it is safe by construction; this test targets the shared
/// `Document::parse` path both `MarkdownAdapter::build_structure` and
/// `PlainTextAdapter`'s eventual real-world callers sit in front of.
#[test]
fn deeply_nested_block_quotes_do_not_overflow_the_stack() {
    let depth = 200_000;
    let mut source = "> ".repeat(depth);
    source.push_str("bottom\n");

    // Must not panic/abort (stack overflow aborts the process rather than
    // unwinding, so the meaningful assertion is that this line is reached
    // at all).
    let markdown_result = MarkdownAdapter.build_structure(&source);
    assert!(
        markdown_result.is_ok(),
        "must not error, and must not crash"
    );

    let plain_text_result = PlainTextAdapter.build_structure(&source);
    assert!(plain_text_result.is_ok());
    assert_eq!(
        plain_text_result.unwrap().nodes[0]
            .source_range
            .unwrap()
            .len(),
        source.len()
    );
}
