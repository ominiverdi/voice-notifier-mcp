# TTS research and evaluation

## Evaluation goal

Select a fully local speech engine for short completion notifications. The engine should sound substantially more natural than eSpeak while remaining fast enough that speech begins promptly after an agent task finishes.

Primary criteria:

1. naturalness and intelligibility on short messages
2. cold-start and resident synthesis latency
3. time to first audible speech
4. CPU performance and resident memory
5. reliability across punctuation, names, numbers, and technical terms
6. straightforward local integration from Rust
7. permissive licensing
8. continued operation through a simple fallback

Speech Dispatcher through `spd-say` remains the emergency fallback when neural initialization, synthesis, or playback fails.

## Development system

Measurements in this document are local observations, not portable guarantees.

- CPU: AMD Ryzen AI MAX+ 395
- GPU: Radeon 8060S, gfx1151
- Memory: 128 GiB unified memory
- ROCm: 7.1
- Kokoro execution provider tested: ONNX Runtime CPU
- Audio output: desktop PipeWire/PulseAudio path over HDMI

## Candidate summary

### Kokoro 82M

Selected backend for the initial release.

- 82 million parameters
- Apache 2.0 model weights
- 24 kHz output
- 54 voices across eight language groups in v1.0
- 28 English voices: 20 American and 8 British
- no voice cloning
- style embeddings can be blended with weighted voice expressions
- available through Rust and ONNX implementations

Why it fits: high perceived quality for its size, sub-second synthesis of the test notification, a working Rust implementation, and a modest model download.

Important limitation: the upstream voice documentation warns that voices may perform worse on utterances shorter than 10 to 20 tokens. Notifications are often near or below this range, so short-prompt testing is mandatory.

### Pocket TTS

Strong second candidate.

- 100 million parameters
- CPU-focused
- supports English, French, German, Portuguese, Italian, and Spanish in the current upstream release
- audio streaming and voice cloning
- upstream reports roughly 200 ms to the first audio chunk and approximately six times real time on a MacBook Air M4
- uses two CPU cores according to its model card
- provides a stock voice catalog and accepts reference WAV files
- can export processed voice states as `.safetensors` for quick reuse
- official implementation is Python/PyTorch
- community implementations include Rust/Candle, Rust/XN, C++/ONNX Runtime, and sherpa-onnx

Why it remains interesting: voice cloning, low reported streaming latency, and a model size close to Kokoro. The `babybirdprd/pocket-tts` Rust/Candle port includes a native CLI, streaming, and an OpenAI-compatible localhost service, making a Python-free comparison practical.

### Chatterbox Nano

Promising later comparison.

- 110 million parameters
- MIT licensed
- Resemble AI reports three times real-time CPU generation
- zero-shot voice cloning from a reference clip
- supports paralinguistic tags such as laughter, sighs, coughs, and gasps
- generated speech carries Resemble AI's PerTh watermark
- primary distribution uses Python tooling

Why it remains interesting: expressive control and cloning in a small model. These features are less important for completion notifications than Kokoro's simple stock voices, and the Python-first runtime increases integration cost.

### Larger models

Qwen3-TTS, Fish Audio models, and larger Chatterbox variants remain relevant quality references, but their size and runtime complexity are currently disproportionate to short desktop notifications. They should only be tested if compact models fail the quality or pronunciation requirements.

### Benchmark projects

`tts-bench` provides useful listening samples and automated scores. Its reported scores are useful for candidate discovery, but they are not a replacement for local notification tests:

- its prompts and aggregate metrics may not represent very short utterances
- human preference remains decisive for voice character
- hardware and runtime choices affect latency
- WER is more useful as a failure detector than a fine quality ranking

A separate Picovoice on-device TTS benchmark defines useful metrics such as time to first byte, first-token-to-speech, model size, and peak memory. This project will reuse the metric concepts without adopting benchmark dependencies or remote services.

## Kokoro implementation tested

Repository evaluated:

```text
lucasjinreal/Kokoros local checkout (not used in production because it declares no license)
```

Runtime assets:

```text
checkpoints/kokoro-v1.0.onnx   approximately 310 MiB
data/voices-v1.0.bin           approximately 27 MiB
```

Build dependencies added on Ubuntu:

```text
libsonic-dev
libpcaudio-dev
```

The release build succeeded with the local Rust toolchain. The CLI and resident OpenAI-compatible HTTP server both generated valid audio.

## Kokoro measurements

Test text:

```text
Lorenzo, your Ferrum task has finished. The results are ready for review.
```

The test contains 12 words and is representative of the intended use, while still near Kokoro's documented weak range for short utterances.

### CLI synthesis

Each sample used a separate CLI process and the CPU execution provider. The times below are the engine's reported synthesis time, not complete process wall time.

