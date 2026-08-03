# RFC-063 Handoff

Companion execution package for:

- RFC: `rfcs/proposed/063-command-line-file-argument.md`
- Sequenced ahead of RFC-054 J7

## What this work is

`omriss <path>` opens that file, through the load path the Recent Files list
already uses.

## Why it comes before J7

J7 is the largest user-visible slice in RFC-054 and the one whose rendering
claim most needs evidence. Every rendering check since J3 has depended on
synthetic input reaching the WebView — which failed outright once and worked
once. This makes that check depend on a real product capability instead.

## Size

One slice. Most of the machinery exists: `file_dialog::open_markdown_path`
already takes a path, checks existence, and returns an `OpenOutcome`; `app.rs`
already calls it for Recent Files. The change is where the path comes from.

Contents:

- `implementation-handoff.md`
- `acceptance-qa-checklist.md`
