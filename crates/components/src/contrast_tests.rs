//! RFC-030 acceptance, "text contrast meets target tokens" (WCAG AA, 4.5:1),
//! as a property of the stylesheet's token values.
//!
//! Contrast needs no renderer: it is arithmetic on the colours `style.css`
//! declares. This reads the stylesheet (it lives in `crates/app`, which is not
//! a default member; this crate is, so the guard runs in plain `cargo test`) and
//! fails when
//!
//! * a `color:` declaration is anything but `var(--…)` — a text colour written
//!   as a literal on a background that is a theme token is right for one theme
//!   and wrong for the other, which is how the welcome screen's *Open…* button
//!   came to be 3.71:1; or
//! * a declared text/background pair falls below 4.5:1 in either theme,
//!   hover states included (`filter: brightness(1.1)` scales the whole element,
//!   text and background alike).
//!
//! Run with `--nocapture` to see the table of every pair.

use std::collections::BTreeMap;
use std::path::PathBuf;

const AA: f64 = 4.5;

/// A `color:` that is not a plain token, allowed with its reason. A disabled
/// control is exempt from WCAG 1.4.3 ("inactive user interface component").
const LITERAL_COLOR_EXEMPT: &[(&str, &str)] = &[(
    ".toolbar-icon-btn:disabled",
    "disabled control: exempt from WCAG 1.4.3",
)];

/// The stylesheet with its `/* … */` comments removed, so a comment can neither
/// hide a declaration nor be mistaken for one.
fn stylesheet() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../app/assets/style.css");
    strip_comments(&std::fs::read_to_string(path).expect("style.css is readable"))
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

// ── colours ─────────────────────────────────────────────────────────────────

type Rgb = [f64; 3];

fn parse_hex(value: &str) -> Rgb {
    let hex = value.trim().trim_start_matches('#');
    assert!(
        hex.len() == 6,
        "only #rrggbb tokens are expected: {value:?}"
    );
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f64;
    [byte(0), byte(2), byte(4)]
}

