use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;
use walkdir::WalkDir;

// Main structure to represent a skin mod with all necessary information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkinMod {
    name: String,                    // Display name of the mod
    path: String,                    // Original path in fossmodmanager/mods
    enabled: bool,                   // Whether this mod is currently enabled
    thumbnail_path: Option<String>,  // Path to preview image
    author: Option<String>,          // Mod author if available
    version: Option<String>,         // Version information if available
    description: Option<String>,     // Mod description if available
    files: Vec<ModFile>,             // Files included in this mod for conflict detection
    installed_timestamp: i64,        // When this mod was installed
}

// Structure to track individual files within a mod for conflict resolution
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModFile {
    relative_path: String,           // Path relative to game root
    original_path: String,           // Path in the original mod folder
    file_type: ModFileType,          // Type of file (PAK, natives, etc.)
    enabled: bool,                   // Whether this specific file is enabled
    size_bytes: u64,                 // File size for information
}

// Enum to categorize mod files
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModFileType {
    PakFile,                         // .pak file
    NativesFile,                     // File inside natives directory
    Other,                           // Other files
}

// Central registry for all installed skin mods
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ModRegistry {
    installed_mods: Vec<SkinMod>,    // All installed mods
    last_updated: i64,               // When registry was last updated
}

// Image cache entry metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
struct CacheEntry {
    original_path: String,           // Original image path
    timestamp: i64,                  // When cached
}

//--------- Registry Management Functions ---------//

// Load the mod registry from disk
fn load_registry(app_handle: &AppHandle) -> Result<ModRegistry, String> {
    let registry_path = get_registry_path(app_handle)?;
    
    if !registry_path.exists() {
        return Ok(ModRegistry::default());
    }
    
    match fs::read_to_string(&registry_path) {
        Ok(content) => {
            if content.is_empty() {
                return Ok(ModRegistry::default());
            }
            
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse mod registry: {}", e))
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(ModRegistry::default())
            } else {
                Err(format!("Failed to read mod registry: {}", e))
            }
        }
    }
}

// Save the mod registry to disk
fn save_registry(app_handle: &AppHandle, registry: &ModRegistry) -> Result<(), String> {
    let registry_path = get_registry_path(app_handle)?;
    
    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize mod registry: {}", e))?;
    
    fs::write(&registry_path, content)
        .map_err(|e| format!("Failed to write mod registry: {}", e))
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
    
    Ok(config_dir.join("skinmods_registry.json"))
}

//--------- Mod Management Commands ---------//

