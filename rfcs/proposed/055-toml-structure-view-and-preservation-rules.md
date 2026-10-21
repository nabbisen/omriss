# RFC-055: TOML Structure View and Preservation Rules

**Project:** omriss — Omriss Editor
**Milestone:** M12 — Structured Format Support
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-053, RFC-054
**Related RFCs:** RFC-056

---

## 1. Summary

This RFC defines TOML support for omriss.

TOML is a strong fit for omriss because it is structured, text-based, widely used for configuration, and common in Rust projects. However, TOML editing must be more conservative than JSON because comments, ordering, table layout, inline tables, and arrays of tables are important to users.

Initial TOML support should be staged:

1. read-only Document Map view;
2. focused value editing where source preservation is proven;
3. table/group raw focused editing only after strict validation;
4. structural operations only after a separate implementation plan if needed.

## 2. Goals

- Open `.toml` files.
- Build a Document Map from TOML tables, arrays of tables, keys, and values.
- Preserve comments, ordering, spacing, and table layout.
- Support focused scalar value editing where safe.
- Avoid full-file reserialization or normalization during normal save.
- Provide friendly labels and errors.
- Keep TOML support compatible with the UI split from RFC-048–051.

## 3. Non-goals

- This RFC does not require full TOML structural editing in the first release.
- This RFC does not require drag-and-drop for TOML tables or keys.
- This RFC does not implement schema validation.
- This RFC does not format the whole TOML file.
- This RFC does not remove or rewrite comments.
- This RFC does not promise editing of every TOML syntax form in the first editable version.

## 4. User experience

Example TOML:

```toml
[package]
name = "omriss"
version = "0.1.0"

[dependencies]
serde = "1"
pulldown-cmark = "0.13"
```

Document Map:

```text
Document Map

▾ package
  name
  version
▾ dependencies
  serde
  pulldown-cmark
```

Right panel for selected table:

```text
package

This group contains 2 items.
Use the Document Map to choose an item.

[Show this part as text]
```

Right panel for selected value:

```text
name

Text
[ omriss ]

[Done]
```

### 4.1 The open dialog's filters

Task 016 made the open dialog default to an **"All supported"** filter built by
chaining the per-format extension lists (`crates/app/src/file/file_dialog.rs`),
so that a format added to one list cannot be missing from the default view.
TOML is the first format to arrive after that, and two things follow.

**Order is load-bearing, and not obviously so.** `rfd`'s Windows backend calls
`IFileDialog::SetDefaultExtension` with the **first extension of the first
filter** (`add_filters`, rfd 0.15.4), so that extension is what a Save-As dialog
appends when the user types a bare name. `md` is first today. Append
`TOML_EXTENSIONS` to the chain; do not prepend it, or Save-As silently starts
suggesting `.toml` for Markdown documents.

**TOML stays out of "All supported" until its preservation tests pass.**
RFC-052 §14.4 makes TOML *visible, labeled experimental*. An aggregate filter
entry has nowhere to carry that label: a user who opens a `.toml` from "All
supported" is never shown the word "experimental". Offering TOML only through
its own explicitly-labeled filter — `TOML (experimental)` — is how §14.4's two
halves survive together in a dialog. When §13.2's preservation tests pass and
the experimental label is dropped, chain `TOML_EXTENSIONS` into the default and
remove this exception.

## 5. Plain labels

| TOML concept | omriss label |
|--------------|--------------|
| table | group |
| array of tables | repeated group |
| key | name |
| string | text |
| integer/float | number |
| boolean | on/off |
| datetime | date/time |
| array | list |
| inline table | small group |

## 6. Preservation rules

TOML support must follow strict preservation rules.

### 6.1 Comments

Comments must be preserved exactly unless the user edits the specific focused range containing them through a raw text mode.

### 6.2 Key ordering

Key order must be preserved.

### 6.3 Table ordering

Table order must be preserved.

### 6.4 Spacing

Spacing around unrelated keys, blank lines, and comments must be preserved.

### 6.5 Inline forms

Inline tables and arrays must not be automatically expanded or normalized.

### 6.6 Quoting style

String quote style should be preserved where possible. If changing the quote style is unavoidable, the adapter must do so only within the selected value range and only after validation.

### 6.7 Line endings

LF/CRLF style must be preserved outside edited ranges.

## 7. Recommended parsing/editing strategy

Use a lossless or formatting-preserving TOML editing strategy. A value-only parser that discards comments and formatting is not sufficient for normal saves.

The TOML adapter must provide:

- table/key structure;
- source ranges for editable values;
- source ranges for table/group summaries where safe;
- preservation of comments and ordering;
- validation before mutation.

If exact range information is not available for a TOML feature, that feature should be read-only until a safe editing method is designed.

