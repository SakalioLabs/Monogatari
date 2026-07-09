//! Text-to-Speech integration with Windows SAPI and API providers.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub provider: String,
    pub api_url: Option<String>,
    pub api_region: Option<String>,
    pub api_voice_id: Option<String>,
    pub api_key: Option<String>,
    pub default_voice: Option<String>,
    pub language: String,
    pub speed: f32,
    pub pitch: f32,
}
impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: "system".to_string(),
            api_url: None,
            api_region: None,
            api_voice_id: None,
            api_key: None,
            default_voice: None,
            language: "ja".to_string(),
            speed: 1.0,
            pitch: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVoice {
    pub character_id: String,
    pub voice_id: String,
    pub voice_name: String,
    pub language: String,
    pub speed: f32,
    pub pitch: f32,
    pub emotion_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool,
    pub audio_path: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

static TTS_CONFIG: Lazy<RwLock<TtsConfig>> = Lazy::new(|| RwLock::new(TtsConfig::default()));
static CHARACTER_VOICES: Lazy<RwLock<HashMap<String, CharacterVoice>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn tts_output_dir(project_root: &Path) -> PathBuf {
    project_root.join("assets").join("tts")
}

fn tts_output_path(
    project_root: &Path,
    provider: &str,
    character_id: &str,
    timestamp: u64,
    extension: &str,
) -> Result<PathBuf, String> {
    let extension = match extension
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "wav" => "wav",
        "mp3" => "mp3",
        _ => return Err("TTS output extension must be wav or mp3.".to_string()),
    };
    let output_dir = tts_output_dir(project_root);
    let character = safe_tts_file_component(character_id);
    let provider = safe_tts_file_component(provider);
    let path = output_dir.join(format!("{character}_{provider}_{timestamp}.{extension}"));

    if path.parent() != Some(output_dir.as_path()) {
        return Err(
            "TTS output path must stay inside the project assets/tts directory.".to_string(),
        );
    }

    Ok(path)
}

fn safe_tts_file_component(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if ch == '-' || ch == '_' {
            if !last_was_separator {
                output.push(ch);
                last_was_separator = true;
            }
        } else if !last_was_separator {
            output.push('_');
            last_was_separator = true;
        }

        if output.len() >= 64 {
            break;
        }
    }

    let output = output.trim_matches(|ch| ch == '_' || ch == '-').to_string();
    if output.is_empty() {
        "voice".to_string()
    } else {
        output
    }
}

fn ensure_tts_output_parent(output_path: &Path) -> Result<(), String> {
    let Some(parent) = output_path.parent() else {
        return Err("TTS output path has no parent directory.".to_string());
    };
    std::fs::create_dir_all(parent).map_err(|e| format!("TTS output directory failed: {e}"))
}

fn write_tts_output_bytes(
    output_path: &Path,
    bytes: &[u8],
    provider: &str,
) -> Result<String, String> {
    ensure_tts_output_parent(output_path)?;
    std::fs::write(output_path, bytes).map_err(|e| format!("{provider} TTS write failed: {e}"))?;
    Ok(output_path.to_string_lossy().to_string())
}

fn redact_tts_error_text(text: &str) -> String {
    let token_redacted = redact_tts_token_like_values(text);
    redact_tts_secret_assignments(&token_redacted)
}

fn redact_tts_token_like_values(text: &str) -> String {
    let mut redacted = String::with_capacity(text.len());
    let mut cursor = 0;
    while cursor < text.len() {
        if let Some((prefix, min_len)) = tts_token_prefix_at(text, cursor) {
            let token_end = tts_token_end(text, cursor);
            let body_len = text[cursor + prefix.len()..token_end]
                .chars()
                .filter(|ch| is_tts_token_char(*ch))
                .count();
            if body_len >= min_len {
                redacted.push_str("<redacted>");
                cursor = token_end;
                continue;
            }
        }

        let ch = text[cursor..]
            .chars()
            .next()
            .expect("cursor at char boundary");
        redacted.push(ch);
        cursor += ch.len_utf8();
    }

    redacted
}

fn tts_token_prefix_at(text: &str, cursor: usize) -> Option<(&'static str, usize)> {
    let rest = &text[cursor..];
    for (prefix, min_len) in [("github_pat_", 20), ("ghp_", 20), ("sk-", 20)] {
        if rest.starts_with(prefix) {
            return Some((prefix, min_len));
        }
    }
    None
}

fn tts_token_end(text: &str, cursor: usize) -> usize {
    text[cursor..]
        .char_indices()
        .find_map(|(offset, ch)| (!is_tts_token_char(ch)).then_some(cursor + offset))
        .unwrap_or(text.len())
}

