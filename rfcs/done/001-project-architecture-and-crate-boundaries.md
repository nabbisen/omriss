<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-001: Project Architecture and Crate Boundaries

**Project:** omriss — Omriss Editor  
**Milestone:** M0 — Technical Spike  
**Status.** Implemented (v0.1.0)  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Define the workspace architecture for omriss. The primary decision is that the source-preserving editor engine lives in a UI-independent Rust crate. Dioxus Desktop is allowed to render and dispatch UI actions, but it must not own document semantics.

## 2. Goals

- Create a Rust workspace with clear crate responsibilities.
- Keep `omriss` free from Dioxus, WebView, desktop dialog, and OS integration dependencies.
- Define how UI commands cross into core operations without custom frontend/backend IPC.
- Enable core correctness tests to run through `cargo test` without launching a desktop shell.

## 3. Non-Goals

- No plugin architecture.
- No public stable API guarantee for third-party consumers in M0.
- No final packaging layout.

## 4. Design

### Workspace Layout

Adopted layout (design review, v0.1.0):

```text
omriss/
  Cargo.toml
  crates/
    core/
      src/document.rs
      src/outline.rs
      src/index.rs
      src/range.rs
      src/revision.rs
      src/edit.rs
      src/history.rs
      src/error.rs
      src/tests/        (unit tests)
      tests/            (golden integration tests + fixtures)
    omriss-ui/
      src/session.rs
      src/view_state.rs
      src/i18n/
      src/tests/
    app/
      src/main.rs
      src/app.rs        (Dioxus components)
      src/file_dialog.rs
```

The original draft placed the Dioxus components in `omriss-ui`. Review moved
them one level out, to the app package, and made `omriss-ui` fully
renderer-independent (session state, focus navigation, i18n catalogs — plain
Rust, no Dioxus dependency at all). Rationale: this extends the RFC's own
goal — headless `cargo test` — from core to the entire editor logic, and it
confines the WebView/windowing dependency to a single leaf crate that hosts
nothing but the thin `rsx!` projection and platform glue. If the component
layer grows beyond a thin projection, splitting a Dioxus-dependent
`omriss-components` crate out of the app package is the planned follow-up.

### Dependency Direction

```text
omriss -> omriss-ui -> omriss-core
```

No reverse dependency is allowed. `omriss-core` and `omriss-ui` must compile
on stable Rust with no desktop runtime feature; only the `omriss` app package links
the platform WebView stack, and it is excluded from the workspace default
members so `cargo build`/`cargo test` work on hosts without GUI libraries.

### Boundary Rules

| Crate | Owns | Must Not Own |
|---|---|---|
| `omriss-core` | Markdown text, outline index, ranges, edit commands, validation | Dioxus components, file dialogs, WebView state, user-facing prose |
| `omriss-ui` | Editor session, dirty tracking, focus/view navigation state, i18n catalogs | Dioxus/WebView dependencies, raw filesystem policy, parser internals |
| `omriss` | entrypoint, Dioxus components, platform integration, native file dialogs, app lifecycle | section edit semantics, document state |

### Command Flow

```text
User event -> omriss-ui command -> omriss-core operation -> EditResult -> UI state update
```

The command flow is an in-process Rust call boundary, not a serialized IPC protocol.

## 5. Internal Design Notes

### Internal Dependency Enforcement

- Use workspace-level linting to deny accidental dependencies from `omriss-core` to UI/runtime crates.
- Add a CI check that runs `cargo tree -p omriss-core` and fails if forbidden crates appear.
- Keep `omriss-core` feature flags minimal. Optional parser or rope features must remain internal implementation details until stabilized.

## 6. Validation and Test Plan

- `cargo test -p omriss-core` runs without desktop dependencies.
- Forbidden dependency smoke test for `omriss-core`.
- Example CLI-like test can parse and edit Markdown using core only.

## 7. Acceptance Criteria

- Workspace builds with the three-crate dependency direction.
- No Dioxus, wry, tao, or file-dialog dependency is present in `omriss-core`.
- Core tests can validate source-preserving replacement independently of UI.
## 8. Dependencies

- None. This RFC is foundational for the workspace.

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