// Scan for skin mods in the fossmodmanager/mods directory
#[tauri::command]
pub async fn scan_for_skin_mods(
    app_handle: AppHandle,
    game_root_path: String
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
    let existing_mods: HashMap<String, SkinMod> = registry.installed_mods
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
            log::debug!("Inspecting potential mod folder: {:?}", path);
            
            // Get mod path as string
            let mod_path = path.to_string_lossy().to_string();
            
            // Check if we already have this mod in the registry
            if let Some(existing_mod) = existing_mods.get(&mod_path) {
                scanned_mods.push(existing_mod.clone());
                continue;
            }

            // Check if this directory has a 'natives' folder or .pak files
            let has_natives_folder = WalkDir::new(path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path().is_dir() && 
                    e.path().file_name()
                       .map_or(false, |n| n.to_string_lossy().to_lowercase() == "natives")
                });

            // Check for .pak files
            let has_pak_files = WalkDir::new(path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path().is_file() &&
                    e.path().extension()
                       .map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "pak")
                });

            // Skip if not a valid skin mod
            if !has_natives_folder && !has_pak_files {
                log::debug!("Skipping directory {:?} - not a skin mod", path);
                continue;
            }

            // Get folder name and extract cleaner display name
            let folder_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let display_name = extract_mod_name_from_folder(&folder_name);

            // Find mod info and screenshot
            let (mod_info, screenshot_path) = find_mod_info_and_screenshot(path);
            
            // Index mod files for conflict detection
            let mod_files = index_mod_files(path, &game_root)?;
            
            // Create the mod entry
            let skin_mod = SkinMod {
                name: mod_info.name.unwrap_or(display_name),
                path: mod_path,
                enabled: false, // New mods start disabled
                thumbnail_path: screenshot_path,
                author: mod_info.author,
                version: mod_info.version,
                description: mod_info.description,
                files: mod_files,
                installed_timestamp: chrono::Utc::now().timestamp(),
            };
            
            scanned_mods.push(skin_mod);
        }
    }

    // Update registry with newly found mods
    registry.installed_mods = scanned_mods.clone();
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
    mod_path: String
) -> Result<(), String> {
    log::info!("Enabling skin mod: {}", mod_path);
    
    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }
    
    // Load the registry
    let mut registry = load_registry(&app_handle)?;
    
    // Find the mod to enable
    let mod_index = registry.installed_mods.iter().position(|m| m.path == mod_path)
        .ok_or_else(|| format!("Mod not found in registry: {}", mod_path))?;
    
    // Clone the data we need from the mod
    let mod_files = registry.installed_mods[mod_index].files.clone();
    let mod_path_clone = registry.installed_mods[mod_index].path.clone();
    
    // Check for conflicts with currently enabled mods
    let conflicts = find_conflicts_with_enabled_mods(&registry, &mod_files, &mod_path_clone);
    
    // Disable conflicting files in other mods
    if !conflicts.is_empty() {
        log::info!("Found {} conflicts to resolve", conflicts.len());
        resolve_conflicts(&game_root, &mut registry, &conflicts)?;
    }
    
    // Enable the mod's files
    for file in &mod_files {
        let game_file_path = game_root.join(&file.relative_path);
        let disabled_path = PathBuf::from(format!("{}.disabled", game_file_path.to_string_lossy()));
        
        if disabled_path.exists() {
            // File exists but is disabled, enable it
            fs::rename(&disabled_path, &game_file_path)
                .map_err(|e| format!("Failed to enable file {}: {}", 
                    file.relative_path, e))?;
        } else if !game_file_path.exists() {
            // File doesn't exist, install it from the original mod
            let original_file = PathBuf::from(&file.original_path);
            if original_file.exists() {
                // Ensure parent directory exists
                if let Some(parent) = game_file_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory {}: {}", 
                            parent.display(), e))?;
                }
                
                fs::copy(&original_file, &game_file_path)
                    .map_err(|e| format!("Failed to copy file from {} to {}: {}", 
                        original_file.display(), game_file_path.display(), e))?;
            }
        }
    }
    
    // Update the mod status in registry
    registry.installed_mods[mod_index].enabled = true;
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
    mod_path: String
) -> Result<(), String> {
    log::info!("Disabling skin mod: {}", mod_path);
    
    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }
    
    // Load the registry
    let mut registry = load_registry(&app_handle)?;
    
    // Find the mod to disable
    let mod_index = registry.installed_mods.iter().position(|m| m.path == mod_path)
        .ok_or_else(|| format!("Mod not found in registry: {}", mod_path))?;
    
    // Get the mod to disable
    let mod_to_disable = &registry.installed_mods[mod_index];
    
    // Disable the mod's files
    for file in &mod_to_disable.files {
        let game_file_path = game_root.join(&file.relative_path);
        let disabled_path = PathBuf::from(format!("{}.disabled", game_file_path.to_string_lossy()));
        
        if game_file_path.exists() {
            // Rename to .disabled
            fs::rename(&game_file_path, &disabled_path)
                .map_err(|e| format!("Failed to disable file {}: {}", 
                    file.relative_path, e))?;
        }
    }
    
    // Update the mod status in registry
    registry.installed_mods[mod_index].enabled = false;
    registry.last_updated = chrono::Utc::now().timestamp();
    save_registry(&app_handle, &registry)?;
    
    log::info!("Successfully disabled mod: {}", mod_path);
    Ok(())
}

