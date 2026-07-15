# Platform Support

omriss targets three desktop platforms. This document defines the support
matrix, known constraints, and the policy for adding new platform-specific
behavior.

---

## Supported Platforms

| Platform | Status | Runtime | Notes |
|----------|--------|---------|-------|
| Linux (X11) | Supported | WebKitGTK via Dioxus/wry | Install system packages first |
| Linux (Wayland) | Supported (via XWayland) | WebKitGTK | Native Wayland support follows wry upstream |
| macOS | Supported | WKWebView | No extra packages required |
| Windows | Supported | WebView2 | Requires WebView2 runtime (bundled in recent Windows) |

---

## Linux System Packages

**These packages must be installed before running `cargo build` or `cargo run`.**
If they are missing, the build will fail with a `webkit2gtk-4.1 was not found`
error from pkg-config.

**Debian / Ubuntu (22.04+):**

```sh
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
                 libgtk-3-dev libxdo-dev libssl-dev
```

**Fedora / RHEL:**

```sh
sudo dnf install webkit2gtk4.1-devel javascriptcoregtk4.1-devel \
                 gtk3-devel openssl-devel
```

**Arch Linux:**

```sh
sudo pacman -S webkit2gtk-4.1 gtk3 openssl xdotool
```

### Troubleshooting: "PKG_CONFIG_PATH is not set"

If the packages are installed but the build still fails with
`Package 'webkit2gtk-4.1' not found`, pkg-config cannot find the `.pc` files.
Find and expose them manually:

```sh
# Locate the .pc file
find /usr -name 'webkit2gtk-4.1.pc' 2>/dev/null

# Export its directory and retry
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig   # adjust path as found
cargo run -p omriss
```

On Debian/Ubuntu the `.pc` files are usually at
`/usr/lib/x86_64-linux-gnu/pkgconfig/`. On Fedora they are under
`/usr/lib64/pkgconfig/`.

---

## Platform-Specific Behavior

### Keyboard Modifiers

On macOS the Cmd key is used where Linux/Windows use Ctrl. The keyboard
handler in `crates/app/src/input/keyboard.rs` normalises this automatically
using the `keyboard-types` crate's modifier detection.

### File Dialogs

Native file dialogs (`rfd` crate) use the platform's standard picker:

- Linux: GTK portal or fallback file chooser
- macOS: NSOpenPanel / NSSavePanel
- Windows: IFileOpenDialog / IFileSaveDialog

If the portal is unavailable on a headless Linux system, `rfd` may silently
return no file. This is documented behavior.

### Config Directory

Settings are stored in platform-appropriate locations (RFC-036):

| Platform | Path |
|----------|------|
| Linux | `~/.config/omriss/settings.toml` |
| macOS | `~/Library/Application Support/omriss/settings.toml` |
| Windows | `%APPDATA%\omriss\settings.toml` |

---

## Policy for Platform-Specific Code

All platform-specific behavior must be isolated in the `omriss` app package.
`omriss-core` and `omriss-ui` must compile and test on any host without
GUI libraries (RFC-001, RFC-010 boundary rule).

Platform-specific workarounds should be documented inline with a reference
to the upstream issue where applicable.

---

## Release Platform Matrix

A release may only claim support for a platform that has passed the smoke
test checklist in `docs/smoke-tests.md`. Platforms tested and passing are
listed in the release notes under "Platform Notes".

---

## Known Constraints

- No mobile (iOS/Android) support.
- No browser/web app distribution.
- No ARM/RISC-V testing in initial releases (community contributions welcome).
- Wayland: some compositor-specific window management edge cases may apply.
  Report issues with your compositor name and version.
- Unsigned macOS builds will show a Gatekeeper warning. See
  `RELEASE_CHECKLIST.md` for verification instructions.
