# RFC-069 — Task Breakdown and PR Plan

Related RFC: `../../proposed/069-draft-lifecycle-and-session-guards.md`

Lifecycle status: inherited from RFC-069.

Three independent PRs, in any order.

---

## PR 1 — §2.3, exact dirtiness

`SavedFingerprint` keeps the revision fast path and compares saved text
exactly. Tests in `omriss-ui`:

| Case | Expect |
|---|---|
| open, no edit | clean |
| edit | dirty |
| edit, undo back to the saved text | **clean** |
| edit, save | clean |
| two different texts of equal length | dirty — the case a digest could miss |

## PR 2 — §2.4, one commit path

`on_navigate` through `commit_or_block`; the source-scanning guard test in
`omriss-components`.

Show that the guard bites: put a direct `commit_focused_body` call back into
`app.rs` and watch the test fail, then restore.

## PR 3 — §2.5, typed requests

`StructureRequest`, the signal, the `use_effect` consumer; the sentinel strings
and the dead split branch removed. Then the GUI check in the handoff §5.

Show by grep that no `"struct.*.pending"` string remains anywhere.
