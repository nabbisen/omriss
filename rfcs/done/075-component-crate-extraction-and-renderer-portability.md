# RFC-075: Component Crate Extraction and Renderer Portability

**Project:** omriss — Omriss Editor
**Milestone:** M13 — architecture (0.18.0)
**Status.** Implemented (main, unreleased) — landed as RFC-075's PRs 1, 2 and
4 on 2026-11-03; PR 3 was deliberately empty (§9.3's question, answered in the
review by construction: `input/keyboard.rs` stays in the host). The crate is
`omriss-components`, `publish = false`. Two things the RFC did not predict are
recorded in §9.4. — extraction implemented on `main` (PRs 1–4: `63c02cb`, `febb23c`, no code change for PR 3, and the documentation commit), awaiting architect review; §10.7 (linking §7 from RFC-064) is not done, so the RFC has not moved to `done/`.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, release engineer
**Depends on:** RFC-001, RFC-062
**Related RFCs:** RFC-053, RFC-064, RFC-066, RFC-073

---

## 1. Summary

The question that prompted this RFC was whether `omriss-ui` could be reused to
build a web editor. The answer is **no, because `omriss-ui` contains no
components at all** — and the real answer is better than the question assumed:
the components exist, they are already free of platform coupling, and
everything they depend on builds for `wasm32-unknown-unknown` today without a
single change.

What stands in the way is not the code. It is the crate the code lives in.

This RFC moves the Dioxus components out of the application crate into a crate
of their own, so that the boundary omriss already respects by discipline is
enforced by the compiler — and so that a second host, web or otherwise, is a
matter of writing a host rather than untangling a crate.

**It does not build a web product, and it should be done whether or not one is
ever built.**

## 2. What is true today, measured

### 2.1 `omriss-ui` has no components

Its entire dependency list is:

```toml
[dependencies]
omriss-core = { workspace = true }
```

No Dioxus. What it holds is view state, navigation, search, statistics, i18n
catalogs, the session bridges and the document-map projection — the
renderer-independent layer, exactly as RFC-001 intended.

**Its published description is wrong.** On crates.io today, `omriss-ui` 0.16.0
reads *"View-state logic, i18n catalogs, and GUI components for Omriss"*. The
last clause has never been true. §8.1.

### 2.2 The components are in the application crate, and are already portable

Twenty-four files under `crates/app/src/components/`. Their complete import
profile:

| Imported | Occurrences |
|---|---|
| `omriss_ui` | 35 |
| `dioxus` | 22 |
| `omriss_core` | 4 |
| `dioxus_swdir_tree` | 4 |
| `keyboard_types` | 1 |
| `crate::shell::draft_sync` | **3** |

And the absences matter more than the presences. Across all twenty-four files
there is **no `std::fs`, no `SystemTime`, no `rfd`, no `dirs`, no `std::env`,
and not one `#[cfg(...)]`**. Every file-system and platform call lives in
`file/`, `storage/`, `locale.rs` and `shell/actions.rs`.

Four components reach the DOM through `dioxus::document::eval` — `modal_focus`,
`menu_focus`, `preview_pane`, `command_palette`. All of it is ordinary DOM
(`querySelector`, `focus()`, `getClientRects`); none of it touches
dioxus-desktop's IPC or any desktop global. On the web that code is more at
home than it is now, not less.

### 2.3 Everything needed already builds for the web target

Not inferred from manifests — built, on 2026-11-01, against
`wasm32-unknown-unknown`:

| Crate | Result |
|---|---|
| `omriss-core` | **builds**, unmodified (its only dependency is `pulldown-cmark`) |
| `omriss-ui` | **builds**, unmodified |
| `dioxus-swdir-tree` with `default-features = false` — how omriss already uses it | **builds** (it reaches `swdir`, which does use `std::fs`, but that path is dead for the `ItemTree` API omriss uses) |
| `keyboard-types` | **builds** |

So the portability question is settled before any work is proposed. The
blocker is that `crates/app` also contains `dioxus-desktop`, `tao`, `wry` and
`rfd`, and a crate cannot be half-built.

## 3. The only coupling: `draft_sync`

