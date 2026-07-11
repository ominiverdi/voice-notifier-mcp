# Changelog

All notable changes to this project will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Documented the glibc 2.38 runtime requirement discovered during clean-container testing.
- Declared and added CI coverage for Rust 1.88 as the minimum supported toolchain.

## [0.1.0] - 2026-07-11

### Added

- MCP `voice_notify` tool over standard-input JSON-RPC.
- Independent desktop, terminal-bell, and spoken notification channels.
- Embedded, lazily loaded Kokoro speech through licensed `kokoro-en` 0.1.4.
- `bf_emma` as the default voice and configurable speech speed.
- In-memory PipeWire playback through `pw-play`.
- `spd-say` emergency fallback for neural initialization, synthesis, or playback failures.
- Local asset configuration and checksum-verifying installer.

[Unreleased]: https://github.com/ominiverdi/voice-notifier-mcp/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ominiverdi/voice-notifier-mcp/releases/tag/v0.1.0
