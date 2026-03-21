use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::i18n::Language;

const DEFAULT_CONFIG_FILE_NAME: &str = "config.json";
const APP_SETTINGS_FILE_NAME: &str = "settings.json";
const IMPORTED_CONFIG_DIR_NAME: &str = "ftl";

#[cfg(target_os = "macos")]
fn app_config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Pearl Calculator"),
    )
}

#[cfg(target_os = "windows")]
fn app_config_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(appdata).join("Pearl Calculator"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn app_config_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("pearl-calculator"));
    }
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join("pearl-calculator"))
}

fn app_settings_file_path() -> Option<PathBuf> {
    Some(app_config_dir()?.join(APP_SETTINGS_FILE_NAME))
}

fn imported_config_dir_path() -> Option<PathBuf> {
    Some(app_config_dir()?.join(IMPORTED_CONFIG_DIR_NAME))
}

fn load_app_settings() -> Map<String, Value> {
    let Some(path) = app_settings_file_path() else {
        return Map::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Map::new();
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

fn save_app_settings(root: &Map<String, Value>) -> Result<(), String> {
    let path =
        app_settings_file_path().ok_or_else(|| "config directory is unavailable".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let value = Value::Object(root.clone());
    let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub(crate) fn ensure_store_layout() -> Result<(), String> {
    let config_dir =
        app_config_dir().ok_or_else(|| "config directory is unavailable".to_string())?;
    let imported_dir = config_dir.join(IMPORTED_CONFIG_DIR_NAME);
    std::fs::create_dir_all(&imported_dir).map_err(|e| e.to_string())?;

    let default_config = config_dir.join(DEFAULT_CONFIG_FILE_NAME);
    if !default_config.exists() {
        std::fs::write(default_config, "").map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn load_language() -> Option<Language> {
    let root = load_app_settings();
    let language = root.get("language")?.as_str()?;
    Language::from_code(language)
}

pub(crate) fn save_language(language: Language) -> Result<(), String> {
    let mut root = load_app_settings();
    root.insert(
        "language".to_string(),
        Value::String(language.code().to_string()),
    );
    save_app_settings(&root)
}

pub(crate) fn load_selected_config() -> Option<String> {
    let root = load_app_settings();
    root.get("selected_config")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub(crate) fn save_selected_config(selected: Option<&str>) -> Result<(), String> {
    let mut root = load_app_settings();
    match selected {
        Some(value) => {
            root.insert(
                "selected_config".to_string(),
                Value::String(value.to_string()),
            );
        }
        None => {
            root.insert("selected_config".to_string(), Value::String(String::new()));
        }
    }
    save_app_settings(&root)
}

pub(crate) fn list_imported_configs() -> Result<Vec<String>, String> {
    let dir =
        imported_config_dir_path().ok_or_else(|| "config directory is unavailable".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            result.push(name.to_string());
        }
    }
    result.sort();
    Ok(result)
}

pub(crate) fn imported_config_exists(file_name: &str) -> bool {
    let Some(dir) = imported_config_dir_path() else {
        return false;
    };
    dir.join(file_name).exists()
}

pub(crate) fn imported_config_file_path(file_name: &str) -> Option<PathBuf> {
    Some(imported_config_dir_path()?.join(file_name))
}

pub(crate) fn import_config_file_as(
    source_path: &std::path::Path,
    file_name: &str,
    overwrite: bool,
) -> Result<String, String> {
    let dir =
        imported_config_dir_path().ok_or_else(|| "config directory is unavailable".to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let normalized_name = if file_name.ends_with(".json") {
        file_name.to_string()
    } else {
        format!("{file_name}.json")
    };
    let target = dir.join(&normalized_name);
    if target.exists() && !overwrite {
        return Err("target config already exists".to_string());
    }
    std::fs::copy(source_path, target).map_err(|e| e.to_string())?;
    Ok(normalized_name)
}
