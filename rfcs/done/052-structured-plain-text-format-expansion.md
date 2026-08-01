# RFC-052: Structured Plain-Text Format Expansion

**Project:** omriss — Omriss Editor
**Milestone:** M11 — Format Adapter Foundation
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-048, RFC-049, RFC-050, RFC-051
**Related RFCs:** RFC-053, RFC-054, RFC-055, RFC-056

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

Supported initial mapping:

```text
.md, .markdown → Markdown
.json          → JSON
.toml          → TOML
.yaml, .yml    → YAML candidate / feasibility mode
```

Unknown extensions should open as plain text only if that behavior is explicitly designed later. This RFC does not require generic plain-text editing.

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

Suggested status labels:

| Format | Documentation status |
|--------|----------------------|
| Markdown | Fully supported |
| JSON | Supported after RFC-054 acceptance |
| TOML | Experimental until preservation gates pass |
| YAML | Under investigation |

## 13. Acceptance criteria

This RFC is accepted when:

- product scope statement is updated from “Markdown editor” to “structured plain-text editor” without weakening Markdown priority;
- RFC-048–051 remain the UI foundation;
- format support order is agreed: JSON first, TOML second, YAML feasibility later;
- full-file reserialization is prohibited as normal save behavior;
- RFC-053 is approved as the required architecture for future formats;
- documentation clearly distinguishes stable, experimental, and feasibility states.

## 14. Open questions

1. Should JSON/TOML support be enabled by default or hidden behind “experimental formats” at first?
2. Should omriss support JSONC or only strict JSON?
3. Should unknown text files open in raw read-only mode?
4. Should format-specific commands appear in Quick Actions only when the active format supports them?

## 15. Final decision summary

omriss may expand beyond Markdown if and only if the expansion preserves the original source text, keeps the Document Map / focused editor model, and introduces formats in a staged order. JSON is the first implementation target. TOML follows with strict preservation rules. YAML remains a feasibility spike.
