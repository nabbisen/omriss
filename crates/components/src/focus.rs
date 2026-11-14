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
//! | `data-focus-scope="panel"` | the search panel, the preview: takes initial focus, does **not** contain Tab |
//! | `data-focus-arrows` | on a scope: arrow keys, Home and End move between its controls |
//! | `autofocus` | the control a scope focuses first; otherwise its first focusable control |
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
  window.__omrissFocus = true;

  const FOCUSABLE =
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
    'textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

  const visible = (el) =>
    !el.hidden && el.getClientRects().length > 0 &&
    window.getComputedStyle(el).visibility !== 'hidden';

  const focusables = (scope) =>
    Array.from(scope.querySelectorAll(FOCUSABLE)).filter(visible);

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

  new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node.nodeType !== 1) continue;
        if (node.matches('[data-focus-scope]')) enter(node);
        node.querySelectorAll('[data-focus-scope]').forEach(enter);
      }
    }
  }).observe(document.body, { childList: true, subtree: true });

  const ARROWS = ['ArrowLeft', 'ArrowUp', 'ArrowRight', 'ArrowDown', 'Home', 'End'];

  document.addEventListener('keydown', (e) => {
    const scope = topmost();
    if (!scope) return;
    // `key` is not always "Tab" for Shift+Tab (X11 reports ISO_Left_Tab), so
    // the physical `code` counts too.
    const isTab = e.key === 'Tab' || e.code === 'Tab';
    if (!isTab && !(ARROWS.includes(e.key) && scope.hasAttribute('data-focus-arrows'))) return;

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
