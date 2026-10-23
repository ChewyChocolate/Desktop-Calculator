# Distribution

Build release artifacts on the intended operating system. The Cargo release profile uses thin LTO, a single codegen unit and stripped symbols. No installers have been signed or published.

## Linux: per-user installation

From the repository after `cargo build --release --locked`:

```sh
install -Dm755 target/release/cindral-calculator "$HOME/.local/bin/cindral-calculator"
install -Dm644 packaging/com.cindral.calculator.desktop "$HOME/.local/share/applications/com.cindral.calculator.desktop"
install -Dm644 assets/com.cindral.calculator.svg "$HOME/.local/share/icons/hicolor/scalable/apps/com.cindral.calculator.svg"
```

Ensure `~/.local/bin` is on PATH. Launch from the desktop menu or run `cindral-calculator`. For distribution, build packages on a suitably old supported glibc baseline. Arch-built binaries are not guaranteed to work on older distributions. A distro package should declare system graphics/windowing dependencies.

## Windows

The release `.exe` runs standalone against the OS graphics/windowing libraries. For an installer, package it with WiX or Inno Setup, add a Start Menu shortcut and an uninstall entry, and sign the installer and executable with the publisher's certificate. No certificate is included.

## macOS

For an application bundle, an optional external tool is `cargo-bundle` (`cargo install cargo-bundle --locked`, then `cargo bundle --release`). Cargo metadata includes the application name, identifier and category. Supply an `.icns` icon converted from the source SVG for a shipping bundle. Build for the intended architecture, or combine separately built Apple Silicon and Intel binaries into a universal bundle. Sign with Developer ID and notarize with Apple tools before distribution. No signing identity is included.

The SVG in `assets/` is the editable source icon. A PNG rendition is embedded for native window icons. Windows executable resources and macOS bundle icons remain platform packaging steps.
