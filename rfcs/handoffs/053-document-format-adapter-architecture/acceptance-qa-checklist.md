# RFC-053 Acceptance / QA Checklist

Record `Pass`, `Fail`, or `Blocked` for each item. **Unchecked boxes are not
evidence.** Capture the source identity (commit or worktree hash) for each run.

---

## Per-slice gates

Run for **every** slice, not only the last:

```text
[ ] cargo fmt --check                                      clean
[ ] cargo test --workspace                                 no suite lost, counts only grow
[ ] cargo clippy --workspace --all-targets -- -D warnings  exit 0
[ ] bash scripts/check-rfcs.sh                             passes
```

Baseline to compare against (`52f20b9`, 2026-07-30): **239 tests, 12 suites.**

---

## S1 — detection

```text
[ ] .md, .markdown, .mdown, .txt all classify as Markdown
[ ] .json classifies as Json; .toml as Toml
[ ] .yaml, .yml classify as YamlExperimental
[ ] unknown extension classifies as PlainText
[ ] no path (unsaved buffer) is handled without panic
[ ] nothing calls detection yet; no behavior change anywhere
```

## S2 — core vocabulary

```text
[ ] all types compile in omriss-core with no Dioxus in the dependency tree
[ ] Capability::Disabled carries a typed reason, not a string or catalog key
[ ] capability computation covers root / first / last / only / deep nodes
[ ] omriss-ui is untouched by this slice
```

## S3 — reconciliation (the zero-change slice)

```text
[ ] grep for MapCapability and MapNodeCapabilities in crates/ui and crates/app
    returns nothing
[ ] all eight capability.disabled.* catalog keys are byte-identical to before
[ ] English and Japanese catalogs are otherwise unchanged
[ ] document_map_tests pass with assertions unchanged in meaning
[ ] i18n_tests pass with assertions unchanged in meaning
[ ] omriss-core does not reference any catalog key
[ ] DocumentMapNode still lives in omriss-ui
```

**Manual check — required, cannot be automated:** open a Markdown file, select a
top-level section, open the `⋯` row menu, and confirm each disabled action shows
the *same* explanatory text as before the slice. A reason silently mapping to a
different string is the most likely defect in this work, and no test will catch
it.

## S4 — MarkdownAdapter

```text
[ ] all 239 pre-existing tests pass UNMODIFIED
[ ] each structure command wraps its shipped implementation; call sites named
    in the review request
[ ] Markdown node ids are unchanged from the shipped scheme
[ ] rebuild determinism test passes
[ ] focus-survival test passes (edit unrelated node, focus preserved)
[ ] no adapter mutates text outside Document's replacement path
```

**Manual check:** perform each structural action from the Document Map — move
up, move down, move inside previous, move out one level, rename, merge, delete —
then Undo each. Source text must return byte-exactly. This deliberately repeats
M10 QA scenarios: S4 rewires the path they exercise.

## S5 — PlainText and recovery

```text
[ ] malformed input keeps source text intact
[ ] recovery route to plain file text is offered
[ ] no panic on malformed, empty, or deeply nested input
[ ] PlainText yields exactly one node
[ ] PlainText editing capabilities are Hidden, not Disabled
[ ] no new catalog keys were added
```

---

## Scope discipline

```text
[ ] no JSON, TOML, or YAML parser exists in the diff
[ ] .json / .toml were not added to any file-dialog filter
[ ] no global "experimental formats" setting was introduced
[ ] no SourceText or LineEnding type was introduced
[ ] ByteRange, NodeId, DocumentRevision were reused, never redefined
[ ] no mod.rs anywhere
[ ] no file exceeds 500 ELOC
```

---

## RFC-053 §19 criteria

Reproduce the eleven-row table from RFC-053 §19 with per-row status. Criterion 5
is blocking.

```text
[ ] criteria table reproduced in the review request
[ ] criterion 5 met: no shipped Markdown test was modified
```

---

## Final decision

```text
Decision:        Pending / Accepted / Corrections required
Date:
Recorder:
Source identity:
Evidence reviewed:
Deferred items:
```
