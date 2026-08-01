# RFC-053 Implementation Handoff

**Governing RFC:** [RFC-053](../../done/053-document-format-adapter-architecture.md)
**Milestone:** M11 — Format Adapter Foundation
**Supersedes:** the RFC-053 stub, whose own checklist ended with
"Replace this stub before RFC-053 implementation begins."
**Prerequisite:** dev-team task 002 (file split + clippy gate) should land first.
It touches `document_map_pane.rs` and `structural.rs`, which S3 and S4 also
touch; landing it first keeps both diffs readable.

---

## 1. Purpose

Introduce the document format adapter boundary in `omriss-core` so that
JSON (RFC-054), TOML (RFC-055), and the YAML spike (RFC-056) can attach to the
shipped Document Map / Writing Area interface without format-specific logic
leaking into the Dioxus layer.

**No new user-facing format ships in this work.** Markdown behavior must be
byte-for-byte identical when it is done.

## 2. Background

M10 shipped part of RFC-053's vocabulary into `omriss-ui` before RFC-053
existed — RFC-048 listed "RFC-053 type core" as a dependency and implementation
proceeded anyway. As a result `omriss-ui` currently owns capability types that
RFC-053 assigns to `omriss-core`, and the English/Japanese catalogs already
carry all eight `CapabilityReason` variants, including strings for formats that
do not exist yet.

Reconciling that is **part of this work, not new feature work**.

## 3. Applicable requirements

- RFC-053 in full. It is the type authority; where anything disagrees with it,
  it wins.
- RFC-052 §5.1 (binding extension mapping), §5.2 (`PlainText` / `Unsupported`
  policy), §14.4 (per-format visibility, owner-decided).
- RFC-001 crate boundaries. RFC-033 render boundary. RFC-043 i18n.
- App Requirements v0.4.0 §11, External Design v0.4.0 §8.

## 4. Slices

Five slices, in order. Each is one reviewable unit; each must be green before
the next begins. Per-PR detail is in `task-breakdown-pr-plan.md`.

| Slice | Content | Proves |
|---|---|---|
| S1 | `DocumentFormat` + `formats::detection` | files classify correctly; nothing else changes |
| S2 | Structure vocabulary in `omriss-core` | the types exist and are testable without Dioxus |
| S3 | Reconcile `omriss-ui` onto the S2 types | **zero** user-visible change |
| S4 | `MarkdownAdapter` wrapping shipped `Document` ops | Markdown behavior unchanged |
| S5 | `PlainTextAdapter` + parse-failure recovery | safe failure works end to end |

**S4 is the risk concentration point.** It must wrap the shipped RFC-023
(promote/demote), RFC-024 (move), and RFC-025 (split/merge/delete)
implementations. It must not reimplement them.

## 5. Non-change scope — read before starting

1. **No behavior change of any kind is authorized by S1–S5.** No new UI control,
   no new label, no new catalog key, no changed message.
2. **Do not change any i18n catalog key.** The eight `capability.disabled.*`
   keys stay exactly as they are. `omriss-ui` keeps sole responsibility for
   mapping a typed reason to localized text (RFC-043).
3. **Do not change Markdown node-id assignment.** RFC-053 §13.2 is explicit:
   the outline builder's existing scheme stays. RFC-023/024/025 focus-restoration
   tests depend on it.
4. **Do not move `DocumentMapNode` to `omriss-core`.** RFC-053 §14 keeps it in
   `omriss-ui`: it is a view projection that nests children by value and carries
   `is_selected`, which is session state, not document structure.
5. **Do not move `CapabilityReason::catalog_key()` into `omriss-core`.** The
   reason type moves; the catalog mapping must not. `omriss-core` must never
   know a catalog key — that would invert RFC-001. Reimplement the mapping as a
   function or trait impl inside `omriss-ui`.
6. **Do not implement JSON, TOML, or YAML.** Not even a stub parser. The enum
   variants may exist; the adapters must return "unsupported" until RFC-054+.
7. **Do not add `.json` / `.toml` to any file-dialog filter.** Detection may
   classify them; the open dialog must not offer them until RFC-054/055.
8. **Do not introduce `SourceText` or `LineEnding`.** RFC-053 §5.1 withdrew both.
9. **Do not introduce a global "experimental formats" setting.** RFC-052 §14.4
   decided per-format visibility instead.
10. No `mod.rs`. Files over 500 ELOC get split.

If any of these appears necessary, **stop and file a clarification request.**

## 6. Required design constraints

