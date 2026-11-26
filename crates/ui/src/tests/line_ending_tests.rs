//! RFC-084: editing a section's text must not rewrite line endings the user did
//! not touch, and must write the newlines it inserts in the document's style.
//!
//! A browser text area delivers `\n` whatever the file uses, so every draft here
//! is the original *as a text area shows it* with an edit applied — exactly what
//! the page hands to the application.

use omriss_core::{DocumentFormat, NodeId, normalize_to_lf};

use crate::file::file_profile::FileTextProfile;
use crate::interface::document_map::{DocumentMapNode, node_id_from_raw};
use crate::session::EditorSession;

fn find_in(node: &DocumentMapNode, title: &str) -> Option<NodeId> {
    if node.title == title {
        return Some(node_id_from_raw(node.id));
    }
    node.children.iter().find_map(|c| find_in(c, title))
}

fn titles(node: &DocumentMapNode, out: &mut Vec<String>) {
    if !node.title.is_empty() {
        out.push(node.title.clone());
    }
    for child in &node.children {
        titles(child, out);
    }
}

fn open(source: &str) -> EditorSession {
    EditorSession::open(source.to_string(), None).unwrap()
}

fn focus(s: &mut EditorSession, title: &str) {
    let id = find_in(&s.document_map_nodes(), title).unwrap_or_else(|| panic!("no {title:?}"));
    s.focus(id).unwrap();
}

fn body(s: &EditorSession) -> String {
    s.current_snapshot().unwrap().body
}

/// Commits `draft` to the focused section.
fn commit(s: &mut EditorSession, draft: &str) {
    let snapshot = s.current_snapshot().unwrap();
    s.commit_focused_body(&snapshot, draft.to_string()).unwrap();
}

fn lf_doc() -> String {
    "# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody three\n".into()
}
fn crlf(s: &str) -> String {
    s.replace('\n', "\r\n")
}
/// Alternating endings: CRLF on every other line.
fn mixed(s: &str) -> String {
    s.split_inclusive('\n')
        .enumerate()
        .map(|(i, line)| {
            if i % 2 == 0 {
                line.replace('\n', "\r\n")
            } else {
                line.to_string()
            }
        })
        .collect()
}

// ── the property (RFC-084 §4.5) ─────────────────────────────────────────────

/// Every line break a bare CR.
fn bare_cr(s: &str) -> String {
    s.replace('\n', "\r")
}
/// Line endings in rotation: LF, CRLF, CR, LF, …
fn rotating(s: &str) -> String {
    s.split_inclusive('\n')
        .enumerate()
        .map(|(i, line)| match i % 3 {
            0 => line.to_string(),
            1 => line.replace('\n', "\r\n"),
            _ => line.replace('\n', "\r"),
        })
        .collect()
}
/// Bare CRs on the first lines, LF on the rest.
fn some_cr(s: &str) -> String {
    s.split_inclusive('\n')
        .enumerate()
        .map(|(i, line)| {
            if i < 3 {
                line.replace('\n', "\r")
            } else {
                line.to_string()
            }
        })
        .collect()
}

fn count(s: &str, c: char) -> usize {
    s.matches(c).count()
}

fn bare_lf(s: &str) -> usize {
    let b = s.as_bytes();
    (0..b.len())
        .filter(|&i| b[i] == b'\n' && (i == 0 || b[i - 1] != b'\r'))
        .count()
}

/// Source bytes spanned by the first `n` characters a text area shows of `text`
/// (a CRLF is one shown character and two bytes).
fn source_len_of_view_head(text: &str, n: usize) -> usize {
    let mut chars = text.char_indices().peekable();
    let mut shown = 0;
    let mut at = 0;
    while shown < n {
        let Some((i, c)) = chars.next() else { break };
        at = i + c.len_utf8();
        if c == '\r' && chars.peek().map(|&(_, n)| n) == Some('\n') {
            let (j, n) = chars.next().unwrap();
            at = j + n.len_utf8();
        }
        shown += 1;
    }
    at
}

/// Source bytes spanned by the last `n` characters a text area shows of `text`.
fn source_len_of_view_tail(text: &str, n: usize) -> usize {
    let bytes = text.as_bytes();
    let mut at = bytes.len();
    for _ in 0..n {
        if at >= 2 && bytes[at - 2] == b'\r' && bytes[at - 1] == b'\n' {
            at -= 2;
        } else {
            at -= text[..at].chars().next_back().map_or(0, char::len_utf8);
        }
    }
    bytes.len() - at
}

