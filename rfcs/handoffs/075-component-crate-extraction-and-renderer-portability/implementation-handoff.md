# RFC-075 — Implementation Handoff

Related RFC: `../../proposed/075-component-crate-extraction-and-renderer-portability.md`

Lifecycle status: inherited from RFC-075.

---

## 1. What this is, in one sentence

Move the 24 Dioxus component files out of the application crate into
`omriss-components`, so that a boundary omriss already respects by discipline
is enforced by the compiler.

**It is a move.** If the diff contains logic changes, something has gone wrong.

## 2. The target shape

```text
omriss-core        no UI, no renderer        pulldown-cmark only
omriss-ui          no renderer               omriss-core only
omriss-components  Dioxus components         omriss-ui + dioxus + tree widget + keyboard-types
omriss             the desktop host          + dioxus-desktop, rfd, dirs, tao/wry
```

`crates/components/`, `publish = false`, and this description verbatim — the
field answers the `ui`-versus-`components` question at the point it arises, and
RFC-075 §4.2 explains why that sentence is load-bearing:

```toml
description = "Dioxus components for Omriss, built on omriss-ui's renderer-independent view state"
```

**`omriss-ui` must still have exactly one dependency when you are done.** If
`dioxus` has appeared in it, the extraction went the wrong way.

## 3. Why this is less work than it looks

Measured before the RFC was written, not assumed:

- the 24 files import only `omriss_ui` (35), `dioxus` (22), `omriss_core` (4),
  `dioxus_swdir_tree` (4), `keyboard_types` (1), and `crate::shell::draft_sync`
  (3);
- across all of them there is **no `std::fs`, no `SystemTime`, no `rfd`, no
  `dirs`, no `std::env`, and not one `#[cfg(...)]`**;
- four of them reach the DOM through `dioxus::document::eval`, and all of that
  is ordinary DOM — no dioxus-desktop IPC, no desktop global.

So the only real coupling is the three imports of `draft_sync`, a 100-line
module whose own imports are `dioxus`, `omriss_core` and `omriss_ui`. It moves
with the components because it is renderer-independent logic that happens to
live under `shell/`.

## 4. The move discipline

1. **`git mv`, not copy-and-edit.** The review asks for a diff that is moves
   plus import paths, and nothing else.
2. **No renames, no reordering, no "while we are here".** A refactor that
   changes behaviour while claiming to be a move is the single way this work
   could hurt, and it would be invisible in a diff this large.
3. **`rustfmt` only.** If formatting changes beyond what `cargo fmt` does to a
   moved file, say which file and why.
4. The import rewrites are mechanical: `crate::components::X` becomes
   `omriss_components::X` in the host, and `crate::shell::draft_sync` becomes a
   sibling path inside the new crate.

## 5. Two questions to answer by construction

Not by sampling. The RFC-067 §3.2 lesson — enumerate every caller, do not list
the ones you happened to touch — applies to both.

**5.1 `input/keyboard.rs`.** It maps `keyboard-types` codes to commands. If it
is renderer-independent it belongs in `omriss-ui`; if it encodes desktop key
conventions it stays in the host. Read it and decide **with a reason**, rather
than moving it because it is nearby. Either answer is acceptable; an unexamined
answer is not.

**5.2 Anything else reaching back into `crate::`.** After the move, no file in
`omriss-components` may import `crate::` anything outside its own crate. Prove
it with a grep in the review request, not with a claim.

## 6. What stays in `omriss`

`main.rs`, the shell wiring (`app.rs`, `app_ctx.rs`, `dispatch.rs`,
`actions.rs`), `file/`, `storage/`, `cli.rs`, `locale.rs`, `csp.rs`. That is
the host: platform IO, platform configuration, the window, and the desktop's
answers to the questions a host must answer.

## 7. The acceptance test that matters

```sh
cargo build -p omriss-components --target wasm32-unknown-unknown
```

If this fails, a platform dependency survived the split and the boundary is not
where this RFC says it is. It is the difference between a boundary that is
claimed and one that is demonstrated.

The target is installed on the maintainer's machine; `omriss-core`,
`omriss-ui`, `dioxus-swdir-tree` (with `default-features = false`) and
`keyboard-types` were each built against it unchanged on 2026-11-01.

## 8. Out of scope

- **No web host.** This makes one possible; it does not write one.
- **No renderer abstraction** — no trait, no `Backend`, no indirection for
  renderers that do not exist (RFC-075 §5).
- **No publishing.** `publish = false`, decided.
- **No behaviour change of any kind.**
