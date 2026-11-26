//! RFC-059 §12: every programmatic focus move, in one place.
//!
//! **Rust declares, one script acts.** Components say in markup what focus
//! should do; one script, installed once by [`use_focus_script`], does it. No
//! component calls `document::eval` to move focus (`focus_path_tests` fails if
//! one does), and there is no per-overlay or per-transition focus code.
//!
//! | Markup | Meaning |
//! |---|---|
//! | `data-focus-scope="contain"` | a dialog, Quick Actions or a menu: takes initial focus, keeps Tab and Shift+Tab inside |
//! | `data-focus-scope="panel"` | the preview: takes its own place in the layout, takes initial focus, does **not** contain Tab. Anything drawn *over* other content is `contain` instead (§12.3a) |
//! | `data-focus-arrows` | on a scope: arrow keys, Home and End move between its controls |
//! | `autofocus` | the control a scope focuses first; otherwise its first focusable control |
//! | `data-focus-return="ID"` | on a control that opens a scope: focus returns to element `ID`, not to the control |
//! | `data-focus-primary` | at most one per view: the view's primary surface, where focus falls back to |
//!
//! `autofocus` is not honoured by the WebView on its own (Task 019), which is
//! why the script looks it up and focuses it. Shortcuts stay in Rust on
//! `div.app`; the script only decides focus, and so needs no keymap.

use dioxus::prelude::*;

