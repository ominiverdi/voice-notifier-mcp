# Third-party components

Voice Notifier MCP does not bundle Kokoro model or voice assets. Users download them separately from their upstream distributor.

| Component | Version or asset | License | Source |
| --- | --- | --- | --- |
| `kokoro-en` | 0.1.4 | Apache-2.0 | <https://crates.io/crates/kokoro-en/0.1.4> |
| Kokoro 82M ONNX model | `onnx/model.onnx` | Apache-2.0 | <https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX> |
| Kokoro voice embeddings | `af_bella`, `af_heart`, `af_nicole`, `af_sarah`, `am_michael`, and `bf_emma` | Apache-2.0 | <https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX> |

The complete Rust dependency graph and exact versions are recorded in `Cargo.lock`. Binary distributors are responsible for satisfying all applicable dependency-license notice requirements.