/// Every `(i, j)` char span of `view` on char boundaries, with a handful of
/// inserted snippets: the edits a person makes by typing, deleting, pasting.
fn drafts(view: &str) -> Vec<String> {
    let idx: Vec<usize> = view
        .char_indices()
        .map(|(i, _)| i)
        .chain([view.len()])
        .collect();
    let mut out = Vec::new();
    for (a, &i) in idx.iter().enumerate() {
        for &j in &idx[a..] {
            for snippet in ["", "x", "\n", "x\n", "\ny", "é", "a\nb\n"] {
                let draft = format!("{}{}{}", &view[..i], snippet, &view[j..]);
                if draft != view {
                    out.push(draft);
                }
            }
        }
    }
    out
}

/// The property, for one document: for every section, every edit as a text area
/// would deliver it —
///
/// * the text area shows exactly the draft afterwards;
/// * bytes outside the changed span are unchanged (the unchanged head and tail
///   of the body, and everything outside the body);
/// * every newline the edit inserted is the document's: no bare `\n` appears in a
///   file that uses CRLF, no `\r` appears in a file that does not;
/// * undo restores the original exactly.
///
/// A section followed by a heading whose edited text does not end in a line
/// break gains exactly one — core's separator, in the document's newline.
fn property(source: &str) -> usize {
    let probe = open(source);
    let mut all = Vec::new();
    titles(&probe.document_map_nodes(), &mut all);
    let mut checked = 0;
    for title in all {
        let mut s = open(source);
        focus(&mut s, &title);
        let original = body(&s);
        let view = normalize_to_lf(&original);
        let is_last = source.ends_with(&original);
        for draft in drafts(&view) {
            let mut s = open(source);
            focus(&mut s, &title);
            commit(&mut s, &draft);
            let after = s.source().to_string();
            let ctx = format!("{title:?}, draft {draft:?}, doc {source:?}");

            // The text area shows what the user made — plus, in a section that
            // is followed by a heading, the one line break that keeps that
            // heading on its own line (core's `joining_separator`).
            let shown_draft = normalize_to_lf(&draft);
            let separator = if !is_last && !draft.is_empty() && !draft.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            assert_eq!(
                normalize_to_lf(&body(&s)),
                format!("{shown_draft}{separator}"),
                "the editor must show the draft and nothing else: {ctx}"
            );
            // Everything outside the changed span is the same bytes: the head
            // and the tail the text area did not change, and the document around
            // the body.
            let after_body = body(&s);
            let head = view
                .chars()
                .zip(draft.chars())
                .take_while(|(x, y)| x == y)
                .count();
            let tail = view
                .chars()
                .rev()
                .zip(draft.chars().rev())
                .take_while(|(x, y)| x == y)
                .count()
                .min(view.chars().count() - head)
                .min(draft.chars().count() - head);
            // The one exception: a lone CR (or a run of them) right before the
            // edit, where the edit puts a line break the CR would fuse with into a
            // CRLF, is written as the document's newline.
            let head_bytes = &original[..source_len_of_view_head(&original, head)];
            assert!(
                after_body.starts_with(head_bytes)
                    || after_body.starts_with(head_bytes.trim_end_matches('\r')),
                "the unchanged head of the body changed: {ctx} -> {after_body:?}"
            );
            assert!(
                after_body.ends_with(
                    &original[original.len() - source_len_of_view_tail(&original, tail)..]
                ),
                "the unchanged tail of the body changed: {ctx} -> {after_body:?}"
            );
            let body_start = source.find(&original).expect("the body is in the source");
            assert!(
                after.starts_with(&source[..body_start]),
                "bytes before the body changed: {ctx}"
            );
            assert!(
                after.ends_with(&source[body_start + original.len()..]),
                "bytes after the body changed: {ctx}"
            );
            // Inserted newlines are the document's (RFC-084 §10.3: its newline is
            // CRLF if it has any CRLF, else CR if its breaks are bare CRs only,
            // else LF).
            match omriss_core::document_newline(source) {
                "\r\n" => assert!(
                    bare_lf(&after) <= bare_lf(source),
                    "a bare LF was inserted into a CRLF file: {ctx} -> {after:?}"
                ),
                "\r" => assert!(
                    !after.contains('\n'),
                    "an LF was inserted into a bare-CR file: {ctx} -> {after:?}"
                ),
                _ if !source.contains('\r') => {
                    assert!(!after.contains('\r'), "a CR appeared in an LF file: {ctx}")
                }
                // LF beside lone CRs: the edit may rewrite a lone CR beside the
                // text it touches, and writes at most one CR itself — the first
                // newline after a heading ended by a lone CR, which an LF would
                // fuse with (the heading's ending would change).
                _ => assert!(
                    count(&after, '\r') <= count(source, '\r') + 1,
                    "CRs were inserted into an LF file: {ctx} -> {after:?}"
                ),
            }
            // Undo is byte-exact.
            s.undo().unwrap();
            assert_eq!(s.source(), source, "undo must restore the original: {ctx}");
            checked += 1;
        }
    }
    checked
}

