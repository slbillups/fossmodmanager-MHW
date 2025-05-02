// mod_registry.rs - Place this in src-tauri/src/utils/ directory
#![allow(dead_code)]
use log::{error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};

/// Core representation of a mod in the registry
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(unused_imports)]
pub struct Mod {
    // Core identification
    pub name: String,           // Display name (user-friendly)
    pub directory_name: String, // Folder name or identifier
    pub path: String,           // Original path in mods directory

    // Status
    pub enabled: bool, // Whether this mod is currently enabled

    // Metadata
    pub author: Option<String>,      // Author information if available
    pub version: Option<String>,     // Version information if available
    pub description: Option<String>, // Mod description if available
    pub source: Option<String>,      // Where the mod came from (e.g., "local_zip", "nexus")
    pub installed_timestamp: i64,    // When this mod was installed (unix timestamp)

    // File specific info
    pub installed_directory: String, // Relative path from game root
    pub mod_type: ModType,           // Type categorization
}

/// Types of mods that can be installed
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModType {
    REFrameworkPlugin,  // Installed to reframework/plugins/
    REFrameworkAutorun, // Installed to reframework/autorun/
    SkinMod,            // Various appearance mods
    NativesMod,         // Files for the natives directory
    Other,              // Any other mod type
}

/// For skin mods with additional capabilities
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinMod {
    #[serde(flatten)]
    pub base: Mod, // Include all base mod fields
    pub thumbnail_path: Option<String>, // Path to preview image
    pub conflicts: Vec<String>,         // List of other mods this conflicts with
    pub files: Vec<ModFile>,            // Individual files included in this skin mod
    pub installed_files: Vec<String>,   // List of files installed by this mod
    pub installed_pak_path: Option<String>, // Path to the installed (numbered) .pak file
}

/// Structure to track individual files within a mod for conflict resolution
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub relative_path: String,  // Path relative to game root
    pub original_path: String,  // Path in the original mod folder
    pub file_type: ModFileType, // Type of file (PAK, natives, etc.)
    pub enabled: bool,          // Whether this specific file is enabled
    pub size_bytes: u64,        // File size for information
}

/// Enum to categorize mod files
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModFileType {
    PakFile,     // .pak file
    NativesFile, // File inside natives directory
    Other,       // Other files
}

/// The complete registry containing all mods
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModRegistry {
    pub mods: Vec<Mod>,          // Regular mods (REFramework plugins/autorun)
    pub skin_mods: Vec<SkinMod>, // Skin mods with additional metadata
    pub last_updated: i64,       // When registry was last updated (unix timestamp)
    pub format_version: u32,     // For future migration needs (start with 1)
}