Three component files import `crate::shell::draft_sync`. That module is 100
lines and imports `dioxus`, `omriss_core` and `omriss_ui` — nothing else. It is
renderer-independent logic that happens to sit under `shell/`, which otherwise
holds the desktop wiring and the file actions.

It is the only thing standing between the components and a clean crate
boundary, and moving it is a move, not a redesign.

## 4. Design

### 4.1 Three layers, and why the components do not join `omriss-ui`

```text
omriss-core        no UI, no renderer          pulldown-cmark only
omriss-ui          no renderer                 omriss-core only
omriss-components  Dioxus components           omriss-ui + dioxus + tree widget
omriss             the desktop host            + dioxus-desktop, rfd, dirs, tao/wry
```

The tempting shortcut is to put the components into `omriss-ui`, since the name
already says UI. **That would destroy the property that makes `omriss-ui`
worth having.** Today a consumer can take the view-state layer with one
dependency and no renderer — a server-side renderer, a different framework, a
test harness, a future TUI. Adding Dioxus drags `wasm-bindgen` and `web-sys`
into all of them.

The layer boundary is the asset. The new crate sits below the host and above
the renderer-independent layer, and `omriss-ui` keeps its single dependency.

### 4.2 The new crate is named `omriss-components`

> **Decided by the project owner on 2026-11-03: `omriss-components`, with
> `publish = false` (§4.3).** These are final decisions, not recommendations.

**The decisive fact is §4.3's.** An unpublished crate's name is cheap to change
— a `git mv` and three manifest lines — so the criterion is not "safest
forever" but "clearest to someone opening the repository today". An earlier
draft of this section recommended `omriss-dioxus` on a permanence argument that
an unpublished crate does not support.

`components` is not an empty category word here. It is Dioxus and React
vocabulary for precisely what the crate holds, so a Rust developer who reads it
expects RSX and finds RSX. `crates/components` also sits correctly beside
`crates/core`, `crates/ui` and `crates/app`, the generic directory scheme
RFC-062's question 3 settled.

It survives the likely change, too. Replacing one renderer with another is far
likelier than running two at once — and §5 refuses the multi-renderer
abstraction that would justify a framework suffix. After such a swap
`omriss-components` is still true; `omriss-dioxus` would be a lie.

**The `ui` versus `components` question is answered in the description, not the
name:**

```toml
description = "Dioxus components for Omriss, built on omriss-ui's renderer-independent view state"
```

That is the first line anyone reads beside a crate name, and this project has
direct evidence that the field is load-bearing: `omriss-ui`'s description
claimed GUI components it never had, on a public registry, for a year.

**Rejected, with reasons, so they are not re-proposed:**

| Rejected | Why |
|---|---|
| `omriss-gui` | `ui` and `gui` are three letters apart and would name two different layers. That is exactly the pair a tired reader picks wrong |
| `omriss-widgets` | understates and slightly misleads: the crate holds dialogs, panes and the shell's composition, not reusable controls |
| `omriss-ui-dioxus` | the most precise option, and the right one **if the crate were published**. Unpublished, it is precision aimed at an audience that does not exist |
| `omriss-dioxus` | bakes a third-party framework into a permanent identity for a benefit that only exists on a public registry |

### 4.3 The new crate is **not published** — owner decision, 2026-11-03

`publish = false`, until a second consumer actually exists.

This looks like a small thing and is the most consequential recommendation
here. Publishing is not packaging; it is a promise. Every published crate turns
its API into a contract, and RFC-062 has already shown what a package-identity
change costs this project once made public. The split delivers its entire
engineering benefit — a compiler-enforced boundary, a web-buildable component
set, components testable without the platform WebView — **with the crate
unpublished**.

When a real second consumer exists, publishing is one line plus a stability
statement, written then, about an API that has by then been exercised by two
products instead of imagined for one.

For a single maintainer, the cost of a published API is not the publish
command; it is every future breaking change paid for twice.

### 4.4 What moves

| From | To | Note |
|---|---|---|
| `crates/app/src/components/**` (24 files) | `crates/components/` | unchanged content |
| `crates/app/src/shell/draft_sync.rs` | `crates/components/` | §3; it is not shell |
| `crates/app/src/input/keyboard.rs` | **stays** — decided by whether it is renderer-independent; see §9.3 | |