fn is_tts_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn redact_tts_secret_assignments(text: &str) -> String {
    let mut redacted = text.to_string();
    for key in [
        "api_key",
        "apiKey",
        "access_token",
        "accessToken",
        "token",
        "secret",
        "password",
        "authorization",
        "x-api-key",
        "api-key",
        "xi-api-key",
        "ocp-apim-subscription-key",
    ] {
        redacted = redact_tts_assignment_values(&redacted, key);
    }
    redacted
}

fn redact_tts_assignment_values(text: &str, key: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(relative) = find_tts_key_case_insensitive(&text[cursor..], key) {
        let key_start = cursor + relative;
        let key_end = key_start + key.len();
        if !is_tts_key_boundary(text, key_start, key_end) {
            result.push_str(&text[cursor..key_end]);
            cursor = key_end;
            continue;
        }

        let Some((value_start, value_end)) = tts_assignment_value_span(text, key_start, key_end)
        else {
            result.push_str(&text[cursor..key_end]);
            cursor = key_end;
            continue;
        };

        result.push_str(&text[cursor..value_start]);
        result.push_str("<redacted>");
        cursor = value_end;
    }

    result.push_str(&text[cursor..]);
    result
}

fn find_tts_key_case_insensitive(text: &str, key: &str) -> Option<usize> {
    text.to_ascii_lowercase().find(&key.to_ascii_lowercase())
}

fn is_tts_key_boundary(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn tts_assignment_value_span(
    text: &str,
    key_start: usize,
    key_end: usize,
) -> Option<(usize, usize)> {
    let mut idx = skip_tts_ws(text, key_end);

    if let Some(quote) = text[..key_start]
        .chars()
        .next_back()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        if text[idx..].starts_with(quote) {
            idx += quote.len_utf8();
            idx = skip_tts_ws(text, idx);
        }
    }

    if let Some(quote) = text[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        idx += quote.len_utf8();
        idx = skip_tts_ws(text, idx);
    }

    let separator = text[idx..].chars().next()?;
    if !matches!(separator, ':' | '=') {
        return None;
    }
    idx += separator.len_utf8();
    idx = skip_tts_ws(text, idx);

    if let Some(quote) = text[idx..]
        .chars()
        .next()
        .filter(|ch| matches!(ch, '"' | '\''))
    {
        let value_start = idx + quote.len_utf8();
        let value_end = text[value_start..]
            .char_indices()
            .find_map(|(offset, ch)| (ch == quote).then_some(value_start + offset))
            .unwrap_or(text.len());
        return Some((value_start, value_end));
    }

    let value_start = idx;
    let value_end = text[value_start..]
        .char_indices()
        .find_map(|(offset, ch)| {
            (ch.is_whitespace() || matches!(ch, '&' | ',' | '}' | ']' | ';'))
                .then_some(value_start + offset)
        })
        .unwrap_or(text.len());
    (value_start < value_end).then_some((value_start, value_end))
}

fn skip_tts_ws(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(start + offset))
        .unwrap_or(text.len())
}

fn tts_provider_error_message(provider: &str, status: reqwest::StatusCode, body: &str) -> String {
    let safe_body = redact_tts_error_text(body);
    let safe_body = safe_body.trim();
    if safe_body.is_empty() {
        format!("{provider} TTS error: {status}")
    } else {
        format!(
            "{provider} TTS error {status}: {}",
            truncate_tts_error_body(safe_body)
        )
    }
}

fn truncate_tts_error_body(value: &str) -> String {
    const MAX_ERROR_CHARS: usize = 500;
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(MAX_ERROR_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[tauri::command]
pub async fn configure_tts(config: TtsConfig) -> Result<String, String> {
    tracing::info!(
        "TTS configured: provider={}, language={}",
        config.provider,
        config.language
    );
    *TTS_CONFIG.write().await = config;
    Ok("TTS configured successfully".to_string())
}

#[tauri::command]
pub async fn set_character_voice(
    _state: State<'_, AppState>,
    voice: CharacterVoice,
) -> Result<String, String> {
    tracing::info!(
        "Voice set for {}: {} ({})",
        voice.character_id,
        voice.voice_name,
        voice.voice_id
    );
    CHARACTER_VOICES
        .write()
        .await
        .insert(voice.character_id.clone(), voice.clone());
    Ok(format!("Voice set for {}", voice.character_id))
}

/// Synthesize speech using Azure Cognitive Services API
async fn azure_tts(
    text: &str,
    api_key: &str,
    region: &str,
    voice: &str,
    output_path: &Path,
) -> Result<String, String> {
    let url = format!(
        "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
        region
    );
    let body = format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US"><voice name="{}">{}</voice></speak>"#,
        voice, text
    );
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Content-Type", "application/ssml+xml")
        .header(
            "X-Microsoft-OutputFormat",
            "audio-16khz-32kbitrate-mono-mp3",
        )
        .body(body)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Azure TTS request failed: {}",
                redact_tts_error_text(&e.to_string())
            )
        })?;
    if resp.status().is_success() {
        let bytes = resp.bytes().await.map_err(|e| {
            format!(
                "Azure TTS read failed: {}",
                redact_tts_error_text(&e.to_string())
            )
        })?;
        write_tts_output_bytes(output_path, &bytes, "Azure")
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(tts_provider_error_message("Azure", status, &body))
    }
}