/// Frontend-friendly view of a mod (for compatibility with existing frontend code)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub directory_name: String,      // Identifier for the mod
    pub name: Option<String>,        // Display name
    pub version: Option<String>,     // Version if available
    pub author: Option<String>,      // Author if available
    pub description: Option<String>, // Description if available
    pub enabled: bool,               // Whether enabled or not
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyModMetadata {
    pub parsed_name: String,
    pub original_zip_name: String,
    pub installed_directory: String,
    pub source: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinMetadata {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub thumbnail_path: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModListContainer {
    pub mods: Vec<LegacyModMetadata>,
    pub skins: Vec<SkinMetadata>,
}

// --------------------------------
// ModRegistry Implementation
// --------------------------------

impl ModRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            mods: Vec::new(),
            skin_mods: Vec::new(),
            last_updated: chrono::Utc::now().timestamp(),
            format_version: 1,
        }
    }

    /// Get the path to the registry file
    pub fn get_registry_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to get app config dir: {}", e))?;

        // Ensure the directory exists
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        Ok(config_dir.join("mod_registry.json"))
    }

    /// Validate the registry file
    /// Returns Ok if the file doesn't exist or is valid JSON.
    /// Returns Err only if the file exists but cannot be parsed.
    pub fn validate_registry(app_handle: &AppHandle) -> Result<(), String> {
        let registry_path = Self::get_registry_path(app_handle)?;

        if !registry_path.exists() {
            log::debug!("Mod registry file does not exist, validation skipped.");
            return Ok(()); // Not existing is valid
        }

        match fs::read_to_string(&registry_path) {
            Ok(content) => {
                if content.is_empty() {
                    log::warn!("Mod registry file is empty, considering valid for now.");
                    return Ok(()); // Empty is technically parsable, consider valid for now
                }
                // Attempt to parse, discard the result, only care about errors
                match serde_json::from_str::<Self>(&content) {
                    Ok(_) => {
                        log::info!("Mod registry validation successful.");
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Mod registry validation failed: {}", e);
                        Err(format!("Failed to parse mod_registry.json: {}", e))
                    }
                }
            }
            Err(e) => {
                // Errors other than NotFound during read are problematic
                log::error!("Failed to read mod_registry.json for validation: {}", e);
                Err(format!(
                    "Failed to read mod_registry.json for validation: {}",
                    e
                ))
            }
        }
    }

    /// Load the registry from disk
    pub fn load(app_handle: &AppHandle) -> Result<Self, String> {
        let registry_path = Self::get_registry_path(app_handle)?;

        // If registry doesn't exist, return a new empty one
        if !registry_path.exists() {
            info!("No existing mod registry found, creating new one");
            return Ok(Self::new());
        }

        // Read the file contents
        match fs::read_to_string(&registry_path) {
            Ok(content) => {
                if content.is_empty() {
                    info!("Registry file exists but is empty, creating new registry");
                    return Ok(Self::new());
                }

                // Try to parse as ModRegistry
                match serde_json::from_str::<Self>(&content) {
                    Ok(registry) => {
                        info!(
                            "Successfully loaded mod registry with {} mods and {} skin mods",
                            registry.mods.len(),
                            registry.skin_mods.len()
                        );
                        Ok(registry)
                    }
                    Err(e) => {
                        // Handle legacy format
                        warn!("Failed to parse registry file as ModRegistry: {}", e);
                        Self::migrate_from_legacy(content, app_handle)
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Should never happen as we already checked existence
                Ok(Self::new())
            }
            Err(e) => {
                error!("Failed to read registry file: {}", e);
                Err(format!("Failed to read mod registry: {}", e))
            }
        }
    }

    /// Save the registry to disk
    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let registry_path = Self::get_registry_path(app_handle)?;

        // Serialize to JSON
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize mod registry: {}", e))?;

        // Write to file
        fs::write(&registry_path, content)
            .map_err(|e| format!("Failed to write mod registry: {}", e))?;

        info!("Successfully saved mod registry");
        Ok(())
    }

    /// Migrate from old format to new format
    fn migrate_from_legacy(content: String, app_handle: &AppHandle) -> Result<Self, String> {
        info!("Attempting to migrate from legacy format");

        // Try to handle various formats
        let mut registry = Self::new();

        // First try the intermediate "ModListContainer" format
        match serde_json::from_str::<ModListContainer>(&content) {
            Ok(container) => {
                info!(
                    "Found legacy ModListContainer format with {} mods and {} skins",
                    container.mods.len(),
                    container.skins.len()
                );

                // Convert ModMetadata to Mod
                for legacy_mod in container.mods {
                    let new_mod = Mod {
                        name: legacy_mod.parsed_name.clone(),
                        directory_name: legacy_mod.parsed_name,
                        path: legacy_mod.original_zip_name,
                        enabled: true, // We'll check actual status later
                        author: None,
                        version: legacy_mod.version,
                        description: None,
                        source: Some(legacy_mod.source),
                        installed_timestamp: chrono::Utc::now().timestamp(),
                        installed_directory: legacy_mod.installed_directory.clone(),
                        mod_type: if legacy_mod.installed_directory.contains("/autorun/") {
                            ModType::REFrameworkAutorun
                        } else if legacy_mod.installed_directory.contains("/plugins/") {
                            ModType::REFrameworkPlugin
                        } else {
                            ModType::Other
                        },
                    };
                    registry.mods.push(new_mod);
                }

                // Convert SkinMetadata to SkinMod
                for legacy_skin in container.skins {
                    let base_mod = Mod {
                        name: legacy_skin.name.clone(),
                        directory_name: Path::new(&legacy_skin.path)
                            .file_name()
                            .and_then(|os_str| os_str.to_str())
                            .unwrap_or(&legacy_skin.name)
                            .to_string(),
                        path: legacy_skin.path,
                        enabled: legacy_skin.enabled,
                        author: legacy_skin.author,
                        version: legacy_skin.version,
                        description: legacy_skin.description,
                        source: Some("local".to_string()),
                        installed_timestamp: chrono::Utc::now().timestamp(),
                        installed_directory: "".to_string(), // Will be updated on refresh
                        mod_type: ModType::SkinMod,
                    };

                    let skin_mod = SkinMod {
                        base: base_mod,
                        thumbnail_path: legacy_skin.thumbnail_path,
                        conflicts: Vec::new(),
                        files: Vec::new(),           // Will be populated on refresh
                        installed_files: Vec::new(), // Will be populated on refresh
                        installed_pak_path: None,
                    };

                    registry.skin_mods.push(skin_mod);
                }
            }
            Err(_) => {
                // Fall back to older ModList format (Vec<ModMetadata>)
                match serde_json::from_str::<Vec<crate::ModMetadata>>(&content) {
                    Ok(mod_list) => {
                        info!("Found legacy ModList format with {} mods", mod_list.len());

                        // Convert ModMetadata to Mod
                        for legacy_mod in mod_list {
                            let new_mod = Mod {
                                name: legacy_mod.parsed_name.clone(),
                                directory_name: legacy_mod.parsed_name,
                                path: legacy_mod.original_zip_name,
                                enabled: true, // We'll check actual status later
                                author: None,
                                version: legacy_mod.version,
                                description: None,
                                source: Some(legacy_mod.source),
                                installed_timestamp: chrono::Utc::now().timestamp(),
                                installed_directory: legacy_mod.installed_directory.clone(),
                                mod_type: if legacy_mod.installed_directory.contains("/autorun/") {
                                    ModType::REFrameworkAutorun
                                } else if legacy_mod.installed_directory.contains("/plugins/") {
                                    ModType::REFrameworkPlugin
                                } else {
                                    ModType::Other
                                },
                            };
                            registry.mods.push(new_mod);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse legacy mod list: {}", e);
                        return Err(format!("Failed to migrate from legacy format: {}", e));
                    }
                }
            }
        }

        // Save the migrated registry
        registry.last_updated = chrono::Utc::now().timestamp();
        registry.save(app_handle)?;

        info!("Successfully migrated to new registry format");
        Ok(registry)
    }

    /// Convert a standard Mod to a frontend-friendly ModInfo
    pub fn to_mod_info(m: &Mod) -> ModInfo {
        ModInfo {
            directory_name: m.directory_name.clone(),
            name: Some(m.name.clone()),
            version: m.version.clone(),
            author: m.author.clone(),
            description: m.description.clone(),
            enabled: m.enabled,
        }
    }

    /// Convert a SkinMod to a frontend-friendly ModInfo
    pub fn skin_to_mod_info(sm: &SkinMod) -> ModInfo {
        ModInfo {
            directory_name: sm.base.directory_name.clone(),
            name: Some(sm.base.name.clone()),
            version: sm.base.version.clone(),
            author: sm.base.author.clone(),
            description: sm.base.description.clone(),
            enabled: sm.base.enabled,
        }
    }

    /// Get all mods as ModInfo objects (for frontend compatibility)
    pub fn get_all_mod_info(&self) -> Vec<ModInfo> {
        let mut result = Vec::new();

        // Add standard mods
        for m in &self.mods {
            result.push(Self::to_mod_info(m));
        }

        // Add skin mods
        for sm in &self.skin_mods {
            result.push(Self::skin_to_mod_info(sm));
        }

        result
    }

    /// Get REFramework mods as ModInfo objects
    pub fn get_reframework_mod_info(&self) -> Vec<ModInfo> {
        self.mods
            .iter()
            .filter(|m| {
                m.mod_type == ModType::REFrameworkPlugin
                    || m.mod_type == ModType::REFrameworkAutorun
            })
            .map(Self::to_mod_info)
            .collect()
    }

    /// Get skin mods as ModInfo objects
    pub fn get_skin_mod_info(&self) -> Vec<ModInfo> {
        self.skin_mods.iter().map(Self::skin_to_mod_info).collect()
    }

    /// Find a mod by directory name
    pub fn find_mod(&self, directory_name: &str) -> Option<&Mod> {
        self.mods
            .iter()
            .find(|m| m.directory_name == directory_name)
    }

    /// Find a mod by directory name (mutable)
    pub fn find_mod_mut(&mut self, directory_name: &str) -> Option<&mut Mod> {
        self.mods
            .iter_mut()
            .find(|m| m.directory_name == directory_name)
    }

    /// Find a skin mod by directory name
    pub fn find_skin_mod(&self, directory_name: &str) -> Option<&SkinMod> {
        self.skin_mods
            .iter()
            .find(|m| m.base.directory_name == directory_name)
    }

    /// Find a skin mod by directory name (mutable)
    pub fn find_skin_mod_mut(&mut self, directory_name: &str) -> Option<&mut SkinMod> {
        self.skin_mods
            .iter_mut()
            .find(|m| m.base.directory_name == directory_name)
    }

    /// Update the enabled status of a mod based on filesystem state
    pub fn update_mod_enabled_status(&mut self, game_root_path: &Path) -> Result<(), String> {
        // Update regular mods
        for mod_entry in &mut self.mods {
            let mod_dir_rel = PathBuf::from(&mod_entry.installed_directory);
            let mod_dir_abs = game_root_path.join(&mod_dir_rel);
            let disabled_dir_str = format!("{}.disabled", mod_entry.installed_directory);
            let disabled_dir_abs = game_root_path.join(PathBuf::from(&disabled_dir_str));

            let is_enabled = mod_dir_abs.is_dir(); // Enabled if directory exists without .disabled

            // Log warnings for unusual states
            if is_enabled && disabled_dir_abs.exists() {
                warn!(
                    "Mod '{}' has both enabled and disabled directories present! Assuming enabled.",
                    mod_entry.name
                );
            } else if !is_enabled && !disabled_dir_abs.exists() {
                warn!("Mod '{}' directory not found in either enabled or disabled state. Assuming disabled.",
                     mod_entry.name);
            }

            mod_entry.enabled = is_enabled;
        }

        // Update skin mods - their enabled status is tracked separately
        // This would be implemented based on how skin mods are actually enabled/disabled

        self.last_updated = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// Add a new mod to the registry
    pub fn add_mod(&mut self, new_mod: Mod) {
        // Remove any existing mod with same directory name
        self.mods
            .retain(|m| m.directory_name != new_mod.directory_name);
        // Add the new mod
        self.mods.push(new_mod);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Add a new skin mod to the registry
    pub fn add_skin_mod(&mut self, new_skin_mod: SkinMod) {
        // Remove any existing skin mod with same directory name
        self.skin_mods
            .retain(|m| m.base.directory_name != new_skin_mod.base.directory_name);
        // Add the new skin mod
        self.skin_mods.push(new_skin_mod);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Remove a mod from the registry
    pub fn remove_mod(&mut self, directory_name: &str) -> bool {
        let initial_count = self.mods.len();
        self.mods.retain(|m| m.directory_name != directory_name);
        let removed = self.mods.len() != initial_count;

        if removed {
            self.last_updated = chrono::Utc::now().timestamp();
        }

        removed
    }

    /// Remove a skin mod from the registry
    pub fn remove_skin_mod(&mut self, directory_name: &str) -> bool {
        let initial_count = self.skin_mods.len();
        self.skin_mods
            .retain(|m| m.base.directory_name != directory_name);
        let removed = self.skin_mods.len() != initial_count;

        if removed {
            self.last_updated = chrono::Utc::now().timestamp();
        }

        removed
    }

    /// Toggle a mod's enabled state
    pub fn toggle_mod_enabled(&mut self, directory_name: &str, enable: bool) -> Result<(), String> {
        // Find the mod
        if let Some(mod_entry) = self.find_mod_mut(directory_name) {
            mod_entry.enabled = enable;
            self.last_updated = chrono::Utc::now().timestamp();
            Ok(())
        } else {
            Err(format!("Mod '{}' not found in registry", directory_name))
        }
    }

    /// Toggle a skin mod's enabled state
    pub fn toggle_skin_mod_enabled(
        &mut self,
        directory_name: &str,
        enable: bool,
    ) -> Result<(), String> {
        // Find the skin mod
        if let Some(skin_mod) = self.find_skin_mod_mut(directory_name) {
            skin_mod.base.enabled = enable;
            self.last_updated = chrono::Utc::now().timestamp();
            Ok(())
        } else {
            Err(format!(
                "Skin mod '{}' not found in registry",
                directory_name
            ))
        }
    }
}

// Utility functions

/// Toggle a mod's enabled state through the registry and on filesystem
#[tauri::command]
pub async fn toggle_mod_enabled_state(
    app_handle: AppHandle,
    game_root_path: String,
    mod_name: String,
    enable: bool,
) -> Result<(), String> {
    log::info!(
        "Toggling mod '{}' to enabled={} in game root: {}",
        mod_name,
        enable,
        game_root_path
    );
    let game_root = PathBuf::from(&game_root_path);

    // Load the registry
    let mut registry = ModRegistry::load(&app_handle)?;

    // Find the mod
    let mod_entry = match registry.find_mod(&mod_name) {
        Some(m) => m.clone(), // Clone to avoid borrow issues
        None => {
            // Try to find it as a skin mod
            if registry.find_skin_mod(&mod_name).is_some() {
                return Err(format!(
                    "Mod '{}' is a skin mod. Please use toggle_skin_mod_enabled instead.",
                    mod_name
                ));
            }

            return Err(format!("Mod '{}' not found in registry", mod_name));
        }
    };

    // Get paths for filesystem operations
    let installed_dir_rel = PathBuf::from(&mod_entry.installed_directory);
    let installed_dir_abs = game_root.join(&installed_dir_rel);
    let disabled_dir_str = format!("{}.disabled", mod_entry.installed_directory);
    let disabled_dir_abs = game_root.join(PathBuf::from(&disabled_dir_str));

    if enable {
        // Enable: Rename *.disabled to * (if it exists)
        if disabled_dir_abs.exists() {
            log::info!(
                "Enabling mod '{}': Renaming {:?} -> {:?}",
                mod_name,
                disabled_dir_abs,
                installed_dir_abs
            );
            fs::rename(&disabled_dir_abs, &installed_dir_abs).map_err(|e| {
                format!(
                    "Failed to rename {:?} to {:?}: {}",
                    disabled_dir_abs, installed_dir_abs, e
                )
            })?;
        } else if installed_dir_abs.exists() {
            log::info!(
                "Mod '{}' is already enabled (directory {:?} exists).",
                mod_name,
                installed_dir_abs
            );
            // Already in desired state
        } else {
            return Err(format!(
                "Cannot enable mod '{}': Neither directory {:?} nor {:?} found.",
                mod_name, installed_dir_abs, disabled_dir_abs
            ));
        }
    } else {
        // Disable: Rename * to *.disabled (if it exists)
        if installed_dir_abs.exists() {
            log::info!(
                "Disabling mod '{}': Renaming {:?} -> {:?}",
                mod_name,
                installed_dir_abs,
                disabled_dir_abs
            );
            fs::rename(&installed_dir_abs, &disabled_dir_abs).map_err(|e| {
                format!(
                    "Failed to rename {:?} to {:?}: {}",
                    installed_dir_abs, disabled_dir_abs, e
                )
            })?;
        } else if disabled_dir_abs.exists() {
            log::info!(
                "Mod '{}' is already disabled (directory {:?} exists).",
                mod_name,
                disabled_dir_abs
            );
            // Already in desired state
        } else {
            return Err(format!(
                "Cannot disable mod '{}': Neither directory {:?} nor {:?} found.",
                mod_name, installed_dir_abs, disabled_dir_abs
            ));
        }
    }

    // Update registry and save
    registry.toggle_mod_enabled(&mod_name, enable)?;
    registry.save(&app_handle)?;

    log::info!(
        "Successfully toggled mod '{}' to enabled={}",
        mod_name,
        enable
    );
    Ok(())
}

/// Extract a cleaner mod name from folder name
pub fn extract_mod_name_from_folder(folder_name: &str) -> String {
    // Common delimiters used in mod folder names
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

/// Find screenshot in a mod directory (more robust version)
fn find_screenshot(mod_dir: &Path) -> Option<String> {
    let image_extensions = ["png", "jpg", "jpeg", "webp", "gif", "bmp"]; // Added more extensions

    // 1. Search in the root directory first (quick check)
    if let Ok(entries) = fs::read_dir(mod_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if image_extensions.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
                        log::debug!("Found screenshot in root: {}", path.display());
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    log::debug!(
        "No screenshot found in root of {}, searching subdirectories.",
        mod_dir.display()
    );

    // 2. If not found in root, search recursively up to 3 levels deep
    // WalkDir depth is relative to the starting path.
    // max_depth(1) means only the root.
    // max_depth(2) means root + 1 level down.
    // max_depth(4) means root + 3 levels down.
    for entry in WalkDir::new(mod_dir)
        .max_depth(4) // Search mod_dir + 3 levels of subdirectories
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path() != mod_dir && e.file_type().is_file()) // Skip root, only files
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if image_extensions.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
                log::debug!("Found screenshot recursively: {}", path.display());
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    log::debug!("No screenshot found for: {}", mod_dir.display());
    None
}

/// Helper function to find the next available patch number in the game root directory
fn find_next_available_patch_number(game_root: &Path) -> Result<u32, String> {
    let pak_regex = Regex::new(r"re_chunk_000\.pak\.sub_000\.pak\.patch_(\d{3})\.pak(?:\.disabled)?$").unwrap();
    let mut max_num: u32 = 0;
    let mut found_any = false;

    log::debug!("Scanning {} for existing patch files", game_root.display());

    match fs::read_dir(game_root) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(caps) = pak_regex.captures(file_name) {
                            if let Some(num_str) = caps.get(1) {
                                if let Ok(num) = num_str.as_str().parse::<u32>() {
                                    log::trace!("Found patch file: {} with number {}", file_name, num);
                                    max_num = max_num.max(num);
                                    found_any = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Failed to read game root directory {}: {}",
                game_root.display(),
                e
            ));
        }
    }

    let next_num = if found_any { max_num + 1 } else { 1 }; // Start from 001 if none found
    log::debug!("Next available patch number determined: {}", next_num);
    Ok(next_num)
}

/// Scans REFramework directories, compares with registry, and updates registry state.
fn scan_and_update_reframework_mods(registry: &mut ModRegistry, game_root_path: &Path) -> Result<(), String> {
    log::debug!("Scanning REFramework directories in {}", game_root_path.display());
    let mut found_on_disk = HashSet::new();
    let mut disk_mod_info = HashMap::new(); // Store details like enabled status and path

    let plugins_dir = game_root_path.join("reframework").join("plugins");
    let autorun_dir = game_root_path.join("reframework").join("autorun");

    // Helper closure to scan a directory - mark as mutable
    let mut scan_dir = |dir: &Path, mod_type: ModType| -> Result<(), String> {
        if !dir.exists() {
            log::warn!("REFramework directory not found: {}, skipping scan.", dir.display());
            return Ok(());
        }
        log::debug!("Scanning directory: {}", dir.display());
        for entry in fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry in {}: {}", dir.display(), e))?;
            let path = entry.path();
            if path.is_dir() { // Check if it's a directory
                let file_name_os = entry.file_name();
                if let Some(name_str) = file_name_os.to_str() {
                    let is_enabled = !name_str.ends_with(".disabled");
                    let base_name = if is_enabled {
                        name_str.to_string()
                    } else {
                        name_str.trim_end_matches(".disabled").to_string()
                    };

                    if !base_name.is_empty() {
                        let rel_path = path.strip_prefix(game_root_path)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| name_str.to_string()); // Fallback to original name

                        let installed_dir = if is_enabled {
                            rel_path.clone()
                        } else {
                            rel_path.trim_end_matches(".disabled").to_string()
                        };

                        log::trace!("Found mod directory: {} (Enabled: {}) -> Base: {}, InstalledDir: {}",
                                    name_str, is_enabled, base_name, installed_dir);

                        // Store info, potentially overwriting if both enabled/disabled exist (prefer enabled)
                        if !disk_mod_info.contains_key(&base_name) || is_enabled {
                             disk_mod_info.insert(base_name.clone(), (is_enabled, installed_dir, mod_type.clone()));
                        }
                        found_on_disk.insert(base_name);
                    }
                }
            }
        }
        Ok(())
    };

    // Scan both directories
    scan_dir(&plugins_dir, ModType::REFrameworkPlugin)?;
    scan_dir(&autorun_dir, ModType::REFrameworkAutorun)?;

    log::debug!("Found {} potential REFramework mods on disk: {:?}", found_on_disk.len(), found_on_disk);

    // --- Compare with Registry ---
    let _mods_to_remove_from_registry: Vec<String> = Vec::new();
    let mut registry_mod_names = HashSet::new();

    // First pass: Update existing mods in registry and check for removals
    for mod_entry in registry.mods.iter_mut() {
        // Only process REFramework mods
        if mod_entry.mod_type != ModType::REFrameworkPlugin && mod_entry.mod_type != ModType::REFrameworkAutorun {
            continue;
        }

        let mod_name = &mod_entry.directory_name;
        registry_mod_names.insert(mod_name.clone());

        if let Some((disk_enabled, disk_installed_dir, disk_mod_type)) = disk_mod_info.get(mod_name) {
            // Mod exists on disk, update status in registry
            if mod_entry.enabled != *disk_enabled {
                 log::info!("Updating status for mod '{}': {} -> {}", mod_name, mod_entry.enabled, disk_enabled);
                 mod_entry.enabled = *disk_enabled;
            }
            // Optionally update installed_directory if it differs? Or assume registry is correct if source wasn't manual?
            if mod_entry.installed_directory != *disk_installed_dir && mod_entry.source == Some("manual_scan".to_string()) {
                 log::info!("Updating installed directory for manually scanned mod '{}': '{}' -> '{}'",
                           mod_name, mod_entry.installed_directory, disk_installed_dir);
                 mod_entry.installed_directory = disk_installed_dir.clone();
            }
            // Update mod type if it changed (e.g., moved from autorun to plugins)
             if mod_entry.mod_type != *disk_mod_type {
                 log::info!("Updating mod type for mod '{}': {:?} -> {:?}", mod_name, mod_entry.mod_type, disk_mod_type);
                 mod_entry.mod_type = disk_mod_type.clone();
             }

        } else {
            // Mod is in registry but not found on disk (neither enabled nor disabled)
            log::warn!("Mod '{}' found in registry but not on disk. Marking as disabled.", mod_name);
            mod_entry.enabled = false;
            // Optionally, we could completely remove it here if source is "manual_scan"
            // if mod_entry.source == Some("manual_scan".to_string()) {
            //    mods_to_remove_from_registry.push(mod_name.clone());
            // }
        }
    }

    // Remove mods marked for removal (currently unused, see above comment)
    // for mod_name in mods_to_remove_from_registry {
    //     registry.remove_mod(&mod_name);
    // }

    // Second pass: Add mods found on disk but not in registry
    let mut added_new_mod = false;
    for disk_mod_name in found_on_disk.difference(&registry_mod_names) {
        if let Some((disk_enabled, disk_installed_dir, disk_mod_type)) = disk_mod_info.get(disk_mod_name) {
            log::info!("Found manually added mod '{}' on disk. Adding to registry.", disk_mod_name);
            let new_mod = Mod {
                name: disk_mod_name.clone(), // Use directory name as display name initially
                directory_name: disk_mod_name.clone(),
                path: "Manually Detected".to_string(), // Indicate it wasn't installed via manager
                enabled: *disk_enabled,
                author: None,
                version: None,
                description: None,
                source: Some("manual_scan".to_string()),
                installed_timestamp: chrono::Utc::now().timestamp(),
                installed_directory: disk_installed_dir.clone(),
                mod_type: disk_mod_type.clone(),
            };
            registry.mods.push(new_mod);
            added_new_mod = true;
        }
    }

    if added_new_mod {
        registry.last_updated = chrono::Utc::now().timestamp();
        // No need to save here, assuming the caller (list_mods) will save if needed.
    }

    Ok(())
}

#[tauri::command]
pub async fn list_mods(
    app_handle: AppHandle,
    game_root_path: String,
) -> Result<Vec<ModInfo>, String> {
    log::info!(
        "Listing REFramework mods based on registry for game root: {}",
        game_root_path
    );

    let game_root = PathBuf::from(&game_root_path);
    let mut registry = ModRegistry::load(&app_handle)?;

    // --- Scan filesystem and update registry FIRST --- 
    log::debug!("Running scan_and_update_reframework_mods before listing...");
    if let Err(e) = scan_and_update_reframework_mods(&mut registry, &game_root) {
        log::error!("Error during REFramework mod scan: {}. Proceeding with potentially stale registry data.", e);
        // Decide if this should be a hard error. For now, log and continue.
    }
    // Also update general enabled status based on filesystem AFTER scan might have added/updated mods
    // Note: scan_and_update_reframework_mods already updates enabled status for discovered mods.
    // This `update_mod_enabled_status` might be redundant or could overwrite manual_scan status?
    // Let's comment it out for now and rely on the scan function's update logic.
    // registry.update_mod_enabled_status(&game_root)?;

    // --- Save registry IF changes were made by the scan --- 
    // Check if the scan modified the registry (e.g., added manual mods, changed status)
    // We need a way to track if scan_and_update_reframework_mods actually changed anything.
    // Let's modify scan_and_update_reframework_mods to return a bool indicating changes.
    // For now, let's just save unconditionally after scan, accepting potential unnecessary writes.
    if let Err(e) = registry.save(&app_handle) {
         log::error!("Failed to save registry after scan: {}", e);
         // Proceed anyway, but log the error
    }

    // Now get the mod info from the potentially updated registry
    let mods_info = registry.get_reframework_mod_info();

    log::info!(
        "Finished processing mod list. Returning {} REFramework mods to frontend.",
        mods_info.len()
    );
    Ok(mods_info)
}

// --------- Skin Mod Management Commands (Consolidated) --------- //

#[tauri::command]
pub async fn scan_and_update_skin_mods(
    app_handle: AppHandle,
    game_root_path: String,
) -> Result<Vec<SkinMod>, String> {
    log::info!(
        "Scanning for skin mods in {} and updating registry",
        game_root_path
    );

    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }

    // Look in <game_root>/fossmodmanager/mods
    let mods_dir = game_root.join("fossmodmanager").join("mods");
    log::debug!("Looking for mods in {:?}", mods_dir);

    if !mods_dir.exists() || !mods_dir.is_dir() {
        log::info!("Mods directory does not exist: {:?}", mods_dir);
        // Load existing registry anyway to return its current state
        let registry = ModRegistry::load(&app_handle)?;
        return Ok(registry.skin_mods);
    }

    // Load the existing registry
    let mut registry = ModRegistry::load(&app_handle)?;
    let mut existing_mods: HashMap<String, SkinMod> = registry
        .skin_mods
        .iter()
        .map(|m| (m.base.path.clone(), m.clone())) // Use base.path here
        .collect();

    let mut updated_or_new_mods = Vec::new();
    let mut found_mod_paths = std::collections::HashSet::new();

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

            // --- Filter Check (Recursive, limited depth) ---
            let mut is_valid_skin_mod = false;
            // Use WalkDir to check recursively up to depth 4 (root + 3 levels)
            for inner_entry in WalkDir::new(path)
                .max_depth(4)
                .into_iter()
                .filter_map(Result::ok)
            {
                let inner_path = inner_entry.path();

                // Check if it's a directory named "natives"
                if inner_path.is_dir() && inner_entry.file_name().to_str() == Some("natives") {
                    is_valid_skin_mod = true;
                    log::debug!("Found 'natives' directory inside: {}", inner_path.display());
                    break; // Found one condition, no need to check further
                }

                // Check if it's a file with a .pak extension
                if inner_path.is_file() {
                    if let Some(ext) = inner_path.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("pak") {
                            is_valid_skin_mod = true;
                            log::debug!("Found .pak file inside: {}", inner_path.display());
                            break; // Found one condition, no need to check further
                        }
                    }
                }
            }

            // Skip if neither condition was met during the recursive check
            if !is_valid_skin_mod {
                log::debug!("Skipping directory {:?}: No 'natives' subdir or .pak file found within depth 4.", path);
                continue;
            }
            // --- End Filter Check ---

            // Get mod path as string
            let mod_path = path.to_string_lossy().to_string();
            found_mod_paths.insert(mod_path.clone());

            // Check if we already have this mod in the registry
            if let Some(mut existing_mod) = existing_mods.remove(&mod_path) {
                // Make existing_mod mutable

                // --- Re-apply name extraction logic for existing mods ---
                let folder_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&existing_mod.base.directory_name) // Fallback to existing dir name if needed
                    .to_string();

                let delimiters: &[char] = &['_', '-', ' ', '!', '#', '$', '.', '(', '['];
                let cleaned_folder_name: String = folder_name
                    .chars()
                    .filter(|c| !c.is_whitespace() && *c != '\\')
                    .collect();

                // --- Refined Name Extraction Logic (Handles MHW/MHWs prefix) ---
                let display_name = match cleaned_folder_name.find(delimiters) {
                    Some(first_delim_index) => {
                        let prefix = &cleaned_folder_name[..first_delim_index];
                        if prefix.eq_ignore_ascii_case("mhw") || prefix.eq_ignore_ascii_case("mhws")
                        {
                            // Found MHW(s) prefix, look at the part *after* the delimiter
                            let suffix = &cleaned_folder_name[first_delim_index + 1..];
                            match suffix.find(delimiters) {
                                Some(second_delim_index) => {
                                    suffix[..second_delim_index].to_string()
                                } // Take part before next delimiter
                                None => suffix.to_string(), // No more delimiters, take the whole suffix
                            }
                        } else {
                            // Prefix is not MHW(s), just use the prefix
                            prefix.to_string()
                        }
                    }
                    None => cleaned_folder_name, // No delimiters found, use the whole cleaned name
                };
                // --- End Refined Name Extraction ---

                // Update the name in the existing mod struct if it changed
                if existing_mod.base.name != display_name {
                    log::debug!(
                        "Updating name for existing mod '{}': '{}' -> '{}'",
                        mod_path,
                        existing_mod.base.name,
                        display_name
                    );
                    existing_mod.base.name = display_name;
                }
                // --- End re-applying name extraction ---

                // --- Always re-check for screenshot for existing mods --- 
                let current_screenshot_path = find_screenshot(path);
                if existing_mod.thumbnail_path != current_screenshot_path {
                    log::debug!(
                        "Updating thumbnail path for existing mod '{}': {:?} -> {:?}",
                        mod_path,
                        existing_mod.thumbnail_path,
                        current_screenshot_path
                    );
                    existing_mod.thumbnail_path = current_screenshot_path;
                }
                // --- End screenshot re-check ---

                // --- Re-check installed files if mod is enabled ---
                if existing_mod.base.enabled {
                    // If the mod is marked as enabled in registry, but installed files are missing, mark as disabled
                    let all_files_exist = existing_mod.installed_files.iter().all(|f| PathBuf::from(f).exists());
                    if !all_files_exist {
                        log::warn!("Mod '{}' was enabled but installed files are missing. Disabling in registry.", mod_path);
                        existing_mod.base.enabled = false;
                        existing_mod.installed_files.clear();
                        existing_mod.installed_pak_path = None;
                        // We should probably trigger a save here or after the loop
                    }
                }
                // --- End re-check installed files ---

                updated_or_new_mods.push(existing_mod); // Push the potentially updated mod
                log::debug!("Found existing mod in registry: {}", mod_path);
                continue;
            }

            // If not in registry, it's a new mod
            log::debug!("Found new potential skin mod: {}", mod_path);
            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            // --- Refined Name Extraction ---
            let delimiters: &[char] = &['_', '-', ' ', '!', '#', '$', '.', '(', '['];
            let cleaned_folder_name: String = folder_name
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '\\')
                .collect();

            let display_name = match cleaned_folder_name.find(delimiters) {
                Some(first_delim_index) => {
                    let prefix = &cleaned_folder_name[..first_delim_index];
                    if prefix.eq_ignore_ascii_case("mhw") || prefix.eq_ignore_ascii_case("mhws") {
                        // Found MHW(s) prefix, look at the part *after* the delimiter
                        let suffix = &cleaned_folder_name[first_delim_index + 1..];
                        match suffix.find(delimiters) {
                            Some(second_delim_index) => suffix[..second_delim_index].to_string(), // Take part before next delimiter
                            None => suffix.to_string(), // No more delimiters, take the whole suffix
                        }
                    } else {
                        // Prefix is not MHW(s), just use the prefix
                        prefix.to_string()
                    }
                }
                None => cleaned_folder_name, // No delimiters found, use the whole cleaned name
            };
            // --- End Refined Name Extraction ---

            let screenshot_path = find_screenshot(path);

            // Create the base Mod struct
            let base_mod = Mod {
                name: display_name.clone(),
                directory_name: folder_name, // Keep original folder name as directory_name
                path: mod_path.clone(),
                enabled: false,    // New mods start disabled
                author: None,      // TODO: Parse from modinfo.ini
                version: None,     // TODO: Parse from modinfo.ini
                description: None, // TODO: Parse from modinfo.ini
                source: Some("local_scan".to_string()),
                installed_timestamp: chrono::Utc::now().timestamp(),
                installed_directory: mod_path.clone(), // Use mod path as identifier for skins
                mod_type: ModType::SkinMod,
            };

            // Create the SkinMod struct
            let skin_mod = SkinMod {
                base: base_mod,
                thumbnail_path: screenshot_path,
                conflicts: Vec::new(),
                files: Vec::new(), // Files are populated on enable
                installed_files: Vec::new(),
                installed_pak_path: None,
            };
            log::info!(
                "Adding new skin mod: Name='{}', Path='{}'",
                display_name,
                mod_path
            );
            updated_or_new_mods.push(skin_mod);
        }
    }

    // Update registry with the latest list (removes mods no longer found on disk)
    registry.skin_mods = updated_or_new_mods;
    registry.last_updated = chrono::Utc::now().timestamp();
    registry.save(&app_handle)?;

    log::info!(
        "Scan complete. Registry contains {} skin mods",
        registry.skin_mods.len()
    );
    Ok(registry.skin_mods)
}

