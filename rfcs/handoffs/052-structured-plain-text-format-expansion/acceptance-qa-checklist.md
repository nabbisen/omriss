# RFC-052 Acceptance / QA Checklist

Record `Pass`, `Fail`, or `Blocked`. **Unchecked boxes are not evidence.**

---

## Scope discipline

```text
[ ] git diff --stat shows ZERO paths under crates/
[ ] no file-dialog filter was changed
[ ] .txt and .mdown still open as Markdown
[ ] no i18n catalog key was added or changed
[ ] no new user-facing UI string was introduced
```

## Content accuracy — the overclaim check

This is the failure mode RFC-052 exists to prevent. Check every changed file.

```text
[ ] no document states or implies that JSON editing works today
[ ] no document states or implies that TOML editing works today
[ ] no document states or implies that YAML is supported
[ ] the status table uses RFC-052 §12's "Status to document now" column,
    not the "after its implementing RFC is accepted" column
[ ] no format was promoted out of its stage by this documentation change
[ ] Markdown is still described as the primary experience
```

## Documentation gates

```text
[ ] docs/src/file-formats.md exists
[ ] it is linked from docs/src/SUMMARY.md
[ ] mdbook build docs succeeds
[ ] the new page appears in the rendered navigation
[ ] README.md remains concise; it links to the new page rather than inlining
    a format matrix
```

## Unchanged-state gates

```text
[ ] cargo test --workspace       239 passed, 12 suites
[ ] cargo fmt --check            clean
[ ] bash scripts/check-rfcs.sh   passes
```

## RFC-052 §13 criteria

Criteria 1, 2, 3, 4, and 6 must each be closable by pointing at a specific file.
Criterion 8 depends on RFC-053's acceptance and is not closable here.

```text
[ ] 1  product scope wording          → file:
[ ] 2  format support order recorded  → file:
[ ] 3  extension compatibility stated → file:
[ ] 4  PlainText / Unsupported policy → file:
[ ] 6  status distinctions documented → file:
[ ] 7  no open question left to the implementer
```

---

## Final decision

```text
Decision:        Pending / Accepted / Corrections required
Date:
Recorder:
Source identity:
Evidence reviewed:
```
