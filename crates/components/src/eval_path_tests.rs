//! RFC-060 §12: JavaScript source is never built from data.
//!
//! Every script omriss runs in the page is a constant. Data reaches a script as
//! data, through the eval channel (`send` and `dioxus.recv()`), and is never
//! written into source: a file's name is anyone's to choose, and an escaper is a
//! hand-written security boundary. This scans the sources of the whole UI layer
//! (see `source_scan`): the argument of every `eval(` must be a string literal, or
//! the name of a `const` that is defined in the UI layer.

use std::collections::BTreeSet;

use crate::source_scan::{production_code, ui_layer_files};

/// What follows `eval(` at `open`, judged: `Ok` if it is a literal or a constant
/// name in `consts`, `Err(what was there)` otherwise.
fn judge_argument(text: &str, open: usize, consts: &BTreeSet<String>) -> Result<(), String> {
    let rest = text[open..].trim_start();
    let rest = rest.strip_prefix('&').unwrap_or(rest).trim_start();
    let shown = |r: &str| r.lines().next().unwrap_or("").trim().to_string();

    // A string literal, plain or raw, followed by the call's end.
    let after_literal = if let Some(body) = rest.strip_prefix('"') {
        let mut escaped = false;
        let mut end = None;
        for (i, c) in body.char_indices() {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                end = Some(i + 1);
                break;
            }
        }
        end.map(|e| &body[e..])
    } else if let Some(raw) = rest.strip_prefix('r') {
        let hashes = raw.chars().take_while(|&c| c == '#').count();
        raw[hashes..].strip_prefix('"').and_then(|body| {
            let close = format!("\"{}", "#".repeat(hashes));
            body.find(&close).map(|i| &body[i + close.len()..])
        })
    } else {
        None
    };
    if let Some(after) = after_literal {
        return match after.trim_start().chars().next() {
            Some(')') | Some(',') => Ok(()),
            _ => Err(shown(rest)),
        };
    }

    // A constant: an upper-case name, then the call's end, defined as a `const`.
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
        .collect();
    let after = rest[name.len()..].trim_start();
    if !name.is_empty()
        && consts.contains(&name)
        && matches!(after.chars().next(), Some(')') | Some(','))
    {
        return Ok(());
    }
    Err(shown(rest))
}

/// Every `eval(` call in `text` (a call to `document::eval`, or `eval` imported),
/// with the line it is on, that is not given a literal or a constant.
fn bad_calls(text: &str, consts: &BTreeSet<String>) -> Vec<(usize, String)> {
    let mut bad = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("eval(") {
        let at = from + i;
        from = at + "eval(".len();
        let before = text[..at].chars().next_back();
        // `my_eval(` is another function; `use_eval(` is the hook form of this one and
        // is refused outright. `.eval(` is judged like the rest: `document().eval(…)`
        // is the same call.
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            if text[..at].ends_with("use_") {
                bad.push((
                    text[..at].matches('\n').count() + 1,
                    "use_eval(".to_string(),
                ));
            }
            continue;
        }
        if let Err(what) = judge_argument(text, from, consts) {
            bad.push((text[..at].matches('\n').count() + 1, what));
        }
    }
    bad
}

/// The names of every `const` item in `text`.
fn const_names(text: &str) -> impl Iterator<Item = String> + '_ {
    text.lines().filter_map(|l| {
        let l = l.trim_start();
        let l = l
            .strip_prefix("pub(crate) ")
            .or(l.strip_prefix("pub "))
            .unwrap_or(l);
        let l = l.strip_prefix("const ")?;
        let name: String = l
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        (!name.is_empty() && l[name.len()..].starts_with(':')).then_some(name)
    })
}

fn production_text(path: &std::path::Path) -> String {
    let text = std::fs::read_to_string(path).unwrap();
    production_code(&text)
        .into_iter()
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_script_is_a_literal_or_a_constant() {
    let files = ui_layer_files();
    assert!(files.len() > 30, "scan looks wrong: {} files", files.len());
    let texts: Vec<_> = files.iter().map(|p| (p, production_text(p))).collect();
    let consts: BTreeSet<String> = texts.iter().flat_map(|(_, t)| const_names(t)).collect();
    let mut calls = 0;
    let mut offenders = Vec::new();
    for (path, text) in &texts {
        calls += text.matches("document::eval(").count();
        for (line, what) in bad_calls(text, &consts) {
            offenders.push(format!("{}:~{line}: eval({what}", path.display()));
        }
    }
    assert!(
        calls >= 3,
        "the guard sees only {calls} calls; the scan is wrong"
    );
    assert!(
        offenders.is_empty(),
        "a script is built from data (RFC-060 §12): give `eval` a literal or a `const`, and send \
         the data with `Eval::send` / read it with `dioxus.recv()`:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_judge_accepts_what_it_should_and_refuses_what_it_should() {
    let consts: BTreeSet<String> = ["FOCUS_SCRIPT".to_string()].into();
    let bad = |src: &str| !bad_calls(src, &consts).is_empty();
    // Accepted: a literal, a raw literal, a multi-line literal call, a known const.
    assert!(!bad("let _ = document::eval(\"a.b()\");"));
    assert!(!bad("let _ = document::eval(\n    \"a.b()\",\n);"));
    assert!(!bad("document::eval(r#\"a \"quoted\" b\"#)"));
    assert!(!bad("document::eval(FOCUS_SCRIPT)"));
    assert!(!bad("document::eval(&FOCUS_SCRIPT)"));
    // Refused: built, borrowed, computed, unknown, or the hook form.
    assert!(bad("document::eval(&format!(\"x = {}\", y))"));
    assert!(bad("document::eval(&page)"));
    assert!(bad("document::eval(script)"));
    assert!(bad("document::eval(\"a\".to_string() + &b)"));
    assert!(bad("document::eval(\"a\" + b)"));
    assert!(bad("document::eval(OTHER_SCRIPT)"));
    assert!(bad("eval(&page)"));
    assert!(bad("let e = use_eval(\"x\");"));
    // The method form is the same call.
    assert!(bad("document().eval(&page)"));
    assert!(!bad("document().eval(\"a.b()\")"));
    // Another function's name is not.
    assert!(!bad("my_eval(x)"));
}

#[test]
fn the_constants_are_found() {
    let found: Vec<_> =
        const_names("const A_B: &str = \"x\";\npub const C: &str = \"y\";\nlet x = 1;\n").collect();
    assert_eq!(found, ["A_B", "C"]);
}