| Voice | Reported synthesis time | Reported words/second |
| --- | ---: | ---: |
| `af_heart` | 499 ms | 24.05 |
| `af_sarah` | 534 ms | 22.48 |
| `af_nicole` | 824 ms | 14.57 |
| `af_sky` | 555 ms | 21.64 |
| `am_michael` | 584 ms | 20.53 |
| `bf_emma` | 497 ms | 24.16 |

These are single observations, not statistically rigorous benchmarks. The samples were generated sequentially, and filesystem cache state differed between the first and later runs.

### Resident HTTP service

A localhost-only Kokoros server was started with one model instance.

| Measurement | Observation |
| --- | ---: |
| Resident RSS | 1,315,288 KiB, approximately 1.25 GiB |
| Warm WAV request 1 | 835 ms total |
| Warm WAV request 2 | 741 ms total |
| Streaming PCM HTTP time to first byte | 29 ms |
| Streaming PCM total request time | 877 ms |

The 29 ms result is HTTP time to first byte, not yet verified time to first audible non-silent sample. Audio-device wake time and playback buffering must be included before treating it as user-perceived latency.

### Listening results

All six initial voices were judged very good.

Local preferences:

- `af_sarah`: clarity and crispness
- `bf_emma`: clarity and crispness
- `af_nicole`: ASMR-like, soft presentation

Official Kokoro voice metadata relevant to these choices:

| Voice | Upstream trait | Target quality | Training duration | Overall grade |
| --- | --- | --- | --- | --- |
| `af_sarah` | American female | B | 1 to 10 hours | C+ |
| `bf_emma` | British female | B | 10 to 100 hours | B- |
| `af_nicole` | American female, headphone trait | B | 10 to 100 hours | B- |
| `af_bella` | American female, expressive trait | A | 10 to 100 hours | A- |
| `af_heart` | American female, heart trait | not separately listed | not separately listed | A |

Upstream grades estimate the quality and quantity of associated training data. They do not fully describe voice character and should not override local listening preferences.

## Kokoro voice inventory

The local v1.0 voice pack contains 54 embeddings.

| Group | Voices |
| --- | ---: |
| American female | 11 |
| American male | 9 |
| British female | 4 |
| British male | 4 |
| Japanese | 5 |
| Mandarin Chinese | 8 |
| Spanish | 3 |
| French | 1 |
| Hindi | 4 |
| Italian | 2 |
| Brazilian Portuguese | 3 |

The OpenAI-compatible server also reports compatibility names such as `sage`, `coral`, and `fable`. These are aliases mapped to Kokoro embeddings, not additional voices.

Kokoros supports weighted blending, for example:

```text
af_sarah.7+af_nicole.3
bf_emma.8+af_nicole.2
```

Cross-accent blending should be treated as experimental because phonemization language remains a separate setting.

## Pocket TTS implementation status

