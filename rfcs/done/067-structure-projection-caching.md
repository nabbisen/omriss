# RFC-067: Structure Projection Caching and Commit Cost

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate (conditional, see §7)
**Status.** Implemented (main, unreleased) — C1 (the revision-keyed cache), C3
(RFC-054 §0.4's amendment) and C4 (measured limitations text) landed. Measured
on a 374 KB / 8000-item JSON fixture: per-keystroke `structured_draft_state`
21.1 ms → 4.7 ms, `focus` 10.4 ms → 2.6 ms, which is what §7's ship gate asked
for.

Three deliberate exclusions, each recorded: **C2** (skipping `build_outline` for
non-Markdown) is deferred to 0.18.0 — §3.2's "never read" premise is false and
the correction is recorded inline; **C5** (splice inversion) and **C6** (the
search hot loop) were always 0.18.0. One measurement is honestly short of the
others: `document_map_nodes` on a warm cache is ~9 ms, because the
`DocumentStructure` → `DocumentMapNode` projection is a second layer this RFC
did not cache. That is a candidate follow-up, not a regression.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, QA engineer
**Depends on:** RFC-053, RFC-054
**Related RFCs:** RFC-031, RFC-032, RFC-033, RFC-052

---

## 1. Summary

RFC-054 §0.4's rule — build structure fresh from source, never cache it — is a
sound correctness rule that was applied without a memoisation layer beneath it.
The result is four to five full JSON re-parses **per keystroke**, and another
per render.

This RFC adds the missing layer without weakening the rule, by keying a single
cached `DocumentStructure` on `DocumentRevision`.

**Source of this RFC:** AUDIT-0170-010 and -011.

## 2. Measured cost

Release build, 405 KB JSON, 8000 array items:

```text
document_map_nodes()          11.3 ms  (first)      7.4 ms  (per render)
focus()                        6.9 ms
structured_draft_state()      15.7 ms  per keystroke
                              ─────────
per-keystroke total           ~25 ms at 400 KB → ~250 ms at 4 MB
```

Every render of `StructuredFocusView` calls `focused_structured_content()` (one
parse), then `structured_draft_state()` → `structured_draft_differs()` (one to
two more) → `validate_structured_draft()` (one more); `document_map_nodes()`
adds another. `oninput` writes the draft signal and triggers a re-render, so the
whole set runs per character.

`known-limitations.md` currently says JSON is "re-parsed in full after each
committed edit and each undo… not measured against a large file". The real shape
is worse than that sentence: per keystroke and per render, not per commit. **I
wrote that sentence**, and it under-describes the behaviour; correcting it is
part of this RFC.

## 3. Design

### 3.1 Revision-keyed structure cache

Cache one `DocumentStructure` on `EditorSession`, keyed by `DocumentRevision`.

This preserves what RFC-054 §0.4 actually guarantees. The rule exists to prevent
serving structure built from **stale source**; revisions are monotonic and every
mutation bumps exactly one, so a revision match is a sound proof of freshness —
strictly stronger than "we rebuilt it recently". A draft does not change the
revision, which is why five parses per keystroke collapse to zero.

§0.4 should be amended to state the rule as *never serve structure whose
revision does not match the document's*, which is what it meant.

### 3.2 Skip the Markdown outline for non-Markdown formats

`open_detected` calls `Document::parse` regardless of format, so a JSON document
carries a Markdown outline rebuilt on **every edit**.

> **Correction, 2026-09-08: "never read" was wrong, and §3.2 cannot be
> implemented as written.** `document_map_nodes`, `focus` and
> `prune_dead_history` do branch away from the outline for non-Markdown formats
> — that part is accurate — but they are not the complete list of what reads
> `document.outline()`.
>
> **`outline_items()` reads it unconditionally, on every JSON open.**
> `shell/actions.rs:56` deliberately skips auto-focus for non-Markdown, so a
> JSON file opens in `ViewMode::Outline`; `shell/app.rs:319` branches on
> `ViewMode` with no format guard and renders `OverviewPane`, which calls
> `outline_items()` → `focus_snapshot(root_id())` → `.expect("root always
> exists")` (`session.rs:243`).
>
> Skipping `build_outline`, or returning a degenerate `Outline`, therefore turns
> that `expect` into a **panic on opening a JSON file** — the most basic JSON
> interaction there is. `actions.rs:48-54`'s own comment already said as much
> ("`outline_items`/`focus` both still read the Markdown heading parse"); this
> RFC contradicted a comment that was already in the tree.
>
> The error was method: §3.2 enumerated three readers and concluded "never
> read". The readers should have been found by construction — every caller of
> `document.outline()` — not by listing the ones this RFC happened to touch.
>
> **Deferred to 0.18.0.** Doing it properly means making `OverviewPane` and
> `SearchPanel` format-aware first, which is real scope in two more app
> components, not the "cheap win" §3.2 called it.

### 3.3 Invert the splice instead of copying the document

`apply_replacement` takes a full `String` copy purely as a rollback buffer. It
already knows the range and can invert its own splice on failure. This composes
with AUDIT-0170-018's request for `old_text` in the same place.

### 3.4 Search hot loop

`find_all` allocates a `Vec<char>` **per character comparison** in the innermost
loop; 1.08 MB document, common term, 40 000 matches → 58.7 ms on the UI thread
per query. Lowercase each body once into a reusable buffer and use
`str::match_indices`; hoist the breadcrumb path out of the per-match loop.

Two Unicode caveats to document while there: only the first char of a multi-char
lowercase expansion is compared, and there is no normalisation, so NFC "é" does
not match NFD "é".

## 4. Non-goals

1. **No incremental parsing.** Full re-parse per revision is fine; per keystroke
   is not. Incremental parsing is a much larger change with much larger risk to
   the preservation guarantee.
2. **No caching across documents**, and no cache invalidation policy beyond the
   revision key. One entry, replaced when the revision moves.
3. **No change to RFC-053's adapter statelessness.** Adapters still own no
   state; the cache lives on `EditorSession`.

## 5. Validation and test plan

- A test that mutating the document invalidates the cache: same node, new
  revision, structure reflects the edit.
- A test that a draft edit does **not** invalidate it.
- Re-measure §2's numbers and record before/after in the review request.
- The existing JSON suites pass unmodified — this is a performance change with
  no behavioural difference, and any golden-test change means it is not.

## 6. Acceptance criteria

1. Per-keystroke parses reduced to zero on an unchanged revision.
2. Markdown outline not built for non-Markdown documents.
3. `known-limitations.md` updated with real measured numbers, replacing the
   "not measured" sentence.
4. RFC-054 §0.4 amended to state the revision-freshness rule precisely.
5. RFC-031's thresholds set from these measurements (see RFC-066 §7).

## 7. Ship-gate question for the owner

JSON is 0.17.0's headline feature. At 400 KB it costs ~25 ms per keystroke; at
4 MB, ~250 ms — the latter is not usable, and JSON files of that size are
ordinary.

The fix is roughly twenty lines and low risk. **Recommendation: include §3.1 and
§3.2 in the 0.17.0 gate**, and defer §3.3 and §3.4 to 0.18.0. Shipping the
flagship feature in a state where large real files are unusable is the kind of
overclaim RFC-052 §12 exists to prevent.