#[tauri::command]
pub async fn enable_skin_mod_via_registry(
    app_handle: AppHandle,
    game_root_path: String,
    mod_path: String, // Use the original path as identifier
) -> Result<(), String> {
    log::info!("Enabling skin mod via registry: {}", mod_path);

    let game_root = PathBuf::from(&game_root_path);
    if !game_root.exists() || !game_root.is_dir() {
        return Err(format!("Invalid game root path: {}", game_root_path));
    }

    let mod_dir = PathBuf::from(&mod_path);
    if !mod_dir.exists() || !mod_dir.is_dir() {
        return Err(format!("Invalid mod path: {}", mod_path));
    }

    // Load the registry
    let mut registry = ModRegistry::load(&app_handle)?;

    // Find the mod to enable
    let mod_index = registry
        .skin_mods
        .iter()
        .position(|m| m.base.path == mod_path)
        .ok_or_else(|| format!("SkinMod with path '{}' not found in registry", mod_path))?;

    // Check if already enabled
    if registry.skin_mods[mod_index].base.enabled {
        log::info!("SkinMod '{}' is already enabled.", mod_path);
        // Optionally, verify installed files here and reinstall if needed?
        // For now, just return Ok.
        return Ok(());
    }

    // Get mutable reference to the mod we are enabling
    // Do this early to ensure we can update it later
    let skin_mod_entry = registry.skin_mods.get_mut(mod_index).unwrap();

    // Clear any potentially stale installed file data before starting
    skin_mod_entry.installed_files.clear();
    skin_mod_entry.installed_pak_path = None;

    let mut installed_files_tracker = Vec::new();
    let mut installed_pak_path_tracker: Option<String> = None;


    // Walk the mod directory to find .pak and natives/ files
    log::debug!("Scanning mod directory {} for files to install", mod_dir.display());
    let natives_prefix = mod_dir.join("natives");
    let game_natives_dir = game_root.join("natives");

    for entry_res in WalkDir::new(&mod_dir).into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                log::warn!("Error walking mod directory {}: {}", mod_dir.display(), err);
                continue; // Skip problematic entries
            }
        };

        let source_path = entry.path();

        // Skip directories
        if !source_path.is_file() {
            continue;
        }

        // --- Handle .pak files ---
        if source_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("pak")) && source_path.parent() == Some(&mod_dir) {
            // Only process .pak files directly in the mod root for now
            // TODO: Decide if we need to handle .pak in subdirs differently

            let next_patch_num = find_next_available_patch_number(&game_root)?;
            let pak_file_name = format!("re_chunk_000.pak.sub_000.pak.patch_{:03}.pak", next_patch_num);
            let dest_path = game_root.join(&pak_file_name);

            log::info!(
                "Installing .pak file: {} -> {} (as {})",
                source_path.display(),
                dest_path.display(),
                pak_file_name
            );

            fs::copy(source_path, &dest_path).map_err(|e| {
                format!(
                    "Failed to copy .pak file {} to {}: {}",
                    source_path.display(),
                    dest_path.display(),
                    e
                )
            })?;

            let dest_path_str = dest_path.to_string_lossy().to_string();
            installed_files_tracker.push(dest_path_str.clone());
            // Assume only one pak file per mod for now, overwrite if multiple found
            installed_pak_path_tracker = Some(dest_path_str);

        // --- Handle natives files ---
        } else if source_path.starts_with(&natives_prefix) {
            let rel_path = match source_path.strip_prefix(&natives_prefix) {
                Ok(p) => p,
                Err(_) => {
                    log::warn!("Failed to strip prefix for natives file: {}", source_path.display());
                    continue; // Skip if path logic fails
                }
            };

            let dest_path = game_natives_dir.join(rel_path);

            // Ensure parent directory exists in game natives
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!("Failed to create natives subdirectory {}: {}", parent.display(), e)
                    })?;
                    log::debug!("Created directory: {}", parent.display());
                }
            }

            log::info!(
                "Installing natives file: {} -> {}",
                source_path.display(),
                dest_path.display()
            );
            fs::copy(source_path, &dest_path).map_err(|e| {
                format!(
                    "Failed to copy natives file {} to {}: {}",
                    source_path.display(),
                    dest_path.display(),
                    e
                )
            })?;
            installed_files_tracker.push(dest_path.to_string_lossy().to_string());
        } else {
             log::trace!("Skipping file during install (not .pak in root or under natives/): {}", source_path.display());
        }
    }


    // --- Update the registry entry ---
    // We already have skin_mod_entry as a mutable reference
    skin_mod_entry.base.enabled = true;
    skin_mod_entry.installed_files = installed_files_tracker; // Store the collected list
    skin_mod_entry.installed_pak_path = installed_pak_path_tracker; // Store the installed pak path

    log::info!(
        "Updated registry for '{}'. Enabled: {}, Installed Pak: {:?}, Total Installed Files: {}",
        mod_path,
        skin_mod_entry.base.enabled,
        skin_mod_entry.installed_pak_path,
        skin_mod_entry.installed_files.len()
    );

    // --- Save the updated registry ---
    registry.last_updated = chrono::Utc::now().timestamp();
    if let Err(e) = registry.save(&app_handle) {
        // Attempt to clean up installed files if save fails? This could be complex.
        // For now, just return the save error.
        log::error!("Failed to save registry after enabling mod {}: {}", mod_path, e);
        return Err(format!("Failed to save registry state after enabling mod: {}", e));
    }

    log::info!("Successfully enabled skin mod '{}' via registry.", mod_path);
    Ok(())
}

