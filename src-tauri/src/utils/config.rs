use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use std::env;

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

// New command to validate game path and return GameData without writing config
#[tauri::command]
pub async fn validate_game_installation(executable_path: String) -> Result<GameData, String> {
    info!(
        "Validating game installation from executable: {}",
        executable_path
    );
    let (game_root_path_buf, _) = find_game_paths_from_exe(&executable_path)?;
    let game_root_path_str = game_root_path_buf
        .to_str()
        .ok_or("Game root path contains invalid UTF-8")?
        .to_string();

    // TODO: Add optional check for dinput8.dll presence as per todo.md

    let game_data = GameData {
        game_root_path: game_root_path_str.clone(),
        game_executable_path: executable_path.clone(),
    };

    info!("Validation successful for: {}", executable_path);
    Ok(game_data)
}

// New function to explicitly save GameData
#[tauri::command] // Expose saving as a separate command
pub async fn save_game_config(app_handle: AppHandle, game_data: GameData) -> Result<(), String> {
    info!("Saving game config: {:?}", game_data);
    let config_path = get_config_path(&app_handle)?;
    fs::create_dir_all(config_path.parent().unwrap()) // Ensure dir exists
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    fs::write(
        &config_path,
        serde_json::to_string_pretty(&game_data)
            .map_err(|e| format!("Failed to serialize GameData: {}", e))?,
    )
    .map_err(|e| format!("Failed to write config to {:?}: {}", config_path, e))?;

    info!("Successfully saved game config to {:?}", config_path);
    Ok(())
}

#[tauri::command]
pub async fn load_game_config(app_handle: AppHandle) -> Result<Option<GameData>, String> {
    let config_path = get_config_path(&app_handle)?;
    match fs::read_to_string(&config_path) {
        Ok(json) => {
            let data = serde_json::from_str(&json).map_err(|e| {
                error!("Failed to parse userconfig.json: {}. Backing up.", e);
                // Backup corrupted file
                let backup_path = config_path.with_extension(format!(
                    "json.corrupt-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                ));
                if let Err(backup_err) = fs::rename(&config_path, &backup_path) {
                    error!(
                        "Failed to backup corrupted config file to {:?}: {}",
                        backup_path, backup_err
                    );
                } else {
                    info!("Backed up corrupted config file to {:?}", backup_path);
                }
                e.to_string()
            })?;
            Ok(Some(data))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read config: {}", e)),
    }
}

#[tauri::command]
pub async fn nuke_settings_and_relaunch(app_handle: AppHandle) -> Result<(), String> {
    info!("Attempting to delete all application configuration, data, and cache.");

    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;

    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get app cache dir: {}", e))?;

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let mut errors = Vec::new();

    // Attempt to remove config directory
    if config_dir.exists() {
        match fs::remove_dir_all(&config_dir) {
            Ok(_) => info!("Successfully deleted config directory: {:?}", config_dir),
            Err(e) => {
                let err_msg = format!("Failed to delete config directory {:?}: {}", config_dir, e);
                error!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    } else {
        info!(
            "Config directory does not exist, skipping deletion: {:?}",
            config_dir
        );
    }

    // Attempt to remove data directory
    if data_dir.exists() {
        match fs::remove_dir_all(&data_dir) {
            Ok(_) => info!("Successfully deleted data directory: {:?}", data_dir),
            Err(e) => {
                let err_msg = format!("Failed to delete data directory {:?}: {}", data_dir, e);
                error!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    } else {
        info!(
            "Data directory does not exist, skipping deletion: {:?}",
            data_dir
        );
    }

    // Attempt to remove cache directory
    if cache_dir.exists() {
        match fs::remove_dir_all(&cache_dir) {
            Ok(_) => info!("Successfully deleted cache directory: {:?}", cache_dir),
            Err(e) => {
                let err_msg = format!("Failed to delete cache directory {:?}: {}", cache_dir, e);
                error!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    } else {
        info!(
            "Cache directory does not exist, skipping deletion: {:?}",
            cache_dir
        );
    }

    if !errors.is_empty() {
        // If there were errors deleting, return them instead of restarting
        return Err(errors.join("; "));
    }

    // --- Environment variable cleanup ---
    info!("Attempting to clear potential AppImage environment variables before relaunch.");
    if let Ok(val) = env::var("APPIMAGE") {
        info!("Found APPIMAGE variable: {}, removing.", val);
        env::remove_var("APPIMAGE");
    } else {
        info!("APPIMAGE variable not found.");
    }
    if let Ok(val) = env::var("APPDIR") {
         info!("Found APPDIR variable: {}, removing.", val);
        env::remove_var("APPDIR");
    } else {
         info!("APPDIR variable not found.");
    }
    // --- End environment variable cleanup ---

    info!("Configuration cleared successfully. Requesting application restart.");
    // Restart the application. This function does not return.
    app_handle.restart();

    // Note: Code execution will not reach here if restart is successful.
    // We still need a return type for the function signature, but Ok(()) is effectively unreachable.
    // Ok(())
}

fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    Ok(dir.join("userconfig.json"))
}
