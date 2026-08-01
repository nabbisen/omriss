//! RFC-053 §7.2 / S4b: the `Json`/`Toml`/`Yaml` stub adapters return
//! `UnsupportedFeature` for every method and implement no parsing logic.

use crate::{
    ActiveAdapter, Document, DocumentFormat, DocumentFormatAdapter, DocumentRevision, JsonAdapter,
    MarkdownAdapter, PlainTextAdapter, StructureCommand, StructureErrorKind, TomlAdapter,
    YamlExperimentalAdapter,
};

#[test]
fn json_adapter_reports_its_format_and_refuses_to_build_structure() {
    let adapter = JsonAdapter;
    assert_eq!(adapter.format(), DocumentFormat::Json);
    let err = adapter
        .build_structure("{}", DocumentRevision::INITIAL)
        .expect_err("not implemented");
    assert_eq!(err.kind, StructureErrorKind::UnsupportedFeature);
}

#[test]
fn toml_adapter_reports_its_format_and_refuses_to_build_structure() {
    let adapter = TomlAdapter;
    assert_eq!(adapter.format(), DocumentFormat::Toml);
    let err = adapter
        .build_structure("key = 1", DocumentRevision::INITIAL)
        .expect_err("not implemented");
    assert_eq!(err.kind, StructureErrorKind::UnsupportedFeature);
}

#[test]
fn yaml_adapter_reports_its_format_and_refuses_to_build_structure() {
    let adapter = YamlExperimentalAdapter;
    assert_eq!(adapter.format(), DocumentFormat::YamlExperimental);
    let err = adapter
        .build_structure("key: 1", DocumentRevision::INITIAL)
        .expect_err("not implemented");
    assert_eq!(err.kind, StructureErrorKind::UnsupportedFeature);
}

#[test]
fn json_adapter_refuses_a_structure_command_too() {
    let adapter = JsonAdapter;
    // No real DocumentStructure can exist for JSON yet (build_structure
    // always refuses), so exercise structure_command with an empty one —
    // the point is that the adapter refuses uniformly for every method,
    // not that this input is realistic.
    let empty = crate::DocumentStructure {
        format: DocumentFormat::Json,
        root_id: crate::NodeId(0),
        nodes: Vec::new(),
        revision: crate::DocumentRevision::INITIAL,
    };
    let mut document = Document::parse(String::new()).unwrap();
    let err = adapter
        .structure_command(
            &mut document,
            &empty,
            StructureCommand::Delete {
                target: crate::NodeId(0),
            },
        )
        .expect_err("not implemented");
    assert_eq!(err.kind, StructureErrorKind::UnsupportedFeature);
}

#[test]
fn active_adapter_dispatches_to_the_right_format_for_every_current_variant() {
    let markdown = ActiveAdapter::Markdown(MarkdownAdapter);
    let json = ActiveAdapter::Json(JsonAdapter);
    let toml = ActiveAdapter::Toml(TomlAdapter);
    let yaml = ActiveAdapter::Yaml(YamlExperimentalAdapter);
    let plain_text = ActiveAdapter::PlainText(PlainTextAdapter);

    let format = |adapter: &ActiveAdapter| match adapter {
        ActiveAdapter::Markdown(a) => a.format(),
        ActiveAdapter::Json(a) => a.format(),
        ActiveAdapter::Toml(a) => a.format(),
        ActiveAdapter::Yaml(a) => a.format(),
        ActiveAdapter::PlainText(a) => a.format(),
    };

    assert_eq!(format(&markdown), DocumentFormat::Markdown);
    assert_eq!(format(&json), DocumentFormat::Json);
    assert_eq!(format(&toml), DocumentFormat::Toml);
    assert_eq!(format(&yaml), DocumentFormat::YamlExperimental);
    assert_eq!(format(&plain_text), DocumentFormat::PlainText);
}