/// Synthesize speech using ElevenLabs API
async fn elevenlabs_tts(
    text: &str,
    api_key: &str,
    voice_id: &str,
    output_path: &Path,
) -> Result<String, String> {
    let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id);
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"text": text, "model_id": "eleven_monolingual_v1"}))
        .send()
        .await
        .map_err(|e| {
            format!(
                "ElevenLabs request failed: {}",
                redact_tts_error_text(&e.to_string())
            )
        })?;
    if resp.status().is_success() {
        let bytes = resp.bytes().await.map_err(|e| {
            format!(
                "ElevenLabs read failed: {}",
                redact_tts_error_text(&e.to_string())
            )
        })?;
        write_tts_output_bytes(output_path, &bytes, "ElevenLabs")
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(tts_provider_error_message("ElevenLabs", status, &body))
    }
}

#[tauri::command]
pub async fn synthesize_speech(
    state: State<'_, AppState>,
    character_id: String,
    text: String,
    emotion: Option<String>,
) -> Result<TtsResult, String> {
    tracing::info!(
        "TTS synthesis for {}: \"{}\" (emotion: {:?})",
        character_id,
        text,
        emotion
    );
    let config = TTS_CONFIG.read().await.clone();
    let assigned_voice = CHARACTER_VOICES.read().await.get(&character_id).cloned();
    let voice_id = assigned_voice
        .as_ref()
        .map(|voice| voice.voice_id.clone())
        .or_else(|| config.api_voice_id.clone())
        .or_else(|| config.default_voice.clone());
    let project_root = state.current_project_data_root().await;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    match config.provider.as_str() {
        "azure" => {
            if let (Some(api_key), Some(region), Some(voice)) = (
                config.api_key.as_deref(),
                config.api_region.as_deref(),
                voice_id.as_deref(),
            ) {
                let output_path =
                    tts_output_path(&project_root, "azure", &character_id, ts, "mp3")?;
                return match azure_tts(&text, api_key, region, voice, &output_path).await {
                    Ok(path) => Ok(tts_success(path)),
                    Err(error) => Ok(tts_failure(error)),
                };
            }
        }
        "elevenlabs" => {
            if let (Some(api_key), Some(voice)) = (config.api_key.as_deref(), voice_id.as_deref()) {
                let output_path =
                    tts_output_path(&project_root, "elevenlabs", &character_id, ts, "mp3")?;
                return match elevenlabs_tts(&text, api_key, voice, &output_path).await {
                    Ok(path) => Ok(tts_success(path)),
                    Err(error) => Ok(tts_failure(error)),
                };
            }
        }
        _ => {}
    }

    let output_path = tts_output_path(&project_root, "system", &character_id, ts, "wav")?;
    ensure_tts_output_parent(&output_path)?;

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
        match std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps])
            .output()
        {
            Ok(out) if out.status.success() && output_path.exists() => {
                let size = std::fs::metadata(&output_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                return Ok(TtsResult {
                    success: true,
                    audio_path: Some(output_path.to_string_lossy().to_string()),
                    duration_ms: Some(size * 8 / 256),
                    error: None,
                });
            }
            Ok(out) => tracing::warn!("SAPI failed: {}", String::from_utf8_lossy(&out.stderr)),
            Err(e) => tracing::warn!("PowerShell failed: {}", e),
        }
    }
    Ok(TtsResult {
        success: false,
        audio_path: None,
        duration_ms: None,
        error: Some(
            "TTS requires Windows SAPI or API provider. Configure in Settings.".to_string(),
        ),
    })
}

fn tts_success(path: String) -> TtsResult {
    TtsResult {
        success: true,
        audio_path: Some(path),
        duration_ms: None,
        error: None,
    }
}

