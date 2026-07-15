<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Added during architecture/design review to close the i18n requirements gap
Language: English
-->
# RFC-043: GUI Internationalization and Locale Policy

**Project:** omriss — Omriss Editor
**Milestone:** M2 — Basic Desktop UX (catalog infrastructure) / M8 — Cross-Platform Delivery (locale switching UX)
**Status.** Implemented (v0.1.0) — deferred: explicit locale setting persistence awaits RFC-036; startup detection is environment-based until then
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer

---

## 1. Summary

Define how the omriss GUI supports multiple languages. The project requirements
mandate a multilingual GUI, but RFC-001 through RFC-042 either ignored
localization or excluded it as a non-goal. This RFC closes that gap: every
user-facing string in `omriss-ui` is resolved through a message catalog keyed by
stable identifiers, with English as the authoritative fallback locale.

## 2. Goals

- Externalize all user-visible GUI strings (labels, dialogs, status messages, errors).
- Define a stable message-key convention and an English fallback chain.
- Define locale detection at startup and explicit locale selection in settings.
- Keep `omriss` entirely locale-free: core returns structured errors, never prose.
- Ship English (`en`) and Japanese (`ja`) catalogs as the initial proof of the model.

## 3. Non-Goals

- No translation of user documents. Markdown content is never touched.
- No runtime download of translation packs.
- No locale-dependent Markdown parsing or formatting behavior.
- No RTL layout work in the initial implementation (tracked as a future RFC when an RTL locale is added).
- No pluralization/gender grammar engine in the first version; keys needing it are deferred until a target locale requires it.

## 4. Design

### Layer Responsibility

```text
omriss    structured errors and data only; no user-facing prose
omriss-ui      owns message catalogs and key lookup; renders localized text
The `omriss` app package detects OS locale at startup; persists explicit user choice in settings
```

This mirrors the RFC-009 result boundary: core emits `EditError::RevisionMismatch`,
UI maps it to the localized message for key `error.revision-mismatch`.

### Message Keys

Keys are lowercase, dot-namespaced, stable identifiers:

```text
app.title
action.open / action.save / action.save-as
view.overview / view.focus / view.raw-source
breadcrumb.root
status.saved / status.unsaved / status.index-warning
dialog.unsaved-changes.title / dialog.unsaved-changes.body
error.revision-mismatch / error.stale-node / error.invalid-utf8
```

Renaming a key is a reviewed change; catalogs for all locales must update in the
same commit (same rule as RFC 000 applies to status fields).

### Catalog Format

Catalogs are compiled into the binary as static key→string tables, one module per
locale inside `omriss-ui/src/i18n/`. No file I/O is required to render the UI, which
preserves the local-first and no-hidden-state principles. A heavier framework
(e.g. Fluent) may replace the static tables later behind the same lookup API.

### Lookup and Fallback

```rust
pub enum Locale { En, Ja }

pub fn t(locale: Locale, key: &str) -> &'static str;
```

Resolution order: requested locale → English → the key itself (visible fallback
that makes missing translations diagnosable rather than silent).

### Locale Selection

```text
startup: explicit setting (RFC-036) if present, else OS locale, else English
runtime: user may switch locale in Settings; takes effect without restart
```

## 5. Internal Design Notes

- A test-only exhaustiveness check asserts every key present in `en` is present
  in every shipped locale (or intentionally listed as pending).
- Keyboard shortcut labels are localized; the shortcuts themselves are not
  (RFC-014 owns the bindings).
- Date/number formatting is out of scope until a feature needs it.

## 6. Validation and Test Plan

- Catalog exhaustiveness test across locales.
- Fallback test: unknown key returns the key; key missing in `ja` falls back to `en`.
- UI smoke test renders the shell in both locales without panics.
- No string literal destined for the user appears outside the catalog (review checklist; lint later if practical).

## 7. Acceptance Criteria

- All M2 shell strings resolve through `t()`.
- English and Japanese catalogs ship and pass exhaustiveness tests.
- `omriss` contains no user-facing prose.
- Locale can be selected explicitly and persists via RFC-036 settings.

## 8. Dependencies

- RFC-009
- RFC-010
- RFC-029
- RFC-036
- RFC-039

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
