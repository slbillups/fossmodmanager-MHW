// src-tauri/src/utils/skinmanager.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;
use walkdir::WalkDir;

// Main structure to represent a skin mod with all necessary information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkinMod {
    pub name: String,                   // Display name of the mod
    pub path: String,                   // Original path in fossmodmanager/mods
    pub enabled: bool,                  // Whether this mod is currently enabled
    pub thumbnail_path: Option<String>, // Path to preview image
    pub author: Option<String>,         // Mod author if available
    pub version: Option<String>,        // Version information if available
    pub description: Option<String>,    // Mod description if available
    pub installed_timestamp: i64,       // When this mod was installed
    pub installed_files: Vec<String>,   // List of files installed by this mod
}

// Central registry for all installed skin mods
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SkinRegistry {
    installed_skins: Vec<SkinMod>, // All installed skins
    last_updated: i64,             // When registry was last updated
}

//--------- Registry Management Functions ---------//

// Load the mod registry from disk
fn load_registry(app_handle: &AppHandle) -> Result<SkinRegistry, String> {
    let registry_path = get_registry_path(app_handle)?;

    if !registry_path.exists() {
        return Ok(SkinRegistry::default());
    }

    match fs::read_to_string(&registry_path) {
        Ok(content) => {
            if content.is_empty() {
                return Ok(SkinRegistry::default());
            }

            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse skin registry: {}", e))
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(SkinRegistry::default())
            } else {
                Err(format!("Failed to read skin registry: {}", e))
            }
        }
    }
}

// Save the skin registry to disk
fn save_registry(app_handle: &AppHandle, registry: &SkinRegistry) -> Result<(), String> {
    let registry_path = get_registry_path(app_handle)?;

    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize skin registry: {}", e))?;

    fs::write(&registry_path, content).map_err(|e| format!("Failed to write skin registry: {}", e))
}

// Get the path to the registry file
fn get_registry_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;

    // Ensure the directory exists
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(config_dir.join("skin_registry.json"))
}

//--------- Skin Management Commands ---------//

// Scan for skin mods in the fossmodmanager/mods directory
#[tauri::command]
pub async fn scan_for_skin_mods(
    app_handle: AppHandle,
    game_root_path: String,
) -> Result<Vec<SkinMod>, String> {
    log::info!("Scanning for skin mods in {}", game_root_path);

    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }

    // Look in <game_root>/fossmodmanager/mods
    let mods_dir = game_root.join("fossmodmanager").join("mods");
    log::debug!("Looking for mods in {:?}", mods_dir);

    if !mods_dir.exists() || !mods_dir.is_dir() {
        log::info!("Mods directory does not exist: {:?}", mods_dir);
        return Ok(Vec::new());
    }

    // Load the existing registry to combine data
    let mut registry = load_registry(&app_handle)?;
    let existing_mods: HashMap<String, SkinMod> = registry
        .installed_skins
        .iter()
        .map(|m| (m.path.clone(), m.clone()))
        .collect();

    let mut scanned_mods = Vec::new();

    // Scan the mods directory
    for entry in WalkDir::new(&mods_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip the root directory itself
        if path == mods_dir {
            continue;
        }

        if path.is_dir() {
            log::debug!("Inspecting potential skin mod folder: {:?}", path);

            // Get mod path as string
            let mod_path = path.to_string_lossy().to_string();

            // Check if we already have this mod in the registry
            if let Some(existing_mod) = existing_mods.get(&mod_path) {
                scanned_mods.push(existing_mod.clone());
                continue;
            }

            // Get folder name and extract cleaner display name
            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let display_name = extract_mod_name_from_folder(&folder_name);

            // Find mod info and screenshot
            let screenshot_path = find_screenshot(path);

            // Create the mod entry
            let skin_mod = SkinMod {
                name: display_name,
                path: mod_path,
                enabled: false, // New mods start disabled
                thumbnail_path: screenshot_path,
                author: None,
                version: None,
                description: None,
                installed_timestamp: chrono::Utc::now().timestamp(),
                installed_files: Vec::new(),
            };

            scanned_mods.push(skin_mod);
        }
    }

    // Update registry with newly found mods
    registry.installed_skins = scanned_mods.clone();
    registry.last_updated = chrono::Utc::now().timestamp();
    save_registry(&app_handle, &registry)?;

    log::info!("Found {} skin mods", scanned_mods.len());
    Ok(scanned_mods)
}

