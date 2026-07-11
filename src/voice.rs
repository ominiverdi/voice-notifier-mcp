use kokoro_en::{KokoroTts, Voice};
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_VOICE: &str = "bf_emma";
const LOAD_TIMEOUT: Duration = Duration::from_secs(10);
const SYNTHESIS_TIMEOUT: Duration = Duration::from_secs(15);
const PLAYBACK_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct VoiceEngine {
    tts: Option<KokoroTts>,
}

impl VoiceEngine {
    pub async fn speak(&mut self, message: &str, speed: f32) -> Result<(), String> {
        if self.tts.is_none() {
            let (model_path, voice_path) = asset_paths()?;
            let tts = timeout(
                LOAD_TIMEOUT,
                KokoroTts::new(model_path.as_path(), voice_path.as_path()),
            )
            .await
            .map_err(|_| "Kokoro model loading timed out".to_owned())?
            .map_err(|error| format!("Kokoro model loading failed: {error}"))?;
            self.tts = Some(tts);
        }

        let tts = self.tts.as_ref().expect("Kokoro initialized above");
        let voice_name =
            env::var("VOICE_NOTIFIER_VOICE").unwrap_or_else(|_| DEFAULT_VOICE.to_owned());
        let (audio, _) = timeout(
            SYNTHESIS_TIMEOUT,
            tts.synth(message, Voice::new(voice_name).with_speed(speed)),
        )
        .await
        .map_err(|_| "Kokoro synthesis timed out".to_owned())?
        .map_err(|error| format!("Kokoro synthesis failed: {error}"))?;

        play_audio(&audio).await
    }
}

fn asset_paths() -> Result<(PathBuf, PathBuf), String> {
    let data_dir = if let Some(path) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(path)
    } else {
        let home = env::var_os("HOME").ok_or_else(|| {
            "HOME is unset; configure VOICE_NOTIFIER_MODEL_PATH and VOICE_NOTIFIER_VOICE_PATH"
                .to_owned()
        })?;
        PathBuf::from(home).join(".local/share")
    };
    let default_dir = data_dir.join("voice-notifier-mcp");
    let model_path = env::var_os("VOICE_NOTIFIER_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_dir.join("model.onnx"));
    let voice_path = env::var_os("VOICE_NOTIFIER_VOICE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_dir.join("bf_emma.bin"));
    Ok((model_path, voice_path))
}

async fn play_audio(audio: &[f32]) -> Result<(), String> {
    let mut child = Command::new("pw-play")
        .args(["--format", "f32", "--rate", "24000", "--channels", "1", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("pw-play failed to start: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "pw-play stdin was unavailable".to_owned())?;
    let mut pcm = Vec::with_capacity(std::mem::size_of_val(audio));
    for sample in audio {
        pcm.extend_from_slice(&sample.to_ne_bytes());
    }
    stdin
        .write_all(&pcm)
        .await
        .map_err(|error| format!("pw-play input failed: {error}"))?;
    drop(stdin);

    let output = timeout(PLAYBACK_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| "pw-play timed out".to_owned())?
        .map_err(|error| format!("pw-play failed: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("pw-play exited with {}", output.status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_asset_paths_are_local_user_data() {
        if env::var_os("VOICE_NOTIFIER_MODEL_PATH").is_none()
            && env::var_os("VOICE_NOTIFIER_VOICE_PATH").is_none()
        {
            let (model, voice) = asset_paths().unwrap();
            assert!(model.ends_with("voice-notifier-mcp/model.onnx"));
            assert!(voice.ends_with("voice-notifier-mcp/bf_emma.bin"));
        }
    }
}
