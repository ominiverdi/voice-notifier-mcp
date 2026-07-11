# Voice Notifier MCP

A local MCP notification server with neural voice, desktop, and terminal channels.

It exposes `voice_notify` with three independent notification channels:

- desktop notification through `notify-send` (enabled by default)
- terminal bell
- local neural speech through Kokoro, with `spd-say` as an emergency fallback

Kokoro loads lazily on the first spoken notification and remains resident. The default voice is `bf_emma`. Audio is sent as in-memory 24 kHz PCM to PipeWire through `pw-play`; notification text is never passed through a shell or sent to a remote API.

## Requirements

- Linux with glibc 2.38 or newer and PipeWire's `pw-play`
- `notify-send` for desktop notifications
- `spd-say` for speech fallback
- Rust 1.88 or newer and a native build toolchain when building from source
- approximately 312 MB for the Kokoro model and Emma voice assets

The current ONNX Runtime binary requires glibc 2.38 symbols. Ubuntu 24.04 and Debian 13 are tested; Debian 12 and Ubuntu 22.04 are not currently supported by the default build. Release binaries and packages are built on Ubuntu 24.04 and require glibc 2.39 or newer. On Debian and Ubuntu, the runtime commands are provided by `pipewire-bin`, `libnotify-bin`, and `speech-dispatcher`. On Fedora, they are provided by `pipewire-utils`, `libnotify`, and `speech-dispatcher`.

The licensed integration uses [`kokoro-en` 0.1.4](https://crates.io/crates/kokoro-en), licensed Apache-2.0. The Kokoro model weights are also Apache-2.0.

## Install

### Debian or Ubuntu package

Download the `.deb` and `SHA256SUMS` files for the release, verify the package, and install it:

```bash
sha256sum --check --ignore-missing SHA256SUMS
sudo apt install ./voice-notifier-mcp_*_amd64.deb
voice-notifier-install-assets
```

### Fedora or RPM package

Download the `.rpm` and `SHA256SUMS` files for the release, verify the package, and install it:

```bash
sha256sum --check --ignore-missing SHA256SUMS
sudo dnf install ./voice-notifier-mcp-*.x86_64.rpm
voice-notifier-install-assets
```

The distribution packages install `voice-notifier-mcp` and `voice-notifier-install-assets` under `/usr/bin`. Runtime dependencies are installed through the package manager. `speech-dispatcher` is recommended rather than required because it is only the emergency fallback.

### Portable archive

The release also provides a `linux-x86_64.tar.gz` archive containing the server, asset installer, documentation, and licenses. Extract it and copy the two executables to a directory on `PATH`, then run `voice-notifier-install-assets`.

### Build from source

```bash
git clone https://github.com/ominiverdi/voice-notifier-mcp.git
cd voice-notifier-mcp
./scripts/install-kokoro-assets.sh
cargo install --locked --path .
```

The asset installer downloads the official model and Emma voice, verifies their SHA-256 hashes, and places them under `${XDG_DATA_HOME:-$HOME/.local/share}/voice-notifier-mcp`. Once installed, synthesis is fully local.

Expected SHA-256 values for the installed assets:

```text
8fbea51ea711f2af382e88c833d9e288c6dc82ce5e98421ea61c058ce21a34cb  model.onnx
669fe0647f9dd04fcab92f1439a40eeb4c8b4ab1f82e4996fe3d918ce4a63b73  bf_emma.bin
```

Override the defaults with `VOICE_NOTIFIER_MODEL_PATH`, `VOICE_NOTIFIER_VOICE_PATH`, and `VOICE_NOTIFIER_VOICE`. When selecting another voice, point `VOICE_NOTIFIER_VOICE_PATH` to its file or to a directory containing voice files.

## Build and test

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
./scripts/package-release.sh
```

`package-release.sh` creates a portable archive, Debian package, RPM package, and SHA-256 manifest under `dist/`. It requires `dpkg-deb` and `cargo-generate-rpm` 0.21.0 in addition to the Rust build tools.

## Tool input

```json
{
  "title": "Ferrum complete",
  "message": "Lorenzo, the codebase review is done. The report is ready.",
  "desktop": true,
  "bell": true,
  "voice": true,
  "speech_speed": 1.0
}
```

Only `message` is required. `speech_speed` accepts 0.5 through 2.0 and defaults to 1.0. Spoken messages are limited to 500 Unicode characters. Process arguments are passed directly without a shell.

The server negotiates MCP protocol versions `2024-11-05`, `2025-06-18`, and `2025-11-25`. Interoperability has been verified with Ferrum, Pi 0.80.3 through its MCP bridge extension, and the official MCP Inspector 0.22.0. The tool is annotated as non-destructive, non-idempotent, and local-only.

## Ferrum configuration

```toml
[[mcp.servers]]
name = "voice-notifier"
command = "/home/your-user/.cargo/bin/voice-notifier-mcp"
enabled = true
```

Use the absolute binary path returned by `command -v voice-notifier-mcp`; MCP clients do not necessarily expand `~` or environment variables in command paths. Restart Ferrum after installing a new binary or changing its configuration.

## Documentation

- [Third-party components and licenses](THIRD_PARTY.md)
- [Project overview and architecture](docs/PROJECT.md)
- [Local TTS research and benchmarks](docs/TTS_RESEARCH.md)
