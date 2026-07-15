<!--
Project: omriss — Omriss Editor
Document Set: RFC detailed design bundle
Generated for architecture/design review
Language: English
-->
# RFC-035: Cross-Platform Desktop Runtime Policy

**Project:** omriss — Omriss Editor  
**Milestone:** M8 — Cross-Platform Delivery  
**Status.** Implemented (v0.11.0)  
**Document type:** Detailed RFC design  
**Primary audience:** Architect, Rust developer, UI/UX designer, QA engineer  

---

## 1. Summary

Define platform behavior and constraints for Linux, macOS, and Windows desktop delivery.

## 2. Goals

- Document runtime assumptions.
- Handle Wayland/X11, macOS, and Windows differences.
- Define file dialog and native menu policy.
- Specify fallback behavior.

## 3. Non-Goals

- No mobile support.
- No web app distribution.
- No platform-specific feature divergence unless necessary.

## 4. Design

### Platform Scope

```text
Linux: Wayland and X11 where Dioxus/wry stack supports it
macOS: current supported desktop versions chosen by release policy
Windows: current supported desktop versions chosen by release policy
```

### Runtime Principle

The WebView is a rendering layer. Document logic remains native Rust. Platform-specific behavior must be isolated in the `omriss` app package.

### Known Caveat Policy

If a platform/runtime issue affects file dialogs, rendering, keyboard shortcuts, or accessibility, the release notes must document it and smoke tests must cover it.

## 5. Validation and Test Plan

- Open/save smoke path per OS.
- Keyboard shortcut modifier per OS.
- Wayland/X11 launch smoke tests where available.
- Native file dialog fallback documented.

## 6. Acceptance Criteria

- The supported platform matrix is explicit.
- Platform-specific code is isolated.
- A release cannot claim support for an untested platform path.

## 7. Dependencies

- RFC-010
- RFC-015
- RFC-038

---

## Implementation Reminder

This RFC must preserve the project-wide invariant: editing one section must not rewrite unrelated Markdown source bytes unless the RFC explicitly describes and justifies a structural source transformation.
