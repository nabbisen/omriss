# Handoff: RFC-064 H3 — Wire the Content-Security-Policy

Related RFC: `../../done/064-preview-html-sanitization-and-webview-trust-boundary.md`

Lifecycle status: inherited from RFC-064.

**Item 2 of 0.18.0's scope.** H1 and H2 shipped in 0.17.0; H3 is the deferred
defence-in-depth, and §9's reason for deferring it has expired.

Files:

- `implementation-handoff.md` — what is wired, the three things in the tree
  that contradict it today, and the behaviour it changes
- `acceptance-qa-checklist.md` — the gates, and the two failure modes that must
  be looked for rather than waited for

**Read `implementation-handoff.md` §2 first.** This is not the small wiring job
0.18.0's scope assumed: the constant in the tree is the pre-correction policy,
its tests assert the old text, and one of them asserts the opposite of what the
policy now needs.
