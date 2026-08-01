# RFC-052: Structured Plain-Text Format Expansion

**Project:** omriss — Omriss Editor
**Milestone:** M11 — Format Adapter Foundation
**Status.** Implemented (main, unreleased) — documentation artifacts landed in
commit `693fcb5`. Seven of the eight §13 acceptance criteria are closed;
criterion 8 ("RFC-053 is accepted as the required architecture") is a dependency
on RFC-053's own acceptance and is tracked there, not here. The §14.4 per-format
visibility decision is binding on RFC-054, RFC-055, and RFC-056.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-049, RFC-050, RFC-051 — all Implemented in v0.16.0
**Related RFCs:** RFC-053, RFC-054, RFC-055, RFC-056
**Scope of change:** Documentation and policy only. This RFC authorizes no code.

---

## 1. Summary

This RFC expands the omriss product direction from a Markdown outline editor to a broader **structured plain-text editor** while preserving Markdown as the primary and first-class experience.

The proposed direction:

> omriss helps users navigate and edit structured plain-text files one level at a time, without hiding or corrupting the original file.

Markdown support remains central. JSON support is proposed as the first non-Markdown target. TOML follows after preservation rules are proven. YAML is investigated separately as a feasibility spike because of its complexity.

## 2. Motivation

omriss is built around the idea that complex information can be understood and edited by moving through structure level by level.

This idea applies naturally to Markdown:

```text
Document
  Chapter
    Section
      Subsection
```

It also applies to structured plain-text data:

```text
settings
  editor
    theme
    autosave
  files
    recent
```

Many users and projects rely on structured plain-text files:

- project settings;
- configuration files;
- package manifests;
- data fixtures;
- scenario/worldbuilding databases;
- personal knowledge files;
- local app settings.

Supporting such files can make omriss more useful without changing its core philosophy, as long as the product remains source-preserving and local-first.

## 3. Product philosophy update

The product should no longer be described only as a Markdown editor.

Recommended wording:

```text
omriss is a local-first structured plain-text editor.
It starts with Markdown outlines and grows to support structured data files such as JSON and TOML.
Your original file remains yours.
```

Shorter product line:

```text
Think and edit in outlines, without leaving plain text.
```

## 4. Core principles

### 4.1 Source text remains canonical

For every supported format:

```text
Canonical document = original source text
Derived structure  = format-specific map/index
Focused editor     = projection of one selected node/range
```

omriss must not parse a file into a generic data object and serialize the entire file back as the normal save path.

### 4.2 No proprietary sidecar format

Supported files must remain ordinary `.md`, `.json`, `.toml`, or `.yaml`/`.yml` files. omriss must not require hidden metadata files for document recovery.

### 4.3 UI role split remains unchanged

The UI model from RFC-048–051 remains the foundation:

```text
Document Map = navigate and organize structure
Right panel  = edit focused content
```

### 4.4 Markdown remains first-class

Structured data support must not make Markdown writing harder, slower, or visually more complex.

### 4.5 Format support is staged

Do not implement every format at once. Each format must pass preservation and usability gates before becoming editable.

## 5. Supported format roadmap

| Format | Extension | Initial state | Editing priority | Notes |
|--------|-----------|---------------|------------------|-------|
| Markdown | `.md`, `.markdown` | Existing first-class support | Existing | Primary product mode. |
| JSON | `.json` | New support | First | Strict grammar, good first adapter target. |
| TOML | `.toml` | New support | Second | Important for Rust/config files; comments and order must be preserved. |
| YAML | `.yaml`, `.yml` | Feasibility only | Not promised | Complex syntax; editing requires careful proof. |

### 5.1 Extension policy and shipped-behavior compatibility

The table above is incomplete against shipped behavior and must not be
implemented literally. As of v0.16.0 the file dialog accepts
`["md", "markdown", "mdown", "txt"]` and parses **all four** as Markdown
(`crates/app/src/file/file_dialog.rs`). `.txt` files are therefore fully
editable Markdown documents today.

Binding compatibility rule:

