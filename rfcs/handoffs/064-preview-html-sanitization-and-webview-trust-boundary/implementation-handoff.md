# RFC-064 H3 — Implementation Handoff

Related RFC: `../../done/064-preview-html-sanitization-and-webview-trust-boundary.md`

Lifecycle status: inherited from RFC-064.

---

## 1. What H3 is

Apply RFC-064 §4.3's Content-Security-Policy to the application's WebView, via
`Config::with_custom_head` in `crates/app/src/main.rs`.

H1 (escaping raw HTML) and H2 (the scheme allowlist) shipped in 0.17.0 and
close the finding on their own. H3 is defence in depth: it makes an escaping
bypass, if one ever existed, fail against the browser's own policy as well.

**§9's deferral has expired on its own terms.** It waited because "0.17.0's
Windows smoke run has not yet been performed"; it has been. The third platform
§9 worried about is macOS, which the artifact matrix no longer claims.

## 2. Three things in the tree contradict the policy — read before coding

This is not a one-line wiring job, and the scope that called it "small code"
was written before anyone looked at `csp.rs`.

**2.1 The constant is the pre-correction policy.**
`crates/app/src/shell/csp.rs` holds

```text
default-src 'self'; script-src 'self'; img-src 'self' data:; connect-src 'none'
```

which is the text RFC-064 §4.3 was *corrected away from*. The current policy,
after two corrections, is:

```text
default-src 'self'; script-src 'self' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src ws://127.0.0.1:*
```

**2.2 Its tests assert the old policy, and one asserts the opposite of the new
one.** `never_loosens_to_unsafe_inline` asserts the policy does **not** contain
`'unsafe-inline'`. The corrected policy requires it twice.

That test must be **re-aimed, not deleted**. Its intent — `'unsafe-inline'`
never creeps in unnoticed — is still right; what changed is that two specific
concessions are now deliberate. Replace it with a test that pins exactly which
directives may carry it and fails if a third ever does. A deleted guard is how
the next loosening arrives unannounced. (RFC-078 §4 makes the same point about
a different test; the principle is the project's, not that RFC's.)

**2.3 The module's doc comment describes the old blocker** and says the policy
is unwired because `connect-src 'none'` kills the websocket. Rewrite it to
describe what is true: the policy is wired, and inline script and style are
conceded to the framework for the reasons §4.3 now records.

## 3. What it changes for the user

**Remote images in the Markdown preview stop loading.** `img-src 'self' data:`
is the clause that blocks RFC-064 §9a's tracking pixel, and a tracking pixel is
only a remote image with bad intentions — the policy cannot tell them apart.

That is the RFC's intent, not a side effect. But it removes something that
works today, so it needs:

- a `CHANGELOG.md` entry under **Changed**, not buried in Security;
- **`PRIVACY.md` §4 rewritten.** It currently tells every reader that opening a
  preview may fetch remote images and reveal their IP address. After this, that
  is false, and a privacy document that overstates what the application does is
  as misleading as one that understates it;
- an entry in `docs/src/known-limitations.md` — a user whose image does not
  render deserves to find out why, with `Owner: RFC-064` as §9's audit now
  requires of every entry.

## 4. Scope

1. Update `CONTENT_SECURITY_POLICY` in `csp.rs` to §4.3's corrected text.
2. Re-aim the three tests (§2.2).
3. Rewrite the module doc comment (§2.3).
4. Wire it: `Config::new().with_window(window).with_menu(None).with_custom_head(csp::content_security_policy_meta_tag())`
   in `main.rs`, and remove the comment there explaining why it is not wired.
5. Drop the `#[allow(dead_code)]` attributes — the code is now live, and
   leaving them would hide the next thing that falls out of use.
6. The three documentation changes in §3.

## 5. Constraints

1. **No `'unsafe-eval'`, ever.** Two concessions are recorded and reasoned; a
   third is an escalation, not a fix.
2. **Do not add a nonce.** §4.3 explains why it would be theatre alongside
   `script-src 'unsafe-inline'`.
3. **Do not widen `img-src` to make remote images work again.** That is the
   clause the policy exists for. If the owner decides remote images matter more
   than the pixel, that is their call and it changes the RFC, not the code.
4. No change to `omriss-core`'s escaping. H1 is what closes the finding; this
   is the layer behind it.

## 6. The two failure modes, and why they must be looked for

Both are catastrophic and silent in a build log:

| Symptom | Cause |
|---|---|
| **Blank window** | `connect-src` or `script-src` too narrow — the framework's loopback websocket or its templated module script is blocked, and no UI is ever streamed |
| **Unstyled window** | `style-src` missing — `app.rs:281` renders the whole stylesheet as an inline `<style>` element |

A CSP violation appears in the WebView console, not on stderr. Check the
console, do not infer from the absence of an error.

## 7. Escalate rather than decide

If the policy cannot be made to work without a third `'unsafe-*'`, **stop**.
The answer is then to leave H3 unwired and record why — the scope freeze's one
exception exists precisely for this, and unwiring is always permitted.