**The move must be byte-identical.** No renames, no "while we are here"
improvements, no reformatting beyond what `rustfmt` does to a moved file. The
whole test suite is the proof, and the review request states the diff is a move
and nothing else. A refactor that changes behaviour while claiming to be a move
is the single way this work could hurt.

### 4.5 What stays in `omriss`

`main.rs`, the shell wiring (`app.rs`, `app_ctx.rs`, `dispatch.rs`,
`actions.rs`), `file/`, `storage/`, `cli.rs`, `locale.rs`, `csp.rs`. That is
the host: platform IO, platform configuration, the window, and the desktop's
answers to the questions a host must answer.

## 5. Non-goals

1. **No renderer abstraction.** No trait, no `Backend`, no indirection layer
   for renderers that do not exist. Dioxus RSX *is* the component; a TUI would
   need its own components and would reuse `omriss-ui`, which is precisely why
   `omriss-ui` must stay renderer-free. Inventing the abstraction now would be
   speculative generality, and it would make the code harder to read in
   exchange for a flexibility nobody has asked for.
2. **No web host.** This RFC makes one possible; it does not write one.
3. **No API stability promise** — §4.3.
4. **No behaviour change of any kind.** §4.4.
5. **Not a fix for anything in 0.17.0.** This is 0.18.0 work and must not touch
   the tag (`ROADMAP.md`'s scope freeze).

## 6. What a web host would still have to write

Stated so the estimate is honest rather than encouraging:

| Concern | Desktop today | A web host |
|---|---|---|
| Open / save | `rfd` + `std::fs`, `shell/actions.rs` (289 lines) | HTTP, or the File System Access API |
| Settings | `dirs::config_dir()` + TOML | `localStorage`, or server-side per account |
| Locale | Windows `GetUserPreferredUILanguages`, else env | `navigator.language` |
| External modification | mtime comparison | ETag, revision, or a real conflict protocol |
| Trust boundary | RFC-064, one local user | **§7** |

The components do no IO, which is why this is a port rather than a rewrite.
But the host is where the real work is, and it is not small.

## 7. The threat model does not survive the move — gate, not footnote

**RFC-064 assumes the document is the user's own file, opened from their own
disk.** Every judgement in it rests on that: the preview renders text the user
already had, so escaping protects them from a file they chose to open.

In a web service that assumption is simply false. Documents arrive from other
users, from a shared store, from an upload. The preview stops being a
local-trust question and becomes a **stored-XSS boundary between tenants**, and
the same escaping that is sufficient locally is necessary-but-not-sufficient
there.

Concretely, before any web host renders a document it did not get from the
person looking at it:

1. `preview.rs` is re-audited against stored XSS, not local trust. RFC-064's
   evidence does not transfer, because its threat model does not.
2. A real Content-Security-Policy is served by the host. RFC-064 §4.3's policy
   was written around dioxus-desktop's loopback WebSocket and does not apply.
3. **RFC-064 §9a changes severity.** A remote image in a preview is a tracking
   pixel; locally it tells a sender when the recipient opened their own file. In
   a multi-user service it tells one user when another read their document —
   from an application that promises in `PRIVACY.md` that it collects nothing.
4. `PRIVACY.md` is rewritten for the host. The current document describes a
   local application and would be false for a service in almost every section.

**No web product ships against this RFC's extraction without that audit.** The
extraction is safe; what the extraction makes easy is not automatically safe,
and the easiest mistake available after this RFC lands is to assume the
security work came along with the components.

## 8. Two things to correct regardless

### 8.1 `omriss-ui`'s description is false on a public registry

*"View-state logic, i18n catalogs, and GUI components for Omriss"* — the crate
has never contained a component and cannot, since it has no renderer
dependency. It is published and reads that way today.

One line in `crates/ui/Cargo.toml`, published with the next release of that
crate. Suggested: *"Renderer-independent view state, session bridges and i18n
catalogs for Omriss"*.

### 8.2 RFC-062's open questions are now due

RFC-062 §6.6 and its open questions 4 and 6 left crates.io deprecation and the
fate of the published `omriss` package to be decided before a release. The
registry state on 2026-11-01:

| Package | crates.io |
|---|---|
| `omriss` 0.16.0 | **published — described as the engine**, the role RFC-062 moved to `omriss-core` |
| `omriss-core` | **not published** |
| `omriss-ui` 0.16.0 | published, with §8.1's description |

So `cargo add omriss` today installs the old engine, under the name that now
belongs to the desktop application — which is `publish = false` and will
therefore never supersede it. RFC-062 anticipated exactly this ("Crates.io
expectation risk"); what is missing is the decision, and 0.17.0 is the release
that makes the mismatch permanent.

This is a release question, not an architecture one, and it belongs to the
0.17.0 pre-tag items rather than to this RFC. It is recorded here because this
RFC is what would add a fourth name to that family.

## 9. Validation and test plan

1. `git diff` for the component move shows **moves only**. State it as a
   command and its output.
2. `cargo test --workspace` passes with the same count as before the move, or
   the difference is explained test by test.
3. `cargo test` (default members, no `--workspace`) now **compiles the
   components**, which it cannot today: `crates/app` is outside
   `default-members` because it needs the platform WebView (RFC-001). The new
   crate does not.

   **Corrected 2026-11-03, after the implementation measured it:** an earlier
   draft called this "a genuine coverage gain". It is a *compile* gain, not a
   test gain — `omriss-components` contains **zero tests**, by design
   (`TESTING.md` defines no component-level harness), and a move adds none. The
   default-members count is 494 before and 494 after. What is actually gained
   is that `cargo build`, `cargo test` and `clippy` on a machine with no WebKit
   now cover all 23 components, where before they were reachable only with the
   platform libraries installed. Worth having; not worth overstating.
4. `cargo build -p omriss-components --target wasm32-unknown-unknown` succeeds. This
   is the acceptance test for the whole RFC: if it fails, a platform dependency
   survived the split, and the boundary is not where this RFC says it is.
5. The desktop application behaves identically. No smoke re-run is required by
   the move itself, but the next scheduled run must pass unchanged.
6. `scripts/check-rfcs.sh`, `cargo clippy -- -D warnings`, `cargo fmt --check`.

### 9.4 Two things this RFC did not predict

**The workspace `dioxus` entry had to change.** It pinned
`dioxus = { version = "0.7", features = ["desktop"] }`, and a workspace
dependency can only *add* features — so every crate taking
`{ workspace = true }`, the new one included, would have pulled in
`dioxus-desktop` and the wasm build would have failed. `desktop` moved to
`crates/app`'s own dependency line. The host's feature set is unchanged. This
is build configuration, not code, and it is the only change outside "moves and
import paths" that the extraction required.

**Seven functions became `pub`.** They were `pub(crate)` because caller and
callee shared a crate; the host now calls them across a crate boundary.
Visibility is the only non-import change in the moved files, and the boundary
forces it — which is, in a small way, the boundary doing its job: the crate's
public surface is now exactly what the host actually uses.

### 9.3 One thing to decide during implementation

`input/keyboard.rs` maps `keyboard-types` codes to commands. If it is
renderer-independent it belongs in `omriss-ui`; if it encodes desktop key
conventions it stays in the host. **Read it and decide with a reason** rather
than moving it because it is nearby.

## 10. Acceptance criteria

1. The components build for `wasm32-unknown-unknown` (§9.4).
2. `omriss-ui` still has exactly one dependency.
3. No component imports `crate::` anything.
4. The move is byte-identical and the suite is unchanged (§9.1, §9.2).
5. `omriss-components` is `publish = false` with the reason recorded in its manifest,
   unless the owner decides otherwise (§4.3).
6. The name is the owner's decision (§4.2), recorded here in RFC-052 §14.4's
   form before implementation begins.
7. §7's threat-model statement is linked from RFC-064, so a future web host
   cannot reach the extraction without meeting the audit.

## 11. Decisions — all resolved, 2026-11-03

1. **The crate name** — `omriss-components` (§4.2).
2. **Published or not** — `publish = false` (§4.3).
3. **Whether to do this at all if the web product is not pursued** — yes. It is
   item 1 of 0.18.0's scope, taken for the compiler-enforced boundary, the
   components entering the default `cargo test`, and the smaller host crate,
   not for a web product that may never be built.

Nothing in this RFC is waiting on a decision.