```text
.md, .markdown, .mdown, .txt  -> Markdown        (unchanged; no regression)
.json                         -> Json
.toml                         -> Toml
.yaml, .yml                   -> YamlExperimental (feature-gated)
anything else                  -> PlainText or Unsupported (see below)
```

Routing `.txt` to `PlainText` would remove editing from files users can edit
today. That is a user-visible regression and is **prohibited** by this RFC. If
`.txt` should ever stop meaning Markdown, that requires its own RFC and a
migration note.

### 5.2 Reconciliation with RFC-053 format variants

RFC-053 §5 defines two variants this RFC must give policy for:

| Variant | Policy |
|---|---|
| `PlainText` | The file opens, shows plain file text, and is **read-only with no synthetic structure**. omriss must not invent a hierarchy for a format it does not understand. |
| `Unsupported` | The file does not open into the workspace; a plain "This file type is not supported yet" message is shown. Reserved for content omriss should not attempt, such as non-UTF-8 or oversized input. |

Neither variant may be used as a fallback that silently downgrades a format
omriss claims to support. If a `.json` file fails to parse, the user sees a JSON
error with a plain-file-text escape hatch — not a silent reclassification to
`PlainText`.

## 6. UX model by format

### 6.1 Markdown

```text
Document Map = headings / sections
Right panel  = section body writer and preview
```

### 6.2 JSON

```text
Document Map = objects, arrays, and values
Right panel  = focused value editor or group/list summary
```

Plain labels:

| JSON concept | omriss label |
|--------------|--------------|
| object | group |
| array | list |
| property | name |
| string | text |
| number | number |
| boolean | on/off |
| null | No value |

### 6.3 TOML

```text
Document Map = tables, arrays of tables, keys, and values
Right panel  = focused value editor or table summary
```

Plain labels:

| TOML concept | omriss label |
|--------------|--------------|
| table | group |
| array of tables | repeated group |
| key | name |
| value | content |

### 6.4 YAML

```text
Document Map = best-effort hierarchy
Right panel  = read-only or safely editable only after feasibility gates
```

YAML must not be advertised as fully editable until anchors, aliases, tags, indentation, and multiline values are understood well enough.

## 7. Functional requirements

### FR-052-001: Format detection

omriss must detect the document format from extension and, where useful, content inspection.

The binding mapping is §5.1, which includes the shipped `.mdown` and `.txt`
Markdown extensions. Detection order is extension first, then lightweight
content validation where the extension is ambiguous, then a user-facing message
— never a silent guess (RFC-053 §10).

Where extension and content disagree, omriss must not take destructive action.
The required behavior is to keep the source text, explain the mismatch in plain
language, and offer plain file text.

Unknown extensions open as `PlainText` per §5.2: viewable, not editable, no
synthetic structure. This RFC does not introduce generic plain-text editing.

### FR-052-002: Format-specific Document Map

The Document Map must render the structure produced by the active format adapter.

### FR-052-003: Focused content editing

The right panel must render the focused content editor appropriate to the active format and selected node.

### FR-052-004: Validation before apply

For structured data formats, changes must be validated before applying to canonical source text.

### FR-052-005: Safe unsupported state

If a node or format cannot be safely edited, omriss must show a clear read-only state and offer `Show plain file text`.

### FR-052-006: Preservation tests

Each editable format must have preservation tests proving that unrelated source text remains byte-identical after a focused edit.

### FR-052-007: No global reformat by default

omriss must not reformat an entire file as part of normal editing or saving.

A future explicit command such as `Format file` may be considered, but it must not be part of this RFC.

## 8. Non-functional requirements

- Opening files up to ordinary project-config scale must feel immediate.
- Editing a focused value must provide feedback within the same interaction turn.
- Invalid changes must be explained in plain language.
- The app must not crash on malformed files.
- Malformed files must remain viewable as plain file text when possible.
- No network access is required.
- No telemetry is introduced.

## 9. Error handling policy