#[test]
fn an_lf_file_keeps_its_bytes_outside_the_edit() {
    assert!(property(&lf_doc()) > 500);
}

#[test]
fn a_crlf_file_keeps_its_crlf_outside_the_edit_and_inserts_crlf() {
    assert!(property(&crlf(&lf_doc())) > 500);
}

#[test]
fn a_mixed_file_keeps_each_untouched_ending_and_inserts_crlf() {
    assert!(property(&mixed(&lf_doc())) > 500);
}

#[test]
fn a_bare_cr_file_keeps_its_bytes_outside_the_edit_and_inserts_cr() {
    // Task 028 D6: a file whose lines end in a bare CR (classic Mac OS).
    assert!(property(&bare_cr(&lf_doc())) > 500);
}

#[test]
fn a_file_rotating_lf_crlf_and_cr_keeps_each_untouched_ending() {
    assert!(property(&rotating(&lf_doc())) > 500);
}

#[test]
fn a_file_with_some_bare_crs_keeps_each_untouched_ending() {
    assert!(property(&some_cr(&lf_doc())) > 500);
}

#[test]
fn task_028s_minimal_example_keeps_the_headings_cr_and_the_blank_line() {
    // `# A\rb\r`, the body `b\r`; the user types a line break at its start. The
    // heading's CR stays, the typed line break is a CR, the blank line exists.
    let mut s = open("# A\rb\r");
    focus(&mut s, "A");
    commit(&mut s, "\nb\n");
    assert_eq!(s.source(), "# A\r\rb\r");
    s.undo().unwrap();
    assert_eq!(s.source(), "# A\rb\r");

    // The same where the document also has LFs, so its newline is LF and an LF
    // after the heading's lone CR would have fused with it into a CRLF.
    let doc = "# A\rb\n\n# C\nc\n";
    let mut s = open(doc);
    focus(&mut s, "A");
    commit(&mut s, "\nb\n\n");
    assert_eq!(s.source(), "# A\r\rb\n\n# C\nc\n");
    s.undo().unwrap();
    assert_eq!(s.source(), doc);
}

#[test]
fn deleting_the_text_between_a_lone_cr_and_an_lf_does_not_fuse_them() {
    // The heading's `\r` and the body's `\n`s would become one CRLF if the `x`
    // between them went: the heading's ending stays, and the body still shows its
    // two breaks. (The body's own breaks are read as `\r` to keep it so.)
    let doc = "# A\rx\n\n# C\nc\n";
    let mut s = open(doc);
    focus(&mut s, "A");
    commit(&mut s, "\n\n");
    assert!(s.source().starts_with("# A\r"), "{:?}", s.source());
    assert_eq!(normalize_to_lf(&body(&s)), "\n\n");
    assert!(!s.source().starts_with("# A\r\n"));
    s.undo().unwrap();
    assert_eq!(s.source(), doc);
}

#[test]
fn a_bare_cr_files_inserted_newlines_are_cr() {
    let doc = "# A\ra\r\r# B\rb\r";
    let mut s = open(doc);
    focus(&mut s, "A");
    commit(&mut s, "a\nmore\n\n");
    assert_eq!(s.source(), "# A\ra\rmore\r\r# B\rb\r");
    assert!(!s.source().contains('\n'));
}