// Enable a skin mod
#[tauri::command]
pub async fn enable_skin_mod(
    app_handle: AppHandle,
    game_root_path: String,
    mod_path: String,
) -> Result<(), String> {
    log::info!("Enabling skin mod: {}", mod_path);

    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }

    let mod_dir = PathBuf::from(&mod_path);
    if !mod_dir.exists() || !mod_dir.is_dir() {
        return Err(format!("Invalid mod path: {}", mod_path));
    }

    // Load the registry
    let mut registry = load_registry(&app_handle)?;

    // Find the mod to enable
    let mod_index = registry
        .installed_skins
        .iter()
        .position(|m| m.path == mod_path)
        .ok_or_else(|| format!("Mod not found in registry: {}", mod_path))?;

    // Scan for .pak files in the mod directory to install
    let mut installed_files = Vec::new();

    // Find and copy .pak files to game root
    for entry in WalkDir::new(&mod_dir)
        .max_depth(3) // Don't go too deep in directory structure
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|ext| ext.to_string_lossy().to_lowercase() == "pak")
        })
    {
        let source_path = entry.path();
        let file_name = source_path
            .file_name()
            .ok_or_else(|| format!("Invalid filename in path: {}", source_path.display()))?;

        // Destination is in game root
        let dest_path = game_root.join(file_name);

        log::info!(
            "Installing .pak file: {} -> {}",
            source_path.display(),
            dest_path.display()
        );

        // Copy the file to game root
        fs::copy(source_path, &dest_path).map_err(|e| {
            format!(
                "Failed to copy file {} to {}: {}",
                source_path.display(),
                dest_path.display(),
                e
            )
        })?;

        installed_files.push(dest_path.to_string_lossy().to_string());
    }

    // Look for natives directory and copy contents
    let natives_dir = mod_dir.join("natives");
    if natives_dir.exists() && natives_dir.is_dir() {
        let game_natives_dir = game_root.join("natives");

        // Ensure game natives directory exists
        if !game_natives_dir.exists() {
            fs::create_dir_all(&game_natives_dir)
                .map_err(|e| format!("Failed to create natives directory in game root: {}", e))?;
        }

        // Copy all files from mod's natives directory to game's natives directory
        for entry in WalkDir::new(&natives_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let source_path = entry.path();

            // Calculate relative path from natives dir
            let rel_path = source_path
                .strip_prefix(&natives_dir)
                .map_err(|e| format!("Path error: {}", e))?;

            let dest_path = game_natives_dir.join(rel_path);

            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {}: {}", parent.display(), e)
                })?;
            }

            log::info!(
                "Installing natives file: {} -> {}",
                source_path.display(),
                dest_path.display()
            );

            // Copy the file
            fs::copy(source_path, &dest_path).map_err(|e| {
                format!(
                    "Failed to copy file {} to {}: {}",
                    source_path.display(),
                    dest_path.display(),
                    e
                )
            })?;

            installed_files.push(dest_path.to_string_lossy().to_string());
        }
    }

    // Store the list of installed files in the registry entry
    registry.installed_skins[mod_index].enabled = true;

    // Store installed files in the registry (you'll need to add this field to SkinMod struct)
    if let Some(skin_mod) = registry.installed_skins.get_mut(mod_index) {
        // Store the installed files info for later removal
        skin_mod.installed_files = installed_files;

        log::info!(
            "Installed {} files for skin mod {}",
            skin_mod.installed_files.len(),
            mod_path
        );
    }

    registry.last_updated = chrono::Utc::now().timestamp();
    save_registry(&app_handle, &registry)?;

    log::info!("Successfully enabled mod: {}", mod_path);
    Ok(())
}

