# RFC-064: Preview HTML Sanitization and WebView Trust Boundary

**Project:** omriss — Omriss Editor
**Milestone:** M12 hardening — 0.17.0 ship gate
**Status.** Implemented (main, unreleased) — H1 (escaping), H2 (scheme
checking) and H4 (documentation) landed and close the finding. **H3 (the CSP) is
deliberately not wired**, deferred to 0.18.0 for cross-platform verification —
§4.3 records why the originally-specified policy was wrong and §9 why the
corrected one waits. `crates/app/src/shell/csp.rs` holds it, tested and
dead-coded, in the meantime.
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, security reviewer, QA engineer
**Depends on:** RFC-045
**Related RFCs:** RFC-010, RFC-017, RFC-033, RFC-039, RFC-042

---

## 1. Summary

The Markdown preview renders author-supplied HTML verbatim into the
application's own WebView, with no Content-Security-Policy and no scheme
checking on links. Opening someone else's Markdown file and pressing Preview
executes their JavaScript inside omriss.

This RFC closes that, and — more importantly — writes down a trust boundary the
project has never named. Sixty-three RFCs discuss preserving the user's bytes;
none asks what those bytes are allowed to *do* when rendered.

**Source of this RFC:** AUDIT-0170-001, reproduced independently by the
architect before acceptance.

> **If omriss is ever hosted on the web, this RFC's threat model does not
> transfer (added 2026-11-01).** Everything here assumes the document is the
> user's own file, opened from their own disk, so escaping protects a person
> from a file they chose to open. A web service renders documents written by
> *other* users, which makes the preview a stored-XSS boundary between tenants
> and turns §9a's remote image from a local annoyance into one user learning
> when another read their document. **RFC-075 §7** states the audit any such
> host must pass first; it is a gate, not a recommendation.

## 2. The defect

`render_html` passes `Event::Html` and `Event::InlineHtml` through unchanged
(`crates/core/src/doc/preview.rs:165-168`), and the result is handed to Dioxus
`dangerous_inner_html` (`crates/app/src/components/preview_pane.rs:51`). The
window is built with no CSP (`crates/app/src/shell/app.rs:39-44`). Link
destinations are HTML-escaped but never scheme-checked.

Executed against `omriss_core::section_html`, verbatim:

```text
in   <img src=x onerror="alert(1)">
out  <img src=x onerror="alert(1)">

in   [c](javascript:alert(1))
out  <p><a href="javascript:alert(1)">c</a></p>
```

Both round-trip unescaped.

### Why this matters more than a normal XSS

A browser tab's XSS is bounded by an origin. This is not a browser tab. The
WebView is the application, so script running in it shares the process with the
document the user has open — it can read the open document out of the DOM and
exfiltrate it, overlay a convincing dialog on the app's own chrome, or navigate
the view. This is the only defect in the 0.17.0 audit whose blast radius extends
beyond the file being edited.

The delivery path is ordinary: a shared note, a README, a downloaded template.
Markdown is a format people exchange, and omriss's entire premise is opening
files other people wrote.

## 3. Non-goals

1. **Not a sandbox.** The WebView keeps full-trust access to the app; this RFC
   does not attempt process isolation.
2. **Not a change to what is stored.** Raw HTML stays in the user's source text
   byte-for-byte. This RFC changes only what the *preview* renders. Source
   preservation is untouched and must remain so.
3. **No HTML sanitizer dependency.** Escaping is a smaller, more auditable
   answer than allow-listing tags, and matches what the preview is for.

## 4. Design

Three independent layers. Layer 1 alone closes the finding; 2 and 3 are defence
in depth and cost nothing.

### 4.1 Escape authored HTML in the preview

```rust
Event::Html(raw) | Event::InlineHtml(raw) => {
    // Show authored HTML as text. Rendering it would execute it: the
    // preview is injected into the app's own WebView.
    out.push_str(&html_escape(&raw));
}
```

This is a deliberate product decision, not only a security one: the preview is a
**reading aid for structure**, not a browser. A user who wants their HTML
rendered as HTML has the source view and an external tool. Showing the markup as
text is also more honest about what omriss is.

### 4.2 Scheme-check link and image destinations

Allow `http`, `https`, `mailto`, and relative paths in the `Tag::Link` and
`Tag::Image` arms. Drop the attribute otherwise, keeping the link text. `data:`
is excluded deliberately — `data:text/html` is a script vector.