// Get list of installed skin mods
#[tauri::command]
pub async fn list_installed_skin_mods(
    app_handle: AppHandle
) -> Result<Vec<SkinMod>, String> {
    log::info!("Listing installed skin mods");
    
    // Load the registry
    let registry = load_registry(&app_handle)?;
    
    Ok(registry.installed_mods)
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
    // Also check if it looks like a PAK file name
    if folder_name.to_lowercase().ends_with(".pak") || folder_name.contains("chunk") {
        // Try to extract a meaningful name from PAK files
        if let Some(match_pos) = folder_name.find("chunk") {
            if match_pos > 0 {
                return folder_name[..match_pos]
                    .trim_end_matches('_')
                    .trim_end_matches('-')
                    .to_string();
            }
        }
        return "Custom Skin".to_string();
    }

    folder_name.to_string()
}

// Basic mod information structure
#[derive(Default, Debug)]
struct ModInfo {
    name: Option<String>,
    author: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

// Find mod info and screenshot from a mod directory
fn find_mod_info_and_screenshot(mod_dir: &Path) -> (ModInfo, Option<String>) {
    let mut mod_info = ModInfo::default();
    let mut screenshot_path = None;
    
    // Look for screenshot with common names
    let screenshot_candidates = vec![
        "preview.jpg", "preview.png", "screenshot.jpg", "screenshot.png", "1.png", "1.jpg"
    ];
    
    // Check in the main directory
    for candidate in &screenshot_candidates {
        let candidate_path = mod_dir.join(candidate);
        if candidate_path.exists() && candidate_path.is_file() {
            screenshot_path = Some(candidate_path.to_string_lossy().to_string());
            break;
        }
    }
    
    // If not found, check in immediate subdirectories
    if screenshot_path.is_none() {
        if let Ok(entries) = fs::read_dir(mod_dir) {
            for entry in entries.filter_map(Result::ok) {
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    for candidate in &screenshot_candidates {
                        let candidate_path = sub_path.join(candidate);
                        if candidate_path.exists() && candidate_path.is_file() {
                            screenshot_path = Some(candidate_path.to_string_lossy().to_string());
                            break;
                        }
                    }
                }
                if screenshot_path.is_some() {
                    break;
                }
            }
        }
    }
    
    // Check for modinfo.ini in various locations
    let modinfo_locations = vec![
        mod_dir.join("modinfo.ini"),
        mod_dir.join("Texture").join("modinfo.ini"),
    ];
    
    // Add checks for subdirectories
    if let Ok(entries) = fs::read_dir(mod_dir) {
        for entry in entries.filter_map(Result::ok) {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                let modinfo_path = sub_path.join("modinfo.ini");
                if modinfo_path.exists() && modinfo_path.is_file() {
                    if let Some(info) = parse_modinfo_file(&modinfo_path) {
                        mod_info = info;
                        break;
                    }
                }
            }
        }
    }
    
    // Check all predefined locations if we haven't found info yet
    if mod_info.name.is_none() {
        for location in modinfo_locations {
            if location.exists() && location.is_file() {
                if let Some(info) = parse_modinfo_file(&location) {
                    mod_info = info;
                    break;
                }
            }
        }
    }
    
    (mod_info, screenshot_path)
}

