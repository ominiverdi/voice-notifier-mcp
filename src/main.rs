mod voice;

use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::process::Command;
use voice::VoiceEngine;

const MAX_SPEECH_CHARS: usize = 500;

const SERVER_NAME: &str = "voice-notifier";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2025-11-25";
const LEGACY_PROTOCOL_VERSION: &str = "2025-06-18";
const INITIAL_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct NotifyArgs {
    message: String,
    #[serde(default = "default_title")]
    title: String,
    #[serde(default = "default_true")]
    desktop: bool,
    #[serde(default)]
    bell: bool,
    #[serde(default)]
    voice: bool,
    #[serde(default = "default_speech_speed")]
    speech_speed: f32,
}

fn default_title() -> String {
    "Ferrum".to_owned()
}

fn default_true() -> bool {
    true
}

fn default_speech_speed() -> f32 {
    1.0
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut voice_engine = VoiceEngine::default();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Invalid JSON-RPC message: {error}");
                continue;
            }
        };

        if let Some(response) = handle_request(&request, &mut voice_engine).await {
            serde_json::to_writer(&mut stdout, &response)?;
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }
    }

    Ok(())
}

async fn handle_request(request: &Value, voice_engine: &mut VoiceEngine) -> Option<Value> {
    let method = request.get("method")?.as_str()?;
    let id = request.get("id")?.clone();

    let result = match method {
        "initialize" => json!({
            "protocolVersion": negotiated_protocol(request),
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        }),
        "ping" => json!({}),
        "tools/list" => json!({
            "tools": [{
                "name": "voice_notify",
                "description": "Send a local desktop, terminal, and optional spoken notification after a task completes or needs attention.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Short notification and spoken message."
                        },
                        "title": {
                            "type": "string",
                            "default": "Ferrum",
                            "description": "Desktop notification title."
                        },
                        "desktop": {
                            "type": "boolean",
                            "default": true,
                            "description": "Send a desktop notification with notify-send."
                        },
                        "bell": {
                            "type": "boolean",
                            "default": false,
                            "description": "Write an audible bell to the MCP server terminal."
                        },
                        "voice": {
                            "type": "boolean",
                            "default": false,
                            "description": "Speak locally with Kokoro using bf_emma by default; fall back to spd-say if neural speech is unavailable."
                        },
                        "speech_speed": {
                            "type": "number",
                            "minimum": 0.5,
                            "maximum": 2.0,
                            "default": 1.0,
                            "description": "Kokoro speech speed multiplier."
                        }
                    },
                    "required": ["message"],
                    "additionalProperties": false
                }
            }]
        }),
        "tools/call" => return Some(handle_tool_call(request, id, voice_engine).await),
        _ => {
            return Some(error_response(
                id,
                -32601,
                format!("Method not found: {method}"),
            ));
        }
    };

    Some(success_response(id, result))
}

fn negotiated_protocol(request: &Value) -> &str {
    match request
        .pointer("/params/protocolVersion")
        .and_then(Value::as_str)
    {
        Some(PROTOCOL_VERSION) => PROTOCOL_VERSION,
        Some(LEGACY_PROTOCOL_VERSION) => LEGACY_PROTOCOL_VERSION,
        Some(INITIAL_PROTOCOL_VERSION) => INITIAL_PROTOCOL_VERSION,
        _ => PROTOCOL_VERSION,
    }
}

async fn handle_tool_call(request: &Value, id: Value, voice_engine: &mut VoiceEngine) -> Value {
    let name = request
        .pointer("/params/name")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if name != "voice_notify" {
        return error_response(id, -32602, format!("Unknown tool: {name}"));
    }

    let arguments = request
        .pointer("/params/arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let args: NotifyArgs = match serde_json::from_value(arguments) {
        Ok(args) => args,
        Err(error) => {
            return tool_result(id, format!("Invalid arguments: {error}"), true);
        }
    };

    if args.message.trim().is_empty() {
        return tool_result(id, "Message must not be empty".to_owned(), true);
    }

    if args.voice && args.message.chars().count() > MAX_SPEECH_CHARS {
        return tool_result(
            id,
            format!("Spoken messages must not exceed {MAX_SPEECH_CHARS} characters"),
            true,
        );
    }

    if !(0.5..=2.0).contains(&args.speech_speed) {
        return tool_result(
            id,
            "speech_speed must be between 0.5 and 2.0".to_owned(),
            true,
        );
    }

    match send_notification(&args, voice_engine).await {
        Ok(summary) => tool_result(id, summary, false),
        Err(error) => tool_result(id, error, true),
    }
}