| Situation | User-facing behavior |
|-----------|----------------------|
| Unsupported file extension | “This file type is not supported yet.” |
| Malformed JSON | “This file does not look like valid JSON.” |
| Malformed TOML | “This file does not look like valid TOML.” |
| Complex YAML feature unsupported | “This part can be viewed, but safe editing is not ready yet.” |
| Unsafe replacement | “This change could not be made safely.” |
| Validation failure | “This text does not look valid yet.” |

Never show byte offsets, parser stack traces, node IDs, or crate names in normal UI.

## 10. Implementation strategy

Structured format support must be implemented through a format adapter boundary described in RFC-053.

Staged delivery:

```text
Stage A: UI migration stable for Markdown.
Stage B: Format detection and adapter shell.
Stage C: JSON read-only structure view.
Stage D: JSON focused value editing.
Stage E: TOML read-only structure view.
Stage F: TOML safe focused editing after preservation proof.
Stage G: YAML feasibility spike.
```

## 11. Product risk analysis

### 11.1 Risk: omriss becomes too broad

Mitigation:

- Keep Markdown primary.
- Keep JSON/TOML/YAML support behind the same simple Document Map / focused editor model.
- Avoid schema design, database editing, cloud sync, or complex form builders.

### 11.2 Risk: source preservation becomes harder

Mitigation:

- Require byte-preservation tests per format.
- Forbid full-file serialization as default save path.
- Introduce editability gradually.

### 11.3 Risk: YAML complexity consumes the project

Mitigation:

- Treat YAML as feasibility only.
- Do not promise editable YAML in product materials until accepted by a later RFC.

### 11.4 Risk: non-technical users see too many concepts

Mitigation:

- Use plain labels.
- Hide advanced actions.
- Show read-only summaries for complex groups/lists.
- Keep detailed syntax visible only in `Show plain file text`.

## 12. Documentation changes

User documentation should say:

```text
omriss works best with Markdown documents.
It can also help you view and, where supported, safely edit structured plain-text files.
```

Do not claim equal maturity across all formats.

Status labels are **time-dependent**, and documentation must state the status
that is true on the day it ships — never the status a format will earn later.

| Format | Status to document now (RFC-052) | Status after its implementing RFC is accepted |
|---|---|---|
| Markdown | Fully supported | unchanged |
| JSON | Planned — not available yet | Supported (after RFC-054) |
| TOML | Planned — not available yet | Experimental until preservation gates pass (after RFC-055) |
| YAML | Under investigation | determined by the RFC-056 go/no-go result |

The left column is binding for any documentation written under RFC-052. The
right column is what the implementing RFC's own handoff will change it to. A
documentation change that promotes a format before its RFC is accepted is a
defect, not an optimization.

## 13. Acceptance criteria

This RFC is documentation-only. It is complete when the following artifacts
exist and can be inspected — not merely when the policy is agreed in principle:

| # | Criterion | Evidence |
|---|---|---|
| 1 | Product scope wording moves from "Markdown editor" to "structured plain-text editor" without weakening Markdown priority | `README.md`, `docs/src/introduction.md` |
| 2 | Format support order recorded as JSON → TOML → YAML feasibility | This RFC §5, `ROADMAP.md` |
| 3 | Extension mapping records the shipped `.mdown` / `.txt` Markdown behavior as a compatibility constraint | This RFC §5.1 |
| 4 | `PlainText` and `Unsupported` have stated policy, not just type definitions | This RFC §5.2 |
| 5 | Full-file reserialization prohibited as normal save behavior | This RFC §4.1, §7 FR-052-007 |
| 6 | Documentation distinguishes stable, experimental, and feasibility states | new `docs/src/file-formats.md` (note: `languages.md` covers GUI i18n, not file formats) |
| 7 | No open question is left for the implementer to decide | This RFC §14 |
| 8 | RFC-053 is accepted as the required architecture for future formats | RFC-053 status |

Criterion 8 is a dependency on RFC-053's own acceptance and is the only item
that cannot be closed by RFC-052 work alone.

**Not in scope for acceptance:** any code, any parser, any adapter. The first
implementation work under M11 belongs to RFC-053.

## 14. Resolved questions and remaining decisions

