# RFC-052 Task Breakdown / PR Plan

RFC-052 is documentation and policy only. One PR is appropriate — the changes
form a single coherent statement of product scope and would read oddly if split.

Read `implementation-handoff.md` §5 (non-change scope) first. The short version:
**zero files under `crates/`.**

---

## PR-1 — Product scope and format status documentation

### 1. New page — `docs/src/file-formats.md`

Content required:

- the per-format status table, using the **"Status to document now"** column of
  RFC-052 §12:

  | Format | Status |
  |---|---|
  | Markdown | Fully supported |
  | JSON | Planned — not available yet (RFC-054) |
  | TOML | Planned — not available yet (RFC-055) |
  | YAML | Under investigation (RFC-056) |

- the extension behavior users can rely on **today**: `.md`, `.markdown`,
  `.mdown`, and `.txt` all open as Markdown;
- a plain statement that omriss preserves the original file and never rewrites
  unrelated parts of it.

Do not describe adapters, parsers, node kinds, or capabilities. This page is for
users.

### 2. `docs/src/SUMMARY.md`

Add the page after `Languages`.

### 3. `docs/src/introduction.md`

Widen the framing to include structured plain-text files as the product
direction, keeping Markdown as the described primary experience. Currently it
opens "A Markdown document is more than a wall of text" — that stays true and
primary; the addition is directional, not a claim of present capability.

### 4. `README.md`

Align the Overview framing with RFC-052 §3. Keep it concise per project rules:
no feature table, no format matrix — link to `docs/src/file-formats.md`.

### 5. `ROADMAP.md`

Verify only. It already names M11 as the active theme. Change nothing unless it
contradicts RFC-052 §5.

### 6. `CHANGELOG.md`

One `[Unreleased]` → `### Changed` entry, stating explicitly that no format
support is added.

---

## Verification

```sh
mdbook build docs          # new page renders and appears in the nav
cargo test --workspace     # unchanged: 239 passed
cargo fmt --check          # unchanged
bash scripts/check-rfcs.sh # unchanged
git diff --stat            # zero paths under crates/
```

The three "unchanged" gates are listed because an unchanged result is the
evidence that this work stayed inside its scope.
