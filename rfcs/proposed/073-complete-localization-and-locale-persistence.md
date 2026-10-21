# RFC-073: Complete Localization and Locale Persistence

**Project:** omriss — Omriss Editor
**Milestone:** M13 — localization completeness (0.18.0)
**Status.** Proposed
**Document type:** Detailed RFC design
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer
**Depends on:** RFC-036, RFC-043
**Related RFCs:** RFC-039, RFC-053, RFC-060, RFC-071

---

## 1. Summary

omriss ships a complete Japanese catalog with enforced `en`/`ja` parity, and a
Japanese user still meets English in the places that matter most: counts in the
UI chrome, every error cause, and one screen-reader label. A language they pick
by hand is then forgotten on restart.

So "omriss supports Japanese" is not yet true from the user's seat. This RFC
makes it true, and closes the two structural reasons it is not:

1. **No interpolation**, so any message containing a number was written as a
   Rust literal instead of a catalog entry.
2. **Error causes are pre-formatted `String`s**, so the text is decided in the
   file layer where no locale is in scope.

**Source:** AUDIT-0170-026; J7A-IMPL-002; RFC-043's recorded follow-up 3.

## 2. Why this is not a Minor

The audit filed the i18n gap as Minor and that was a reasonable triage against
a release gate. Against the project's stated principle — *APIs and UI/UX for
users not to be confused or misunderstand* — it is not, for one specific
reason:

**A Japanese user hits English exactly when something has gone wrong.** Opening
a file that is not UTF-8, a file that has moved, a document whose structure
cannot be parsed — these are the moments a user most needs to understand the
application, and they are precisely the moments it stops speaking their
language. The catalogs are excellent everywhere except where they matter most.

The same reasoning applies to persistence: a user who finds the language
switcher, uses it, and loses the setting on restart learns that the feature does
not work. That is worse than not offering it.

## 3. The two structural causes

### 3.1 `t()` cannot interpolate

```rust
pub fn t<'key>(locale: Locale, key: &'key str) -> &'key str
```

It returns a borrowed `&str` — deliberately, so a missing key can fall back to
returning the key itself and never render an empty label (RFC-043 §4). Nothing
is wrong with it; it simply cannot produce `"3 subsections"`.

So those strings were written where the number was: as Rust literals.

| Literal | Site |
|---|---|
| `"{count} subsections"` | `components/overview_pane.rs:65` |
| `"{line_count} lines"` | `components/raw_source.rs:23` |
| `" ({child_count} subsections)"` | `components/confirm_delete_dialog.rs:43` |

**Not in scope, and must not be "fixed":** `"{label}"`, `"{title}"`,
`"{query}"`, `"{file}"` and similar in `breadcrumb.rs`, `command_palette.rs`,
`document_map_pane.rs`, `status_bar.rs`. Those render *the user's own content*,
not UI chrome. They are correctly not catalog keys, and routing them through one
would be a regression.

The distinction is the test: **is the English word part of the message, or is
the whole string data?**

### 3.2 Error causes are decided where no locale exists

```rust
pub enum OpenOutcome {
    Failed { cause: String },   // file_dialog.rs:24
}
```

`cause` is built in `crates/app/src/file/`, which has no `Locale` and should not
acquire one. Consequently:

| English text | Source |
|---|---|
| `"File is not valid UTF-8. Omriss requires UTF-8 Markdown files."` | `file_dialog.rs:77` |
| `"File not found: {path_str}"` | `file_dialog.rs:121` |
| `"Could not parse the file structure."` | `shell/actions.rs:78` |
| a raw OS error, via `e.to_string()` | `file_dialog.rs:67` |

The last one is the worst: an untranslated, unstyled operating-system message
shown to the user as an explanation.

**This is a type problem, and the project already solved it twice.**
`CapabilityReason` and `StructureErrorKind` are typed, core-owned reasons that
`omriss-ui` maps to catalog keys at render time, with exhaustive matches so a new
variant is a compile error (RFC-001, RFC-053). `OpenOutcome::Failed` must take
the same shape.

## 4. Design

### 4.1 Interpolation

Add alongside `t()`, not replacing it:

```rust
/// Translates `key` and substitutes named placeholders.
pub fn t_args(locale: Locale, key: &str, args: &[(&str, &str)]) -> String
```

Catalog values use **named** placeholders — `{count}`, not `{}` — so they are
order-independent and self-documenting, and so a translator can move them.

`t()` stays exactly as it is for the ~100 existing static keys. Only messages
that genuinely carry a value use `t_args`.

**Enforced invariant:** the placeholder *set* in every catalog must be identical
across locales for a given key. A `ja` entry that omits `{count}` would silently
drop the number — a defect no parity test on key names can see. `i18n_tests`
gains this check.

### 4.2 Plurals

`"1 subsections"` is the kind of detail that reads as unfinished, so the
mechanism has to handle it, and the smallest thing that does is a two-category
selector:

```rust
pub fn t_plural(locale: Locale, key: &str, n: usize, args: &[(&str, &str)]) -> String
```

Keys carry a `.one` / `.other` suffix:

```text
("overview.subsection_count.one",   "{count} subsection")
("overview.subsection_count.other", "{count} subsections")
```

