# RFC-052 Implementation Handoff

**Governing RFC:** [RFC-052](../../done/052-structured-plain-text-format-expansion.md)
**Milestone:** M11 — Format Adapter Foundation
**Supersedes:** the RFC-052 handoff stub, whose stated exit condition
("after M10 is accepted and stable") was met when RFC-048–051 shipped in
v0.16.0.

---

## 1. Purpose

RFC-052 is a **documentation and policy RFC**. It authorizes no code, no
parser, and no adapter. This handoff covers the documentation work that closes
RFC-052's acceptance criteria so that RFC-053 can begin against a settled
product position.

## 2. Background

The product currently describes itself as a Markdown editor. RFC-052 widens
that to "structured plain-text editor" while keeping Markdown primary and
staging JSON → TOML → YAML-feasibility behind their own acceptance gates.

The risk this handoff exists to prevent is **overclaiming**: documentation that
implies JSON, TOML, or YAML support before the implementing RFCs have passed
their gates.

## 3. Applicable requirements

- RFC-052 §5.1 (extension policy), §5.2 (`PlainText` / `Unsupported` policy),
  §12 (documentation changes), §13 (acceptance criteria).
- Project rule: `README.md` stays concise; full documentation lives in
  `docs/src` as an mdBook.
- Project rule: English for all documentation.

## 4. Change scope

### 4.1 New page: `docs/src/file-formats.md`

Must state the per-format status using the **"Status to document now"** column
of RFC-052 §12 — the left column, not the right one:

| Format | Status |
|---|---|
| Markdown | Fully supported |
| JSON | Planned — not available yet (RFC-054) |
| TOML | Planned — not available yet (RFC-055) |
| YAML | Under investigation (RFC-056) |

RFC-052 §12's right column is what a *later* handoff will change these to once
each implementing RFC is accepted. Using it now would promote a format that
does not exist yet.

The page must also record the extension behavior users can rely on **today**:
`.md`, `.markdown`, `.mdown`, and `.txt` all open as Markdown.

Add the page to `docs/src/SUMMARY.md` after `Languages`.

### 4.2 `docs/src/introduction.md`

Widen the framing from "A Markdown document is more than a wall of text" to
include structured plain-text files as a direction, **without** claiming
JSON/TOML support exists. Markdown must remain the described primary experience.

### 4.3 `README.md`

Update the Overview/Design-notes framing to match §3 of RFC-052. Keep it
concise per project rules — no feature table, no format matrix; link to
`docs/src/file-formats.md`.

### 4.4 `ROADMAP.md`

Already updated for M11 as the active theme. Verify only; change nothing unless
it contradicts RFC-052 §5.

## 5. Non-change scope

1. **No code.** Not one line in `crates/`. If you find yourself editing Rust,
   stop — you are outside RFC-052.
2. **Do not add `.json` or `.toml` to `MD_EXTENSIONS`** or to any file dialog
   filter. Detection belongs to RFC-053.
3. **Do not change** `.txt` or `.mdown` handling. RFC-052 §5.1 makes preserving
   them binding.
4. **Do not write documentation in the future tense as though shipped.**
   "omriss edits JSON" is prohibited; "JSON support is planned" is correct.
5. Do not modify `docs/src/languages.md` — it covers GUI i18n, a different
   subject.
6. Do not update the i18n catalogs. No new user-facing strings are in scope.

## 6. Required implementation

1. Write `docs/src/file-formats.md` per §4.1.
2. Register it in `docs/src/SUMMARY.md`.
3. Adjust `docs/src/introduction.md` per §4.2.
4. Adjust `README.md` per §4.3.
5. Verify `ROADMAP.md` per §4.4.

Single PR is appropriate; this is one coherent documentation unit.

## 7. Required tests

No unit tests apply. Required gates:

```sh
mdbook build docs          # must succeed; new page must appear in the nav
cargo fmt --check          # unchanged
cargo test --workspace     # unchanged: 239 passed
bash scripts/check-rfcs.sh # unchanged
```

`cargo test` and `check-rfcs.sh` are listed not because this work touches them,
but because an unchanged result is the evidence that it didn't.

## 8. Acceptance criteria

1. `docs/src/file-formats.md` exists, is linked from `SUMMARY.md`, and renders
   in `mdbook build`.
2. The status table matches RFC-052 §12 with no format upgraded in tone.
3. The `.md` / `.markdown` / `.mdown` / `.txt` behavior is documented.
4. No document claims JSON, TOML, or YAML support exists.
5. `README.md` remains concise and links to the new page rather than inlining a
   format matrix.
6. `git diff --stat` shows **zero** files under `crates/`.
7. RFC-052 §13 criteria 1, 2, 3, 4, 6 can each be closed by pointing at a
   specific file.

## 9. Documentation updates

This handoff *is* the documentation update. Add a `CHANGELOG.md`
`[Unreleased]` → `### Changed` entry noting the product-scope wording change and
the new formats page, and stating explicitly that no format support is added.

## 10. Known risks

| Risk | Severity | Mitigation |
|---|---|---|
| Documentation implies JSON/TOML already work | **High** — this is the exact overclaim RFC-052 exists to prevent | §5.4 and acceptance criterion 4 |
| Scope creep into detection or dialog filters | Medium | §5.1, §5.2, acceptance criterion 6 |
| `.txt` handling changed as a "cleanup" | Medium | §5.3; it is a shipped user-visible behavior |

## 11. Required review-request content

Per the organization workflow §9.2, plus:

- the exact status table as written, quoted in the review request;
- confirmation that `git diff --stat` contains no `crates/` path;
- `mdbook build docs` output.

## 12. Escalation triggers

Stop and ask the architect if:

- closing an acceptance criterion appears to require a code change;
- RFC-052 §5.1 conflicts with observed file-dialog behavior;
- you believe a format's documented status should differ from RFC-052 §12.