#[tauri::command]
pub async fn disable_skin_mod_via_registry(
    app_handle: AppHandle,
    _game_root_path: String, // Not strictly needed if paths are absolute, kept for consistency
    mod_path: String,        // Use the original path as identifier
) -> Result<(), String> {
    log::info!("Disabling skin mod via registry: {}", mod_path);

    // Load the registry
    let mut registry = ModRegistry::load(&app_handle)?;

    // Find the mod to disable
    let mod_index = registry
        .skin_mods
        .iter()
        .position(|m| m.base.path == mod_path)
        .ok_or_else(|| format!("SkinMod with path '{}' not found in registry", mod_path))?;

    // Check if already disabled
    if !registry.skin_mods[mod_index].base.enabled {
        log::info!("SkinMod '{}' is already disabled.", mod_path);
        return Ok(());
    }

    // Get the list of installed files TO REMOVE
    // Clone it so we don't borrow registry while modifying filesystem
    let installed_files_to_remove = registry.skin_mods[mod_index].installed_files.clone();

    // Get mutable reference to the mod entry BEFORE removing files
    let skin_mod_entry = registry.skin_mods.get_mut(mod_index).unwrap();

    log::info!(
        "Removing {} installed files for mod: {}",
        installed_files_to_remove.len(),
        mod_path
    );

    // Remove installed files from the filesystem
    let mut removal_errors = Vec::new();
    for file_path_str in &installed_files_to_remove {
        let file_path = PathBuf::from(file_path_str);
        if file_path.exists() {
            log::debug!("Removing file: {}", file_path.display());
            if let Err(e) = fs::remove_file(&file_path) {
                // Log error but continue trying to remove other files
                log::warn!("Failed to remove file {}: {}", file_path.display(), e);
                removal_errors.push(format!("Failed to remove {}: {}", file_path.display(), e));
            }
        } else {
            log::warn!(
                "File listed in registry for '{}' not found during removal at path: {}",
                mod_path,
                file_path.display()
            );
            // File might have been manually deleted, which is okay for disabling.
        }
    }

    // --- Update the registry entry ---
    // This happens regardless of removal errors to reflect the *desired* state
    skin_mod_entry.base.enabled = false;
    skin_mod_entry.installed_files.clear(); // Clear the list
    skin_mod_entry.installed_pak_path = None; // Clear the pak path

    log::info!(
        "Updated registry for '{}'. Enabled: {}, Cleared installed files and pak path.",
        mod_path,
        skin_mod_entry.base.enabled
    );


    // --- Save the updated registry ---
    registry.last_updated = chrono::Utc::now().timestamp();
    if let Err(e) = registry.save(&app_handle) {
        log::error!("Failed to save registry after disabling mod {}: {}", mod_path, e);
        // Even if save fails, files might have been removed. State is inconsistent.
        return Err(format!("Failed to save registry state after disabling mod: {}", e));
    }


    // Report any errors encountered during file removal, but don't fail the operation
    if !removal_errors.is_empty() {
        log::error!(
            "Errors occurred during file removal for '{}': {}. Registry state updated anyway.",
            mod_path,
            removal_errors.join("; ")
        );
        // Consider if this should be an error communicated to the user,
        // even if the registry update succeeded. For now, log it as error but return Ok.
    }

    log::info!(
        "Successfully disabled skin mod '{}' via registry.",
        mod_path
    );
    Ok(())
}

