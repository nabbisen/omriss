# RFC-075 — Task Breakdown and PR Plan

Related RFC: `../../done/075-component-crate-extraction-and-renderer-portability.md`

Lifecycle status: inherited from RFC-075.

Four PRs. The split exists so that the large mechanical diff arrives on its own,
with nothing else in it to hide behind.

---

## PR 1 — Create the crate, move `draft_sync`

**Why first:** it is the only coupling (§3 of the handoff), it is 100 lines, and
moving it alone proves the new crate compiles and is wired into the workspace
before 24 files land on top.

- `crates/components/` with `Cargo.toml`: `publish = false`, the description
  verbatim from the handoff §2, dependencies `omriss-ui`, `dioxus`,
  `dioxus-swdir-tree`, `keyboard-types`.
- Add it to `[workspace] members`, and — deliberately — to
  **`default-members`**, which `crates/app` cannot join because it needs the
  platform WebView. This is the coverage gain, and it should be visible in this
  PR rather than discovered later.
- `git mv crates/app/src/shell/draft_sync.rs crates/components/src/`.
- Fix the three importing call sites.

**Checks:** all four gates. `cargo test` *without* `--workspace` now compiles
the new crate; say so in the PR, with the before and after.

## PR 2 — Move the components

The large one: 24 files, `git mv` and import rewrites only.

**Checks:** all four gates, plus the §5.2 grep showing no `crate::` import
reaches outside `omriss-components`, plus **`cargo build -p omriss-components
--target wasm32-unknown-unknown`**.

State the test count before and after. It must not change — nothing here adds
or removes a test.

## PR 3 — `input/keyboard.rs`, decided by construction

Answer handoff §5.1 and act on the answer. If it moves to `omriss-ui`, the
constraint that `omriss-ui` keeps exactly one dependency still holds —
`keyboard-types` would become its second, so **that is a reason to leave it in
the host** unless the reading says otherwise. Give the reasoning either way.

A PR whose diff is empty and whose body is the reasoning is an acceptable
outcome here.

## PR 4 — Documentation

`PLATFORMS.md`, `README.md` and the mdBook wherever they describe the crate
layout, plus RFC-075's Status line recording what landed.

Small, and last, so the documentation describes what exists rather than what
was planned.

---

## Ordering notes

PRs 1 and 2 are strictly sequential. PR 3 is independent of PR 2 and may be
done in either order. PR 4 is last.

**Do not fold PR 2 into any other.** A 24-file move reviewed alongside anything
else is a 24-file move nobody reviewed.
