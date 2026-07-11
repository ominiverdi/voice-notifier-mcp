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
- approximately 314 MB for the Kokoro model and bundled voice selection

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

The asset installer downloads the official model and six voices, verifies their SHA-256 hashes, and places them under `${XDG_DATA_HOME:-$HOME/.local/share}/voice-notifier-mcp`. Existing verified assets are not downloaded again. Once installed, synthesis is fully local.

Expected SHA-256 values for the installed assets:

```text
8fbea51ea711f2af382e88c833d9e288c6dc82ce5e98421ea61c058ce21a34cb  model.onnx
f69d836209b78eb8c66e75e3cda491e26ea838a3674257e9d4e5703cbaf55c8b  af_bella.bin
d583ccff3cdca2f7fae535cb998ac07e9fcb90f09737b9a41fa2734ec44a8f0b  af_heart.bin
cd2191ab31b914ed7b318416b0e4440fdf392ddad9106a060819aa600a64f59a  af_nicole.bin
4409fbc125afabacc615d94db5398d847006a737b0247d6892b7a9a0007a2f0a  af_sarah.bin
1d1f21dd8da39c30705cd4c75d039d265e9bc4a2a93ed09bc9e1b1225eb95ba1  am_michael.bin
669fe0647f9dd04fcab92f1439a40eeb4c8b4ab1f82e4996fe3d918ce4a63b73  bf_emma.bin
```

### Add extra voices

The installer intentionally provides a small verified selection. To add another compatible Kokoro voice, obtain its raw little-endian float32 `.bin` embedding from a trusted source and copy it into the asset directory:

```bash
asset_dir="${XDG_DATA_HOME:-$HOME/.local/share}/voice-notifier-mcp"
install -d -m 0755 "$asset_dir"
install -m 0644 /path/to/custom_voice.bin "$asset_dir/custom_voice.bin"
```

The filename stem becomes the `voice_name` value and must contain only ASCII letters, digits, `_`, or `-`. PyTorch `.pt` voice files are not directly supported; obtain or convert the corresponding raw `.bin` embedding rather than merely renaming the file. Verify the source, license, and checksum before installation. Restart every MCP client after adding, replacing, or removing voices so it reloads both the tool schema and voice data.

Override the defaults with `VOICE_NOTIFIER_MODEL_PATH`, `VOICE_NOTIFIER_VOICE_PATH`, and `VOICE_NOTIFIER_VOICE`. `VOICE_NOTIFIER_VOICE_PATH` may name one `.bin` file or a directory containing voice files. The server lists safe `.bin` file stems dynamically in the `voice_name` schema; compatible custom embeddings require no catalog update.

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
  "voice_name": "af_heart",
  "speech_speed": 1.0
}
```

Only `message` is required. `voice_name` selects any installed compatible `.bin` voice exposed by the tool schema and defaults to `bf_emma` when available. `speech_speed` accepts 0.5 through 2.0 and defaults to 1.0. Spoken messages are limited to 500 Unicode characters. Process arguments are passed directly without a shell.

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