### 4.3 Content-Security-Policy on the WebView

**The policy below is corrected. The original was wrong and would have shipped a
blank window.** It read:

```text
default-src 'self'; script-src 'self'; img-src 'self' data:; connect-src 'none'
```

justified as costing nothing "since the app makes no network requests by
design". That claim is true of *omriss* and false of the framework it runs on,
which is what matters to a CSP. Verified in `dioxus-desktop` 0.7.9's own source:

- **`connect-src 'none'` blocks every UI update.** `edits.rs` streams all
  mutations to the page over a loopback websocket —
  `TcpListener::bind((IpAddr::from([127, 0, 0, 1]), 0))`, an ephemeral port that
  can change mid-session. Blocked, the page loads and then never receives a
  single edit. The crate's own doc comment says it plainly: *"Using websockets
  does mean we need to handle security and content security policies
  ourselves."*
- **`script-src 'self'` blocks the interop bridge.** `protocol.rs`'s
  `module_loader` injects an inline `<script type="module">` whose content
  varies per launch (it embeds the edits path and connection key), which rules
  out a hash allowlist. `dioxus-desktop` 0.7 exposes no nonce for it.

Reproduced empirically as well as traced: the literal policy renders a blank
window; loosening `script-src`/`style-src` alone still renders blank, isolating
`connect-src` as sufficient on its own to break the app.

The corrected policy:

```text
default-src 'self'; script-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src ws://127.0.0.1:*
```

**Corrected again, 2026-11-03: `style-src 'self' 'unsafe-inline'` is missing
and must be added.**

```text
default-src 'self'; script-src 'self' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src ws://127.0.0.1:*
```

The first correction fixed the two blockers the evidence showed — the loopback
websocket and the framework's templated module script — and did not enumerate
*everything inline* the page emits. `shell/app.rs:281` renders
`style { {STYLE} }`, the whole of `assets/style.css` as an inline `<style>`
element. Under `default-src 'self'` with no `style-src`, that is blocked, and
the application comes up **unstyled**. Same failure class as the blank window,
found the same way: by reading what the page actually contains rather than what
the policy was written against.

§8 says a styling conflict is escalated, never papered over with
`'unsafe-inline'`. It is escalated here, and the architect's answer is to allow
it, for a reason that only became true after the first correction: **once
`script-src` concedes `'unsafe-inline'` — and it must, because the framework
templates its loader per launch and a hash cannot be computed — a nonce on our
own `<style>` would be theatre.** An attacker who can inject script does not
need to inject style. Serving the CSS as an asset instead would be the clean
alternative and is worth doing on its own merits one day; it is not a
prerequisite for the protections this policy actually delivers.

Those protections, stated plainly so nobody mistakes what is left:
`img-src 'self' data:` stops remote images, which is the §9a tracking pixel;
`connect-src` confines the page to the loopback websocket; `default-src 'self'`
stops frames, objects and fonts from elsewhere. Inline script and style are
conceded to the framework, and the policy is still worth wiring.

`connect-src` remains the load-bearing clause. Narrowed to the loopback
websocket scheme and host, it still turns an injected `fetch('https://…')` into
a console error — which is the exfiltration path that matters — while permitting
the framework's own channel. `'unsafe-inline'` on `script-src` is a real loss of
the backstop against event-handler attributes, and is accepted because §4.1's
escaping, not the CSP, is the actual fix; the CSP is defence in depth on top of
it.

## 5. Validation and test plan

- `section_html` and `document_html` never emit `<script`, `onerror=`,
  `onmouseover=`, `<iframe`, or `href="javascript:` for any input. Table-driven,
  with the four vectors from §2 plus `data:text/html` and `srcdoc`.
- Escaped HTML still round-trips: the source text is unchanged after a preview
  render, asserted byte-for-byte.
- The existing preview tests continue to pass, and any that assert raw HTML
  passthrough are **scaffolding** by the RFC-054 handoff's definition — their
  premise is what this RFC invalidates. Flag them in the review request.
- A fuzz target over `render_html` once RFC-066 lands.

## 6. Acceptance criteria

1. No authored HTML from document source reaches the WebView unescaped.
2. No `javascript:` or `data:` destination survives into rendered output.
3. A CSP is set on the window, and its absence fails a test. **Deferred to
   0.18.0** — see §9.