async fn send_notification(
    args: &NotifyArgs,
    voice_engine: &mut VoiceEngine,
) -> Result<String, String> {
    let mut delivered = Vec::new();
    let mut failures = Vec::new();

    if args.desktop {
        match Command::new("notify-send")
            .arg("--")
            .arg(&args.title)
            .arg(&args.message)
            .status()
        {
            Ok(status) if status.success() => delivered.push("desktop"),
            Ok(status) => failures.push(format!("notify-send exited with {status}")),
            Err(error) => failures.push(format!("notify-send failed: {error}")),
        }
    }

    if args.bell {
        if let Err(error) = write_terminal_bell() {
            failures.push(format!("terminal bell failed: {error}"));
        } else {
            delivered.push("bell");
        }
    }

    if args.voice {
        match voice_engine.speak(&args.message, args.speech_speed).await {
            Ok(()) => delivered.push("voice (Kokoro)"),
            Err(neural_error) => {
                eprintln!("Neural voice failed; using spd-say fallback: {neural_error}");
                match Command::new("spd-say")
                    .arg("--")
                    .arg(&args.message)
                    .status()
                {
                    Ok(status) if status.success() => delivered.push("voice (spd-say fallback)"),
                    Ok(status) => {
                        eprintln!("spd-say fallback exited with {status}");
                        failures.push("voice unavailable".to_owned());
                    }
                    Err(error) => {
                        eprintln!("spd-say fallback failed: {error}");
                        failures.push("voice unavailable".to_owned());
                    }
                }
            }
        }
    }

    if delivered.is_empty() && failures.is_empty() {
        return Err("No notification channel was enabled".to_owned());
    }

    if delivered.is_empty() {
        return Err(failures.join("; "));
    }

    let mut summary = format!("Notification sent via {}", delivered.join(", "));
    if !failures.is_empty() {
        summary.push_str(&format!("; failures: {}", failures.join("; ")));
    }
    Ok(summary)
}

fn write_terminal_bell() -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(b"\x07")?;
    stderr.flush()
}

fn success_response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn tool_result(id: Value, text: String, is_error: bool) -> Value {
    success_response(
        id,
        json!({
            "content": [{"type": "text", "text": text}],
            "isError": is_error
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_voice_notify_tool() {
        let mut voice_engine = VoiceEngine::default();
        let response = handle_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
            &mut voice_engine,
        )
        .await
        .unwrap();

        assert_eq!(
            response.pointer("/result/tools/0/name"),
            Some(&json!("voice_notify"))
        );
        assert_eq!(
            response.pointer("/result/tools/0/inputSchema/required/0"),
            Some(&json!("message"))
        );
    }

    #[tokio::test]
    async fn negotiates_supported_and_unknown_protocol_versions() {
        let mut voice_engine = VoiceEngine::default();
        for (requested, expected) in [
            (INITIAL_PROTOCOL_VERSION, INITIAL_PROTOCOL_VERSION),
            (LEGACY_PROTOCOL_VERSION, LEGACY_PROTOCOL_VERSION),
            (PROTOCOL_VERSION, PROTOCOL_VERSION),
            ("2099-01-01", PROTOCOL_VERSION),
        ] {
            let response = handle_request(
                &json!({
                    "jsonrpc": "2.0",
                    "id": 10,
                    "method": "initialize",
                    "params": {"protocolVersion": requested}
                }),
                &mut voice_engine,
            )
            .await
            .unwrap();

            assert_eq!(
                response.pointer("/result/protocolVersion"),
                Some(&json!(expected))
            );
        }
    }

    #[tokio::test]
    async fn rejects_unknown_tool() {
        let mut voice_engine = VoiceEngine::default();
        let response = handle_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {"name": "missing", "arguments": {}}
            }),
            &mut voice_engine,
        )
        .await
        .unwrap();

        assert_eq!(response.pointer("/error/code"), Some(&json!(-32602)));
    }

    #[tokio::test]
    async fn rejects_empty_message_without_running_commands() {
        let mut voice_engine = VoiceEngine::default();
        let response = handle_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "voice_notify",
                    "arguments": {"message": "", "desktop": false}
                }
            }),
            &mut voice_engine,
        )
        .await
        .unwrap();

        assert_eq!(response.pointer("/result/isError"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn rejects_invalid_speech_speed_without_running_commands() {
        let mut voice_engine = VoiceEngine::default();
        let response = handle_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "voice_notify",
                    "arguments": {
                        "message": "test",
                        "desktop": false,
                        "voice": true,
                        "speech_speed": 2.1
                    }
                }
            }),
            &mut voice_engine,
        )
        .await
        .unwrap();

        assert_eq!(response.pointer("/result/isError"), Some(&json!(true)));
    }
}