An RFC must not hand unresolved design decisions to the implementer. The four
questions raised in review are resolved below, except one that is reserved for
the product owner.

### 14.1 Resolved — strict JSON only, no JSONC

RFC-054 implements **strict JSON (RFC 8259)**. Comments and trailing commas are
not accepted.

Rationale: JSON is chosen as the first adapter because its grammar is small
enough to prove the adapter boundary cheaply. JSONC reintroduces
comment-preservation — the same class of problem that makes TOML the *second*
target, not the first — and would defeat the purpose of the staging order.

Accepted consequence, stated plainly rather than hidden: real-world files such
as `tsconfig.json` and editor settings often contain comments and will be
reported as *"This file does not look like valid JSON."* with a plain-file-text
escape hatch. This is the safe failure mode — omriss shows the file and refuses
to guess, rather than parsing and silently discarding the comments. JSONC may be
revisited in a later RFC if demand is demonstrated.

### 14.2 Resolved — unknown text files open read-only

See §5.2. Unknown extensions map to `PlainText`: viewable, read-only, no
synthetic structure. `.txt` remains Markdown per §5.1.

### 14.3 Resolved — Quick Actions is format-aware

Format-specific commands appear in Quick Actions only when the active format and
selected node support them, driven by the **same** `NodeCapabilities` model that
drives the Document Map row menu.

Rationale: one availability source prevents the palette and the row menu from
disagreeing about the same action. Presentation differs by surface — the palette
**hides** unavailable commands, because a list of dead entries is noise in a
search-driven surface, while the row menu keeps **disabled-with-reason**,
because the user is looking at one specific item and deserves to know why.

### 14.4 Resolved by owner decision — visibility of new formats on first release

> **Decided by the project owner on 2026-07-30: Option C, per-format
> visibility.** This is a final decision, not a recommendation.

**Binding policy.** Each format's visibility is governed by its own risk
profile, not by a single global switch:

| Format | Visibility on first release | Condition |
|---|---|---|
| Markdown | Visible | unchanged |
| JSON | Visible by default | after RFC-054 acceptance; read-only stage first |
| TOML | Visible, labeled experimental | until RFC-055 preservation tests pass; write support stays disabled until then |
| YAML | Hidden; explicit opt-in only | until RFC-056 returns a go result |

**Consequences for downstream RFCs.** RFC-054, RFC-055, and RFC-056 must each
honor this table rather than re-opening the question. Specifically:

- no global "experimental formats" toggle is to be introduced;
- TOML must carry a visible experimental label in the UI, not only in
  documentation;
- YAML must not appear in the file dialog, format detection results, or Quick
  Actions without an explicit opt-in;
- a format may not be promoted out of its stage by a handoff. Promotion
  requires its governing RFC to be accepted.

---

**Original question and analysis, retained as the decision record.**

**Question.** When RFC-054 lands, does JSON support appear for all users by
default, or behind an "experimental formats" setting?

**Options.**

| Option | Benefit | Drawback |
|---|---|---|
| A. Default-on, gated only by each format's own acceptance criteria | No extra settings surface; no discovery problem; the RFC gates already prevent unsafe editing | A defect reaches every user immediately |
| B. Experimental toggle, opt-in per release | Blast radius limited to opted-in users | Adds a settings surface and a discovery problem; risks the feature being invisible and therefore untested by real users |
| C. Per-format: JSON default-on, TOML experimental until preservation proven, YAML opt-in only | Matches actual risk per format | Three different visibility rules to explain in documentation |

**Recommendation was C**, on the grounds that the risk profile genuinely
differs per format: JSON arrives read-only first with a strict grammar; TOML
carries real preservation risk until its golden tests pass; YAML must never
appear without explicit opt-in. A single global toggle would treat these as
equivalent when they are not. The owner accepted this on 2026-07-30.

## 15. Final decision summary

omriss may expand beyond Markdown if and only if the expansion preserves the original source text, keeps the Document Map / focused editor model, and introduces formats in a staged order. JSON is the first implementation target. TOML follows with strict preservation rules. YAML remains a feasibility spike.