4. Source text is byte-identical across preview rendering.
5. `SECURITY.md` exists before this RFC is discussed anywhere public.

## 7. Open question for the owner

Escaping HTML is a **user-visible behaviour change**: a document that today
renders an embedded `<table>` or `<details>` in preview will show the markup as
text instead. That is the correct default for a local-first editor, and it is a
regression for anyone relying on it.

If preserving rendered HTML matters, the follow-up is an allow-list of inert
tags (`table`, `details`, `summary`, `sup`, `sub`, `br`) with all attributes
stripped — strictly more work and strictly more risk. **The recommendation is
escaping.** Ship it, note it in `known-limitations.md`, and revisit only with a
concrete user need.

## 9b. Verified on Windows, 2026-11-03

0.17.0's Windows reduced run executed check I against
`crates/core/tests/fixtures/html_content.md`. Its HTML comment, `<em>` and
`<div>` block all displayed as **visible text**, so §4.1's escaping is active
under WebView2, not only under WebKitGTK.

**What it did not exercise.** That fixture carries no `<img onerror=…>` and no
`javascript:` link, so the window-title detector the dev team used on Linux —
which proves no *script* executed — was not run on Windows. The conclusion
still holds, and the reason it holds is worth stating rather than assuming:
both constructs travel the same escaping path in `preview.rs`, so markup that
reaches the user as text cannot have reached the engine as markup. The `<em>`
result is evidence about the escaping, not about one tag.

A future Windows run can close the gap outright by adding a payload file of its
own. `html_content.md` itself must not be changed for that purpose: it is
`include_str!`-ed by `source_preservation.rs:35` and asserted by
`fixture_catalog.rs:250`.

## 9c. H3 is abandoned — the CSP route is a liability, not a control (2026-11-04)

H3 was attempted under 0.18.0's scope and **stopped**, correctly, under the
handoff's §7. What the attempt measured changes the decision rather than
delaying it.

### What it found

Two failure modes beyond the two anyone had named — and neither is visible by
looking at the window.

**The application renders and accepts no input.** `connect-src` governs
`XMLHttpRequest`, and dioxus-desktop sends *every UI event* by a synchronous
XHR to `<base>/__events`. The policy allowed the edit-streaming websocket, so
the window draws — pixel-identical to before, byte-identical screenshots — and
then silently ignores every click and keystroke. Violation observed:
`connect-src dioxus://index.html//__events`.

**`document::eval` is `eval`.** `dioxus-desktop/src/query.rs:79-80` runs
`new AsyncFunction("dioxus", script)`, which CSP classifies as `eval`. omriss
calls it at **eleven sites in five files**. Without `'unsafe-eval'`, measured by
A/B on the delete-confirmation dialog: focus lands on **nothing** instead of
Cancel — silently undoing Task 019's safest-default and every focus behaviour
RFC-060 measured.

### Why this is not a "fix the policy and try again"

A working policy would need `script-src 'unsafe-inline' 'unsafe-eval'` and
`connect-src 'self' ws://127.0.0.1:*`. Three things then hold at once:

1. **`script-src` constrains nothing.** Inline and eval both conceded is the
   whole of what that directive had to offer.
2. **The policy encodes framework internals that differ per platform.**
   `protocol.rs` uses `dioxus://index.html/` on Linux and macOS and
   `http://dioxus.index.html/` on Windows. An allowance correct on one is wrong
   on another, and `__events` is an implementation detail of one dioxus
   version.
3. **The failure mode is silent, catastrophic and invisible to CI.** A dead
   window looks identical to a live one. There is no automated GUI test that
   could catch it, so a dioxus upgrade would reintroduce it and we would learn
   from a user who cannot type.

A control that constrains nothing in its headline directive, depends on
undocumented per-platform framework internals, and fails by disabling the
application without a symptom is not defence in depth. **H3 is withdrawn.**

The finding this RFC exists for stays closed by **H1** (escaping) and **H2**
(the scheme allowlist), exactly as §9 said: *H1 alone closes this finding*.

### What replaces it

§9a's tracking pixel is the one concrete gap the CSP would have closed, and it
does not need a CSP. The same control belongs in `preview.rs` beside H2, where
it is a unit test rather than a per-platform GUI run: **H4 — drop remote image
destinations in the preview**, proposed in §9d.

