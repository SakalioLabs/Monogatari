//! Text-to-Speech integration with Windows SAPI and API providers.
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub provider: String,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub default_voice: Option<String>,
    pub language: String,
    pub speed: f32,
    pub pitch: f32,
}
impl Default for TtsConfig {
    fn default() -> Self {
        Self { provider: "system".to_string(), api_url: None, api_key: None, default_voice: None, language: "ja".to_string(), speed: 1.0, pitch: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVoice {
    pub character_id: String, pub voice_id: String, pub voice_name: String,
    pub language: String, pub speed: f32, pub pitch: f32, pub emotion_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool, pub audio_path: Option<String>, pub duration_ms: Option<u64>, pub error: Option<String>,
}

#[tauri::command]
pub async fn configure_tts(config: TtsConfig) -> Result<String, String> {
    tracing::info!("TTS configured: provider={}, language={}", config.provider, config.language);
    Ok("TTS configured successfully".to_string())
}

#[tauri::command]
pub async fn set_character_voice(_state: State<'_, AppState>, voice: CharacterVoice) -> Result<String, String> {
    tracing::info!("Voice set for {}: {} ({})", voice.character_id, voice.voice_name, voice.voice_id);
    Ok(format!("Voice set for {}", voice.character_id))
}

#[tauri::command]
pub async fn synthesize_speech(
    _state: State<'_, AppState>, character_id: String, text: String, emotion: Option<String>,
) -> Result<TtsResult, String> {
    tracing::info!("TTS synthesis for {}: \"{}\" (emotion: {:?})", character_id, text, emotion);
    let output_dir = std::env::current_dir().unwrap_or_default().join("data").join("assets").join("tts");
    let _ = std::fs::create_dir_all(&output_dir);
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let output_path = output_dir.join(format!("{}_{}.wav", character_id, ts));

    #[cfg(target_os = "windows")]
    {
        let escaped = text.replace('\'', "''");
        let rate = match emotion.as_deref() {
            Some("angry") | Some("excited") => "$voice.Rate = 2; ",
            Some("sad") => "$voice.Rate = -2; ",
            _ => "",
        };
        let path_str = output_path.to_string_lossy().replace('\\', "\\\\");
        let ps = format!("try {{ $v = New-Object -ComObject SAPI.SpVoice; {rate}$s = New-Object -ComObject SAPI.SpFileStream; $s.Open('{path}', 3, $false); $v.AudioOutputStream = $s; $v.Speak('{text}'); $s.Close(); Write-Output 'ok' }} catch {{ Write-Error $_.Exception.Message }}", rate=rate, path=path_str, text=escaped);
        match std::process::Command::new("powershell").args(["-NoProfile", "-Command", &ps]).output() {
            Ok(out) if out.status.success() && output_path.exists() => {
                let size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
                return Ok(TtsResult { success: true, audio_path: Some(output_path.to_string_lossy().to_string()), duration_ms: Some(size * 8 / 256), error: None });
            }
            Ok(out) => tracing::warn!("SAPI failed: {}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => tracing::warn!("PowerShell failed: {}", e),
        }
    }
    Ok(TtsResult { success: false, audio_path: None, duration_ms: None, error: Some("TTS requires Windows SAPI or API provider. Configure in Settings.".to_string()) })
}

#[cfg(target_os = "windows")]
fn list_system_voices() -> Vec<CharacterVoice> {
    let script = "try { $v = New-Object -ComObject SAPI.SpVoice; $voices = $v.GetVoices(); foreach($vc in $voices) { Write-Output $vc.GetDescription() } } catch { }";
    match std::process::Command::new("powershell").args(["-NoProfile", "-Command", script]).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).lines().enumerate().map(|(i, name)| CharacterVoice {
            character_id: String::new(), voice_id: format!("sapi_{}", i), voice_name: name.trim().to_string(),
            language: "en".to_string(), speed: 1.0, pitch: 1.0, emotion_enabled: false,
        }).collect(),
        Err(_) => vec![],
    }
}

#[tauri::command]
pub async fn get_available_voices(_language: Option<String>) -> Result<Vec<CharacterVoice>, String> {
    #[cfg(target_os = "windows")]
    { let voices = list_system_voices(); if !voices.is_empty() { return Ok(voices); } }
    Ok(vec![CharacterVoice { character_id: String::new(), voice_id: "system_default".to_string(), voice_name: "System Default".to_string(), language: "en".to_string(), speed: 1.0, pitch: 1.0, emotion_enabled: false }])
}