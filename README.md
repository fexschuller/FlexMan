# FlexMan

A configuration GUI for the [FlexASIO](https://github.com/dechamps/FlexASIO) ASIO driver,
built with Tauri 2, React and TypeScript.

FlexASIO is configured through a TOML file and has no UI of its own. FlexMan lets you
pick the backend and input/output devices, edit the TOML directly, run FlexASIO's
self-test against a draft before saving, and collect diagnostics.

> FlexMan is an unofficial hobby project and is not affiliated with FlexASIO or its author.
> Windows only.

<!-- TODO: add a screenshot, e.g. docs/screenshot.png -->

## Installation

1. Install [FlexASIO](https://github.com/dechamps/FlexASIO/releases) first.
2. Download the latest `FlexMan_x.y.z_x64-setup.exe` from the
   [Releases page](https://github.com/fexschuller/FlexMan/releases) and run it.

The installer is not code-signed, so Windows SmartScreen may warn about an unknown
publisher. Click "More info", then "Run anyway".

## Features

- **Config tab:** form for backend, buffer size and per-direction device settings.
  Device lists come from FlexASIO's bundled `PortAudioDevices.exe`.
- **TOML tab:** edit `FlexASIO.toml` directly. Saving from the form only touches the keys
  the form manages, so comments and other keys are preserved.
- **Test:** runs `FlexASIOTest.exe` against the current draft or the saved config.
- **Diagnostics:** FlexASIO version and install folder, toggle for FlexASIO's logging,
  and a system report to paste into bug reports.

## Files FlexMan touches

| Path                                         | Purpose                                                                                                     |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `%USERPROFILE%\FlexASIO.toml`                | FlexASIO's config. It's optional: without it, FlexASIO uses its defaults. FlexMan creates it on first save. |
| `%USERPROFILE%\FlexASIO.log`                 | FlexASIO logs whenever this file exists. The logging toggle creates and deletes it.                         |
| `%USERPROFILE%\FlexASIO.toml.flexman-backup` | Exists only while a draft is being tested (see below).                                                      |
| `%APPDATA%\com.felix.flexman\settings.json`  | FlexMan's own settings, e.g. a manually picked FlexASIO install folder.                                     |

### How testing a draft works

`FlexASIOTest.exe` always reads `%USERPROFILE%\FlexASIO.toml`. To test an unsaved draft,
FlexMan writes a backup marker, temporarily replaces the config with the draft, runs the
test, and restores the original (or deletes the file if there was none before). If FlexMan
is killed mid-test, the leftover backup is restored on the next start.

FlexASIO reloads its config live, so **running a test while a DAW is using FlexASIO resets
the DAW's audio stream twice** (once for the draft, once for the restore).

## ASIO Control Panel integration

The installer writes `HKLM\Software\Fabrikat\FlexASIOGUI\Install\InstallPath`, in both the
64-bit and 32-bit registry views. FlexASIO looks up exactly this key when a DAW's
"ASIO Control Panel" button is clicked and launches `FlexASIOGUI.exe` from that folder. This
is why the executable is named `FlexASIOGUI.exe`.

This key belongs to the original [FlexASIO GUI](https://github.com/flipswitchingmonkey/FlexASIO_GUI).
If both are installed, whichever was installed last owns the control-panel button. On
uninstall, FlexMan only removes the key if it still points at FlexMan's install folder.

Only the NSIS installer is built. An MSI would skip the registry hook.

## Development

Requirements: Node.js, a Rust toolchain, and FlexASIO installed (for device listing and
testing).

```bash
npm install
npm run tauri dev       # run the app
npm test                # frontend tests (vitest)
npm run lint            # ESLint
npm run format          # Prettier
cd src-tauri && cargo test && cargo clippy
npm run tauri build     # NSIS installer in src-tauri/target/release/bundle/nsis
```

## Contributing

Bug reports and pull requests are welcome. For bugs, please include the system report
from the Diagnostics tab.

## License

MIT, see [LICENSE](LICENSE).
