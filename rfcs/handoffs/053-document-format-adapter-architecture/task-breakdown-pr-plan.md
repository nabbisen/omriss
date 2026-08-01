# RFC-053 Task Breakdown / PR Plan

Five slices, strictly ordered. Each slice is one PR unless noted. A slice is not
started until the previous one is green and reviewed.

Read `implementation-handoff.md` §5 (non-change scope) before the first commit.

---

## S1 — `DocumentFormat` and detection

**New:** `crates/core/src/formats.rs` + `crates/core/src/formats/detection.rs`

```rust
pub enum DocumentFormat { Markdown, Json, Toml, YamlExperimental, PlainText, Unsupported }
pub enum DetectionConfidence { No, Maybe, Likely, Certain }

pub fn detect_format(path: Option<&Path>, text: &str) -> DocumentFormat;
pub fn confidence_for(format: DocumentFormat, path: Option<&Path>, text: &str) -> DetectionConfidence;
```

Extension mapping is **RFC-052 §5.1** and is binding:

```text
md, markdown, mdown, txt -> Markdown          (shipped behavior; must not regress)
json                     -> Json
toml                     -> Toml
yaml, yml                -> YamlExperimental
(anything else)          -> PlainText
```

`path` is `Option` because an unsaved buffer has no path.

**Not in this slice:** wiring detection into the session or file dialog. S1 adds
the capability and its tests; nothing calls it yet.

**Tests:** one case per extension above; unknown extension; no path.

**Done when:** `cargo test -p omriss-core` covers every row of the mapping and
the workspace suite is otherwise unchanged.

---

## S2 — Structure vocabulary in `omriss-core`

**New:** `crates/core/src/formats/structure.rs` (split further if it exceeds
300 ELOC)

Per RFC-053 §6, using shipped types — `NodeId`, `ByteRange`, `DocumentRevision`:

```rust
pub struct DocumentStructure { format, root_id, nodes, revision }
pub struct StructureNode { id, parent_id, title, kind, depth,
                           source_range, editable_range, children, capabilities }
pub enum StructureNodeKind { DocumentRoot, MarkdownSection, Group, List, Value,
                             RawRegion, Unsupported }
pub struct NodeCapabilities { /* twelve Capability fields, RFC-053 §6 */ }
pub enum Capability { Allowed, Disabled { reason: CapabilityReason }, Hidden }
pub enum CapabilityReason { /* eight variants, RFC-053 §6 */ }
```

Note `Capability::Disabled` is a **struct variant** (`{ reason }`), whereas the
shipped `omriss-ui` `MapCapability::Disabled` is a tuple variant. RFC-053 is the
authority; the struct form wins. Call-site churn from this lands in S3.

Move `sibling_info` capability computation logic here from
`crates/ui/src/editor/navigation.rs` **only if** it operates purely on
`omriss_core::Outline`. It does today — verify before moving.

**Not in this slice:** deleting anything from `omriss-ui`. S2 is purely
additive; the duplication is temporary and resolved in S3.

> **Execution note (added after the S3 review).** This section scopes itself to
> RFC-053 §6, so it did not create `DraftState` (§9.3), even though §14's
> reconciliation table assigns that type to `omriss-core` too. `DraftState`
> was therefore created in **S3**, ported verbatim. The gap was in this
> document, not in either slice.

**Tests:** capability computation for root, first child, last child, only child,
and a deep node — the cases the existing `document_map_tests` cover, now
asserted at core level.

---

## S3 — Reconcile `omriss-ui` onto the S2 types

**The entire point of this slice is that nothing changes for the user.**

Remove from `crates/ui/src/interface/document_map.rs`:

- `MapCapability` → use `omriss_core::Capability`
- `MapNodeCapabilities` → use `omriss_core::NodeCapabilities`
- `CapabilityReason` → use `omriss_core::CapabilityReason`
- `DraftState` → use `omriss_core::DraftState` (created in this slice; see the
  S2 execution note above)

Keep in `omriss-ui`:

- `DocumentMapNode` — view projection, per RFC-053 §14
- `node_id_from_raw`
- **the catalog-key mapping.** `CapabilityReason::catalog_key()` currently lives
  on the type. The type moves to core; the method must not. Reimplement it in
  `omriss-ui` as a free function or a local trait impl. `omriss-core` must never
  name a catalog key.

Rewrite `crates/ui/src/session/document_map_bridge.rs` so `document_map_nodes()`
projects from the core structure rather than computing capabilities itself.

Update call sites in `crates/app/src/components/document_map_pane.rs` for the
tuple → struct variant change (`MapCapability::Disabled(r)` →
`Capability::Disabled { reason }`).

**Suggested commit split inside the PR:**

1. mechanical variant-shape change;
2. type replacement;
3. bridge rewrite;
4. catalog-key mapping relocation.

