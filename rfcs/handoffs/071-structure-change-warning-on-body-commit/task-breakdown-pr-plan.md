# RFC-071 — Task Breakdown and PR Plan

Related RFC: `../../proposed/071-structure-change-warning-on-body-commit.md`

Lifecycle status: inherited from RFC-071.

Three PRs. The first belongs to a different RFC and is listed here because this
item cannot start without it.

---

## PR 1 — `t_args`, to RFC-073 §4.1

**Credited to RFC-073, not to this RFC.** Interpolation is RFC-073's design;
RFC-071 is its first consumer. Record it in RFC-073's Status so item 9 of
0.18.0 reduces to §4.3.

- `t_args(locale, key, args) -> String` in `omriss-ui`, named placeholders.
- `t()` unchanged — ~100 existing static keys keep using it.
- **The placeholder-parity test**: for every key, the placeholder *set* is
  identical across `en` and `ja`. A `ja` entry missing `{count}` would silently
  drop the number, and no key-name parity test can see that.

**Checks:** gates. State the new test count.

## PR 2 — Detection in `omriss-ui`

The §2.2 rule: expected delta per operation versus actual.

- One function that expresses the rule, so the detector and RFC-066's P2 cannot
  drift apart.
- The §6 enumeration: every source-rewriting entry point routed through it,
  listed in the PR with any exception justified.
- Unit tests in `omriss-ui` — no WebView, no GUI.

**Checks:** gates, plus the enumeration as a table.

Test cases at minimum, each asserting *whether* it warns and *which* shape:

| Case | Expect |
|---|---|
| body commit writing `<!--` above a heading | warn, "no longer headings", count 1 |
| body commit writing `# Foo` | warn, "new sections", count 1 |
| ordinary body commit | **silent** |
| `delete_section` of a 3-node subtree | **silent** — intended |
| `move_section` that changes nothing structurally | **silent** |
| `move_section` of a section whose body holds an unterminated `<?` (RFC-066 §3.1a's case) | warn |
| `split_section` adding exactly one node | **silent** |

The two "intended" rows are the ones that make the feature usable; they deserve
tests as much as the warning cases do.

## PR 3 — The message, in `omriss-components`

- The three shapes, the §4.3 cause sentence, `Ctrl+Z` named.
- Catalog keys in `en` and `ja` with identical placeholder sets.
- Non-modal status surface.

**Checks:** gates, plus one GUI pass under the GUI rules showing the message
appearing for a disappearance case and **not** appearing for an ordinary edit.
The second is the one worth capturing.

---

## Ordering

Strictly sequential: PR 2 needs PR 1's `t_args` only at the message layer, so
it *could* precede it — but the catalog keys are written in PR 1's idiom, and
splitting that costs more than it saves. Do them in order.
