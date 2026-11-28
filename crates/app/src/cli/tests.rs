use super::*;

use std::ffi::OsString;
use std::path::PathBuf;

fn interpret_str(args: &[&str]) -> StartupArg {
    interpret(args.iter().map(OsString::from))
}

#[test]
fn a_plain_path_yields_that_path() {
    assert_eq!(
        interpret_str(&["notes.md"]),
        StartupArg::Path(PathBuf::from("notes.md"))
    );
}

#[test]
fn no_arguments_yields_none() {
    assert_eq!(interpret_str(&[]), StartupArg::None);
}

#[test]
fn extra_arguments_are_ignored_first_wins() {
    assert_eq!(
        interpret_str(&["a.md", "b.md", "c.md"]),
        StartupArg::Path(PathBuf::from("a.md"))
    );
}

#[test]
fn an_argument_beginning_with_dash_is_rejected_as_an_option() {
    assert_eq!(
        interpret_str(&["--help"]),
        StartupArg::RejectedOption(OsString::from("--help"))
    );
    assert_eq!(
        interpret_str(&["-v"]),
        StartupArg::RejectedOption(OsString::from("-v"))
    );
}

#[test]
fn dot_slash_weird_dot_md_is_treated_as_a_path() {
    // The standard convention for opening a file whose name genuinely
    // begins with `-` (RFC-063 §5.3): it does not itself start with `-`,
    // so it is never mistaken for an option.
    assert_eq!(
        interpret_str(&["./-weird.md"]),
        StartupArg::Path(PathBuf::from("./-weird.md"))
    );
}

#[test]
fn a_rejected_option_still_ignores_any_further_arguments() {
    // Mirrors extra_arguments_are_ignored_first_wins for the rejection
    // branch: only the first argument is ever inspected.
    assert_eq!(
        interpret_str(&["--help", "notes.md"]),
        StartupArg::RejectedOption(OsString::from("--help"))
    );
}

// ── RFC-068 §7.5: nothing in the command line is ever required to be text ────────

#[cfg(unix)]
mod non_utf8 {
    use super::*;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    fn bytes(b: &[u8]) -> OsString {
        OsString::from_vec(b.to_vec())
    }

    #[test]
    fn a_path_that_is_not_utf8_is_a_path_with_its_bytes_intact() {
        // `std::env::args()` panics on this; `args_os()` and this do not.
        let arg = bytes(b"caf\xe9 \xff.md");
        match interpret(std::iter::once(arg.clone())) {
            StartupArg::Path(path) => assert_eq!(path.as_os_str().as_bytes(), arg.as_bytes()),
            other => panic!("expected a path, got {other:?}"),
        }
    }

    #[test]
    fn no_argument_of_any_bytes_panics_or_loses_them() {
        for raw in [
            &b""[..],
            b"\xff",
            b"-\xff",
            b"\xc0\xaf",
            b"\xed\xa0\x80",
            b"a\xffb/c\xfe.md",
            b"--\xe9",
        ] {
            let arg = bytes(raw);
            match interpret([arg.clone(), bytes(b"\xff-second")].into_iter()) {
                StartupArg::Path(p) => assert_eq!(p.as_os_str(), arg.as_os_str(), "{raw:?}"),
                StartupArg::RejectedOption(o) => {
                    assert_eq!(o, arg, "{raw:?}");
                    assert_eq!(raw.first(), Some(&b'-'), "only a leading dash is an option");
                }
                StartupArg::None => panic!("an argument was given: {raw:?}"),
            }
        }
    }

    #[test]
    fn an_option_is_recognised_by_its_dash_whatever_follows() {
        assert!(matches!(
            interpret(std::iter::once(bytes(b"-\xff"))),
            StartupArg::RejectedOption(_)
        ));
    }
}

/// The platforms whose strings are always text have nothing to test here, and say so.
#[cfg(not(unix))]
#[test]
#[ignore = "a non-UTF-8 argument cannot be built on this platform; the Unix test covers it"]
fn non_utf8_arguments_are_a_unix_case() {}

/// `std::env::args()` panics on an argument that is not valid Unicode. Nothing in the
/// application may call it; `args_os()` is the only way in (RFC-068 §7.5).
#[test]
fn nothing_in_the_application_reads_argv_as_text() {
    fn walk(dir: &std::path::Path, hits: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, hits);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if name == "tests" || name.ends_with("_tests") {
                    continue;
                }
                for (n, line) in std::fs::read_to_string(&path).unwrap().lines().enumerate() {
                    let code = line.split("//").next().unwrap_or("");
                    if code.contains("env::args()") {
                        hits.push(format!("{}:{}", path.display(), n + 1));
                    }
                }
            }
        }
    }
    let mut hits = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut hits,
    );
    assert!(
        hits.is_empty(),
        "use std::env::args_os(), which cannot panic: {hits:?}"
    );
}