fn tts_failure(error: String) -> TtsResult {
    TtsResult {
        success: false,
        audio_path: None,
        duration_ms: None,
        error: Some(redact_tts_error_text(&error)),
    }
}

#[cfg(target_os = "windows")]
fn list_system_voices() -> Vec<CharacterVoice> {
    let script = "try { $v = New-Object -ComObject SAPI.SpVoice; $voices = $v.GetVoices(); foreach($vc in $voices) { Write-Output $vc.GetDescription() } } catch { }";
    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .enumerate()
            .map(|(i, name)| CharacterVoice {
                character_id: String::new(),
                voice_id: format!("sapi_{}", i),
                voice_name: name.trim().to_string(),
                language: "en".to_string(),
                speed: 1.0,
                pitch: 1.0,
                emotion_enabled: false,
            })
            .collect(),
        Err(_) => vec![],
    }
}

#[tauri::command]
pub async fn get_available_voices(
    _language: Option<String>,
) -> Result<Vec<CharacterVoice>, String> {
    #[cfg(target_os = "windows")]
    {
        let voices = list_system_voices();
        if !voices.is_empty() {
            return Ok(voices);
        }
    }
    Ok(vec![CharacterVoice {
        character_id: String::new(),
        voice_id: "system_default".to_string(),
        voice_name: "System Default".to_string(),
        language: "en".to_string(),
        speed: 1.0,
        pitch: 1.0,
        emotion_enabled: false,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tts_output_dir_stays_inside_project_assets() {
        let root = PathBuf::from("project-data");
        assert_eq!(tts_output_dir(&root), root.join("assets").join("tts"));
    }

    #[test]
    fn tts_output_path_sanitizes_character_ids_and_stays_in_project_assets() {
        let root = PathBuf::from("project-data");
        let path = tts_output_path(&root, "system", "../Sakura\\evil:role", 42, "wav").unwrap();

        assert_eq!(
            path,
            root.join("assets")
                .join("tts")
                .join("sakura_evil_role_system_42.wav")
        );
        assert_eq!(path.parent(), Some(tts_output_dir(&root).as_path()));
    }

    #[test]
    fn api_provider_tts_outputs_are_project_scoped() {
        let root = PathBuf::from("project-data");
        let azure = tts_output_path(&root, "azure", "Luna", 7, "mp3").unwrap();
        let elevenlabs = tts_output_path(&root, "elevenlabs", "Kenji", 8, ".mp3").unwrap();

        assert_eq!(
            azure,
            root.join("assets").join("tts").join("luna_azure_7.mp3")
        );
        assert_eq!(
            elevenlabs,
            root.join("assets")
                .join("tts")
                .join("kenji_elevenlabs_8.mp3")
        );
    }

    #[test]
    fn tts_output_path_rejects_unsupported_extensions() {
        let root = PathBuf::from("project-data");

        assert!(tts_output_path(&root, "system", "sakura", 1, "../wav").is_err());
        assert!(tts_output_path(&root, "system", "sakura", 1, "txt").is_err());
    }

    #[test]
    fn redacts_tts_provider_error_text() {
        let bearer_key = format!("sk-{}", "A".repeat(24));
        let github_key = format!("{}{}", "github_pat_", "B".repeat(24));
        let plain_secret = "plain-tts-secret";
        let header_secret = "header-secret-value";
        let azure_secret = "azure-header-secret";
        let body = format!(
            r#"{{"error":"bad voice","api_key":"{plain_secret}","url":"https://example.test?access_token={github_key}","Authorization":"Bearer {bearer_key}","xi-api-key":"{header_secret}","Ocp-Apim-Subscription-Key":"{azure_secret}"}}"#
        );

        let redacted = redact_tts_error_text(&body);

        assert!(!redacted.contains(&bearer_key));
        assert!(!redacted.contains(&github_key));
        assert!(!redacted.contains(plain_secret));
        assert!(!redacted.contains(header_secret));
        assert!(!redacted.contains(azure_secret));
        assert!(redacted.contains("bad voice"));
        assert!(redacted.contains("<redacted>"));
        assert!(redacted.contains("access_token=<redacted>"));
    }

    #[test]
    fn tts_failure_redacts_error_surface() {
        let provider_key = format!("sk-{}", "C".repeat(24));
        let result = tts_failure(format!(
            "ElevenLabs request failed: xi-api-key={provider_key}"
        ));

        assert!(!result.success);
        let error = result.error.expect("redacted error");
        assert!(!error.contains(&provider_key));
        assert!(error.contains("xi-api-key=<redacted>"));
    }
}
