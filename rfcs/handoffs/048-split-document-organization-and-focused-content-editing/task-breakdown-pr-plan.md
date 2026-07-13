# RFC-048 Task Breakdown / PR Plan

## PR 0 - Reconciliation and Design Lock

Goal: make sure implementation starts from the amended design, not from stale
pre-review assumptions.

Tasks:

- Read RFC-048 through RFC-051 and RFC-053 sections 5, 6, 8, and 9.3.
- Read the committed RFC text after the design-review amendments. If available
  on this machine, also read local artifacts
  `.git-exclude/reviewed/rfc-048-design-review.md` and
  `.git-exclude/reviewed/rfc-048-handoff-review.md`.
- Audit current dirty files for stale assumptions:
  - `[Done]` or primary Done workflow;
  - `Commit -> Done`;
  - `FocusedNodeKind`;
  - structure controls inside the Writing Area;
  - Markdown-only names in generic UI boundaries.
- Decide, file by file, whether the current dirty implementation is kept,
  amended, or replaced.
- Record any remaining design conflict before coding.

Exit criteria:

- No stale RFC-048 blocker remains in planned code.
- Developer confirms the implementation unit is the M10 bundle, not RFC-048
  alone.
- RFC-049, RFC-050, and RFC-051 handoffs exist before PR 1 opens.
- RFC-052 through RFC-056 handoffs exist or are explicitly stubbed with
  RFC-053 canonical names before PR 1 opens.
- No code path treats existing spike behavior as more authoritative than RFCs.

Recommended PR shape:

- Documentation or small code cleanup only.
- No behavioral broadening beyond reconciliation.

## PR 1 - Layout and Naming Boundary

Goal: establish the two-zone UI boundary with stable names.

Tasks:

- Ensure the left panel component is `DocumentMapPane` or an equivalent
  Document Map boundary.
- Ensure the right panel component is `FocusedContentPane` internally and
  presents Markdown as `Writing Area`.
- Remove obsolete `Outline` / `Focus Editor` naming where it describes the new
  primary panels.
- Update i18n labels in English and Japanese.
- Update command labels:
  - `Raw Markdown` -> `Show plain file text`;
  - `Command Palette` -> `Quick Actions` in normal UI.
- Keep internal naming format-neutral where low-risk.

Exit criteria:

- UI text and docs use `Document Map` and Markdown `Writing Area` consistently.
- Internal right-panel boundary is not named as a Markdown-only section editor.
- No JSON/TOML/YAML functionality is introduced.

Suggested checks:

- `cargo fmt`
- `cargo check -p omriss-app`
- `bash scripts/check-rfcs.sh`

## PR 2 - Document Map Owns Structure Operations

Goal: move visible structure organization to the Document Map.

Tasks:

- Render current section/node highlight in the Document Map.
- Provide structure actions from the Document Map:
  - add inside;
  - add after;
  - rename;
  - move up/down;
  - move inside previous / out one level;
  - join with previous;
  - delete;
  - show plain file text.
- Put advanced/destructive actions behind row menus or equivalent disclosure.
- Hide unsupported actions, or show disabled state with plain explanation.
- Ensure destructive operations use confirmation with `Cancel` as safest
  default.
- Keep keyboard access to row menus and actions.

Exit criteria:

- No visible structural operation remains in the Writing Area.
- All existing Markdown structural operations remain reachable from the
  Document Map or compatible command path.
- Delete and other destructive actions are confirmable and undo-safe.

Suggested checks:

- `cargo test -p omriss`
- `cargo test -p omriss-ui`
- `cargo check -p omriss-app`

## PR 3 - Writing Area Draft Lifecycle

Goal: make the right side a focused content editor with apply-on-navigation.

Tasks:

- Remove the primary `Done` or `Commit` workflow.
- Keep focused Markdown body drafts local while typing.
- Apply valid pending draft on:
  - Document Map selection change;
  - Save / Ctrl+S;
  - Preview;
  - search/navigation actions;
  - blur, if supported safely.
- Ensure Ctrl+S applies the pending draft before saving.
- Ensure save/status text reflects a single user-visible dirty state:
  - `Saved`;
  - `Not saved yet`;
  - `Saving...`;
  - `Saved ✓`;
  - friendly save failure text.
- Keep preview focused on selected Markdown content.

Exit criteria:

- No primary Done button is visible.
- Navigating between sections does not lose typed content.
- Saving after a draft edit writes the latest focused content.
- Existing source-preserving body edit behavior is unchanged.

Suggested checks:

- `cargo test -p omriss`
- `cargo test -p omriss-ui`
- `cargo check -p omriss-app`

## PR 4 - Accessibility, Keyboard, and Safety

Goal: make the migrated workflow usable by keyboard and assistive technologies.

Tasks:

- Preserve or add landmarks:
  - toolbar/header;
  - `aside` or equivalent labelled `Document Map`;
  - `main` or equivalent labelled `Writing Area` for Markdown;
  - status/footer live region.
- Verify tab order:
  - toolbar;
  - Document Map;
  - Writing Area;
  - status/recovery actions.
- Ensure focus restoration after add, delete, rename, and move.
- Ensure Esc closes overlays before changing document focus.
- Ensure live status messages are polite except direct action failures.
- Ensure error messages avoid byte ranges, node IDs, parser internals, and raw
  system errors.

Exit criteria:

- Keyboard-only manual scenario succeeds.
- Screen-reader labels are plain and meaningful.
- Dialog focus does not disappear.

Suggested checks:

- component tests where available;
- manual keyboard QA from `acceptance-qa-checklist.md`.

## PR 5 - Documentation, Release Notes, and Final Gate

Goal: align public docs and final verification with the migrated UI.

Tasks:

- Update README only where concise overview terminology changes.
- Update mdbook docs:
  - getting started;
  - editing/history;
  - structural editing;
  - keyboard reference;
  - architecture;
  - known limitations;
  - working in layers.
- Update CHANGELOG with the UI role separation.
- Add a note that structured plain-text support is future work.
- Run the full acceptance checklist.
- Record non-technical-user QA result when available.

Exit criteria:

- Docs no longer describe structure controls in the Writing Area.
- Docs distinguish `Document Map`, Markdown `Writing Area`, `Quick Actions`,
  and `Show plain file text`.
- Full gate commands are observed after final changes.

Suggested checks:

- `cargo fmt`
- `cargo test -p omriss -p omriss-ui`
- `cargo check -p omriss-app`
- `mdbook build docs`
- `bash scripts/check-rfcs.sh`
- `git diff --check`

## PR Ordering Notes

- PR 0 is mandatory if the current dirty implementation remains in the
  worktree.
- PR 1 and PR 2 may be combined only if the diff remains small and reviewable.
- PR 3 should not be mixed with unrelated docs churn; draft lifecycle failures
  are high-risk.
- PR 4 should happen before docs are finalized, because accessibility labels may
  change wording.
- PR 5 is the release-readiness gate, not a place to add new behavior.