`crates/app/src/shell/csp.rs` stays in the tree, unwired, with its six tests
and its doc comment recording both violations verbatim. It is the record of a
route that was tried and measured, which is worth more than a deleted file.

## 9d. H4 — remote images are not fetched until the reader asks (0.19.0)

**Amended 2026-11-04**, after the owner asked whether the first version of this
section matched the project's design principles. It did not, and the test is
worth recording because the failure is a common one.

### The version that was rejected, and why

The first proposal was simply to drop remote image destinations in
`preview.rs`, beside H2's scheme allowlist. It is safe, robust, cross-platform
and a unit test. It also fails on two counts:

**It misleads.** A user opens *their own* notes containing
`![diagram](https://…)` and sees nothing — no image, no gap, no reason. They
would conclude the file is wrong, or the URL is broken, or omriss is broken,
and the application would have declined to do something and said nothing about
it. A `known-limitations.md` entry does not repair that: it asks the reader to
find documentation that explains a blank space.

**It is blunt where the problem is narrow.** §9a's threat is *a document
someone else sent you* learning your IP address and the moment you opened it.
The ordinary case is your own document with your own images, where there is no
threat at all. Dropping every remote image punishes the ordinary case to stop
the rare one, because omriss cannot tell them apart — and "cannot distinguish,
therefore refuse everything" is the crude answer, not the considered one.

### The design

What every mail client settled on for this exact threat:

1. **A remote image renders as a visible placeholder** carrying its URL, so the
   absence is a statement the reader can act on rather than a gap they must
   diagnose.
2. **A per-document control — *Load remote images* — off by default.** One
   action and the document renders as written.
3. **Links are untouched.** Following a link is a deliberate act; loading an
   image is not, and that asymmetry is the whole of §9a.

Nothing reaches the network until the reader asks, which is the property §9a
needs; and a previewer that can never show the images in a Markdown document
has stopped being a previewer, which is the capability the first version threw
away.

### Scheduling

This is a design slice — placeholder output in `preview.rs` and a real
affordance in the UI — not an allowlist edit, and 0.18.0 is at capacity.
**Scheduled to 0.19.0.** §9a has been open since 0.17.0 shipped and needs a
hostile document to matter; that does not justify rushing a design the project
would have to apologise for.

0.18.0's item 2 is therefore empty. That is the honest consequence of §9c's
withdrawal, not a slot to backfill.

## 9a. The CSP also has a privacy dimension (recorded 2026-10-21)

Noted while reviewing `PRIVACY.md` for the Store listing. §4.2's scheme check
allows `http` and `https` for `Tag::Image`, so a Markdown image with a remote
address renders as `<img src="https://…">` and the WebView fetches it on
preview. Verified:

```text
in   ![logo](https://example.com/tracker.png)
out  <p><img src="https://example.com/tracker.png" alt="">logo</p>
```

That is a tracking pixel: a document from someone else learns the reader's IP
address and when they opened it. It is not a security hole — no code executes,
and raw HTML `<img>` is escaped by §4.1 — but it is the only path by which
viewing a document reaches the network, and it contradicts the obvious reading
of "local-first".

**`img-src 'self' data:`, already in §4.3's corrected policy, blocks it.** So
wiring the CSP is not only defence in depth over an already-closed finding; it
also closes a live privacy gap. That strengthens the case for doing it in
0.18.0, and the three-platform verification requirement in §9 is unchanged.

`PRIVACY.md` §4 documents the behaviour and its mitigation in the meantime,
because the policy has to be true of the shipped build.

## 9. H3 deferred to 0.18.0

The corrected §4.3 policy has been verified on **Linux only**. WebKitGTK,
WKWebView, and WebView2 differ in how they honour a `<meta>` CSP, and the
failure mode when one of them disagrees is a blank window — not a degraded
style, the whole application.

0.17.0 ships to the Microsoft Store, and its Windows smoke run has not yet been
performed. Wiring an unverifiable CSP into that release trades a catastrophic,
platform-specific regression risk against defence in depth that §4 already says
is not the fix: **H1 alone closes this finding**, and H1 and H2 are verified.

So `crates/app/src/shell/csp.rs` holds the corrected policy, documented and
unit-tested, marked `#[allow(dead_code)]`, and `main.rs` does not apply it. It
gets wired in 0.18.0, when it can be exercised on all three platforms in the
same smoke run that would catch it breaking.
