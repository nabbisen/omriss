# RFC-060 — Acceptance and QA Checklist

Related RFC: `../../proposed/060-accessibility-screen-reader-validation-plan.md`

Lifecycle status: inherited from RFC-060.

This checklist operationalises RFC-060 §5's required scenarios. The RFC is
authoritative; where this file is narrower, the RFC wins.

---

## 1. Run record — RFC-060 §4

Every run records, before any finding:

- [ ] app version and commit
- [ ] operating system
- [ ] desktop runtime / WebView version
- [ ] assistive technology used — or `AT-SPI tree only, Orca not run`
- [ ] test fixture or document
- [ ] tester
- [ ] pass / fail / blocked per scenario below

A run missing this block is reference, not validation.

## 2. Trees to capture

Dump and keep the full tree for each state, one file per state:

- [ ] **S1** launch, no file open (Welcome screen)
- [ ] **S2** Markdown open, root focused
- [ ] **S3** Markdown open, a subsection focused, editor entry present
- [ ] **S4** **JSON open** — 0.17.0's headline feature, never examined this way
- [ ] **S5** plain file text view ("Show plain file text")

## 3. The unnamed-control audit

From each tree, list every node whose role is one of

```text
button, entry, combo box, landmark, heading, check box, link, list item
```

**and whose accessible name is empty**, with the state it appeared in.

This is the highest-value single output: an unnamed control is one a
screen-reader user cannot identify at all.

Empty names on `section`, `panel` and `filler` are structural and are **not**
findings.

## 4. Scenario coverage — RFC-060 §5

| RFC-060 §5 scenario | How this run proves it |
|---|---|
| Header/toolbar exposes useful accessible names | S2 tree; every toolbar child named; P-4 resolved |
| Document Map discoverable as the structure region | S2 tree; `[landmark]` present **and named** |
| Writing Area discoverable as the primary editing region | S3 tree; landmark named, `[entry]` named and `editable,focusable` |
| Quick Actions discoverable and dismissible | §5 below |
| Show plain file text discoverable and described as read-only | S5 tree; is read-only conveyed, or only implied by the absence of an editor? |
| Row menus expose useful labels and disabled explanations | S2/S3; open a row menu, capture it; disabled items must carry a reason, not just a disabled state |
| Dialogs announce purpose, labels, primary action, safe cancel | §5 below |
| Status/footer exposed through live regions | §6 below |
| Errors expose no byte ranges, node IDs, parser internals, raw system errors | §7 below |
| Known limitations announced or documented clearly | report whether anything in `known-limitations.md` is reachable only visually |

## 5. Dialogs

For each of: unsaved-changes, error, split, confirm-delete, Quick Actions
palette.

- [ ] appears in the tree with role `dialog` or `alert` **and a name** — we
      author `aria-modal` and `aria-labelledby`; confirm they survive mapping
- [ ] focus is inside the dialog on open (`focused` on a descendant)
- [ ] primary action, cancel path and every field are labelled
- [ ] confirm-delete: the **consequence** is described, not only the button
      named — RFC-060 §6 makes an unexplained destructive action Blocking
- [ ] Quick Actions can be dismissed, and focus goes somewhere identifiable

## 6. Live regions — status and footer

`raw_source.rs` authors `aria-live`. The footer's "Saved" / "Unsaved changes"
is the message a user most needs.

- [ ] minimum: tree before and after a Ctrl+S, showing whether the status text
      is present and changed
- [ ] better: register an `Atspi.EventListener` for `object:text-changed` and
      `object:children-changed` during the save and record which events fire.
      If this proves awkward, say so and give the before/after — do not spend
      long on it

## 7. Error text

- [ ] open a non-UTF-8 file — record the **exact** announced string
- [ ] open a malformed `.json` — same

`file_dialog.rs:67` passes `e.to_string()` straight through, so a raw OS error
is reachable in principle. Confirm whether it reaches the user, and quote it.

## 8. Findings

Classify each per RFC-060 §6 — Blocking / Major / Minor. **The classification
is a proposal; the architect decides.** Anything Blocking triggers the
escalation in the implementation handoff §7 immediately, not at the end.

## 9. Required report content

1. The §1 run record.
2. The five trees as files.
3. The §3 unnamed-control table.
4. P-1 … P-4 confirmed or refuted, each with evidence.
5. Findings classified per §8.
6. **What was not run, or not proven.** It was the most useful section of the
   Task 015 report and it is required here.

No build gates apply: this run changes no code. Running `cargo test` means
something has gone wrong with the measure-don't-fix constraint.