// Parse modinfo.ini from a specific path
fn parse_modinfo_file(modinfo_path: &PathBuf) -> Option<ModInfo> {
    if !modinfo_path.exists() || !modinfo_path.is_file() {
        return None;
    }

    let content = fs::read_to_string(modinfo_path).ok()?;

    let mut info = ModInfo::default();

    // Simple INI parsing
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            if !value.is_empty() {
                match key.as_str() {
                    "name" => info.name = Some(value.to_string()),
                    "author" => info.author = Some(value.to_string()),
                    "version" => info.version = Some(value.to_string()),
                    "description" => info.description = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    Some(info)
}

// Index all files in a mod directory for tracking
fn index_mod_files(mod_dir: &Path, game_root: &Path) -> Result<Vec<ModFile>, String> {
    let mut files = Vec::new();
    
    // Index .pak files
    for entry in WalkDir::new(mod_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && 
            e.path().extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "pak")
        })
    {
        let file_path = entry.path();
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename in path: {}", file_path.display()))?;
        
        // PAK files go directly in game root
        let relative_path = filename.to_string(); 
        
        match file_path.metadata() {
            Ok(metadata) => {
                files.push(ModFile {
                    relative_path,
                    original_path: file_path.to_string_lossy().to_string(),
                    file_type: ModFileType::PakFile,
                    enabled: false,
                    size_bytes: metadata.len(),
                });
            },
            Err(e) => {
                log::warn!("Failed to get metadata for {}: {}", file_path.display(), e);
            }
        }
    }
    
    // Index natives files
    for entry in WalkDir::new(mod_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Look for the "natives" directory
        if path.is_dir() && path.file_name()
            .map_or(false, |n| n.to_string_lossy().to_lowercase() == "natives") 
        {
            // Walk the natives directory and index all files
            for native_entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
            {
                let file_path = native_entry.path();
                
                // Calculate relative path from natives dir
                if let Ok(rel_path) = file_path.strip_prefix(path) {
                    let relative_path = PathBuf::from("natives").join(rel_path);
                    
                    match file_path.metadata() {
                        Ok(metadata) => {
                            files.push(ModFile {
                                relative_path: relative_path.to_string_lossy().to_string(),
                                original_path: file_path.to_string_lossy().to_string(),
                                file_type: ModFileType::NativesFile,
                                enabled: false,
                                size_bytes: metadata.len(),
                            });
                        },
                        Err(e) => {
                            log::warn!("Failed to get metadata for {}: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(files)
}

// New function to find conflicts that doesn't need a full mod reference
fn find_conflicts_with_enabled_mods(
    registry: &ModRegistry,
    mod_files: &[ModFile],
    mod_path: &str
) -> Vec<(String, String)> {
    let mut conflicts = Vec::new();
    
    // Get all enabled mods except the one we're checking
    let enabled_mods: Vec<&SkinMod> = registry.installed_mods.iter()
        .filter(|m| m.enabled && m.path != mod_path)
        .collect();
    
    // Build a map of all files from enabled mods
    let mut enabled_files: HashMap<String, String> = HashMap::new();
    for mod_entry in &enabled_mods {
        for file in &mod_entry.files {
            enabled_files.insert(file.relative_path.clone(), mod_entry.path.clone());
        }
    }
    
    // Check for conflicts
    for file in mod_files {
        if let Some(other_mod_path) = enabled_files.get(&file.relative_path) {
            conflicts.push((file.relative_path.clone(), other_mod_path.clone()));
        }
    }
    
    conflicts
}

// Resolve conflicts by disabling conflicting files
fn resolve_conflicts(
    game_root: &PathBuf,
    registry: &mut ModRegistry,
    conflicts: &[(String, String)]
) -> Result<(), String> {
    for (file_path, mod_path) in conflicts {
        // Find the mod with the conflict
        if let Some(mod_entry) = registry.installed_mods.iter_mut()
            .find(|m| m.path == *mod_path) 
        {
            // Find the file in the mod
            if let Some(file_entry) = mod_entry.files.iter_mut()
                .find(|f| f.relative_path == *file_path)
            {
                // Disable the file
                let game_file_path = game_root.join(file_path);
                let disabled_path = PathBuf::from(format!("{}.disabled", game_file_path.to_string_lossy()));
                
                if game_file_path.exists() {
                    // Rename to .disabled
                    fs::rename(&game_file_path, &disabled_path)
                        .map_err(|e| format!("Failed to disable conflicting file {}: {}", 
                            file_path, e))?;
                            
                    file_entry.enabled = false;
                }
            }
        }
    }
    
    Ok(())
}

//--------- Image Caching Functions ---------//

// Function to read mod image files and return as base64
#[tauri::command]
pub async fn read_mod_image(image_path: String) -> Result<String, String> {
    log::info!("Reading mod image from: {}", image_path);

    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("Image file does not exist: {}", image_path));
    }

    // Read the image file
    let img_data = fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?;

    // Convert to base64
    let base64_encoded = general_purpose::STANDARD.encode(&img_data);

    log::info!(
        "Successfully read image: {} ({} bytes)",
        image_path,
        img_data.len()
    );
    Ok(base64_encoded)
}

// Get the image cache directory path
fn get_image_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get app cache dir: {}", e))?
        .join("fossmodmanager")
        .join("images");

    // Ensure the cache directory exists
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create image cache directory: {}", e))?;

    Ok(cache_dir)
}

// Generate a cache key for an image path
fn get_image_cache_key(image_path: &str) -> String {
    // Use a simple hash to ensure the filename is valid for filesystem
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Function to cache a mod image
#[tauri::command]
pub async fn cache_mod_image(
    app_handle: AppHandle,
    image_path: String,
    image_data: String,
) -> Result<(), String> {
    log::debug!("Caching image: {}", image_path);

    // Create a unique cache key
    let cache_key = get_image_cache_key(&image_path);

    // Get the cache directory
    let cache_dir = get_image_cache_dir(&app_handle)?;
    let cache_file_path = cache_dir.join(format!("{}.cache", cache_key));

    // Store the cache entry info
    let cache_info = CacheEntry {
        original_path: image_path.clone(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let cache_info_json = serde_json::to_string(&cache_info)
        .map_err(|e| format!("Failed to serialize cache info: {}", e))?;

    let cache_info_path = cache_dir.join(format!("{}.json", cache_key));
    fs::write(&cache_info_path, cache_info_json)
        .map_err(|e| format!("Failed to write cache info: {}", e))?;

    // Write the image data
    match general_purpose::STANDARD.decode(&image_data) {
        Ok(decoded_data) => {
            fs::write(&cache_file_path, decoded_data)
                .map_err(|e| format!("Failed to write image cache file: {}", e))?;
            log::debug!("Successfully cached image at {:?}", cache_file_path);
            Ok(())
        }
        Err(e) => Err(format!("Failed to decode image data: {}", e)),
    }
}

// Function to get cached mod images
#[tauri::command]
pub async fn get_cached_mod_images(
    app_handle: AppHandle,
    image_paths: Vec<String>,
) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();
    let cache_dir = get_image_cache_dir(&app_handle)?;

    let image_paths_count = image_paths.len();

    // For each requested path
    for path in image_paths {
        let cache_key = get_image_cache_key(&path);
        let cache_file_path = cache_dir.join(format!("{}.cache", cache_key));
        let cache_info_path = cache_dir.join(format!("{}.json", cache_key));

        // Check if both the cache file and info exist
        if cache_file_path.exists() && cache_info_path.exists() {
            // Read and validate cache info
            match fs::read_to_string(&cache_info_path) {
                Ok(info_json) => {
                    match serde_json::from_str::<CacheEntry>(&info_json) {
                        Ok(cache_info) => {
                            // Verify it's for the right path (in case of hash collision)
                            if cache_info.original_path != path {
                                log::warn!(
                                    "Cache key collision: {} vs {}",
                                    cache_info.original_path,
                                    path
                                );
                                continue;
                            }

                            // Check if cache is not too old (e.g., older than 7 days)
                            let now = chrono::Utc::now().timestamp();
                            let age = now - cache_info.timestamp;
                            if age > 7 * 24 * 60 * 60 {
                                // 7 days in seconds
                                log::debug!("Cache entry too old ({}), will reload: {}", age, path);
                                continue;
                            }

                            // Read and return the cached image
                            match fs::read(&cache_file_path) {
                                Ok(data) => {
                                    let base64_data = general_purpose::STANDARD.encode(data);
                                    result.insert(path.clone(), base64_data);
                                    log::debug!("Retrieved image from cache: {}", path);
                                }
                                Err(e) => {
                                    log::warn!("Failed to read cached image data: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to parse cache info: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read cache info: {}", e);
                }
            }
        } else {
            log::debug!("No cache found for: {}", path);
        }
    }

    log::info!(
        "Retrieved {} cached images out of {} requested",
        result.len(),
        image_paths_count
    );
    Ok(result)
}
