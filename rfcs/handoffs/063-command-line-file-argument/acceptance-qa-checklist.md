# RFC-063 Acceptance / QA Checklist

Record `Pass`, `Fail`, or `Blocked`. **Unchecked boxes are not evidence.**

## Gates

```text
[ ] cargo fmt --all --check                                 clean
[ ] cargo test --workspace                                  401+, counts only grow
[ ] cargo clippy --workspace --all-targets -- -D warnings   exit 0
[ ] bash scripts/check-rfcs.sh                              passes
```

## Argument interpretation (unit-tested, no GUI)

```text
[ ] a plain path yields that path
[ ] no arguments yields none
[ ] extra arguments ignored, first wins
[ ] an argument beginning with `-` is rejected as an option
[ ] ./-weird.md is treated as a path
```

## Behavior

```text
[ ] cargo run -p omriss -- some.md      opens with the document rendered
[ ] cargo run -p omriss -- some.json    opens with the Document Map populated
[ ] a nonexistent path: app STARTS, Welcome screen, plain message
[ ] a directory path: app STARTS, plain message
[ ] omriss --help: intelligible message, no crash, no "File not found: --help"
[ ] no argument: Welcome screen and Recent Files unchanged
```

**Screenshot required** for at least one successful open. This artifact is the
slice's real deliverable — it is what makes RFC-054 J7's rendering check
performable without synthetic input.

## Scope discipline

```text
[ ] zero diff under crates/core/ and crates/ui/
[ ] no new dependency (no clap, no argh)
[ ] no flag parsing beyond the `-` rejection message
[ ] docs/src/file-formats.md NOT touched
[ ] no shipped test modified
```

## Final decision

```text
Decision:        Pending / Accepted / Corrections required
Date:
Recorder:
Source identity:
```