/// The installed script. A `MutationObserver` gives a newly added scope its
/// initial focus; a capture-phase `keydown` listener keeps Tab (and, where the
/// scope asks, the arrow keys) inside the topmost `contain` scope.
const FOCUS_SCRIPT: &str = r#"
(() => {
  if (window.__omrissFocus) return;
  window.__omrissFocus = {};

  const FOCUSABLE =
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
    'textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

  const visible = (el) =>
    !el.hidden && el.getClientRects().length > 0 &&
    window.getComputedStyle(el).visibility !== 'hidden';

  const focusables = (scope) =>
    Array.from(scope.querySelectorAll(FOCUSABLE)).filter(visible);

  // Input types that are not text: a checkbox, a radio or a button has no caret.
  const NON_TEXT = ['button', 'checkbox', 'radio', 'submit', 'reset', 'file',
                    'color', 'range', 'image', 'hidden'];
  const isTextInput = (el) =>
    !!el && el.tagName === 'INPUT' && !NON_TEXT.includes((el.type || 'text').toLowerCase());
  // A control with a caret: it keeps the arrows, Home and End.
  const isEditable = (el) =>
    !!el && (el.isContentEditable || el.tagName === 'TEXTAREA' || isTextInput(el));

  // Containment is the topmost `contain` scope: the last in document order.
  const topmost = () => {
    const all = document.querySelectorAll('[data-focus-scope="contain"]');
    return all.length ? all[all.length - 1] : null;
  };

  // `[autofocus]` first: querySelectorAll returns document order, not
  // selector-list order, so a preferred control that is not first has to be
  // looked up on its own.
  const firstControl = (scope) => {
    const preferred = scope.querySelector('[autofocus]');
    if (preferred && visible(preferred)) return preferred;
    return focusables(scope)[0] || null;
  };

  // Two frames: a re-render the opening action triggers lands inside the first
  // (the structure dialogs hit this), and the second fires after that patch.
  const enter = (scope) => {
    requestAnimationFrame(() => requestAnimationFrame(() => {
      if (!scope.isConnected || scope.contains(document.activeElement)) return;
      (firstControl(scope) || scope).focus();
    }));
  };

  // ── Restoration (§12.2 step 2) ──────────────────────────────────────────
  // The WebView leaves focus on <body> when the focused element is removed and
  // fires no blur or focusout, so removal is what the observer watches.

  // The element that had focus, kept from `focusin`: by the time a scope is
  // added its opener may already be gone (a menu item, when the menu closes in
  // the same batch as the dialog it opened).
  let lastFocus = null;
  document.addEventListener('focusin', (e) => { lastFocus = e.target; }, true);

  // `data-focus-return="ID"` on a control that opens a scope: focus returns to
  // element ID, not to the control.
  const returnId = (el) => {
    const holder = el && el.closest ? el.closest('[data-focus-return]') : null;
    return holder ? holder.getAttribute('data-focus-return') : null;
  };

  const open = [];            // scopes that are open: { el, opener, retId }
  let justClosed = null;      // the scope that closed since the last check

  const track = (scope) => {
    open.push({ el: scope, opener: lastFocus, retId: returnId(lastFocus) });
  };

  const settle = () => {
    for (let i = open.length - 1; i >= 0; i--) {
      if (!open[i].el.isConnected) justClosed = open.splice(i, 1)[0];
    }
  };

  const returnTarget = (s) => (s.retId ? document.getElementById(s.retId) : s.opener);

  // Focus left on <body>, one frame after a batch of mutations, and only while
  // this window is the active one. While a `contain` scope is open, focus goes to
  // that scope's first control (§12.3a: an element in it was removed by a
  // re-render). Otherwise the first of (a) where the scope that just closed was
  // opened from, if it is still in the document; (b) the view's primary surface;
  // (c) div.app.
  const check = () => {
    const closed = justClosed;
    justClosed = null;
    const active = document.activeElement;
    if (active && active !== document.body) return;
    if (!document.hasFocus()) return;
    const scope = topmost();
    if (scope) {
      (firstControl(scope) || scope).focus();
      return;
    }
    const targets = [
      closed && returnTarget(closed),
      document.querySelector('[data-focus-primary]'),
      document.querySelector('.app'),
    ];
    for (const target of targets) {
      if (!target || !target.isConnected) continue;
      target.focus();
      if (document.activeElement === target) return;
    }
  };

  let checking = false;
  const scheduleCheck = () => {
    if (checking) return;
    checking = true;
    requestAnimationFrame(() => { checking = false; check(); });
  };

  const adopt = (scope) => { track(scope); enter(scope); };

  new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node.nodeType !== 1) continue;
        if (node.matches('[data-focus-scope]')) adopt(node);
        node.querySelectorAll('[data-focus-scope]').forEach(adopt);
      }
    }
    settle();
    scheduleCheck();
  }).observe(document.body, { childList: true, subtree: true });

  // The script is installed after the first render, so a scope that is already
  // in the document (a dialog opened at start-up) is never "added" to the
  // observer: enter those too. Launch itself leaves focus on <body>: check it,
  // and again whenever the window gains focus.
  document.querySelectorAll('[data-focus-scope]').forEach(adopt);
  window.addEventListener('focus', scheduleCheck);
  scheduleCheck();

  const ARROWS = ['ArrowLeft', 'ArrowUp', 'ArrowRight', 'ArrowDown', 'Home', 'End'];

  document.addEventListener('keydown', (e) => {
    // IME composition is never interrupted: not by Tab, not by anything.
    if (e.isComposing) return;

    const scope = topmost();
    if (!scope) return;

    // Typing returns to the field. A printable key pressed on a control that is
    // not a text field, in a scope whose first control is one (Quick Actions'
    // input, a dialog's title), moves focus there *without* preventDefault, so
    // the character lands in the field. Space on a button is its activation, not
    // text, and is left alone.
    const printable = e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey;
    if (printable) {
      if (e.key === ' ' && e.target && e.target.tagName === 'BUTTON') return;
      const field = firstControl(scope);
      if (field && isTextInput(field) && !isEditable(e.target)) field.focus();
      return;
    }

    // `key` is not always "Tab" for Shift+Tab: WebKitGTK reports "Unidentified"
    // for Shift+Tab sent through xdotool (observed, WebKitGTK 2.54.2, with
    // `code` "Tab"), so the physical `code` counts too.
    const isTab = e.key === 'Tab' || e.code === 'Tab';
    // Editable fields keep their caret keys: arrows, Home and End move the
    // caret there. Tab is unaffected.
    const caretKey = ARROWS.includes(e.key) && isEditable(e.target);
    if (!isTab && !(ARROWS.includes(e.key) && scope.hasAttribute('data-focus-arrows') && !caretKey)) return;

    // Decided here, synchronously, so the key never reaches Rust or the
    // WebView's own traversal.
    e.preventDefault();
    e.stopPropagation();

    const items = focusables(scope);
    if (!items.length) { scope.focus(); return; }
    const i = items.indexOf(document.activeElement);
    const last = items.length - 1;
    const next = items[i < 0 ? 0 : (i + 1) % items.length];
    const prev = items[i < 0 ? last : (i - 1 + items.length) % items.length];
    if (isTab) { (e.shiftKey ? prev : next).focus(); return; }
    switch (e.key) {
      case 'ArrowLeft': case 'ArrowUp': prev.focus(); break;
      case 'ArrowRight': case 'ArrowDown': next.focus(); break;
      case 'Home': items[0].focus(); break;
      case 'End': items[last].focus(); break;
    }
  }, true);
})()
"#;

/// Installs the focus script, once, when the calling component first renders.
/// The host calls it from its root component.
pub fn use_focus_script() {
    use_hook(|| {
        spawn(async move {
            let _ = document::eval(FOCUS_SCRIPT);
        });
    });
}

#[cfg(test)]
pub(crate) mod tests;
