# RFC-062: Crate Package Name Exchange

**Project:** omriss — Omriss Editor
**Milestone:** Post-0.16 release cleanup (proposed)
**Status.** Implemented (main, unreleased) — landed on `main` after the 0.16.0
tag; the "Shipped in" version is assigned at the next release.
**Document type:** Design seed
**Primary audience:** Architect, Rust developer, release engineer
**Depends on:** RFC-001, RFC-037, RFC-042, RFC-047
**Related RFCs:** RFC-040

---

## 1. Summary

Exchange the package names of the core engine and desktop application:

| Role | Current package | Proposed package |
| --- | --- | --- |
| Source-preserving document engine | `omriss` | `omriss-core` |
| Desktop application shell | `omriss-app` | `omriss` |

The goal is to make the product package name match the product and binary name,
so the natural local run command becomes:

```sh
cargo run -p omriss
```

The preferred design is a direct package-name exchange, not an alias, wrapper,
or facade layer.

## 2. Motivation

After the 0.16.0 release, the project name, binary name, repository name, and
user-facing product name are all `omriss`. However, the Cargo package named
`omriss` is still the core library crate, while the application package is
`omriss-app`.

That makes the most natural command for running the product unavailable:

```sh
cargo run -p omriss
```

The current command is:

```sh
cargo run -p omriss-app
```

This is understandable from an internal boundary perspective, but it is less
natural for users and contributors. Over the long term, the package named
`omriss` should mean the product application, while the reusable engine should
carry the explicit `-core` suffix.

## 3. Background

RFC-047 renamed the application and crate layout for v0.14.0:

- `omriss-core` became `omriss`;
- `omriss-desktop` became `omriss-app`;
- the produced binary became `omriss`.

This RFC intentionally revisits that package-name decision after the 0.16.0
release. The product naming is now stable enough that the package identity
should be optimized for long-term clarity.

## 4. Goals

- Make `cargo run -p omriss` run the desktop app.
- Make the source-preserving engine package name explicit: `omriss-core`.
- Keep crate boundaries from RFC-001 intact:
  - `omriss-core` has no Dioxus, WebView, file-dialog, or desktop runtime
    dependencies;
  - `omriss-ui` remains renderer-independent;
  - `omriss` owns the Dioxus desktop shell and platform integration.
- Keep the workspace simple, without wrapper packages or compatibility facades.
- Update documentation, RFC references, release checklist commands, and package
  metadata consistently.
- Define package publishing policy before any renamed package is released.

## 5. Non-goals

- Do not add a new command alias or `xtask` solely to hide the current package
  names.
- Do not introduce a facade package that re-exports core APIs from the app
  package.
- Do not merge `omriss-core`, `omriss-ui`, and `omriss` into one crate.
- Do not change the binary name; it remains `omriss`.
- Do not change product behavior.

## 6. Proposed Design

### 6.1 Package Names

Change package names:

```toml
# crates/core/Cargo.toml
[package]
name = "omriss-core"

# crates/app/Cargo.toml
[package]
name = "omriss"
```

The app package keeps:

```toml
[[bin]]
name = "omriss"
path = "src/main.rs"
```

### 6.2 Dependency Names

Workspace dependencies become:

```toml
[workspace.dependencies]
omriss-core = { version = "0", path = "crates/core" }
omriss-ui   = { version = "0", path = "crates/ui" }
```

`omriss-ui` depends on `omriss-core`.

The app package `omriss` depends on:

```toml
omriss-core = { workspace = true }
omriss-ui   = { workspace = true }
```

### 6.3 Rust Import Paths

Internal Rust import paths change from:

```rust
use omriss::{Document, NodeId};
```

to:

```rust
use omriss_core::{Document, NodeId};
```

This follows Cargo's hyphen-to-underscore crate import rule.

### 6.4 Workspace Defaults

Keep the app package outside default members unless the platform dependency
policy changes:

```toml
default-members = [
    "crates/core",
    "crates/ui",
]
```

This preserves the current behavior where `cargo test` and `cargo build` on
default members avoid desktop WebView dependencies, while app execution becomes:

```sh
cargo run -p omriss
```

### 6.5 Public Compatibility

This is a breaking package/API naming change for Rust library consumers:

- old import: `omriss::Document`;
- new import: `omriss_core::Document`.

The design intentionally accepts that break to avoid permanent compatibility
facades and extra package indirection. The migration should be documented in
the changelog and README.

### 6.6 Publishing And Release Policy

The package-name exchange must not silently change the role of an already
published package.

Publishing policy:

- `omriss-core` is the publishable library package for the source-preserving
  document engine.
- `omriss` is the desktop app package. For the initial package-name exchange,
  it should set `publish = false` unless a separate release review explicitly
  decides that publishing the desktop app package to crates.io is intended.
- `omriss-core` must be publishable and packaged successfully before any future
  release attempts to publish an app package named `omriss`.
- The existing `omriss` library package role change must be called out in the
  changelog and README migration notes. If a crates.io deprecation/migration
  release is needed, it must be planned explicitly before publishing.
