//! RFC-068 §7.1: nothing in the application writes a file except through
//! [`write_durably`](super::write_durably).
//!
//! A source-scanning guard, like the repository's others: it reads the non-test source
//! of every crate under `crates/` and fails on a call that creates, writes, renames or
//! copies a file, anywhere but in `durable_write.rs`. Without it the next convenient
//! `fs::write` quietly brings back everything the primitive removed.

use std::fs;
use std::path::{Path, PathBuf};

/// What may not appear outside the primitive: every way std has of putting bytes in a
/// file or moving one over another. Matched on the code, with comments removed.
const FORBIDDEN: &[&str] = &[
    "fs::write",
    "fs::rename",
    "fs::copy",
    "File::create",
    "File::create_new",
    "OpenOptions",
];

/// The names `use std::fs::…` must not bring in bare (`use std::fs::write;`, or inside
/// braces), which would hide a call behind a plain `write(`.
const FORBIDDEN_IMPORTS: &[&str] = &["write", "rename", "copy", "OpenOptions"];

fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Removes `//` comments (doc comments included) and string literals' contents are left
/// alone: a forbidden call inside a string is not a call, but writing the name in a
/// message is rare enough that a hit there is worth a reworded message.
fn without_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Removes inline `#[cfg(test)] mod name { … }` blocks (brace-matched).
fn without_inline_test_modules(src: &str) -> String {
    let mut out = String::new();
    let mut rest = src;
    while let Some(i) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..i]);
        let after = &rest[i + "#[cfg(test)]".len()..];
        let trimmed = after.trim_start();
        if trimmed.starts_with("mod ") {
            match (trimmed.find('{'), trimmed.find(';')) {
                (Some(open), semi) if semi.is_none_or(|s| open < s) => {
                    let mut depth = 0usize;
                    let mut end = trimmed.len();
                    for (j, c) in trimmed.char_indices().skip(open) {
                        match c {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    end = j + 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    rest = &trimmed[end..];
                    continue;
                }
                // `mod name;`: the module is in its own file, which is skipped by name.
                _ => {}
            }
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// The violations in one file's `src`, as `(line, what)`.
fn violations(src: &str) -> Vec<(usize, String)> {
    let code = without_inline_test_modules(&without_comments(src));
    let mut found = Vec::new();
    for (n, line) in code.lines().enumerate() {
        for token in FORBIDDEN {
            if line.contains(token) {
                found.push((n + 1, format!("`{token}`")));
            }
        }
        if let Some(i) = line.find("std::fs::") {
            let tail = &line[i + "std::fs::".len()..];
            // `use std::fs::write;` or `use std::fs::{self, write};`
            if line.trim_start().starts_with("use ") {
                let names: Vec<&str> = tail
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .filter(|w| !w.is_empty())
                    .collect();
                for name in names {
                    if FORBIDDEN_IMPORTS.contains(&name) {
                        found.push((n + 1, format!("an import of `fs::{name}`")));
                    }
                }
            }
        }
    }
    found
}

/// Test code is where fixtures are written: a file named `tests.rs` or `…_tests.rs`,
/// or anything under a `tests/` or `benches/` directory.
fn is_test_file(path: &Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem == "tests"
        || stem.ends_with("_tests")
        || path
            .components()
            .any(|c| matches!(c.as_os_str().to_str(), Some("tests" | "benches")))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_code_writes_a_file_except_the_durable_write() {
    let mut files = Vec::new();
    for krate in ["core", "ui", "components", "app"] {
        rust_files(&crates_root().join(krate).join("src"), &mut files);
    }
    assert!(
        files.len() > 50,
        "the scan found only {} files: wrong root?",
        files.len()
    );

    let primitive = Path::new("app")
        .join("src")
        .join("file")
        .join("durable_write.rs");
    let mut report = Vec::new();
    let mut scanned = 0;
    for file in &files {
        let relative = file.strip_prefix(crates_root()).unwrap_or(file);
        if relative == primitive || is_test_file(file) {
            continue;
        }
        scanned += 1;
        for (line, what) in violations(&fs::read_to_string(file).unwrap()) {
            report.push(format!("{}:{line}: {what}", relative.display()));
        }
    }
    assert!(scanned > 30, "only {scanned} source files were checked");
    assert!(
        report.is_empty(),
        "files are written only through file::durable_write::write_durably (RFC-068 §7.1):\n{}",
        report.join("\n")
    );
}

#[test]
fn the_guard_sees_what_it_is_meant_to_see() {
    for bad in [
        "fn f() { std::fs::write(p, b).unwrap(); }",
        "fn f() { fs::write(p, b); }",
        "fn f() { let _ = fs::rename(a, b); }",
        "fn f() { std::fs::copy(a, b).ok(); }",
        "fn f() { let f = File::create(p); }",
        "fn f() { let f = std::fs::File::create_new(p); }",
        "fn f() { OpenOptions::new().write(true).open(p); }",
        "use std::fs::write;\nfn f() { write(p, b); }",
        "use std::fs::{self, rename};\n",
        "use std::fs::{copy, read};\n",
        "fn f() {}\n#[cfg(test)]\nmod tests;\nfn g() { fs::write(p, b); }\n",
    ] {
        assert!(!violations(bad).is_empty(), "missed: {bad}");
    }
}

#[test]
fn the_guard_does_not_see_what_it_must_not() {
    for fine in [
        "// fs::write is gone: use write_durably\nfn f() {}",
        "/// Calls `fs::rename` for you.\nfn f() {}",
        "fn f() { let x = fs::read(p); let y = fs::read_to_string(p); fs::create_dir_all(d); fs::remove_file(p); }",
        "use std::fs::{self, read_dir, File};\n",
        "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn t() { std::fs::write(p, b).unwrap(); }\n}\n",
        "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn t() { if x { y } }\n    fn u() { fs::write(p, b); }\n}\nfn g() {}\n",
    ] {
        assert!(
            violations(fine).is_empty(),
            "false alarm: {fine}\n{:?}",
            violations(fine)
        );
    }
}

#[test]
fn a_file_written_after_an_inline_test_module_is_still_seen() {
    let src =
        "#[cfg(test)]\nmod tests { fn t() { fs::write(a, b); } }\nfn g() { fs::write(c, d); }\n";
    assert_eq!(violations(src).len(), 1);
}
