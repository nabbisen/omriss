//! Row-menu keyboard-focus scripts for the Document Map panel.

use dioxus::prelude::*;

fn row_menu_focus_first_script(row_id: u64) -> String {
    format!(
        r#"
requestAnimationFrame(() => requestAnimationFrame(() => {{
  const root = document.querySelector('.row-menu[role="menu"][data-row-menu-id="{row_id}"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[0] || root).focus();
}}))
"#
    )
}

pub(super) const ROW_MENU_FOCUS_NEXT: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  if (!items.length) {
    root.focus();
    return;
  }
  const index = items.indexOf(document.activeElement);
  items[index < 0 ? 0 : (index + 1) % items.length].focus();
})()
"#;

pub(super) const ROW_MENU_FOCUS_PREVIOUS: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  if (!items.length) {
    root.focus();
    return;
  }
  const index = items.indexOf(document.activeElement);
  items[index < 0 ? items.length - 1 : (index - 1 + items.length) % items.length].focus();
})()
"#;

pub(super) const ROW_MENU_FOCUS_FIRST_ITEM: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[0] || root).focus();
})()
"#;

pub(super) const ROW_MENU_FOCUS_LAST_ITEM: &str = r#"
(() => {
  const root = document.querySelector('.row-menu[role="menu"]');
  if (!root) return;
  const items = Array.from(root.querySelectorAll('.row-menu-item:not([disabled])'));
  (items[items.length - 1] || root).focus();
})()
"#;

pub(super) fn focus_open_row_menu(row_id: u64) {
    let script = row_menu_focus_first_script(row_id);
    spawn(async move {
        let _ = document::eval(&script);
    });
}

pub(super) fn move_open_row_menu_focus(script: &'static str) {
    spawn(async move {
        let _ = document::eval(script);
    });
}
