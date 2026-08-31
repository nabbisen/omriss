# RFC-068: Durable File Writes and Path Integrity

**Project:** omriss — Omriss Editor
**Milestone:** M13 — durability hardening (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-015, RFC-016, RFC-018
**Related RFCs:** RFC-035, RFC-036, RFC-039

---

## 1. Summary

The save path is documented as satisfying NFR-REL-003 "atomic". The rename is
atomic; nothing around it is. Saving can lose data on power loss, silently break
a symlinked file, widen file permissions, and — on a non-UTF-8 path — crash the
app at startup before a file is ever opened.

For a product whose single differentiating promise is that it does not damage
your file, the write path deserves the same rigour as the edit path. It does not
currently have it.

**Source of this RFC:** AUDIT-0170-006, -022, -023, -024, and -038.

## 2. The defects

### 2.1 Not crash-safe

`crates/app/src/file/file_dialog.rs:84-103`. No `sync_all` on the temp file and
no directory fsync, so after a power loss the rename can be visible while the
data is not. The file is then empty or truncated — the outcome the atomic-rename
pattern exists to prevent.

### 2.2 Breaks symlinks

`fs::rename` onto a symlinked path **replaces the link with a regular file**. A
vault, a dotfile repository, or a synced folder silently stops receiving edits,
and the user has no way to notice from inside omriss.

### 2.3 Drops permissions

The temp file is created with the default mode, so a file the user set to `0600`
becomes world-readable after save — and is briefly world-readable at a
predictable path *during* it.

### 2.4 Leaks and collides

A failed rename leaves `.tmp.omriss` behind. `with_extension` produces the same
temp name for `notes.md` and `notes.txt` in one directory, so two saves can
collide.

### 2.5 Panics on non-UTF-8 argv

`std::env::args()` panics on non-UTF-8 argv, so launching from a file manager on
a file whose name is not valid UTF-8 crashes at startup. Paths are then carried
as `String` via `path.display()`, which is lossy — a path that survived open
would be **saved to a different path**.

### 2.6 External-modification detection is weak

Detection is `disk_mtime > saved_mtime` only. Equal-second timestamps, and any
tool restoring an older mtime (`git checkout`, `rsync --times`, `touch -d`),
defeat it; omriss then overwrites without asking. The dialog it does show offers
Overwrite / Save As / Cancel — **no Reload**, which RFC-016 §4 requires.

### 2.7 Settings are written non-atomically

`storage/settings.rs:76-87` uses a plain `fs::write`. A crash mid-write leaves
truncated TOML, which `load()` silently discards, resetting every preference
including recent files with no notice. The document path is more careful than
this one; both should use the same primitive.

### 2.8 No size bound on open

`fs::read` loads the whole file, then `text_bytes.to_vec()` copies it again, so
peak memory is ~2× the file — on the GUI thread, with no progress or cancel.
RFC-052 already defines a `TooLarge` error kind that nothing produces.

## 3. Design

One hardened primitive, used everywhere a file is written:

```rust
fn write_atomic(target: &Path, bytes: &[u8]) -> io::Result<()>
```

- `canonicalize` the target first, so a symlink is followed and the link
  survives;
- unique temp name in the target's directory;
- copy the target's permissions onto the temp file on Unix;
- `write_all` → `sync_all` → `rename` → fsync the directory;
- remove the temp file on any failure path.

Then: `args_os()` at startup, and `PathBuf`/`OsString` carried through
`StartupArg`, `OpenOutcome` and `file_name`, with the lossy string kept for
presentation only. `(len, mtime)` for modification detection with a content
re-read on a tie, and a Reload choice in the dialog. A documented size ceiling
producing RFC-052's `TooLarge`.

## 4. Non-goals

1. **No journalling or crash recovery of unsaved buffers.** This RFC makes a
   completed save durable; it does not add autosave.
2. **No change to the source-preservation path.** Bytes handed to
   `write_atomic` are already correct; this is about getting them onto disk.
3. **No Windows ACL work.** Permission preservation is Unix-only; Windows keeps
   default inheritance and this is stated rather than silently skipped.

## 5. Validation and test plan

- `write_atomic` preserves mode `0600` on Unix.
- Saving through a symlink leaves the symlink intact and updates the target.
- A simulated rename failure leaves no temp file behind.
- Two files with the same stem and different extensions save concurrently
  without collision.
- Non-UTF-8 path: open, edit, save, and confirm the bytes landed at the original
  path. Requires a fixture path constructed with `OsString` from raw bytes; on
  platforms where that is not constructible the test is skipped explicitly, not
  silently.
- Settings survive a truncated-write simulation.

## 6. Acceptance criteria

1. All writes route through `write_atomic`, documents and settings alike.
2. Symlinks, permissions and durability covered by tests.
3. No panic reachable from argv; paths round-trip losslessly.
4. Reload offered on external modification, per RFC-016 §4.
5. `TooLarge` produced by a documented ceiling.
6. NFR-REL-003's "atomic" claim is true of the implementation, or the claim is
   narrowed to what is actually guaranteed.
