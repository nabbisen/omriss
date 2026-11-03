use super::*;

/// The policy as `(directive name, value)` pairs, split exactly as a browser
/// does: on `;`, then on the first run of whitespace.
fn directives() -> Vec<(&'static str, &'static str)> {
    CONTENT_SECURITY_POLICY
        .split(';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| d.split_once(char::is_whitespace).unwrap_or((d, "")))
        .map(|(name, value)| (name, value.trim()))
        .collect()
}

fn value_of(name: &str) -> Option<&'static str> {
    directives()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v)
}

#[test]
fn meta_tag_carries_every_required_directive() {
    let tag = content_security_policy_meta_tag();
    for directive in [
        "default-src 'self'",
        "script-src 'self' 'unsafe-inline'",
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data:",
        "connect-src ws://127.0.0.1:*",
    ] {
        assert!(tag.contains(directive), "missing {directive:?} in {tag:?}");
    }
}

/// Re-aimed from `never_loosens_to_unsafe_inline`, which asserted the policy
/// contains no `'unsafe-inline'` at all. The corrected policy needs it twice,
/// but the intent of that guard — it never creeps in unannounced — stands. This
/// pins *which* directives may carry it; a third one fails here, and so does
/// `'unsafe-eval'` or any other `'unsafe-*'` keyword (RFC-064 §4.3, handoff §5).
#[test]
fn unsafe_inline_is_conceded_to_exactly_script_and_style() {
    let mut with_unsafe_inline: Vec<&str> = directives()
        .into_iter()
        .filter(|(_, value)| value.contains("'unsafe-inline'"))
        .map(|(name, _)| name)
        .collect();
    with_unsafe_inline.sort_unstable();
    assert_eq!(
        with_unsafe_inline,
        ["script-src", "style-src"],
        "a third directive carries 'unsafe-inline': escalate (RFC-064 §4.3), do not widen"
    );

    for keyword in ["'unsafe-eval'", "'unsafe-hashes'", "'wasm-unsafe-eval'"] {
        assert!(
            !CONTENT_SECURITY_POLICY.contains(keyword),
            "{keyword} is never conceded (RFC-064 §4.3, handoff §5.1)"
        );
    }
}

/// `img-src` is the clause that stops RFC-064 §9a's tracking pixel. Widening it
/// to make remote images load again changes the RFC, not the code.
#[test]
fn img_src_allows_no_remote_origin() {
    assert_eq!(value_of("img-src"), Some("'self' data:"));
}

/// `connect-src` is the load-bearing clause: the page may reach only the
/// framework's loopback websocket, so an injected `fetch('https://…')` fails.
#[test]
fn connect_src_is_the_loopback_websocket_only() {
    assert_eq!(value_of("connect-src"), Some("ws://127.0.0.1:*"));
}

#[test]
fn default_src_is_self_and_every_directive_is_accounted_for() {
    assert_eq!(value_of("default-src"), Some("'self'"));
    let mut names: Vec<&str> = directives().into_iter().map(|(n, _)| n).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "connect-src",
            "default-src",
            "img-src",
            "script-src",
            "style-src"
        ],
        "a new directive was added or one removed: update RFC-064 §4.3 first"
    );
}

#[test]
fn is_a_well_formed_meta_tag() {
    let tag = content_security_policy_meta_tag();
    assert!(tag.starts_with(r#"<meta http-equiv="Content-Security-Policy" content=""#));
    assert!(tag.ends_with(r#"">"#));
    // The policy sits inside a double-quoted attribute: it must not contain one.
    assert!(!CONTENT_SECURITY_POLICY.contains('"'));
}