#[test]
fn a_file_without_a_final_newline_and_with_nested_sections() {
    let doc = "# A\nintro\n\n## B\nbee\n\n# C\nsea";
    for doc in [
        doc.to_string(),
        crlf(doc),
        mixed(doc),
        bare_cr(doc),
        rotating(doc),
        some_cr(doc),
    ] {
        assert!(property(&doc) > 300);
    }
}

// ── the reported cases ──────────────────────────────────────────────────────

#[test]
fn typing_in_a_crlf_section_keeps_the_sections_other_lines_crlf() {
    // Task 027 case 4a/4b, the part that does not involve a separator: an edit
    // that ends with a newline, and one in the last section.
    let doc = crlf(&lf_doc());
    let mut s = open(&doc);
    focus(&mut s, "Two");
    commit(&mut s, "body two EDITED\n\n");
    assert_eq!(
        s.source(),
        "# One\r\nbody one\r\n\r\n# Two\r\nbody two EDITED\r\n\r\n# Three\r\nbody three\r\n"
    );

    let mut s = open(&doc);
    focus(&mut s, "Three");
    commit(&mut s, "body three\nx");
    // The user typed `\n` at the end of the old body and then `x`: one CRLF.
    assert_eq!(
        s.source(),
        "# One\r\nbody one\r\n\r\n# Two\r\nbody two\r\n\r\n# Three\r\nbody three\r\nx"
    );
}

#[test]
fn a_draft_that_shows_what_the_section_shows_is_not_pending() {
    let mut s = open(&crlf(&lf_doc()));
    focus(&mut s, "Two");
    let as_delivered = normalize_to_lf(&body(&s));
    assert!(
        !s.body_draft_differs(&as_delivered),
        "an untouched CRLF section is not pending"
    );
    assert!(s.body_draft_differs(&format!("{as_delivered}x")));
    // The unchanged draft would commit nothing visible, but the committed bytes
    // are the original either way.
    let before = s.source().to_string();
    commit(&mut s, &as_delivered);
    assert_eq!(s.source(), before);
}

#[test]
fn a_lone_cr_is_not_turned_into_anything_by_an_edit_elsewhere() {
    let doc = "# A\nold line\rkept\r\n\r\n# B\nb\n";
    let mut s = open(doc);
    focus(&mut s, "A");
    let view = normalize_to_lf(&body(&s));
    commit(&mut s, &view.replace("old", "new"));
    assert_eq!(s.source(), "# A\nnew line\rkept\r\n\r\n# B\nb\n");
}

// ── a JSON container's raw-text editor ──────────────────────────────────────

fn open_json(source: &str) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        Some("x.json".into()),
        FileTextProfile::detect(source, false),
        DocumentFormat::Json,
    )
    .unwrap()
}

fn focus_container(s: &mut EditorSession) {
    fn find(node: &DocumentMapNode) -> Option<u64> {
        if node.kind == omriss_core::StructureNodeKind::List {
            return Some(node.id);
        }
        node.children.iter().find_map(find)
    }
    let id = find(&s.document_map_nodes()).expect("a list node");
    s.focus(node_id_from_raw(id)).unwrap();
}

#[test]
fn a_crlf_json_container_edit_keeps_every_untouched_ending() {
    let doc =
        "{\r\n  \"name\": \"omriss\",\r\n  \"tags\": [\r\n    \"a\",\r\n    \"b\"\r\n  ]\r\n}\r\n";
    let mut s = open_json(doc);
    focus_container(&mut s);
    let original = s.focused_structured_raw_text().expect("raw text");
    assert!(
        original.contains("\r\n"),
        "the container's own text is CRLF: {original:?}"
    );

    // As delivered by the text area: LF only, with one value changed.
    let draft = normalize_to_lf(&original).replace("\"b\"", "\"c\"");
    assert!(s.structured_draft_differs(&draft));
    s.commit_structured_draft(&draft).unwrap();
    assert_eq!(
        s.source(),
        "{\r\n  \"name\": \"omriss\",\r\n  \"tags\": [\r\n    \"a\",\r\n    \"c\"\r\n  ]\r\n}\r\n",
        "only the changed value differs; every ending stays CRLF"
    );
}

#[test]
fn an_untouched_crlf_json_container_is_not_pending() {
    let doc = "{\r\n  \"tags\": [\r\n    \"a\"\r\n  ]\r\n}\r\n";
    let mut s = open_json(doc);
    focus_container(&mut s);
    let original = s.focused_structured_raw_text().unwrap();
    assert!(!s.structured_draft_differs(&normalize_to_lf(&original)));
    assert!(!s.structured_draft_differs(&original));
}

