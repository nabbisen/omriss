//! Performance benchmarks for omriss-core (RFC-031).
//!
//! Run with: `cargo bench -p omriss-core`
//!
//! Measurement points per RFC-031 §4:
//! - full parse + index build on small, medium, and large fixtures;
//! - section body replacement (commit path);
//! - structural move on a mid-size document.
//!
//! All fixtures are loaded from disk once per benchmark group; the source
//! string is kept in memory so file I/O does not skew timing.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use omriss_core::{Document, HeadingLevel, MoveTarget, ReplaceSectionBody};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

// ── Parse + index benchmarks ──────────────────────────────────────────────

fn bench_parse_small(c: &mut Criterion) {
    let src = load_fixture("nested_atx.md");
    c.bench_function("parse+index/nested_atx", |b| {
        b.iter(|| Document::parse(black_box(src.clone())).unwrap())
    });
}

fn bench_parse_large(c: &mut Criterion) {
    let src = load_fixture("large-10k-words.md");
    let mut g = c.benchmark_group("parse+index/large-10k");
    g.sample_size(20);
    g.bench_function("parse+index", |b| {
        b.iter(|| Document::parse(black_box(src.clone())).unwrap())
    });
    g.finish();
}

fn bench_parse_academic(c: &mut Criterion) {
    let src = load_fixture("academic-paper.md");
    c.bench_function("parse+index/academic-paper", |b| {
        b.iter(|| Document::parse(black_box(src.clone())).unwrap())
    });
}

// ── Section body replacement (commit path) ────────────────────────────────

fn bench_replace_body_small(c: &mut Criterion) {
    let src = load_fixture("nested_atx.md");
    c.bench_function("replace_body/nested_atx", |b| {
        b.iter(|| {
            let mut doc = Document::parse(src.clone()).unwrap();
            let id = doc.outline().root().children[0];
            doc.replace_section_body(ReplaceSectionBody {
                node_id: id,
                base_revision: doc.revision(),
                new_body: "replacement body\n".to_string(),
            })
            .unwrap();
        })
    });
}

fn bench_replace_body_large(c: &mut Criterion) {
    let src = load_fixture("large-10k-words.md");
    let mut g = c.benchmark_group("replace_body/large-10k");
    g.sample_size(20);
    g.bench_function("replace_body", |b| {
        b.iter(|| {
            let mut doc = Document::parse(src.clone()).unwrap();
            let id = doc.outline().root().children[0];
            doc.replace_section_body(ReplaceSectionBody {
                node_id: id,
                base_revision: doc.revision(),
                new_body: "replacement body\n".to_string(),
            })
            .unwrap();
        })
    });
    g.finish();
}

// ── Structural operations ─────────────────────────────────────────────────

fn bench_promote_section(c: &mut Criterion) {
    let src = load_fixture("large-10k-words.md");
    c.bench_function("structural/promote_large", |b| {
        b.iter(|| {
            let mut doc = Document::parse(src.clone()).unwrap();
            // Promote a deep section; pick the first grandchild.
            let root_child = doc.outline().root().children[0];
            if let Some(&grandchild) = doc
                .outline()
                .node(root_child)
                .and_then(|n| n.children.first())
            {
                doc.demote_section(grandchild, doc.revision()).ok();
            }
        })
    });
}

fn bench_move_section(c: &mut Criterion) {
    let src = load_fixture("large-10k-words.md");
    c.bench_function("structural/move_sibling_large", |b| {
        b.iter(|| {
            let mut doc = Document::parse(src.clone()).unwrap();
            let children = doc.outline().root().children.clone();
            if children.len() >= 2 {
                let a = children[0];
                let b_id = children[1];
                doc.move_section(a, MoveTarget::After(b_id), doc.revision())
                    .ok();
            }
        })
    });
}

// ── Split section ─────────────────────────────────────────────────────────

fn bench_split_section(c: &mut Criterion) {
    let src = load_fixture("academic-paper.md");
    c.bench_function("structural/split_academic", |b| {
        b.iter(|| {
            let mut doc = Document::parse(src.clone()).unwrap();
            let id = doc.outline().root().children[0];
            let body_len = doc.outline().node(id).unwrap().body_range.len();
            doc.split_section(
                id,
                body_len / 2,
                "New Section",
                HeadingLevel::H2,
                doc.revision(),
            )
            .ok();
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_large,
    bench_parse_academic,
    bench_replace_body_small,
    bench_replace_body_large,
    bench_promote_section,
    bench_move_section,
    bench_split_section,
);
criterion_main!(benches);
