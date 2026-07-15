use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "settings.json";

/// On-disk shape. Never serialized back to the frontend as-is — the raw
/// `api_key` must not be reachable from webview JS (see AppSettingsView).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoredSettings {
    pub api_key: Option<String>,
    pub input_language: String,
    pub output_language: String,
    pub theme: String,
    pub preferred_input_device: Option<String>,
    pub preferred_output_device: Option<String>,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            api_key: None,
            input_language: "ml".to_string(),
            output_language: "en".to_string(),
            theme: "dark".to_string(),
            preferred_input_device: None,
            preferred_output_device: None,
        }
    }
}

/// Write-only partial update sent from the frontend; any field left `None`
/// is left unchanged on disk.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsInput {
    pub api_key: Option<String>,
    pub theme: Option<String>,
    pub input_language: Option<String>,
    pub output_language: Option<String>,
    pub preferred_input_device: Option<String>,
    pub preferred_output_device: Option<String>,
}

/// Safe view returned to the frontend — deliberately excludes the raw API key.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsView {
    pub has_api_key: bool,
    pub theme: String,
    pub input_language: String,
    pub output_language: String,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("failed to resolve app config dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create app config dir: {e}"))?;
    Ok(dir.join(SETTINGS_FILE))
}

pub fn load(app: &AppHandle) -> StoredSettings {
    let Ok(path) = settings_path(app) else {
        return StoredSettings::default();
    };
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => StoredSettings::default(),
    }
}

pub fn save(app: &AppHandle, settings: &StoredSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let contents = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, contents).map_err(|e| format!("failed to write settings: {e}"))
}

/// Used internally by session.rs to authenticate with OpenAI. Never exposed
/// as a Tauri command.
pub fn get_api_key(app: &AppHandle) -> Option<String> {
    load(app).api_key.filter(|k| !k.is_empty())
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettingsView, String> {
    let settings = load(&app);
    Ok(AppSettingsView {
        has_api_key: settings.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
        theme: settings.theme,
        input_language: settings.input_language,
        output_language: settings.output_language,
    })
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettingsInput) -> Result<(), String> {
    let mut current = load(&app);
    if let Some(api_key) = settings.api_key {
        current.api_key = Some(api_key);
    }
    if let Some(theme) = settings.theme {
        current.theme = theme;
    }
    if let Some(input_language) = settings.input_language {
        current.input_language = input_language;
    }
    if let Some(output_language) = settings.output_language {
        current.output_language = output_language;
    }
    if let Some(device) = settings.preferred_input_device {
        current.preferred_input_device = Some(device);
    }
    if let Some(device) = settings.preferred_output_device {
        current.preferred_output_device = Some(device);
    }
    save(&app, &current)
}