Japanese has no plural distinction, so `ja` supplies the same text for both.
That is deliberately *not* special-cased: both locales provide both keys, so the
existing key-parity test keeps working unchanged and no locale needs a rule
table.

**No ICU dependency.** `one`/`other` covers `en` and `ja`. A language needing
more categories is a reason to revisit this RFC, not a reason to pull in a
plural-rules engine now.

### 4.3 Typed open/save failures

```rust
pub enum OpenFailure {
    NotFound { path: PathBuf },
    NotUtf8,
    StructureUnparseable,
    Io { kind: std::io::ErrorKind },
}
```

`omriss-ui` maps each to a catalog key at render time, exhaustively. `NotFound`
supplies the path through `t_args`; `Io` maps `ErrorKind` to a plain-language key
with a generic fallback, so **no raw OS string ever reaches the user**.

The path is *data* and must be interpolated, not translated — the same rule as
§3.1.

### 4.4 The remaining label

`"Quick Actions focus boundary"` (`command_palette.rs:136,191`) is a
screen-reader label and gets a catalog key. It is small, and it is exactly the
string a Japanese screen-reader user would hit.

### 4.4a `from_tag` and codeset-only tags

`Locale::from_tag` splits on `-` and `_` and takes the first segment, so
`ja_JP.UTF-8` resolves correctly — verified in the running app by Task 015's
check H. A tag carrying a codeset but **no** region, `ja.UTF-8`, returns `None`,
because `.` is not a separator it splits on.

Real `LANG` values carry a region, so this is theoretical today. While this RFC
is in locale code, split on `.` as well. One line, and it removes a trap for
whoever adds the third locale.

### 4.4b File dialog filter labels

`markdown_dialog()` offers three filters — `"All supported"`, `"Markdown"`,
`"JSON"` — all English literals; the first carries a `TODO(RFC-073)` note
(Task 016). They are
§3.2's problem in a second place: `file_dialog.rs` has no `Locale` and should
not acquire one through a threaded parameter.

`"Markdown"` and `"JSON"` are format names and may reasonably stay as they are
in any locale. `"All supported"` is a translatable phrase, and it is the
*default* filter, so it is the first omriss word a Japanese user reads when
opening a file.

Resolve it the same way as §4.3: the dialog builder takes the already-resolved
strings from its caller in `shell/`, where a `Locale` is in scope, rather than
`file_dialog.rs` reaching for a catalog. A locale does not enter the file layer;
text does.

### 4.5 Locale persistence

```rust
pub struct AppSettings {
    #[serde(default)]
    pub locale: Option<String>,   // BCP 47 tag, e.g. "ja"
    // … existing fields unchanged
}
```

Store the **tag string**, not the `Locale` enum. RFC-036's failure policy
silently discards a settings file that fails to parse, resetting *every*
preference including the recent-files list. An unknown enum variant from a newer
build would therefore wipe the user's settings; an unrecognised `Option<String>`
simply fails `Locale::from_tag` and falls through to OS detection. The weaker
type is the robust choice here, and that is the reason.

Startup order becomes what RFC-043 §92 always specified:

```text
explicit setting (RFC-036) → OS locale (RFC-043, Task 013) → English
```

The toolbar switcher writes to `AppSettings` and persists. `#[serde(default)]`
means existing `settings.toml` files keep working with no migration.

## 5. Non-goals

1. **No new languages.** `en` and `ja` only. This RFC makes the two we have
   correct.
2. **No translation of `omriss-core`.** Core stays catalog-free (RFC-001); it
   gains typed reasons, never message keys.
3. **No ICU / fluent / gettext dependency** — §4.2.
4. **No locale-aware number, date or collation formatting.** Counts are plain
   integers. If that becomes wrong, it is its own RFC.
5. **No change to `t()`'s signature or to the ~100 existing static keys.**
6. **Not RFC-071's warning text**, which is written there — but RFC-071 needs
   `t_plural` and should land after this.

## 6. Validation and test plan

- Every catalog key's placeholder set is identical across `en` and `ja` —
  the §4.1 invariant, as a test.
- Every `.one` key has an `.other` counterpart in both locales, and vice versa.
- `t_plural(En, …, 1)` and `(…, 2)` select different text; `t_plural(Ja, …, 1)`
  and `(…, 2)` select the same text.
- `OpenFailure`'s match is exhaustive with no wildcard arm, so a new variant
  fails the build (the `CapabilityReason` precedent).
- **No `String` cause survives**: `grep` finds no user-facing English literal
  left in `crates/app/src/file/` or `crates/app/src/shell/`. State this in the
  review request as a command and its output, not as a claim.
- Settings round trip: a file written by the previous version loads with
  `locale: None`; an unrecognised tag falls back to OS detection **and leaves
  `recent_files` intact**.
- Pick Japanese, restart, still Japanese. Pick Japanese, delete the settings
  file, restart, back to OS detection.

## 7. Acceptance criteria

1. No user-facing English string is produced outside the catalogs.
2. No raw OS error message is ever shown to the user.
3. Counts are grammatical in both locales at `n = 1` and `n > 1`.
4. Placeholder parity and plural-key parity are enforced by tests.
5. A chosen language survives restart; an unknown stored tag degrades to OS
   detection without discarding other settings.
6. RFC-043's Status is updated: §92's three-step startup order is finally real,
   and follow-up 3 is closed.
