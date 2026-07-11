# Contributing

Contributions are welcome through GitHub issues and pull requests.

## Development setup

Voice Notifier MCP currently targets Linux with PipeWire. Unit tests and static checks do not require model assets or an audio device. A live neural-speech test requires the assets described in the README.

Before submitting a pull request, run:

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
bash -n scripts/install-kokoro-assets.sh
```

Keep MCP-provided text out of shell command strings. Pass external-process arguments directly, keep synthesis local, and preserve `spd-say` as the failure fallback.

## Pull requests

- Keep changes focused and explain user-visible behavior.
- Add tests for protocol and validation changes.
- Update the README or project documentation when interfaces change.
- Do not commit model files, voice files, generated audio, credentials, or machine-specific configuration.

By contributing, you agree that your contribution is licensed under the project's MIT license.