## 8. Structure model

The adapter should produce nodes for:

- root document;
- tables;
- dotted tables;
- arrays of tables;
- keys;
- scalar values;
- arrays;
- inline tables.

Example:

```toml
[workspace]
members = ["crates/core", "crates/app"]

[profile.release]
lto = true
```

Document Map:

```text
workspace
  members
profile
  release
    lto
```

Exact tree representation may either show dotted paths as nested groups or as a single row. The UI should prefer the form that is clearest and least surprising.

## 9. Node identity

TOML node identity should be based on TOML key paths plus occurrence ordinal where needed.

Examples:

```text
/package/name
/dependencies/serde
/profile/release/lto
/bin[0]/name
```

User-facing breadcrumbs should not show slash syntax.

## 10. Editing policy

### 10.1 First editable version

Support only focused scalar value editing where the adapter can safely replace the value range.

Supported candidates:

- string;
- integer;
- float;
- boolean;
- simple arrays, optional;
- date/time, read-only or conservative text mode.

### 10.2 Table/group editing

Tables should initially be summaries with optional raw focused text preview.

Editing an entire table as raw text may be added only if:

- replacement parses as TOML;
- replacement is valid in that table location;
- outside bytes remain unchanged;
- comments outside the range are preserved.

### 10.3 Structural operations

Initial TOML support should not require:

- add key;
- rename key;
- delete key;
- move key;
- move table;
- add table.

These may be added later. TOML structural editing is riskier than Markdown and JSON because comments and layout often carry user intent.

## 11. Validation behavior

| Value kind | Validation |
|------------|------------|
| text | Escaped and written as valid TOML string, preserving style where possible. |
| number | Must be valid TOML integer/float. |
| on/off | Writes `true` or `false`. |
| list | Must parse as a valid TOML array if raw list editing is enabled. |
| date/time | Must parse as valid TOML date/time if editing is enabled. |
| inline table | Read-only until safe editing is designed. |

Friendly messages:

```text
This number is not valid yet.
This date/time is not valid yet.
This list is not valid yet.
```

## 12. Error handling

| Cause | User-facing message |
|-------|---------------------|
| malformed TOML | “This file does not look like valid TOML.” |
| unsupported syntax for editing | “This part can be viewed, but safe editing is not ready yet.” |
| unsafe replacement | “This change could not be made safely.” |
| invalid value | “This value is not valid yet.” |
| unsupported structure action | “This TOML change is not supported yet.” |

## 13. Testing requirements

### 13.1 Structure tests

- root keys before any table;
- simple table;
- dotted table;
- nested dotted keys;
- array of tables;
- inline table;
- arrays;
- strings, numbers, booleans, date/time values;
- comments before/after keys;
- comments before/after tables.

### 13.2 Preservation tests

- edit string value and preserve comments;
- edit number value and preserve key order;
- edit boolean value and preserve blank lines;
- edit nested table value and preserve unrelated tables;
- preserve inline array formatting when unrelated;
- preserve CRLF;
- undo restores exact original source text.

### 13.3 Rejection tests

- malformed TOML rejected safely;
- unsafe inline table edit rejected;
- unsupported array-of-tables operation rejected;
- stale node rejected.

## 14. Implementation phases

### Phase 1 — Read-only TOML tree

- detect `.toml`;
- validate TOML;
- build Document Map;
- show focused group/value summaries;
- show friendly parse failure.

### Phase 2 — Scalar value editing

- strings;
- numbers;
- booleans;
- save/undo/redo integration;
- preservation tests.

### Phase 3 — Conservative raw focused table preview/edit

- preview selected table source;
- validate replacement;
- apply only if safe;
- otherwise show read-only state.

### Phase 4 — Structure operations, later RFC if needed

- add key;
- rename key;
- delete key;
- move key/table.

## 15. Acceptance criteria

- `.toml` files can be opened after JSON support architecture is available.
- Document Map shows TOML hierarchy in plain terms.
- Comments and ordering are preserved in all editable operations.
- Scalar value editing works only where source range replacement is safe.
- Unsupported TOML features show safe read-only messages.
- Normal save does not normalize or reformat the whole file.
- Undo restores exact previous source text.
- Error messages are friendly and non-technical.

## 16. Open questions

1. Should dotted tables display as nested groups or single path rows?
2. Should `Cargo.toml` receive specialized labels or remain generic TOML?
3. Should date/time editing be supported in the first editable version?
4. Should adding new keys be implemented before raw table editing?

## 17. Final decision summary

TOML is a good second structured format for omriss, but it must be implemented conservatively. Read-only structure view comes first. Focused scalar editing follows only with strict preservation tests. Full structural TOML editing is deferred.