- README badges and docs.rs links that currently point to the `omriss` library
  package must move to `omriss-core` after the exchange, unless the link is
  intentionally about the app package.
- `RELEASE_CHECKLIST.md` must gain a package-role check so release execution
  verifies which packages are publishable and prevents publishing a package with
  the wrong role.

## 7. Options Considered

### Option A: Direct exchange, preferred

`omriss-core` is the library package. `omriss` is the app package.

Benefits:

- simple long-term mental model;
- `cargo run -p omriss` works;
- package names match product roles;
- no wrapper/facade maintenance.

Costs:

- breaking change for `omriss` library consumers;
- large documentation and import-path update;
- crates.io package identity changes must be handled carefully.

### Option B: Keep packages and add a command workaround

Keep `omriss` as core and `omriss-app` as app, then add docs or a helper command.

Rejected because it preserves the confusing package identity and adds process
complexity without fixing the root naming model.

### Option C: Facade / compatibility model

Rename packages but keep a facade or re-export path for compatibility.

Rejected for the preferred design because it adds internal complexity, risks
pulling app dependencies into library users, and obscures ownership boundaries.
It should only be reconsidered if the architect decides compatibility must
override simplicity.

## 8. Risks

- **Crates.io expectation risk:** users of the existing `omriss` library package
  may be surprised if a future `omriss` version becomes the app package.
- **Documentation churn:** many RFCs, docs, README entries, and test commands
  reference current package names.
- **Import churn:** internal code and tests must move from `omriss::` to
  `omriss_core::`.
- **Release automation risk:** future CI workflow names and package filters must
  be updated consistently.
- **RFC-001 wording drift:** architecture docs currently use `omriss` to mean
  core. They must be amended or clearly versioned.
- **Publishing role risk:** a future `omriss` package release could be mistaken
  for a continuation of the old core library unless `publish = false`,
  migration notes, badges, and release checklist gates are updated deliberately.

## 9. Implementation Outline

1. Rename package metadata:
   - `crates/core/Cargo.toml`: `omriss` -> `omriss-core`;
   - `crates/app/Cargo.toml`: `omriss-app` -> `omriss`.
2. Update workspace dependency keys and package dependencies.
3. Update Rust imports and doc examples from `omriss::` to `omriss_core::` where
   they refer to the core crate.
4. Update user-facing commands:
   - app run/check command: `cargo run -p omriss`, `cargo check -p omriss`;
   - core test command: `cargo test -p omriss-core`.
5. Update package publishing metadata:
   - make `omriss-core` the publishable library package;
   - set the app package `omriss` to `publish = false` unless a separate release
     decision authorizes publishing it;
   - update README crates.io/docs.rs badges and links to point at `omriss-core`
     for library API documentation.
6. Update README, TESTING, RELEASE_CHECKLIST, PLATFORMS, RFC index references,
   handoff templates, and changelog migration notes.
7. Update architecture boundary checks:
   - commands that mean "core must not depend on UI/runtime crates" must target
     `cargo tree -p omriss-core`, not `cargo tree -p omriss`.
8. Add release checklist protection:
   - verify package roles before publication;
   - verify `omriss-core` is the library package;
   - verify the app package `omriss` is not publishable unless explicitly
     approved for crates.io app distribution.
9. Run verification:
   - `cargo fmt --check`;
   - `cargo test` or `cargo test -p omriss-core -p omriss-ui` for default-member
     non-desktop tests;
   - `cargo test --workspace` only on hosts with the required app system
     dependencies installed;
   - `cargo check -p omriss`;
   - `cargo test -p omriss-core`;
   - `cargo test -p omriss-ui`;
   - `cargo tree -p omriss-core`;
   - `./scripts/check-rfcs.sh`;
   - `git diff --check`.

## 10. Review Questions

1. Should `omriss` package identity mean the product app going forward, with the
   core engine explicitly named `omriss-core`?
2. Is accepting the breaking Rust import-path change preferable to maintaining a
   facade or compatibility layer?
3. Should directory names remain generic (`crates/core`, `crates/app`) or change
   to match packages (`crates/omriss-core`, `crates/omriss`)?
4. Are there crates.io publishing or deprecation steps that must be included
   before implementation?
5. Should this be released as the next minor release with a prominent migration
   note?
6. Should the app package `omriss` remain permanently unpublished, or only
   unpublished for the first exchange release?

## 11. Acceptance Criteria

- Architect review accepts the direct exchange or gives explicit changes.
- `cargo run -p omriss` runs the desktop app.
- `cargo test -p omriss-core` runs core tests without desktop dependencies.
- `cargo test -p omriss-ui` runs UI-logic tests without desktop dependencies.
- `cargo check -p omriss` checks the app package.
- Documentation no longer tells users to run the app with `cargo run -p
  omriss-app`.
- Changelog records the breaking package/import migration.
- README badges/docs.rs links distinguish the app package from the library API
  package and point library users to `omriss-core`.
- `RELEASE_CHECKLIST.md` includes a package-role/publishability gate.
- Core dependency-boundary verification uses `cargo tree -p omriss-core`.
