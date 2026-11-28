//! What opening and editing a large file costs, in this process (RFC-068 §7.8).
//!
//! A measurement tool, not shipped code: it generates a Markdown or JSON file of a given
//! size and then runs the same steps the application runs on open and on one edit,
//! printing the time and the process's resident memory after each. Run it in release:
//!
//! ```text
//! cargo run --release -p omriss-ui --example size_probe -- gen md   50 /tmp/p50.md
//! cargo run --release -p omriss-ui --example size_probe -- gen json 50 /tmp/p50.json
//! cargo run --release -p omriss-ui --example size_probe -- open /tmp/p50.md
//! ```
//!
//! `open` is the application's own sequence: read the bytes, take a BOM off, make the
//! text without copying it, detect the profile and format, open the session, focus the
//! first node as `settle_on_open` does, ask for the Document Map's nodes, then make one
//! edit (a Markdown section body, or a JSON scalar) and commit it. Memory is Linux's
//! `/proc/self/status` (`VmRSS` now, `VmHWM` the peak); elsewhere it prints 0.
//! What is *not* here is the WebView: the page's own memory and the time it takes to
//! draw the map are measured on the running application.

use std::fmt::Write as _;
use std::time::Instant;

use omriss_core::formats::detection::detect_format;
use omriss_ui::{ContentStamp, DocumentMapNode, EditorSession, FileTextProfile, node_id_from_raw};

fn status_kb(field: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(field))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

fn mb(kb: u64) -> String {
    format!("{:>7.1} MB", kb as f64 / 1024.0)
}

struct Clock(Instant);

impl Clock {
    fn step(&mut self, what: &str) {
        let ms = self.0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "{what:<46} {ms:>10.1} ms   rss {}   peak {}",
            mb(status_kb("VmRSS:")),
            mb(status_kb("VmHWM:"))
        );
        self.0 = Instant::now();
    }
}

/// Markdown: chapters of sections, each with a paragraph or two (about 1 KB a section).
fn markdown(target: usize) -> String {
    let mut s = String::with_capacity(target + 4096);
    let para = "The quick brown fox jumps over the lazy dog, and then explains at some length why \
        a text editor ought never to rewrite a byte the user did not touch. ";
    let (mut chapter, mut section) = (0, 0);
    while s.len() < target {
        if section % 50 == 0 {
            chapter += 1;
            let _ = write!(
                s,
                "# Chapter {chapter}\n\nIntroduction to chapter {chapter}.\n\n"
            );
        }
        section += 1;
        let _ = write!(s, "## Section {chapter}.{section}\n\n");
        for _ in 0..(1 + section % 3) {
            s.push_str(para);
            s.push_str("\n\n");
        }
        if section % 7 == 0 {
            let _ = write!(s, "### Detail {section}\n\nA note, kept short.\n\n");
        }
    }
    s
}

/// JSON: an object with an array of small records (about 220 bytes each).
fn json(target: usize) -> String {
    let mut s = String::with_capacity(target + 4096);
    s.push_str("{\n  \"meta\": {\"generator\": \"size_probe\", \"version\": 1},\n  \"items\": [\n");
    let mut i = 0usize;
    while s.len() < target {
        if i > 0 {
            s.push_str(",\n");
        }
        let _ = write!(
            s,
            "    {{\"id\": {i}, \"name\": \"item {i}\", \"tags\": [\"alpha\", \"beta\"], \
             \"note\": \"a short note about record {i}, long enough to look like text\", \
             \"nested\": {{\"x\": {}.5, \"y\": null, \"ok\": true}}}}",
            i % 1000
        );
        i += 1;
    }
    s.push_str("\n  ]\n}\n");
    s
}

fn count(node: &DocumentMapNode) -> usize {
    1 + node.children.iter().map(count).sum::<usize>()
}

/// The first node with no children, found by always taking the first child.
fn first_leaf(node: &DocumentMapNode) -> &DocumentMapNode {
    node.children.first().map_or(node, first_leaf)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["gen", kind, mb, path] => {
            let target = mb.parse::<usize>().expect("MB") * 1024 * 1024;
            let text = if *kind == "json" {
                json(target)
            } else {
                markdown(target)
            };
            std::fs::write(path, &text).unwrap();
            println!("{path}: {} bytes", text.len());
        }
        ["open", path] => open(path),
        _ => eprintln!("usage: size_probe gen md|json MB PATH | size_probe open PATH"),
    }
}

fn open(path: &str) {
    println!("{path}");
    let mut clock = Clock(Instant::now());
    clock.step("start");

    // read, and the stamp of the bytes as read (the application keeps it, RFC-068 §7.6)
    let mut bytes = std::fs::read(path).unwrap();
    clock.step("read the bytes");
    let _stamp = ContentStamp::of(&bytes);
    clock.step("stamp of the bytes (hash)");

    // BOM off, and the text made from the same allocation: no second copy of the file.
    let had_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    if had_bom {
        bytes.drain(..3);
    }
    // SIZE_PROBE_OLD_COPY=1 does what the application did before RFC-068 PR 4: copy the
    // bytes into the text and keep the originals alive meanwhile, to show what that cost.
    let text = if std::env::var_os("SIZE_PROBE_OLD_COPY").is_some() {
        let copied = String::from_utf8(bytes.to_vec()).expect("UTF-8");
        clock.step("text COPIED from the bytes (the old way)");
        drop(bytes);
        copied
    } else {
        let moved = String::from_utf8(bytes).expect("UTF-8");
        clock.step("text from the bytes (moved, not copied)");
        moved
    };

    let profile = FileTextProfile::detect(&text, had_bom);
    clock.step("profile (line endings, trailing newline)");
    let format = detect_format(Some(std::path::Path::new(path)), &text);
    clock.step("format detection");

    let mut session =
        EditorSession::open_detected(text, Some(std::path::PathBuf::from(path)), profile, format)
            .unwrap();
    clock.step("open the session (parse, index, saved copy)");
    session.settle_on_open();
    clock.step("settle on open (focus the first node)");

    let map = session.document_map_nodes();
    let nodes = count(&map);
    clock.step(&format!("Document Map nodes ({nodes} of them)"));

    if session.format() == omriss_core::DocumentFormat::Markdown {
        let id = session.outline_items()[0].id;
        let base = session.focus(id).unwrap();
        clock.step("focus the first section");
        session
            .commit_focused_body(&base, format!("{}x", base.body))
            .unwrap();
        clock.step("one edit: commit a section body");
    } else {
        let leaf = first_leaf(&map);
        session.focus(node_id_from_raw(leaf.id)).unwrap();
        clock.step("focus the first scalar");
        let edited = session.commit_structured_draft("42").is_ok();
        clock.step(&format!(
            "one edit: commit a JSON value (accepted: {edited})"
        ));
    }
    assert!(session.is_dirty() || session.format() != omriss_core::DocumentFormat::Markdown);

    let again = session.document_map_nodes();
    clock.step(&format!(
        "Document Map after the edit ({} nodes)",
        count(&again)
    ));
    let _ = session.is_dirty();
    clock.step("dirty check");
    println!(
        "final: rss {}   PEAK {}",
        mb(status_kb("VmRSS:")),
        mb(status_kb("VmHWM:"))
    );
}