// --------- End Skin Mod Management Commands --------- //

// --------- Delete Mod Commands --------- //

#[tauri::command]
pub async fn delete_reframework_mod(
    app_handle: AppHandle,
    game_root_path: String,
    mod_name: String,
) -> Result<(), String> {
    log::info!("Attempting to delete REFramework mod: {}", mod_name);
    let game_root = PathBuf::from(&game_root_path);

    // Load the registry
    let mut registry = ModRegistry::load(&app_handle)?;

    // Find the mod entry
    let mod_entry = match registry.find_mod(&mod_name) {
        Some(m) => m.clone(), // Clone needed info
        None => return Err(format!("REFramework mod '{}' not found in registry for deletion.", mod_name)),
    };

    // Determine the path(s) to delete (could be enabled or disabled)
    let installed_dir_rel = PathBuf::from(&mod_entry.installed_directory);
    let enabled_path = game_root.join(&installed_dir_rel);
    let disabled_path_str = format!("{}.disabled", mod_entry.installed_directory);
    let disabled_path = game_root.join(PathBuf::from(&disabled_path_str));

    let mut deleted_fs = false;
    let mut fs_errors = Vec::new();

    // Delete enabled directory if it exists
    if enabled_path.exists() {
        log::info!("Removing enabled directory: {}", enabled_path.display());
        if let Err(e) = fs::remove_dir_all(&enabled_path) {
            log::error!("Failed to remove directory {}: {}", enabled_path.display(), e);
            fs_errors.push(format!("Failed to remove {}: {}", enabled_path.display(), e));
        } else {
            deleted_fs = true;
        }
    }

    // Delete disabled directory if it exists
    if disabled_path.exists() {
        log::info!("Removing disabled directory: {}", disabled_path.display());
        if let Err(e) = fs::remove_dir_all(&disabled_path) {
            log::error!("Failed to remove directory {}: {}", disabled_path.display(), e);
            fs_errors.push(format!("Failed to remove {}: {}", disabled_path.display(), e));
        } else {
            deleted_fs = true;
        }
    }

    if !deleted_fs && !fs_errors.is_empty() {
        // If neither path existed but we still got errors somehow?
        log::warn!("Mod '{}' directory not found, but encountered errors: {}", mod_name, fs_errors.join("; "));
        // Proceed to remove from registry anyway
    } else if !deleted_fs {
        log::warn!("Mod '{}' directory not found at expected paths: {} or {}. Proceeding to remove registry entry.",
                   mod_name, enabled_path.display(), disabled_path.display());
    }

    // Remove from registry regardless of filesystem state (if it exists)
    if registry.remove_mod(&mod_name) {
        log::info!("Removed mod '{}' from registry.", mod_name);
        registry.last_updated = chrono::Utc::now().timestamp();
        if let Err(e) = registry.save(&app_handle) {
            log::error!("Failed to save registry after removing mod '{}': {}", mod_name, e);
            // Combine FS errors with save error
            fs_errors.push(format!("Failed to save registry: {}", e));
        }
    } else {
        log::warn!("Mod '{}' was not found in the registry during deletion attempt, maybe already removed?", mod_name);
        // This case should ideally not happen due to the initial find_mod check
    }

    // Return success or failure based on combined errors
    if fs_errors.is_empty() {
        log::info!("Successfully deleted REFramework mod '{}'.", mod_name);
        Ok(())
    } else {
        Err(format!("Errors occurred during deletion of mod '{}': {}", mod_name, fs_errors.join("; ")))
    }
}