The Python-free [`babybirdprd/pocket-tts`](https://github.com/babybirdprd/pocket-tts) Rust/Candle port was selected for the next local comparison.

Local checkout used for evaluation:

```text
babybirdprd/pocket-tts
commit dbf78c816cd83c89f11dbeb9d87290ad0a3dccd0
version 0.6.2
```

The API-only release CLI built successfully without the optional Node-based web interface:

```text
cargo build --release --no-default-features --package pocket-tts-cli
```

This implementation provides:

- a native Rust CLI and library
- CPU inference through Candle
- stateful audio streaming
- stock voices and voice cloning from WAV or `.safetensors`
- explicit pause syntax
- a localhost HTTP server with an OpenAI-compatible endpoint

The initial automated synthesis attempt reached model initialization but Hugging Face returned HTTP 401 for `tts_b6369a24.safetensors`. After the model access terms were accepted, the 236 MB model was downloaded manually through the authenticated browser. A local test configuration points to that file; the public tokenizer and stock voice embeddings are resolved automatically. No Hugging Face token was exposed or stored by the test harness.

### Pocket TTS measurements

Test text was identical to the Kokoro comparison:

```text
Lorenzo, your Ferrum task has finished. The results are ready for review.
```

The native Rust CLI generated valid 24 kHz mono WAV audio. A localhost-only resident server was then started with all eight stock voices prewarmed.

| Measurement | Observation |
| --- | ---: |
| Initial resident RSS after prewarming | 593,176 KiB, approximately 579 MiB |
| Resident RSS after generation tests | 616,820 KiB, approximately 602 MiB |
| First observed warm WAV request | 2.026 seconds |
| Ten warm WAV requests, mean | 1.561 seconds |
| Ten warm WAV requests, minimum | 1.303 seconds |
| Ten warm WAV requests, maximum | 1.793 seconds |
| `alba` sample duration | 5.08 seconds |

Pocket TTS used less than half the resident memory measured for the Kokoros server, but whole-file response generation was approximately twice as slow as Kokoro for this prompt. The `/stream` endpoint returned HTTP response headers immediately and completed in 1.953 seconds, but time to the first non-silent PCM body chunk was not yet instrumented. Header time alone must not be reported as audible latency.

All eight stock voices were generated and played in this order:

1. `alba`
2. `marius`
3. `javert`
4. `jean`
5. `fantine`
6. `cosette`
7. `eponine`
8. `azelma`

Sample durations varied from 4.28 to 8.60 seconds. `javert` was the 8.60-second outlier and should be checked for an abnormal pause, repetition, or other stochastic generation artifact. Subjective preference feedback is pending.

The resident test server was stopped after measurement.

## Embedded Rust comparison

A separate local Rust benchmark workspace was created for this comparison.

It links directly to the local Kokoros library crate and the Pocket TTS Rust/Candle crate. No HTTP server, subprocess synthesis, or Python runtime is involved. Each benchmark:

- starts three independent fresh processes per engine
- loads one model and one voice in each process
- performs one first synthesis per process
- performs 20 sequential warm syntheses per process, 60 warm runs per engine in total
- keeps generated audio in memory during timing
- writes only the first sample after its timed synthesis
- reads resident and high-water memory from `/proc/self/status`

Both engines synthesized the same 12-word notification at 24 kHz. Kokoro used `af_sarah`; Pocket TTS used `alba`.

| Measurement | Embedded Kokoro | Embedded Pocket TTS |
| --- | ---: | ---: |
| Model-load range, 3 processes | 430–827 ms | 151–274 ms |
| Separate voice-load range | included above | 150–265 ms |
| Load through first completed audio | 0.941–1.417 s | 1.672–2.204 s |
| First-synthesis range | 511–590 ms | 1.371–1.665 s |
| Warm mean, 60 runs | 463 ms | 1.434 s |
| Per-process warm median range | 461–466 ms | 1.366–1.421 s |
| Per-process warm 95th-percentile range | 493–502 ms | 1.559–1.660 s |
| Warm minimum across all runs | 407 ms | 1.255 s |
| Warm maximum across all runs | 509 ms | 1.841 s |
| Mean generated duration | 5.325 s | 4.685 s |
| Approximate real-time factor | 0.087 | 0.306 |
| Approximate synthesis speed | 11.5 times real time | 3.3 times real time |
| RSS after model and voice load | approximately 417 MiB | approximately 497 MiB |
| Final RSS | approximately 446 MiB | approximately 507 MiB |
| Peak RSS range | 778–795 MiB | 708–709 MiB |

Across three independent processes, embedded Kokoro was approximately 3.1 times faster during warm synthesis. Pocket TTS consistently loaded its model and voice faster and had an approximately 70–87 MiB lower peak RSS, but retained about 61 MiB more memory after repeated synthesis.

The earlier 1.25 GiB Kokoros HTTP measurement is not representative of a single embedded model. The tested CLI server constructs an initial TTS object before dispatching its mode and then constructs an additional instance for OpenAI server mode. Direct embedding retained only one model and reduced steady RSS to approximately 446 MiB.

Both embedded outputs were played and retained the quality observed in earlier tests. These results currently favor direct Kokoro integration on quality preference, warm latency, and steady resident memory. Pocket TTS remains valuable as a lower-peak-memory, voice-cloning alternative.

The specific Kokoros checkout used for the original benchmark has no `LICENSE` file and its crate manifest declares no license. It remains suitable for local comparison but is not used by Voice Notifier MCP. Licensed Rust alternatives found on crates.io included `kokoro-en` 0.1.4 (Apache 2.0), `hematite-kokoros` 0.1.3 (Apache 2.0), `kokoro-ort` 0.1.0 (MIT), and `kokoroxide` 0.1.5 (MIT or Apache 2.0).

### Licensed `kokoro-en` benchmark

`kokoro-en` 0.1.4 was tested with the official full-precision `model.onnx` and `bf_emma.bin` assets. Both the crate and model weights are Apache-2.0. The crate's default bundled-eSpeak feature failed to link on the test host because its static eSpeak build omitted Sonic and audio symbols. The supported `misaki-lean` feature built cleanly and produced intelligible Emma output, so the integration pins 0.1.4 with default features disabled.

Three fresh processes each performed one first synthesis and 20 resident syntheses of the same 12-word notification:

| Measurement | `kokoro-en` 0.1.4 with `bf_emma` |
| --- | ---: |
| Model-load range | 763–972 ms |
| First-synthesis range | 914–1,081 ms |
| Warm mean, 60 runs | 384 ms |
| Per-process warm median range | 362–364 ms |
| Per-process warm 95th-percentile range | 373–542 ms |
| Warm minimum across all runs | 350 ms |
| Warm maximum across all runs | 1,350 ms |
| Generated duration | 5.100 s |
| RSS after model and voice load | approximately 439 MiB |
| Final and peak RSS | approximately 769–775 MiB |

One 1.35-second outlier raised the aggregate mean; 59 of 60 runs were at or below 542 ms, and typical medians were approximately 363 ms. Compared with the original unlicensed Kokoros baseline, this implementation was typically faster but retained substantially more memory after repeated inference. The licensed implementation was selected because it met local quality and latency requirements while providing explicit Apache-2.0 licensing.

The production path now keeps one `KokoroTts` instance in the MCP process, loads it and all installed voice files lazily, defaults to `bf_emma`, accepts dynamically discovered voice names, sends in-memory float PCM to PipeWire, and falls back to `spd-say` when initialization, synthesis, or playback fails.

Benchmark validation completed successfully:

```text
cargo test --release
cargo fmt --check
cargo clippy --release -- -D warnings
```

## Next test matrix

### Voice and blend selection

Generate and listen to:

- `af_sarah`
- `bf_emma`
- `af_nicole`
- `af_bella`
- `af_heart`
- `af_sarah.7+af_nicole.3`
- `bf_emma.8+bf_alice.2`

Test speed multipliers of 0.95, 1.00, 1.05, and 1.10. Avoid changing voice and speed simultaneously during subjective comparisons.

### Prompt corpus

Use identical prompts across engines:

1. Completion: `Lorenzo, your Ferrum task has finished.`
2. Review: `The implementation is complete and ready for review.`
3. Failure: `The build failed. Check the compiler output.`
4. Numbers: `Completed 127 tests in 42.6 seconds.`
5. Technical terms: `The JSON-RPC and PostgreSQL checks passed.`
6. Paths: `The report is in docs slash TTS research dot M D.`
7. Acronyms: `The MCP, API, and CPU benchmarks are complete.`
8. Very short: `Task complete.`
9. Punctuation: `Finished: build, tests, lint, and documentation.`
10. Longer message: a 40-to-60-word completion summary.

Record pronunciation failures rather than relying only on overall preference.

### Performance protocol

For each engine:

- measure at least five cold starts
- measure at least twenty resident requests
- report median and 95th percentile
- record model-load time separately from synthesis
- record peak RSS and idle RSS
- measure time to first non-silent audio sample at the playback boundary
- test concurrent notification behavior
- test service unavailability and fallback latency

### Integration acceptance criteria

A candidate is ready for default use when:

- short messages are consistently intelligible
- median resident synthesis is below one second on the development host
- speech begins without clipping
- no network access is required after assets are installed
- service failure falls back to `spd-say`
- MCP text is never passed through a shell
- resource usage is acceptable while idle

## Current implementation

Voice Notifier MCP embeds licensed `kokoro-en` 0.1.4 directly in its persistent process. It loads the model and installed voice directory lazily on the first spoken notification, publishes installed safe `.bin` file stems dynamically in the `voice_name` tool schema, prefers `bf_emma` by default, supports speed control, and preserves `spd-say` as the fallback.

The embedded design removes the need for a separate TTS service for the initial single-consumer release. A localhost service may be reconsidered if additional applications need to share one model instance or independent crash/restart isolation becomes more important than deployment simplicity.

Future evaluation should expand the pronunciation corpus, test concurrent notification policy, and measure time to the first non-silent sample at the playback boundary. Chatterbox Nano remains a possible comparison if voice cloning or expressive tags become requirements.

## Sources

Primary sources consulted:

- Kokoro model card: <https://huggingface.co/hexgrad/Kokoro-82M>
- Kokoro voice metadata: <https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md>
- Kokoros Rust implementation: <https://github.com/lucasjinreal/Kokoros>
- Official Kokoro ONNX assets: <https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX>
- Pocket TTS model card: <https://huggingface.co/kyutai/pocket-tts>
- Pocket TTS Rust/Candle port: <https://github.com/babybirdprd/pocket-tts>
- Pocket TTS repository: <https://github.com/kyutai-labs/pocket-tts>
- Chatterbox Nano overview: <https://www.resemble.ai/learn/models/chatterbox-nano>
- Chatterbox repository: <https://github.com/resemble-ai/chatterbox>
- tts-bench scores: <https://5uck1ess.github.io/tts-bench/scores.html>
- Picovoice on-device TTS benchmark: <https://github.com/Picovoice/text-to-speech-benchmark>

Third-party benchmark claims are recorded as research leads. Only measurements explicitly identified as local observations were produced on this development machine.
