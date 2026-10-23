# Project progress

## Implemented

- Rust / eframe 0.33 application, native OpenGL renderer, X11 + Wayland, AccessKit.
- Custom light/dark/system palettes, resizable keypad, scientific controls, history rail/popup, shortcut reference.
- Checked decimal engine, bounded Pratt parser, scientific functions, degree/radian selection, repeat equals.
- Keyboard input, clipboard, last-operand transformations, validation and error feedback.
- Bounded local history and preferences through native eframe storage.
- Release profile, desktop metadata, platform build guidance and CI workflow.

## UI/UX fixes (October 24, 2026)

- Corrected the official display name to Calculator in the window, compact toolbar label, shortcut reference, bundle metadata, desktop entry and documentation. Replaced the monogram launcher/window icon with a plain calculator symbol.
- Preserved the crate/executable name, application ID, storage keys and parent directory; existing preferences and history keep their original paths. No user data was modified.
- Naming correction validation: formatting, strict Clippy, all 31 tests and the Linux release build pass. Refreshed all three native X11 screenshots using temporary sample preferences/history, with the window title confirmed as Calculator.

- Consistent typing after evaluation in the expression editor: digits start fresh, operators continue from the result; caret navigation, selection and clicking still allow explicit corrections.
- Nth root preserves an evaluated base. History recall and subsequent calculations retain approximate-result provenance.
- DEG/RAD changes recalculate the preview and clear repeat state without silently adding history records.
- Preview distinguishes incomplete input from invalid syntax, division by zero and scientific domain errors.
- Escape closes help, then floating history, before clearing the calculation.
- Scientific targets increased from 28 to 32 logical points and status/help text from 11–12 to 13; tighter scientific spacing preserves the minimum 400×660 layout.
- 31 tests pass, including eight new regression tests and light/dark minimum-window geometry checks across ready, incomplete, invalid, approximate and long-result states. Native visual and screen-reader verification remains pending.

## UI/UX improvements (October 23, 2026)

- Editable expressions with caret navigation, selection, native undo/redo, expression paste, and typed scientific function names.
- Explicit ready, live preview, result, approximate, incomplete, and error states. Stale results are dimmed and cannot be copied.
- Fixed feedback space prevents keypad movement; compact scientific spacing retains the full equals button at 400×660.
- History clearing has a session Undo action that preserves newly committed calculations.
- Active history toggle, larger result Copy target, and shortcut hints.
- Current verification: 23 tests pass, including editor input/clipboard/undo, invalid-result copying, history restoration, and themed layout/feedback geometry. Formatting and strict Clippy checks pass.
- This update was checked with headless egui interaction/layout tests. Native screenshots below predate this update; fresh visual and screen-reader checks remain pending.

