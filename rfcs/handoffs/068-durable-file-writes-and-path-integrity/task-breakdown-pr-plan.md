# RFC-068 — Task Breakdown and PR Plan

Related RFC: `../../proposed/068-durable-file-writes-and-path-integrity.md`

Lifecycle status: inherited from RFC-068.

Constraints for every PR:
- `omriss-core` unchanged.
- No new dependency without saying why. A random suffix can come from the
  process id plus a counter and time; a hash for §7.6 can use std's hasher,
  because this is change detection, not security.
- Messages through RFC-073's typed failures, in both locales.
- `docs/src/` and `known-limitations.md` change in the same PR as the behaviour.
- CHANGELOG `Fixed` per user-visible change.

## PR 1 — one durable write (§7.1–§7.4, §7.9)

- The primitive; documents and settings both use it; the old `write_atomic`
  and `settings.rs`'s `fs::write` go.
- Tests (Unix where marked):
  - mode `0600` is kept;
  - through a symlink, the link is intact and the target updated;
  - a forced failure at each step leaves no temp file;
  - same stem, different extension: no collision;
  - an existing file at the temp name is never overwritten;
  - a truncated settings write cannot happen: the old settings file is
    intact until the rename.
- A source-scanning guard: no `fs::write`, `File::create` or `fs::rename` in
  the application crates outside the primitive.

**PR 1 landed** (`a69e43c`).

## PR 2 — paths, losslessly (§7.5)

- First, RFC-068 §8.2: a save whose directory sync fails is saved with a
  warning, a typed outcome with words in both locales. Test it with the
  injected directory-sync failure: the document is marked saved and the status
  is the warning, not a failure.

- `args_os()`; `PathBuf` through to the save target; a display string only for
  display.
- Test, Unix: a path built from non-UTF-8 bytes opens, saves to the same path,
  and is not added to the recent list. Other platforms skip it explicitly.
- No panic reachable from argv: a test feeds non-UTF-8 argv to the CLI
  interpreter.

**PR 2 landed** (`e4165ad`).

## PR 3 — external changes and Reload (§7.6–§7.7)

- Content-hash detection; the `mtime` comparison goes.
- The dialog's Reload: not the default; reload is one undo step and marks the
  document saved; words in both locales.
- Tests:
  - an external change with the **same** mtime and size is detected;
  - an mtime restored to an older value is detected;
  - after Reload, Ctrl+Z restores the unsaved text, and saving it asks again.
- GUI, once: change the file from a shell, save in omriss, choose Reload, undo.

**PR 3 landed** (`571b047`).

## PR 4 — the size ceiling: measure, then stop (§7.8)

- First, RFC-068 §10: Overwrite and Discard drawn as `btn-danger`, never
  primary; a level-2 test over every dialog that no choice that can lose text
  is primary.

- Measure opening and one edit at 10, 50 and 100 MB, Markdown and JSON, release
  build: time and peak RSS. Remove the double copy on open first, and measure
  after that.
- **Stop and report the numbers.** The architect sets the ceiling; then the
  `TooLarge` path and its message go in the same PR.

Gates, each PR: fmt, clippy `-D warnings` with `--locked`, `cargo test
--workspace --locked` and `cargo test` with both counts, `check-rfcs`,
`mdbook build docs`, `check-workflows.sh`.

**PR 4, first part, landed** (`bf9c661` the dialog rule, `118a111` no double
copy). The ceiling, by RFC-068 §11:

1. **Measure first:** plain file text at 100 MB in the GUI (open, scroll,
   commit one edit). If it is not usable, stop and report.
2. The byte ceiling: `OpenFailure::TooLarge { limit }`, checked before reading,
   with words in both locales.
3. The structure ceiling in core: the scanner counts and stops; `TooLarge`;
   a public constant. The UI's sentence states the constant. The file opens in
   plain file text.
4. Tests:
   - a file one byte over the byte ceiling is refused before any read (a
     sparse file is fine);
   - a JSON of 1,000,001 tiny values opens in plain file text with the
     sentence, and peak memory stays bounded (measure it);
   - Markdown is unaffected.
5. Docs (known limitations, file formats) and the CHANGELOG.

`omriss-core` changes for item 3 only.