// Disable a skin mod
#[tauri::command]
pub async fn disable_skin_mod(
    app_handle: AppHandle,
    game_root_path: String,
    mod_path: String,
) -> Result<(), String> {
    log::info!("Disabling skin mod: {}", mod_path);

    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }

    // Load the registry
    let mut registry = load_registry(&app_handle)?;

    // Find the mod to disable
    let mod_index = registry
        .installed_skins
        .iter()
        .position(|m| m.path == mod_path)
        .ok_or_else(|| format!("Mod not found in registry: {}", mod_path))?;

    // Get the list of installed files to remove
    let installed_files = registry.installed_skins[mod_index].installed_files.clone();

    // Check if mod is already disabled
    if !registry.installed_skins[mod_index].enabled {
        log::info!("Mod is already disabled: {}", mod_path);
        return Ok(());
    }

    log::info!(
        "Removing {} installed files for mod: {}",
        installed_files.len(),
        mod_path
    );

    // Remove installed files
    for file_path in &installed_files {
        let path = PathBuf::from(file_path);

        if path.exists() {
            log::info!("Removing file: {}", path.display());

            // Remove file
            if let Err(e) = fs::remove_file(&path) {
                log::warn!("Failed to remove file {}: {}", path.display(), e);
                // Continue with other files even if one fails
            }
        }
    }

    // Update the mod status in registry
    if let Some(skin_mod) = registry.installed_skins.get_mut(mod_index) {
        skin_mod.enabled = false;
        skin_mod.installed_files.clear(); // Clear the list of installed files
    }

    registry.last_updated = chrono::Utc::now().timestamp();
    save_registry(&app_handle, &registry)?;

    log::info!("Successfully disabled mod: {}", mod_path);
    Ok(())
}

// Get list of installed skin mods
#[tauri::command]
pub async fn list_installed_skin_mods(app_handle: AppHandle) -> Result<Vec<SkinMod>, String> {
    log::info!("Listing installed skin mods");

    // Load the registry
    let registry = load_registry(&app_handle)?;

    Ok(registry.installed_skins)
}

//--------- Helper Functions ---------//

// Extract a cleaner mod name from folder name
fn extract_mod_name_from_folder(folder_name: &str) -> String {
    // Common delimiters used in skin mod folder names
    let delimiters = &['_', '-', ' ', '!', '#', '$', '.', '(', '['];

    // Check if there's any delimiter in the folder name
    if let Some(first_delimiter_pos) = folder_name.find(|c| delimiters.contains(&c)) {
        // If found delimiter, return everything before it
        if first_delimiter_pos > 0 {
            return folder_name[..first_delimiter_pos].to_string();
        }
    }

    // If no delimiter found or name would be empty, return the original folder name
    folder_name.to_string()
}

// Find screenshot in a mod directory
fn find_screenshot(mod_dir: &Path) -> Option<String> {
    // Look for screenshot with common names
    let screenshot_candidates = vec![
        "preview.jpg",
        "preview.png",
        "screenshot.jpg",
        "screenshot.png",
        "thumb.jpg",
        "thumb.png",
        "image.jpg",
        "image.png",
        "1.png",
        "1.jpg",
    ];

    // Check in the main directory
    for candidate in &screenshot_candidates {
        let candidate_path = mod_dir.join(candidate);
        if candidate_path.exists() && candidate_path.is_file() {
            return Some(candidate_path.to_string_lossy().to_string());
        }
    }

    // If not found, check in immediate subdirectories
    if let Ok(entries) = fs::read_dir(mod_dir) {
        for entry in entries.filter_map(Result::ok) {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                for candidate in &screenshot_candidates {
                    let candidate_path = sub_path.join(candidate);
                    if candidate_path.exists() && candidate_path.is_file() {
                        return Some(candidate_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}
