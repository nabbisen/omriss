# Privacy Policy

**Effective:** 2026-10-21

## 1. Summary

omriss is a local-first desktop editor. It has no account, no telemetry, no
analytics, no crash reporting, no advertising, and no update check. It sends
nothing about you or your documents anywhere.

There is one exception, and it is the only one: previewing a Markdown document
that links to a remote image will load that image. See §4.

## 2. Information we collect

**None.** omriss collects no personal information, and transmits no information
to us or to any third party. There is no server to send it to.

## 3. Information stored on your device

omriss stores two things locally, and only locally:

- **Your preferences** — font size and line wrap.
- **Your recently opened files** — their full paths, so the list can be shown
  when you reopen the app. These paths may include your user name and your
  document names.

Both live in a single plain-text file on your own device:

| Platform | Location |
|---|---|
| Windows | `%APPDATA%\omriss\settings.toml` |
| Linux | `~/.config/omriss/settings.toml` |
| macOS | `~/Library/Application Support/omriss/settings.toml` |

You can delete that file at any time. omriss will start again with its defaults
and an empty recent-files list. Nothing is kept anywhere else.

Documents you edit are read from, and written to, exactly the paths you choose.
omriss never copies them elsewhere and never scans your filesystem.

## 4. Remote images in Markdown preview

If a Markdown document contains an image with an `http` or `https` address —
`![alt](https://example.com/image.png)` — then opening the **preview** for that
document makes your computer request that image from that server. The server
will see your IP address and the fact that something requested the image. This
is how any Markdown preview works, but you should know it is the one case where
viewing a document reaches the network.

This matters for documents you did not write. A document from someone else can
use a remote image to learn when you opened it.

What omriss does *not* do:

- HTML written directly in a document is never rendered, only shown as text, so
  it cannot load anything.
- Link and image addresses using schemes other than `http`, `https`, `mailto`
  or a relative path are discarded.
- The plain file text view never loads anything remote.

If this concerns you, do not open the preview for documents you do not trust;
the editor and the plain file text view are unaffected.

## 5. File and system access

omriss requests the `runFullTrust` capability on Windows because it is an
ordinary desktop application that reads and writes files you select. All file
handling happens on your device.

omriss displays its interface using the web-view component supplied by your
operating system — WebView2 on Windows, WebKit on Linux, WKWebView on macOS.
That component is part of your operating system and is governed by your OS
vendor's own privacy terms, not by this policy.

## 6. Children's privacy

omriss collects nothing from anyone, including children.

## 7. Contact

For questions about this policy, or anything else, please open an issue at
<https://github.com/nabbisen/omriss/issues>.

To report a security vulnerability, please use the private channel described in
[SECURITY.md](SECURITY.md) rather than a public issue.

## 8. Changes to this policy

If this policy changes, the revised version will be published in this
repository and the **Effective** date above will be updated. Material changes
will also be noted in `CHANGELOG.md`.