#[tauri::command]
pub async fn delete_skin_mod(
    app_handle: AppHandle,
    game_root_path: String, // Needed for potential disable call
    mod_path: String,       // Original source path identifier
) -> Result<(), String> {
    log::info!("Attempting to delete skin mod with source path: {}", mod_path);

    let app_handle_clone = app_handle.clone(); // Clone for potential disable call

    // Load the registry
    let mut registry = ModRegistry::load(&app_handle)?;

    // Find the mod entry by its original source path
    let mod_info = match registry.skin_mods.iter().find(|m| m.base.path == mod_path) {
        Some(m) => {
            // Clone necessary info before potential mutable borrow in disable
            Some((m.base.directory_name.clone(), m.base.enabled))
        }
        None => {
            return Err(format!("Skin mod with source path '{}' not found in registry.", mod_path));
        }
    };

    let (directory_name_to_remove, is_enabled) = mod_info.unwrap(); // We know it exists

    let mut combined_errors = Vec::new();

    // --- Step 1: Disable the mod first if it's enabled --- 
    // This handles removing files from the game directory (.pak, natives/)
    if is_enabled {
        log::info!("Skin mod '{}' is enabled, disabling it first...", directory_name_to_remove);
        if let Err(e) = disable_skin_mod_via_registry(app_handle_clone, game_root_path, mod_path.clone()).await {
            log::error!("Failed to disable skin mod '{}' before deletion: {}. Proceeding with deletion attempt anyway.", directory_name_to_remove, e);
            combined_errors.push(format!("Error during pre-delete disable: {}", e));
            // Reload registry as disable might have failed partially but still saved
            registry = ModRegistry::load(&app_handle)?;
        } else {
            log::info!("Successfully disabled skin mod '{}' before deletion.", directory_name_to_remove);
            // Reload registry as disable function saves it
            registry = ModRegistry::load(&app_handle)?;
        }
    }

    // --- Step 2: Remove the original mod source directory --- 
    let source_mod_dir = PathBuf::from(&mod_path);
    if source_mod_dir.exists() {
        log::info!("Removing original source directory: {}", source_mod_dir.display());
        if let Err(e) = fs::remove_dir_all(&source_mod_dir) {
            log::error!("Failed to remove source directory {}: {}", source_mod_dir.display(), e);
            combined_errors.push(format!("Failed to remove source dir {}: {}", source_mod_dir.display(), e));
        }
    } else {
        log::warn!("Original source directory not found for skin mod '{}' at path: {}. Skipping removal.",
                   directory_name_to_remove, source_mod_dir.display());
    }

    // --- Step 3: Remove the mod from the registry --- 
    if registry.remove_skin_mod(&directory_name_to_remove) {
        log::info!("Removed skin mod '{}' from registry.", directory_name_to_remove);
        registry.last_updated = chrono::Utc::now().timestamp();
        if let Err(e) = registry.save(&app_handle) {
            log::error!("Failed to save registry after removing skin mod '{}': {}", directory_name_to_remove, e);
            combined_errors.push(format!("Failed to save registry: {}", e));
        }
    } else {
        // This might happen if disable failed and registry was reloaded without the mod?
        log::warn!("Skin mod '{}' was not found in the registry during final removal attempt.", directory_name_to_remove);
    }

    // --- Final Result --- 
    if combined_errors.is_empty() {
        log::info!("Successfully deleted skin mod from '{}'.", mod_path);
        Ok(())
    } else {
        Err(format!("Errors occurred during deletion of skin mod from '{}': {}", mod_path, combined_errors.join("; ")))
    }
}

// +++ Add back the list_skin_mods_from_registry command +++
#[tauri::command]
pub async fn list_skin_mods_from_registry(app_handle: AppHandle) -> Result<Vec<SkinMod>, String> {
    log::info!("Listing installed skin mods from registry");
    // Consider adding a scan here too if needed, similar to list_mods
    // For now, just load and return
    let registry = ModRegistry::load(&app_handle)?;
    Ok(registry.skin_mods)
}