fn luminance(rgb: Rgb) -> f64 {
    let lin = |c: f64| {
        let c = c / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// `filter: brightness(1.1)`: every channel × 1.1, clamped — applied to the
/// whole element, so to its text as well as its background.
fn brighten(rgb: Rgb) -> Rgb {
    rgb.map(|c| (c * 1.1).min(255.0))
}

// ── the stylesheet ──────────────────────────────────────────────────────────

/// The declarations of the first block that follows `marker`, as name → value.
fn custom_properties(block: &str) -> BTreeMap<String, String> {
    block
        .split(';')
        .filter_map(|decl| {
            let decl = decl.trim();
            let (name, value) = decl.split_once(':')?;
            let name = name.trim();
            name.starts_with("--")
                .then(|| (name.to_string(), value.trim().to_string()))
        })
        .collect()
}

/// `(dark tokens, light tokens)`; light is dark overridden by the
/// `prefers-color-scheme: light` block.
fn themes(css: &str) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let dark_start = css.find(":root {").expect("a dark :root block") + ":root {".len();
    let dark_end = dark_start + css[dark_start..].find('}').unwrap();
    let dark = custom_properties(&css[dark_start..dark_end]);

    let media = css
        .find("@media (prefers-color-scheme: light)")
        .expect("a light-theme block");
    let light_start = media + css[media..].find(":root {").unwrap() + ":root {".len();
    let light_end = light_start + css[light_start..].find('}').unwrap();
    let mut light = dark.clone();
    light.extend(custom_properties(&css[light_start..light_end]));
    (dark, light)
}

fn token(theme: &BTreeMap<String, String>, name: &str) -> Rgb {
    let value = theme
        .get(name)
        .unwrap_or_else(|| panic!("token {name} is not declared"));
    parse_hex(value)
}

/// Every `color:` declaration (not `border-color`, `background-color`, …) as
/// `(selector, value)`.
fn color_declarations(css: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let bytes = css.as_bytes();
    let mut from = 0;
    while let Some(i) = css[from..].find("color:") {
        let at = from + i;
        from = at + "color:".len();
        let prev = if at == 0 { b' ' } else { bytes[at - 1] };
        if prev == b'-' || prev.is_ascii_alphabetic() {
            continue;
        }
        let end = css[from..].find([';', '}']).map_or(css.len(), |e| from + e);
        let value = css[from..end].trim().to_string();
        let selector_end = css[..at].rfind('{').unwrap_or(0);
        let selector_start = css[..selector_end].rfind(['}', ';']).map_or(0, |s| s + 1);
        let selector = css[selector_start..selector_end].trim().to_string();
        found.push((selector, value));
    }
    found
}

// ── the tests ───────────────────────────────────────────────────────────────

#[test]
fn every_text_colour_is_a_theme_token() {
    let css = stylesheet();
    let decls = color_declarations(&css);
    assert!(
        decls.len() > 50,
        "the scan looks wrong: {} colours",
        decls.len()
    );
    let offenders: Vec<_> = decls
        .iter()
        .filter(|(selector, value)| {
            let plain_token =
                value.starts_with("var(--") && value.ends_with(')') && !value[6..].contains('(');
            !plain_token
                && !LITERAL_COLOR_EXEMPT
                    .iter()
                    .any(|(exempt, _)| selector.contains(exempt))
        })
        .map(|(selector, value)| format!("{selector} {{ color: {value} }}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "text colours must be var(--…) so they follow the theme (RFC-030); use --on-accent \
         / --on-danger on a coloured background:\n{}",
        offenders.join("\n")
    );
}

/// One declared pair: text token on background token, with `brightness(1.1)`
/// applied when it is a hover state.
struct Pair {
    text: &'static str,
    background: &'static str,
    hover: bool,
}

const PAIRS: &[Pair] = &[
    Pair {
        text: "--text",
        background: "--bg",
        hover: false,
    },
    Pair {
        text: "--text",
        background: "--panel",
        hover: false,
    },
    Pair {
        text: "--muted",
        background: "--bg",
        hover: false,
    },
    Pair {
        text: "--muted",
        background: "--panel",
        hover: false,
    },
    Pair {
        text: "--accent",
        background: "--bg",
        hover: false,
    },
    Pair {
        text: "--accent",
        background: "--panel",
        hover: false,
    },
    Pair {
        text: "--danger",
        background: "--bg",
        hover: false,
    },
    Pair {
        text: "--danger",
        background: "--panel",
        hover: false,
    },
    Pair {
        text: "--on-accent",
        background: "--accent",
        hover: false,
    },
    Pair {
        text: "--on-accent",
        background: "--accent",
        hover: true,
    },
    Pair {
        text: "--on-danger",
        background: "--danger",
        hover: false,
    },
    Pair {
        text: "--on-danger",
        background: "--danger",
        hover: true,
    },
];

fn ratio(theme: &BTreeMap<String, String>, pair: &Pair) -> f64 {
    let (mut text, mut background) = (token(theme, pair.text), token(theme, pair.background));
    if pair.hover {
        text = brighten(text);
        background = brighten(background);
    }
    contrast(text, background)
}

#[test]
fn every_declared_pair_meets_aa_in_both_themes() {
    let (dark, light) = themes(&stylesheet());
    println!(
        "\n{:<14} on {:<9} {:<6} {:>6} {:>6}",
        "text", "background", "hover", "dark", "light"
    );
    let mut failures = Vec::new();
    for pair in PAIRS {
        let (d, l) = (ratio(&dark, pair), ratio(&light, pair));
        println!(
            "{:<14} on {:<10} {:<6} {:>6.2} {:>6.2}",
            pair.text,
            pair.background,
            if pair.hover { "yes" } else { "no" },
            d,
            l
        );
        for (name, r) in [("dark", d), ("light", l)] {
            if r < AA {
                failures.push(format!(
                    "{} on {}{} in the {name} theme: {r:.2}:1 (< {AA})",
                    pair.text,
                    pair.background,
                    if pair.hover { " (hover)" } else { "" }
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "below WCAG AA:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_contrast_arithmetic_matches_known_values() {
    // The two extremes, and a pair computed independently (the owner's report:
    // #0b1320 on #1a6fcc was 3.71:1 before the fix).
    assert!((contrast([0.0; 3], [255.0; 3]) - 21.0).abs() < 1e-9);
    assert!((contrast([255.0; 3], [255.0; 3]) - 1.0).abs() < 1e-9);
    let old = contrast(parse_hex("#0b1320"), parse_hex("#1a6fcc"));
    assert!((old - 3.71).abs() < 0.01, "{old}");
}

#[test]
fn the_scanner_reads_both_themes_and_skips_border_colours() {
    let css =
        ".a { border-color: #fff; color: var(--x); }\n.b { background-color: red; color: #fff }";
    let found = color_declarations(css);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0], (".a".to_string(), "var(--x)".to_string()));
    assert_eq!(found[1], (".b".to_string(), "#fff".to_string()));
    let (dark, light) = themes(&stylesheet());
    assert_ne!(dark["--accent"], light["--accent"], "the themes differ");
    assert_eq!(dark["--on-accent"], "#0b1320");
    assert_eq!(light["--on-accent"], "#ffffff");
}
