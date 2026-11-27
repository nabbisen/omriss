//! RFC-085 §8.2: a node's own text is the format's to say, read from the node.
//! One method on every adapter; JSON shares its decoding with `focused_content`.

use crate::{
    ByteRange, DocumentFormatAdapter, DocumentRevision, FocusedContent, JsonAdapter,
    MarkdownAdapter, PlainTextAdapter, StructureNode, TomlAdapter, ValueKind,
    YamlExperimentalAdapter,
};

fn structure_of(adapter: &impl DocumentFormatAdapter, source: &str) -> crate::DocumentStructure {
    adapter
        .build_structure(source, DocumentRevision::INITIAL)
        .unwrap()
}

fn node<'a>(s: &'a crate::DocumentStructure, title: &str) -> &'a StructureNode {
    s.nodes.iter().find(|n| n.title == title).unwrap()
}

#[test]
fn a_markdown_sections_own_text_is_its_body_byte_for_byte() {
    let source = "intro\n\n# A\nbody a\n\n## B\nbody b\n";
    let s = structure_of(&MarkdownAdapter, source);
    let a = MarkdownAdapter.node_text(source, node(&s, "A")).unwrap();
    assert_eq!(a.text, "body a\n\n");
    assert!(a.exact && a.value_kind.is_none());
    assert_eq!(
        &source[a.range.as_range()],
        a.text,
        "exact: the text is the source at the range"
    );
    let root = MarkdownAdapter
        .node_text(source, s.nodes.iter().find(|n| n.id == s.root_id).unwrap())
        .unwrap();
    assert_eq!(root.text, "intro\n\n");
}

#[test]
fn a_json_values_own_text_is_decoded_and_says_what_it_is() {
    let source = r#"{"s": "café \/ \n", "n": 3, "b": true, "z": null, "g": {"k": 1}, "l": [1]}"#;
    let s = structure_of(&JsonAdapter, source);
    let own = |t: &str| JsonAdapter.node_text(source, node(&s, t));

    let text = own("s").unwrap();
    assert_eq!(text.text, "café / \n");
    assert_eq!(text.value_kind, Some(ValueKind::Text));
    assert!(!text.exact, "a decoded string is not the source");
    assert_eq!(
        &source[text.range.as_range()],
        r#""café \/ \n""#,
        "the range is the whole value"
    );

    assert_eq!(own("n").unwrap().text, "3");
    assert_eq!(own("n").unwrap().value_kind, Some(ValueKind::Number));
    assert_eq!(own("b").unwrap().text, "true");
    assert_eq!(own("b").unwrap().value_kind, Some(ValueKind::OnOff));
    let null = own("z").unwrap();
    assert_eq!(
        (null.text.as_str(), null.value_kind),
        ("", Some(ValueKind::NoValue))
    );
    // A group or list has no text of its own: its members are nodes.
    assert!(own("g").is_none() && own("l").is_none());
    assert!(own("k").is_some());
}

#[test]
fn json_shares_one_decoder_between_node_text_and_focused_content() {
    let source = r#"["a\tb", "😀", 1.5e3, false, null, ""]"#;
    let s = structure_of(&JsonAdapter, source);
    for n in s.nodes.iter().filter(|n| n.id != s.root_id) {
        let own = JsonAdapter.node_text(source, n).unwrap();
        let FocusedContent::StructuredValue {
            display_text,
            value_kind,
            ..
        } = JsonAdapter.focused_content(source, &s, n.id).unwrap()
        else {
            panic!("a value");
        };
        assert_eq!(own.text, display_text, "{}", n.title);
        assert_eq!(own.value_kind, Some(value_kind), "{}", n.title);
    }
}

#[test]
fn plain_text_is_one_region_and_the_unbuilt_formats_have_none() {
    let source = "just some text\n";
    let s = structure_of(&PlainTextAdapter, source);
    let own = PlainTextAdapter.node_text(source, &s.nodes[0]).unwrap();
    assert_eq!(own.text, source);
    assert!(own.exact);
    assert_eq!(
        own.range,
        ByteRange {
            start: 0,
            end: source.len()
        }
    );

    // TOML and YAML build no structure, so no node ever reaches them.
    let fake = s.nodes[0].clone();
    assert!(TomlAdapter.node_text(source, &fake).is_none());
    assert!(YamlExperimentalAdapter.node_text(source, &fake).is_none());
}
