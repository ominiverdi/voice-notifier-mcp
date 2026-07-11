use kokoro_en::{KokoroTts, Voice};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
    loaded_voices: Option<Vec<String>>,
}

impl VoiceEngine {
    pub fn available_voices(&self) -> Result<Vec<String>, String> {
        if let Some(voices) = &self.loaded_voices {
            return Ok(voices.clone());
        }
        available_voice_names_at(&voice_path()?)
    }

    pub fn resolve_voice(&self, requested: Option<&str>) -> Result<String, String> {
        let voices = self.available_voices()?;
        let configured = env::var("VOICE_NOTIFIER_VOICE").ok();
        select_voice(&voices, requested, configured.as_deref())
    }

    pub async fn speak(
        &mut self,
        message: &str,
        speed: f32,
        voice_name: &str,
    ) -> Result<(), String> {
        if self.tts.is_none() {
            let (model_path, voices_path) = asset_paths()?;
            let voices = available_voice_names_at(&voices_path)?;
            if !voices.iter().any(|voice| voice == voice_name) {
                return Err(format!("Voice '{voice_name}' is no longer available"));
            }
            let tts = timeout(
                LOAD_TIMEOUT,
                KokoroTts::new(model_path.as_path(), voices_path.as_path()),
            )
            .await
            .map_err(|_| "Kokoro model loading timed out".to_owned())?
            .map_err(|error| format!("Kokoro model loading failed: {error}"))?;
            self.tts = Some(tts);
            self.loaded_voices = Some(voices);
        }

        if !self
            .loaded_voices
            .as_ref()
            .is_some_and(|voices| voices.iter().any(|voice| voice == voice_name))
        {
            return Err(format!(
                "Voice '{voice_name}' was not loaded; restart the MCP server after changing voice files"
            ));
        }

        let tts = self.tts.as_ref().expect("Kokoro initialized above");
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
    let voices_path = env::var_os("VOICE_NOTIFIER_VOICE_PATH")
        .map(PathBuf::from)
        .unwrap_or(default_dir);
    Ok((model_path, voices_path))
}

fn voice_path() -> Result<PathBuf, String> {
    asset_paths().map(|(_, voices)| voices)
}

fn available_voice_names_at(path: &Path) -> Result<Vec<String>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not inspect voice path {}: {error}", path.display()))?;
    let mut voices = Vec::new();

    if metadata.is_dir() {
        let entries = fs::read_dir(path).map_err(|error| {
            format!("Could not read voice directory {}: {error}", path.display())
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!("Could not read an entry in {}: {error}", path.display())
            })?;
            let file_type = entry.file_type().map_err(|error| {
                format!("Could not inspect {}: {error}", entry.path().display())
            })?;
            if file_type.is_file()
                && let Some(name) = voice_name_from_path(&entry.path())
            {
                voices.push(name);
            }
        }
    } else if metadata.is_file()
        && let Some(name) = voice_name_from_path(path)
    {
        voices.push(name);
    }

    voices.sort_unstable();
    voices.dedup();
    Ok(voices)
}

fn voice_name_from_path(path: &Path) -> Option<String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
        return None;
    }
    let name = path.file_stem()?.to_str()?;
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return None;
    }
    Some(name.to_owned())
}

fn select_voice(
    voices: &[String],
    requested: Option<&str>,
    configured: Option<&str>,
) -> Result<String, String> {
    if voices.is_empty() {
        return Err("No Kokoro voice files are installed".to_owned());
    }

    if let Some(name) = requested.or(configured) {
        if voices.iter().any(|available| available == name) {
            return Ok(name.to_owned());
        }
        return Err(format!(
            "Voice '{name}' is not installed; available voices: {}",
            voices.join(", ")
        ));
    }

    if voices.iter().any(|voice| voice == DEFAULT_VOICE) {
        Ok(DEFAULT_VOICE.to_owned())
    } else {
        Ok(voices[0].clone())
    }
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_asset_paths_are_local_user_data() {
        if env::var_os("VOICE_NOTIFIER_MODEL_PATH").is_none()
            && env::var_os("VOICE_NOTIFIER_VOICE_PATH").is_none()
        {
            let (model, voices) = asset_paths().unwrap();
            assert!(model.ends_with("voice-notifier-mcp/model.onnx"));
            assert!(voices.ends_with("voice-notifier-mcp"));
        }
    }

    #[test]
    fn discovers_sorted_safe_voice_file_stems() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "voice-notifier-mcp-voices-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        for name in [
            "bf_emma.bin",
            "af_heart.bin",
            "custom-1.bin",
            "not-a-voice.txt",
            "unsafe voice.bin",
        ] {
            fs::write(directory.join(name), []).unwrap();
        }
        fs::create_dir(directory.join("ignored.bin")).unwrap();

        let voices = available_voice_names_at(&directory).unwrap();
        assert_eq!(voices, ["af_heart", "bf_emma", "custom-1"]);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn selects_requested_configured_and_default_voices() {
        let voices = vec!["af_heart".to_owned(), "bf_emma".to_owned()];
        assert_eq!(
            select_voice(&voices, Some("af_heart"), Some("bf_emma")).unwrap(),
            "af_heart"
        );
        assert_eq!(
            select_voice(&voices, None, Some("af_heart")).unwrap(),
            "af_heart"
        );
        assert_eq!(select_voice(&voices, None, None).unwrap(), "bf_emma");
        assert!(select_voice(&voices, Some("missing"), None).is_err());
        assert!(select_voice(&[], None, None).is_err());
    }

    #[test]
    fn discovers_voice_from_single_file_path() {
        let path = Path::new("/tmp/custom_voice.bin");
        assert_eq!(voice_name_from_path(path).as_deref(), Some("custom_voice"));
        assert_eq!(voice_name_from_path(Path::new("/tmp/voice.wav")), None);
    }
}
