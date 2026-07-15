//! Shared keyboard focus containment for modal dialogs.

use dioxus::prelude::*;

const MODAL_FOCUS_NEXT: &str = r#"
(() => {
  const root = document.querySelector('.modal-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  if (!focusables.length) {
    root.focus();
    return;
  }
  const index = focusables.indexOf(document.activeElement);
  focusables[index < 0 ? 0 : (index + 1) % focusables.length].focus();
})()
"#;

const MODAL_FOCUS_PREVIOUS: &str = r#"
(() => {
  const root = document.querySelector('.modal-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  if (!focusables.length) {
    root.focus();
    return;
  }
  const index = focusables.indexOf(document.activeElement);
  focusables[index < 0 ? focusables.length - 1 : (index - 1 + focusables.length) % focusables.length].focus();
})()
"#;

const MODAL_FOCUS_FIRST: &str = r#"
(() => {
  const root = document.querySelector('.modal-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  (focusables[0] || root).focus();
})()
"#;

const MODAL_FOCUS_LAST: &str = r#"
(() => {
  const root = document.querySelector('.modal-overlay[aria-modal="true"]');
  if (!root) return;
  const focusables = Array.from(root.querySelectorAll(
    'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  (focusables[focusables.length - 1] || root).focus();
})()
"#;

const MODAL_FOCUS_INITIAL: &str = r#"
requestAnimationFrame(() => requestAnimationFrame(() => {
  const root = document.querySelector('.modal-overlay[aria-modal="true"]');
  if (!root) return;
  if (root.contains(document.activeElement)) return;
  const focusables = Array.from(root.querySelectorAll(
    '[autofocus], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])'
  )).filter((el) => {
    const style = window.getComputedStyle(el);
    return !el.hidden && el.getClientRects().length > 0 && style.visibility !== 'hidden';
  });
  (focusables[0] || root).focus();
}))
"#;

pub(crate) fn focus_active_modal() {
    spawn(async move {
        let _ = document::eval(MODAL_FOCUS_INITIAL);
    });
}

pub(crate) fn trap_modal_tab(evt: &Event<KeyboardData>) -> bool {
    let script = match evt.key() {
        Key::Tab if evt.modifiers().shift() => MODAL_FOCUS_PREVIOUS,
        Key::Tab => MODAL_FOCUS_NEXT,
        Key::ArrowLeft | Key::ArrowUp => MODAL_FOCUS_PREVIOUS,
        Key::ArrowRight | Key::ArrowDown => MODAL_FOCUS_NEXT,
        Key::Home => MODAL_FOCUS_FIRST,
        Key::End => MODAL_FOCUS_LAST,
        _ => return false,
    };

    evt.stop_propagation();
    evt.prevent_default();

    spawn(async move {
        let _ = document::eval(script);
    });

    true
}
