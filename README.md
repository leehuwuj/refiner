# Refiner

A small AI-powered app for translating, correcting and refining text. It
lives in the menu bar (system tray on Windows), and is a native rewrite of the
[Tauri version](https://github.com/leehuwuj/refiner) built with
[GPUI Kit](https://github.com/longbridge/gpui-kit) — no webview, no Node.

## Features

- **Translate / Correct / Refine** in the main window, side by side.
- **Global shortcut** (`⌘E` on macOS, `Ctrl+E` on Windows): reads the text
  selected in any app and processes it in the main window or in a compact
  popup next to the cursor (popup shows a translation and a correction).
- **Selection icon** (macOS and Windows, opt-in under Settings → General →
  Shortcut → Selection button): select text in a supported app and a small
  button appears beside it; click it for the compact popup. Windows uses UI
  Automation selection-change events and leaves the clipboard untouched.
- **Providers**: Ollama (local / offline), OpenAI and OpenAI-compatible
  servers, Gemini, Groq. Optional "thinking" switch for reasoning models.
- **Custom prompts** per mode, with `{original_lang}` and `{target_lang}`.
- **Language analysis**: optionally records your inputs and asks the model for
  an HTML proficiency report (CEFR estimate, recurring errors, study plan).
- Light / dark / system theme and three text sizes.

## Screenshots

| Main window | Popup | Settings |
| --- | --- | --- |
| <img src="docs/images/main.png" height="150"> | <img src="docs/images/popup.png" height="150"> | <img src="docs/images/settings.png" height="150"> |

## Run

Requires Rust 1.92+. On macOS only the Command Line Tools are needed (shaders
compile at runtime).

```bash
cargo run --release
```

Build a `Refiner.app` bundle (menu-bar only, no Dock icon):

```bash
./script/bundle-macos.sh
```

### Windows

Install Rust 1.92+ with the MSVC toolchain and Visual Studio Build Tools
with **Desktop development with C++** and a Windows 10/11 SDK. Use the
toolchain matching your machine (`x86_64-pc-windows-msvc` for x64 or
`aarch64-pc-windows-msvc` for ARM64).

```powershell
cargo test --locked
cargo run --release --locked
```

The executable is `target\release\refiner.exe`. Release builds compile GPUI's
HLSL shaders with the Windows SDK's `fxc.exe`. If SDK discovery fails, set
`GPUI_FXC_PATH` to the full path of that executable before building.

Left-click the tray icon to reopen Refiner; right-click for Settings,
Analysis, or Quit. `Ctrl+E` reads the active application's selection through
Windows UI Automation. It never sends Ctrl+C or reads or changes the clipboard.
If the application does not expose its selected text through UI Automation,
no text is submitted. Capturing text from an elevated application requires
appropriate Windows privileges.
Enable **Selection button** in Settings to show an icon beside selections
exposed by UI Automation. The icon does not take focus from the source app;
clearing the selection or changing focus dismisses it.
The Windows icon appears only after mouse release or after releasing a
keyboard selection key (Shift+arrows/Home/End/Page Up/Page Down or Ctrl+A).
Starting another selection hides it; dragging does not create icon windows.
The icon sits just above the top-left corner of the first visible
selected line, using UI Automation's text bounds for mouse and keyboard
selections. If no visible bounds are available, the icon stays hidden.

CI tests and builds the native app on macOS (Apple Silicon and Intel) and
Windows (x64 and ARM64). Each build uploads the same packages used by releases.

### macOS permissions

Refiner reads the selected text through the Accessibility API, so it (or your
terminal, when running from source) needs **System Settings → Privacy &
Security → Accessibility**; macOS asks on first launch. Apps that don't expose
their selection (some terminals, Electron and Java apps) fall back to sending
`⌘C` through System Events, which asks once to allow controlling System
Events. The selection icon never uses `⌘C`, so it only appears in apps that
expose their selection and usable text bounds.

The macOS selection button is a native, nonactivating 22-point circle using
Refiner's app artwork, with a transparent background and no window shadow.
Enable **Settings → General → Shortcut → Selection button**, then select text
by dragging, double/triple-clicking, Shift-clicking, Shift+navigation keys, or
`⌘A`. The button appears above the first selected visible line when the app
exposes line geometry, and stays hidden during the gesture. Beginning another
selection, typing, scrolling, or switching apps dismisses it. Clicking the
button opens the compact result popup. Accessibility reads run on a dedicated
worker and never change the clipboard. Native text ranges and browser text
markers are supported; apps without usable geometry get no button. Display
placement uses native screen coordinates, including Retina and secondary
screens, and respects the menu bar and Dock.

After building, launch the bundle with:

```bash
open target/release/Refiner.app
```

## Releases

The workflows build the root Cargo package directly. They do not use pnpm,
`src-tauri`, or `tauri-action`.

- Every branch push and pull request runs tests, Clippy (including snapshot
  code), and release packaging on all four native platforms.
- macOS artifacts include an architecture-specific DMG and a ZIP containing
  `Refiner.app`. Windows artifacts are portable ZIPs containing `Refiner.exe`,
  the README, and the license; the MSVC runtime is linked statically.
- Pushing a `refiner-v*` tag builds all four platforms, verifies that the tag
  matches `Cargo.toml`, and publishes a GitHub Release with six downloads and
  `SHA256SUMS`. Failed builds or incomplete uploads do not publish a release.
  Existing published releases are never overwritten. Prerelease Cargo versions
  produce GitHub prereleases.

To release version 0.2.0, tag the native-app commit you want to distribute:

```bash
git tag refiner-v0.2.0
git push origin refiner-v0.2.0
```

The **Publish native release** workflow can also be dispatched for an existing
matching tag. Both CI and releases call the same reusable build workflow.
macOS bundles are ad-hoc signed but not notarized; Windows executables are
unsigned. Developer ID signing and notarization are not configured.

To produce the release packages locally:

```bash
./script/package-macos.sh aarch64-apple-darwin
# Or on an Intel Mac:
./script/package-macos.sh x86_64-apple-darwin
```

```powershell
./script/package-windows.ps1 -Target x86_64-pc-windows-msvc
# On a Windows ARM64 machine:
./script/package-windows.ps1 -Target aarch64-pc-windows-msvc
```

Packages are written to `dist/`. macOS scripts accept `CARGO_TARGET_DIR` and
`MACOSX_DEPLOYMENT_TARGET`; the bundle's minimum system version matches the
build target (11.0 by default).

## Data

Settings, history and reports live in the same folder as the Tauri build, so
an upgrade keeps them. On first launch the old `store.bin` is imported.

| OS      | Folder                                                  |
| ------- | ------------------------------------------------------- |
| macOS   | `~/Library/Application Support/dev.leehuwuj.refiner`   |
| Windows | `%APPDATA%\dev.leehuwuj.refiner`                        |

Set `REFINER_DATA_DIR` to use another folder (useful for development).

## Development

```bash
cargo test                       # unit tests
REFINER_LIVE_URL=http://localhost:11434 REFINER_LIVE_MODEL=gemma3 \
  cargo test live -- --ignored   # one real request against Ollama
REFINER_SNAPSHOT=/tmp/shots cargo run --features snapshot
                                 # render every window to PNGs, then quit
```

| Module            | Role                                                     |
| ----------------- | -------------------------------------------------------- |
| `settings.rs`     | Settings model, JSON persistence, Tauri store import     |
| `llm.rs`          | Blocking provider clients (Ollama, OpenAI, Gemini, Groq) |
| `refine.rs`       | Prompts and the translate / correct / refine operations  |
| `history.rs`      | Optional request history                                 |
| `analysis.rs`     | Language analysis report                                 |
| `state.rs`        | GPUI models shared by the windows                        |
| `windows.rs`      | Opening, positioning and close-on-blur of windows        |
| `tray.rs`         | Menu-bar icon and menu                                   |
| `hotkey.rs`       | Global shortcut and selected-text capture                |
| `platform.rs`     | macOS / Windows specifics (⌘C, cursor, accessory app)    |
| `ax.rs`           | macOS Accessibility: read the selected text              |
| `selection_watch.rs` | Platform routing for the selection icon               |
| `macos_selection_watch.rs` | macOS gesture gating and Accessibility observers |
| `macos_selection_button.rs` | Native macOS selection button and screen placement |
| `ui/`             | Main window, popup and settings views                    |

## License

MIT
