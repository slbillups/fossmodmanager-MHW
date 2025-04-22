use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameData {
    pub game_root_path: String,
    pub game_executable_path: String,
}

pub fn find_game_paths_from_exe(executable_path_str: &str) -> Result<(PathBuf, PathBuf), String> {
    let executable_path = PathBuf::from(executable_path_str);

    if !executable_path.is_file() {
        return Err(format!(
            "Provided path is not a file or does not exist: {}",
            executable_path_str
        ));
    }

    let mut current_path = executable_path.parent().ok_or_else(|| {
        format!(
            "Could not get parent directory of executable: {}",
            executable_path_str
        )
    })?;

    loop {
        let parent_path = current_path.parent().ok_or_else(|| {
            format!(
                "Reached filesystem root without finding 'steamapps/common' structure starting from: {}",
                executable_path_str
            )
        })?;

        let parent_dir_name = parent_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Could not get parent directory name for: {:?}", parent_path))?;

        if parent_dir_name == "common" {
            let grandparent_path = parent_path.parent().ok_or_else(|| {
                format!(
                    "Found 'common' but no parent directory above it: {:?}",
                    parent_path
                )
            })?;

            let grandparent_dir_name = grandparent_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    format!(
                        "Could not get grandparent directory name for: {:?}",
                        grandparent_path
                    )
                })?;

            if grandparent_dir_name == "steamapps" {
                return Ok((current_path.to_path_buf(), grandparent_path.to_path_buf()));
            }
        }

        if current_path == parent_path {
            return Err(format!(
                "Path resolution stopped unexpectedly at: {:?}. Could not find 'steamapps/common' structure.",
                current_path
            ));
        }

        current_path = parent_path;
    }
}

// Write config to userconfig.json
#[tauri::command]
pub async fn finalize_setup(
    window: WebviewWindow,
    app_handle: AppHandle,
    executable_path: String,
) -> Result<(), String> {
    let (game_root_path_buf, _) = find_game_paths_from_exe(&executable_path)?;
    let game_root_path_str = game_root_path_buf
        .to_str()
        .ok_or("Game root path contains invalid UTF-8")?
        .to_string();

    let fossmodmanager_path = game_root_path_buf.join("fossmodmanager/mods");
    fs::create_dir_all(&fossmodmanager_path)
        .map_err(|e| format!("Failed to create mods directory {:?}: {}", fossmodmanager_path, e))?;

    let config_path = get_config_path(&app_handle)?;
    fs::create_dir_all(config_path.parent().unwrap())
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let game_data = GameData {
        game_root_path: game_root_path_str.clone(),
        game_executable_path: executable_path.clone(),
    };

    fs::write(&config_path, serde_json::to_string_pretty(&game_data)
        .map_err(|e| e.to_string())?)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }

    if window.label() == "setup" {
        println!("Closing setup window (label: {}).", window.label());
    }

    Ok(())
}

#[tauri::command]
pub async fn load_game_config(app_handle: AppHandle) -> Result<Option<GameData>, String> {
    let config_path = get_config_path(&app_handle)?;
    match fs::read_to_string(&config_path) {
        Ok(json) => {
            let data = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(Some(data))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read config: {}", e)),
    }
}

#[tauri::command]
pub async fn delete_config(app_handle: AppHandle) -> Result<(), String> {
    let config_path = get_config_path(&app_handle)?;
    match fs::remove_file(&config_path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Failed to delete config: {}", e)),
    }
}

fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    Ok(dir.join("userconfig.json"))
}
