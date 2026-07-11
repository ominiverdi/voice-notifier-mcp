# Changelog

All notable changes to this project will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-07-11

### Added

- Added `voice_name` selection to `voice_notify`.
- Added dynamic JSON Schema enumeration of installed safe `.bin` voice file stems.
- Added five additional checksum-verified voices to the asset installer: `af_bella`, `af_heart`, `af_nicole`, `af_sarah`, and `am_michael`.
- Added installed-voice discovery, selection, and validation tests.

### Changed

- Changed the default voice path from `bf_emma.bin` to the complete asset directory while preserving single-file overrides.
- Changed the asset installer to preserve and skip files that already match their expected checksums.

## [0.1.2] - 2026-07-11

### Added

- Added automated x86-64 Linux release packaging for portable archives, Debian packages, and RPM packages.
- Added a packaged-binary subprocess smoke test covering all supported MCP protocol versions and tool annotations.
- Added an installed `voice-notifier-install-assets` command to release artifacts.

### Changed

- Enabled release-profile symbol stripping to reduce download size.

## [0.1.1] - 2026-07-11

### Fixed

- Added protocol negotiation compatibility with MCP `2024-11-05` clients.

### Changed

- Documented the glibc 2.38 runtime requirement discovered during clean-container testing.
- Declared and added CI coverage for Rust 1.88 as the minimum supported toolchain.
- Added explicit MCP tool annotations for non-destructive, local notification side effects.
- Verified interoperability with Ferrum, Pi 0.80.3 through its MCP bridge extension, and the official MCP Inspector 0.22.0.

## [0.1.0] - 2026-07-11

### Added

- MCP `voice_notify` tool over standard-input JSON-RPC.
- Independent desktop, terminal-bell, and spoken notification channels.
- Embedded, lazily loaded Kokoro speech through licensed `kokoro-en` 0.1.4.
- `bf_emma` as the default voice and configurable speech speed.
- In-memory PipeWire playback through `pw-play`.
- `spd-say` emergency fallback for neural initialization, synthesis, or playback failures.
- Local asset configuration and checksum-verifying installer.

[Unreleased]: https://github.com/ominiverdi/voice-notifier-mcp/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ominiverdi/voice-notifier-mcp/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/ominiverdi/voice-notifier-mcp/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ominiverdi/voice-notifier-mcp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ominiverdi/voice-notifier-mcp/releases/tag/v0.1.0
