# Project overview

## Purpose

Voice Notifier MCP is a small local Model Context Protocol server that lets an agent notify the desktop user when a long-running task finishes or needs attention.

The project prioritizes:

- fully local operation
- low notification latency
- natural speech for short messages
- a small, auditable Rust service
- graceful fallback when neural speech is unavailable
- no shell interpolation of tool arguments

## Adopted name

The project name is **Voice Notifier MCP** (`voice-notifier-mcp`):

> A local MCP notification server with neural voice, desktop, and terminal channels.

The MCP server is named `voice-notifier`, and its primary tool is `voice_notify`. The name remains independent of Kokoro because speech-engine details should not define the public interface.

## Current implementation

The server exposes `voice_notify` over JSON-RPC on standard input and output. It supports three independent channels:

| Channel | Implementation | Default |
| --- | --- | --- |
| Desktop | `notify-send` | enabled |
| Terminal | ASCII bell written to standard error | disabled |
| Speech | Embedded Kokoro with `bf_emma`; `spd-say` fallback | disabled |

Only the notification message is required. Process arguments are passed directly with `std::process::Command`; user input is not evaluated by a shell.

The neural model loads lazily on the first spoken notification and remains resident in the MCP process. Synthesis uses the Apache-2.0 `kokoro-en` 0.1.4 crate and official Apache-2.0 ONNX assets. Generated 24 kHz float PCM stays in memory and is written directly to `pw-play` over standard input.

```text
Ferrum -> voice-notifier-mcp
              -> embedded Kokoro -> in-memory PCM -> pw-play -> PipeWire
              -> spd-say -> Speech Dispatcher (fallback only)
```

The default assets are under `$XDG_DATA_HOME/voice-notifier-mcp`, or `~/.local/share/voice-notifier-mcp` when `XDG_DATA_HOME` is unset. Model, voice-file, and voice-name environment variables can override these defaults.

The default `ort`/ONNX Runtime artifact currently requires glibc 2.38 or newer. Clean container tests passed on Debian 13 with Rust 1.88 and 1.90, and failed to link on Debian 12 because its glibc 2.36 lacks the required `__isoc23_*` symbols.

## Architecture

Direct embedding is implemented for the initial single-consumer deployment. One persistent MCP process owns the lazily initialized model, avoiding per-notification model loads and a localhost HTTP sidecar. A separate service remains an option if multiple applications later need to share the model or stronger fault isolation becomes necessary.

## Scope relative to existing voice MCP servers

Existing Kokoro MCP projects generally expose speech synthesis as their primary product. For example, [`giannisanni/kokoro-tts-mcp`](https://github.com/giannisanni/kokoro-tts-mcp) generates, saves, and plays speech through either a local Python Kokoro runtime or an OpenAI-compatible TTS endpoint.

Voice Notifier MCP has a narrower orchestration role: an agent reports a completion, question, or failure once, and the server routes that event through enabled attention channels. It should not grow into a general audiobook, voice-cloning, speech-to-text, or audio-production toolkit. Neural TTS remains a replaceable backend rather than the public identity of the project.

```text
agent event -> attention policy -> desktop + bell + local speech -> fallback
```

This boundary avoids duplicating existing TTS MCP servers and keeps routine completion notifications simple and reliable.

## Planned MCP interface extensions

`voice_notify` already supports a `speech_speed` multiplier from 0.5 through 2.0. Future interface extensions may add:

- `voice_name`: raw engine voice name
- `voice_profile`: curated semantic profile such as `clear-us`, `clear-uk`, or `asmr`

A second tool, tentatively named `voice_list`, can return structured voice metadata:

- identifier and display name
- language and accent
- gender presentation
- official quality and training-data grades where available
- locally assigned listening characteristics
- supported speed range
- whether the entry is a native voice, alias, or blend

The normal notification schema should expose a small curated profile list. Returning the complete catalog only from `voice_list` avoids adding unnecessary context to every tool listing.

Initial curated profiles:

| Profile | Kokoro voice | Listening characteristic |
| --- | --- | --- |
| `clear-us` | `af_sarah` | Clear, crisp American voice |
| `clear-uk` | `bf_emma` | Clear, crisp British voice; default |
| `asmr` | `af_nicole` | Soft, close-microphone presentation |
| `expressive` | `af_bella` | Expressive American voice |
| `warm` | `af_heart` | Balanced, warm American voice |

These descriptions combine official model metadata with local listening results. Subjective labels should remain explicitly distinct from upstream quality grades.

## Reliability and security

Required behavior for neural speech integration:

- bind any HTTP service to `127.0.0.1`, not all interfaces
- enforce a 500-character limit before neural synthesis
- use finite connection and synthesis timeouts
- prefer in-memory audio transfer to playback
- if temporary audio is ever required, use restrictive permissions and delete it after playback
- fall back to `spd-say` on service, synthesis, or playback failure
- do not send message content to remote APIs
- do not invoke a shell with MCP-provided text
- avoid exposing model or service internals through JSON-RPC errors

## Machine-specific audio configuration

Initial speech over HDMI lost the beginning of messages because the HDA device entered power-saving states. This was corrected on the development machine with:

- a WirePlumber rule at `~/.config/wireplumber/main.lua.d/51-keep-hdmi-awake.lua`
- `/etc/modprobe.d/99-disable-hda-power-save.conf`
- runtime `snd_hda_intel` values `power_save=0` and `power_save_controller=N`

These settings are host-specific and are intentionally not installed or managed by this repository.

## Status

| Area | Status |
| --- | --- |
| MCP JSON-RPC server | Working |
| Desktop notification | Working |
| Terminal bell | Working |
| Speech Dispatcher fallback | Working |
| HDMI first-word clipping | Corrected on development host |
| Kokoro Rust/ONNX build | Working |
| Kokoro voice listening test | Completed for six voices |
| Resident Kokoro API test | Working on localhost |
| Neural speech integration | Working with licensed `kokoro-en` 0.1.4 |
| Default neural voice | `bf_emma` |
| Neural audio playback | Working through `pw-play` with in-memory PCM |
| Voice catalog MCP tool | Proposed |
| Kokoro embedded Rust benchmark | Completed; currently preferred |
| Pocket TTS Rust/Candle build | Working with local model file |
| Pocket TTS listening comparison | Completed for eight stock voices |
| Pocket TTS embedded Rust benchmark | Completed |
| Chatterbox Nano comparison | Deferred; Kokoro selected |

See [TTS research and evaluation](TTS_RESEARCH.md) for measurements, candidates, and sources.
