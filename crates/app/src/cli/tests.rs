use super::*;

fn interpret_str(args: &[&str]) -> StartupArg {
    interpret(args.iter().map(|s| s.to_string()))
}

#[test]
fn a_plain_path_yields_that_path() {
    assert_eq!(
        interpret_str(&["notes.md"]),
        StartupArg::Path("notes.md".to_string())
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
        StartupArg::Path("a.md".to_string())
    );
}

#[test]
fn an_argument_beginning_with_dash_is_rejected_as_an_option() {
    assert_eq!(
        interpret_str(&["--help"]),
        StartupArg::RejectedOption("--help".to_string())
    );
    assert_eq!(
        interpret_str(&["-v"]),
        StartupArg::RejectedOption("-v".to_string())
    );
}

#[test]
fn dot_slash_weird_dot_md_is_treated_as_a_path() {
    // The standard convention for opening a file whose name genuinely
    // begins with `-` (RFC-063 §5.3): it does not itself start with `-`,
    // so it is never mistaken for an option.
    assert_eq!(
        interpret_str(&["./-weird.md"]),
        StartupArg::Path("./-weird.md".to_string())
    );
}

#[test]
fn a_rejected_option_still_ignores_any_further_arguments() {
    // Mirrors extra_arguments_are_ignored_first_wins for the rejection
    // branch: only the first argument is ever inspected.
    assert_eq!(
        interpret_str(&["--help", "notes.md"]),
        StartupArg::RejectedOption("--help".to_string())
    );
}