**Tests:** `document_map_tests` and `i18n_tests` pass with **assertions
unchanged in meaning**. Mechanical renames of a type in an assertion are fine;
a changed expected string is not.

**Done when:** `grep -rn "MapCapability\|MapNodeCapabilities" crates/ui crates/app`
returns nothing, all eight catalog keys are unchanged, and the suite is green.

---

## S4 — `MarkdownAdapter`

**New:** `crates/core/src/formats/markdown.rs`

Implement `DocumentFormatAdapter` for Markdown by **wrapping** shipped
operations. The mapping is fixed by RFC-053 §9.2:

| `StructureCommand` | Shipped implementation |
|---|---|
| `Move { InsidePrevious }` | demote (RFC-023) |
| `Move { OutOneLevel }` | promote (RFC-023) |
| `Move { Up \| Down }` | section move via `MoveTarget` (RFC-024) |
| `JoinWithPrevious` | merge (RFC-025) |
| `AddInside` / `AddAfter` / `Rename` / `Delete` | shipped section operations |

`build_structure` projects the shipped `Outline` into `DocumentStructure`,
preserving existing `NodeId` values — **do not re-derive ids** (§5.3 of the
handoff).

Add `ActiveAdapter` (RFC-053 §7.2) with only the `Markdown` and `PlainText`
variants populated; the others may exist and return "unsupported".

**Tests:** the complete existing suite, unchanged, plus RFC-053 §13.3:

- rebuild determinism: same source → same ids;
- focus survival: edit an unrelated node, rebuild, previously focused id still
  resolves to the same logical item.

**Done when:** all 239 pre-existing tests pass **unmodified**, and the new
identity tests pass.

> **Optional split (added after the S3 review).** S4 may be delivered as one
> slice or as two, at the implementer's discretion. If splitting, this is the
> seam worth cutting on — it isolates risk rather than merely halving the diff:
>
> - **S4a — projection.** `build_structure` mapping the shipped `Outline` into
>   `DocumentStructure`, preserving existing `NodeId` values, plus the RFC-053
>   §13.3 identity tests. This half mutates nothing, so criterion 5 is trivially
>   satisfiable and the identity contract is proven on its own.
> - **S4b — mutation.** The `StructureCommand` wrapping of the shipped
>   RFC-023/024/025 operations, plus `ActiveAdapter`. All of the regression risk
>   lives here, and reviewing it against an already-proven projection makes
>   "did this wrap or reimplement?" a sharper question.
>
> Either shape is acceptable. No new task file is required for S4 or for a
> split of it: Developer Task 004 covers all five slices, and the
> one-review-per-slice rule applies to whatever units are actually submitted.

---

## S5 — `PlainTextAdapter` and parse-failure recovery

**New:** `crates/core/src/formats/plain_text.rs`

Per RFC-052 §5.2: a single node, **no synthetic structure**, all editing
capabilities `Hidden`, `can_show_plain_text: Allowed`.

Demonstrate the parse-failure path **at the adapter boundary**: when a format
adapter's `build_structure` fails, falling back to `PlainTextAdapter` yields a
viewable single-node structure with `can_show_plain_text: Allowed`. Source text
is preserved structurally — `build_structure` takes `&str` and mutates nothing,
so a failure cannot damage the document.

> **Scope clarification (added before S5 was dispatched).** An earlier draft of
> this section said "the session keeps the source text and surfaces the
> plain-file-text recovery route." That overstates S5's scope: nothing wires
> `ActiveAdapter` into the session until RFC-054, and S5 must not become the
> slice that does it — that would make the last core slice user-visible and
> pull RFC-054's work forward.
>
> S5 therefore proves the **mechanism** in `omriss-core`: failure returns a
> typed error without mutation, and `PlainTextAdapter` provides the fallback
> structure a recovery path would render. Criterion 7's end-to-end behavior —
> the session actually choosing that fallback and the UI offering
> "Show plain file text" — lands with RFC-054's wiring. No catalog keys, no
> `crates/ui` or `crates/app` changes.

**Tests:**

- malformed input → source text intact, recovery offered, no panic;
- `PlainText` → exactly one node, editing capabilities `Hidden`;
- deep nesting does not overflow the stack (RFC-053 §18).

**Done when:** RFC-053 §19 criteria 7 and 8 are demonstrable, and RFC-054 has a
boundary to build against.

---

## Sequencing summary

```text
task 002 (refactor + clippy gate)
  └─ S1 detection
       └─ S2 core vocabulary
            └─ S3 ui reconciliation      ← zero user-visible change
                 └─ S4 MarkdownAdapter   ← highest regression risk
                      └─ S5 PlainText + recovery
                           └─ RFC-054 may begin
```
