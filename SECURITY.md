# Security Policy

## Reporting a Vulnerability

**Please do not open a public issue for a security problem.**

Report privately through GitHub's [private vulnerability
reporting](https://github.com/nabbisen/omriss/security/advisories/new) for this
repository. That channel is monitored and lets us discuss a fix before any
detail becomes public.

Please include what you have: the version or commit, the platform, what an
attacker gains, and the smallest input or steps that show it. A file that
reproduces the problem is worth more than a description of one.

You should get an acknowledgement within a week. omriss is maintained by one
person, so a fix may take longer than an acknowledgement — you will be told
which is happening.

## Supported Versions

Only the latest release receives security fixes. omriss is pre-1.0 and moves
forward rather than backporting.

## Threat Model

omriss is a local desktop editor. It makes no network requests, has no server
component, no plugin system, no telemetry, and no account. It reads and writes
files you point it at.

That leaves one thing genuinely worth attacking: **omriss opens files other
people wrote.** Markdown and JSON are exchange formats — a shared note, a
README, a downloaded template, a repository you cloned. A document is untrusted
input, and anything a document can make the application do is in scope.

In scope:

- a document causing code execution, or reading or exfiltrating other data,
  when opened, previewed, or edited;
- a document causing omriss to write outside the file being edited;
- **loss or silent corruption of a user's file** — the guarantee the product
  exists to make, and we treat a preservation failure as a security issue even
  though no attacker is involved;
- a crash reachable from document content or from a file path.

Out of scope:

- anything requiring an attacker who already has code execution as your user;
- unsigned builds triggering SmartScreen or Gatekeeper warnings — this is
  documented in `RELEASE_CHECKLIST.md`, not a vulnerability;
- vulnerabilities in the platform WebView (WebKitGTK, WKWebView, WebView2)
  itself; report those upstream, though please still tell us if omriss's use of
  the WebView makes one reachable.

## Dependencies

`cargo audit` findings against transitive dependencies are tracked, but an
advisory in the tree is not by itself a vulnerability in omriss — reachability
matters. If you can show a path from omriss's own code to an advisory, that is
very much worth reporting.