#[test]
fn a_newline_typed_into_a_one_line_container_inside_a_crlf_file_is_crlf() {
    // The container itself holds no line break; the *document's* newline decides.
    let doc = "{\r\n  \"tags\": [\"a\"]\r\n}\r\n";
    let mut s = open_json(doc);
    focus_container(&mut s);
    let original = s.focused_structured_raw_text().unwrap();
    assert_eq!(original, "[\"a\"]");
    s.commit_structured_draft("[\n\"a\"\n]").unwrap();
    assert_eq!(s.source(), "{\r\n  \"tags\": [\r\n\"a\"\r\n]\r\n}\r\n");
}

// ── Task 027's cases, byte-exact (RFC-084 §7) ───────────────────────────────

const A: &str = "# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody three\n";
const B: &str = "# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody three";

/// Types `x` at the end of `section`'s text, as the page does, and commits.
fn type_x_at_the_end(doc: &str, section: &str) -> String {
    let mut s = open(doc);
    focus(&mut s, section);
    let draft = format!("{}x", normalize_to_lf(&body(&s)));
    assert!(s.body_draft_differs(&draft));
    commit(&mut s, &draft);
    // The editor is resynchronised to the committed body: nothing is pending.
    assert!(!s.body_draft_differs(&normalize_to_lf(&body(&s))));
    s.source().to_string()
}

#[test]
fn case_1_a_middle_section_gains_one_document_newline_after_the_typed_text() {
    assert_eq!(
        type_x_at_the_end(A, "Two"),
        "# One\nbody one\n\n# Two\nbody two\n\nx\n# Three\nbody three\n"
    );
    assert_eq!(
        type_x_at_the_end(&crlf(A), "Two"),
        crlf("# One\nbody one\n\n# Two\nbody two\n\nx\n# Three\nbody three\n"),
        "CRLF throughout: the section's own endings and the separator"
    );
}

#[test]
fn case_2_the_last_section_gains_nothing() {
    assert_eq!(
        type_x_at_the_end(A, "Three"),
        "# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody three\nx"
    );
    assert_eq!(
        type_x_at_the_end(&crlf(A), "Three"),
        crlf("# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody three\n") + "x"
    );
}

#[test]
fn case_3_a_file_without_a_final_newline_gains_nothing() {
    assert_eq!(
        type_x_at_the_end(B, "Three"),
        "# One\nbody one\n\n# Two\nbody two\n\n# Three\nbody threex"
    );
}

#[test]
fn case_4_a_crlf_file_stays_crlf_in_every_case() {
    let out = type_x_at_the_end(&crlf(A), "Two");
    assert!(
        !out.replace("\r\n", "").contains('\n'),
        "no bare LF: {out:?}"
    );
    let out = type_x_at_the_end(&crlf(B), "Three");
    assert!(
        !out.replace("\r\n", "").contains('\n'),
        "no bare LF: {out:?}"
    );
}

#[test]
fn case_5_leaving_the_section_commits_the_same_bytes_as_saving() {
    // Saving and leaving both commit the pending draft through the same call
    // (`draft_sync::commit_or_block`); at the session there is one commit.
    let by_save = type_x_at_the_end(&crlf(A), "Two");
    let mut s = open(&crlf(A));
    focus(&mut s, "Two");
    let draft = format!("{}x", normalize_to_lf(&body(&s)));
    commit(&mut s, &draft);
    s.zoom_out();
    assert_eq!(s.source(), by_save);
}

#[test]
fn a_second_commit_after_the_separator_adds_nothing_more() {
    let mut s = open(&crlf(A));
    focus(&mut s, "Two");
    let draft = format!("{}x", normalize_to_lf(&body(&s)));
    commit(&mut s, &draft);
    let once = s.source().to_string();
    // The editor now holds the committed body; typing `y` and committing again
    // puts `y` on the line after `x` and adds no second separator.
    let draft = format!("{}y", normalize_to_lf(&body(&s)));
    commit(&mut s, &draft);
    assert_eq!(
        s.source(),
        once.replace("x\r\n# Three", "x\r\ny\r\n# Three"),
        "one separator, in the document's newline, however often it is committed"
    );
}