## Validation of the original release

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo test --locked`: 18 tests passed (decimal accuracy, domains/overflow/underflow, malformed inputs, state transitions, keyboard symbols, persistence and layout bounds).
- `cargo build --release --locked`: passed on Arch Linux x86_64, Rust 1.98.1.
- Wayland release startup smoke: ran for five seconds without logged errors; interactions were verified on X11.
- Native Linux X11 window launched and visually inspected in light/dark, scientific/standard and compact layouts. Real keyboard calculation, copy/paste, mouse function activation, theme switching, history clearing and restoration exercised.
- Scientific layout fits minimum 400×660 without clipping the equals control; standard also checked at 780×740 and 1200×900. Native screenshots are in `docs/screenshots/`.
- Windows/macOS CI is prepared, not executed locally.

## Remaining platform verification

- Windows and macOS build/run and packaging/signing.
- Manual screen-reader testing, OS theme switching and HiDPI behavior on each platform.
- Shipping installers require platform signing credentials and a distribution destination.

## Design / behavior decisions

- Jade accent, warm neutral surfaces, native window decorations, no background animation loop.
- 15 significant display digits for scientific results; decimal results retain library precision.
- Percent means operand / 100. Unary buttons operate on the last operand. Nth root builds x^(1/n).
- Parser bounded to 2,048 bytes, 256 operation characters and depth 64; history bounded to 500.
- No Antigravity or requested Gemini/Argon models are exposed by supported tools; implementation is direct.

## Calculator interface refinement (October 24, 2026)

- Retained egui/eframe and the calculation engine. Calculator remains the display name; crate name, app ID, persistence keys and parent directory are unchanged.
- Consolidated navigation into a segmented mode selector, compact appearance/shortcuts menu and active history icon. Added Ctrl+1 / Ctrl+2 mode shortcuts without disturbing the editor.
- Unified the right-aligned expression and responsive tabular result, integrated a drawn copy icon, removed idle Ready text and the bottom help strip. Feedback stays inside a reserved display line.
- Centralized palette, error, spacing, width, corner and control-size tokens; restrained radii, immediate interaction states and disabled animation.
- Bounded main content to 480×780 logical points. Default window is 480×680; minimum remains 400×660. Scientific functions fit the minimum without hiding equals.
- History defaults to 232 points, resizes from 208 to 320 as space permits, and floats below 720 points. Compact rows restore original expressions, show exact values in tooltips and offer accessible copy icons. Undo remains available after clearing.
- Bundled fallback covers proportional and monospace math symbols. Copy icons are drawn to avoid missing glyphs.
- Final checks: `cargo fmt --check`, strict all-target Clippy, all 34 tests and Linux release build pass. Regression coverage includes original-expression history restoration, mode shortcuts during editing, history row clipping/recall, minimum layouts, clipboard, undo, scientific calculations and persistence.
- Native Linux X11 release launches and screenshot inspection cover dark Standard with history, light Scientific with/without history at 400×660 and 780×740, compact dark Standard at 480×680, and wide dark Scientific with history at 1200×900. Screenshots are refreshed in `docs/screenshots/`. Sample storage was isolated in temporary directories.
- Native keyboard simulation could not reliably focus the calculator and was stopped; keyboard behavior is verified through egui event tests. Windows/macOS, screen readers, physical HiDPI and OS-driven theme transitions remain unverified.

## Final UI/UX refinement (October 24, 2026)

- Tightened display padding, editor/result spacing and shared right alignment. Retained the native single-line editor, caret scrolling, selection and undo. Long results explicitly remain on one horizontally scrolling line.
- Strengthened dark utility/digit separation within the existing jade palette. Added a subtle scientific surface, 17-point function labels, 20-point advanced symbols and 34-point scientific targets. Compact DEG/RAD buttons visibly retain their selected state.
- Removed redundant number/operator tooltips, including the floating multiplication symbol. Hover changes the surface, pressing darkens it, focus retains a visible inset ring, and a pending operator has an independent underline.
- Standardized copy, backspace, appearance, history and close icons as small vector drawings with a shared stroke and accessible names. Existing 32-point icon hit targets remain.
- History rows are 52 points with 4-point gaps, secondary expressions, result-aligned copy icons, recalled-entry selection and pressed/focus feedback. Recall/copy remain separate targets, with saved history and clear/undo behavior preserved.
- Rounded keypad row heights down to whole logical points to avoid a fractional overflow triggering an unnecessary scrollbar at the minimum window size. Minimum dimensions remain 400×660.
- Percentage semantics are unchanged: postfix operand / 100. Targeted tests verify `200*10% = 20`, `200+10% = 200.1`, `200-10% = 199.9`, `50% = 0.5` and `0% = 0`; README and existing in-app help explain this policy.
- Validation: formatting, strict all-target Clippy, 37 tests and debug build pass. Added coverage for expression/result adjacency and alignment, one-line long results, history copy isolation, and expanded minimum-window scientific-symbol clipping checks.
- Launched the native Linux X11 app with temporary sample storage, captured and inspected both themes and modes, history docked/floating/closed, and sizes from 400×660 through 1400×1000. Updated screenshots in `docs/screenshots/`. Native simulated clicks were unreliable; interaction verification uses egui event tests. User storage was not modified.
- Remaining verification: real screen-reader and physical HiDPI testing, maximization under a desktop window manager, Windows/macOS and OS-driven theme changes. No new dependencies or calculation-engine behavior changes.
