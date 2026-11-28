# RFC-070 — Implementation Handoff

Related RFC: `../../done/070-split-section-input-validation.md`

Lifecycle status: inherited from RFC-070.

---

## 1. Build what RFC-070 §4.1 and §8 say

- `validate_title` in `preflight.rs`, called by rename and split.
- The split offset per RFC-070 §9 (not §8.2): in the section, not strictly
  inside any heading line, on a character boundary, not between CR and LF;
  `InvalidSplitOffset` otherwise.
- RFC-066's P4 for rename and join (RFC-070 §8, last paragraph).

## 2. Constraints

1. This is `omriss-core` work. No change to the GUI's behaviour (RFC-070 §5.2).
   If `omriss-ui` must change to keep compiling, say why.
2. **No clamping** (RFC-070 §5.1).
3. Existing `structural_ops*` tests and the property suite pass unmodified,
   apart from removing the `#[ignore]` and its comment.
4. `rename_section`'s behaviour is unchanged. Moving its validation must not
   alter which titles it accepts; a test pins the before and after.
5. CHANGELOG, under `Fixed`, for library users: `split_section` refuses a
   title `rename_section` would refuse, and an offset outside the section's own
   body. Name the change of accepted inputs plainly: name exactly which
   offsets are now refused (RFC-070 §9).
6. No new dependency. Property case counts as the suite already sets them.