**Adapters own no state.** They take `&str` and return derived data, or take
`&mut Document` to apply an edit. They never own canonical text.

**Mutation goes through `Document`.** RFC-053 §7.1: an adapter handed a raw
mutable buffer could rewrite text without recording an undo entry or
incrementing the revision. Every mutation path must go through the shipped
replacement API so history and revision update as one unit.

**Dispatch is a closed enum** (`ActiveAdapter`), not trait objects. RFC-053 §7.2.

**Detection is free functions** in `omriss_core::formats::detection`, not a
trait method. RFC-053 §10.

**Reuse, never redefine:** `NodeId`, `ByteRange`, `DocumentRevision`. Redefining
`ByteRange` in particular would discard UTF-8 boundary validation.

## 7. Required tests

Each slice adds tests; no slice may modify an existing test's meaning.

- S1: one detection case per extension in RFC-052 §5.1, including `.mdown` and
  `.txt` → Markdown, and an unknown extension → `PlainText`.
- S2: construction and capability-computation unit tests, running under
  `cargo test -p omriss-core` with no Dioxus in the dependency tree.
- S3: the existing `document_map_tests` and `i18n_tests` must pass. If an
  assertion must change in **meaning**, the slice is wrong.
- S4: the full existing suite, unchanged. Plus RFC-053 §13.3 identity tests:
  rebuild determinism, and focus survival across an unrelated edit.
- S5: malformed input keeps source text intact and offers plain file text;
  `PlainText` yields a single node with editing capabilities `Hidden`.

**Baseline to reproduce exactly** (recorded at `52f20b9`, 2026-07-30):

```text
omriss (app unit)                    11
omriss-core (lib unit)               55
fixture_catalog                      20
source_preservation                   9
structural_ops                       34
structural_ops_delete_split_merge    18
structural_ops_move_ops               6
structural_ops_promote_demote         9
structural_ops_revision_guard         1
omriss-ui (lib unit)                 74
doc-tests (core, ui)                1+1
TOTAL                               239
```

Counts may only **grow**, by the new tests each slice adds. A count that drops,
or a suite that changes name, is a defect in the slice.

## 8. Acceptance criteria

RFC-053 §19 is the binding list — eleven criteria, each naming its evidence.
Reproduce that table in the review request with each row marked met or not met.

Criterion 5 is blocking: **if any shipped Markdown test requires modification to
pass, the slice is wrong and the test is not the thing to change.**

Gates, all required per slice:

```sh
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check-rfcs.sh
```

## 9. Documentation updates

- `docs/src/architecture.md`: after S4, describe the adapter boundary and the
  `omriss-ui` → `omriss-core` capability move. Not before — the page must
  describe what is true.
- `CHANGELOG.md` `[Unreleased]`: one entry per slice, each stating explicitly
  that there is no user-facing change.
- Do **not** update `docs/src/known-limitations.md` or write anything implying
  JSON/TOML support exists.

## 10. Known risks

| Risk | Severity | Mitigation |
|---|---|---|
| S3 silently changes a disabled-action reason, so a row menu shows different text | **High** — user-visible, easy to miss | §5.2; `i18n_tests`; diff catalog keys explicitly |
| S4 reimplements a structural op instead of wrapping it, reintroducing a byte-preservation bug | **High** | §4; golden tests; criterion 5 |
| Markdown node ids change, breaking focus restoration | High | §5.3; RFC-053 §13.3 tests |
| `Capability::Disabled` changes from tuple to struct variant, touching many call sites | Medium | mechanical; keep it in its own commit within S3 |
| Adapter mutates text outside `Document`, losing undo | **High** | §6; review every `&mut` in the diff |
| Scope creep into JSON/TOML | Medium | §5.6; `git diff --stat` review |

## 11. Required review-request content

Per organization workflow §9.2, plus, for every slice:

1. the RFC-053 §19 criteria table with per-row status;
2. per-suite test counts against the §7 baseline;
3. clippy output showing exit 0 under `-D warnings`;
4. an explicit statement: **"no user-visible behavior changed, and here is the
   evidence."** For S3 this is the entire point of the slice;
5. for S4, confirmation that each structural operation *wraps* the shipped
   implementation, with the call sites named.

## 12. Escalation triggers

Stop and ask the architect if:

- a shipped test must change to pass;
- capability computation cannot move to `omriss-core` without `omriss-core`
  depending on `omriss-ui`;
- Markdown node identity must change;
- `Document`'s API cannot express an adapter mutation without a new public
  method — propose it, do not add it unilaterally;
- a slice cannot be completed without touching another slice's scope.
