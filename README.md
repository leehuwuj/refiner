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
- **Selection icon** (macOS, opt-in under Settings → Shortcut Behavior): select
  text with the mouse in any app and a small button appears next to the
  cursor; click it for the compact popup.
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

### macOS permissions

Refiner reads the selected text through the Accessibility API, so it (or your
terminal, when running from source) needs **System Settings → Privacy &
Security → Accessibility**; macOS asks on first launch. Apps that don't expose
their selection (some terminals, Electron and Java apps) fall back to sending
`⌘C` through System Events, which asks once to allow controlling System
Events. The selection icon never uses `⌘C`, so it only appears in apps that
expose their selection.

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
| `selection_watch.rs` | Global mouse monitor behind the selection icon        |
| `ui/`             | Main window, popup and settings views                    |

## License

MIT
