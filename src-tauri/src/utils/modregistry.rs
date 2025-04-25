// mod_registry.rs - Place this in src-tauri/src/utils/ directory

use log::{error, info, debug, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Core representation of a mod in the registry
#[derive(Debug, Serialize, Deserialize, Clone)]

#[allow(unused_exports)]

pub struct Mod {
    // Core identification
    pub name: String,                    // Display name (user-friendly)
    pub directory_name: String,          // Folder name or identifier
    pub path: String,                    // Original path in mods directory
    
    // Status
    pub enabled: bool,                   // Whether this mod is currently enabled
    
    // Metadata
    pub author: Option<String>,          // Author information if available
    pub version: Option<String>,         // Version information if available
    pub description: Option<String>,     // Mod description if available
    pub source: Option<String>,          // Where the mod came from (e.g., "local_zip", "nexus")
    pub installed_timestamp: i64,        // When this mod was installed (unix timestamp)
    
    // File specific info
    pub installed_directory: String,     // Relative path from game root
    pub mod_type: ModType,               // Type categorization
}

/// Types of mods that can be installed
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModType {
    REFrameworkPlugin,     // Installed to reframework/plugins/
    REFrameworkAutorun,    // Installed to reframework/autorun/
    SkinMod,               // Various appearance mods
    NativesMod,            // Files for the natives directory
    Other                  // Any other mod type
}

/// For skin mods with additional capabilities
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinMod {
    #[serde(flatten)]
    pub base: Mod,                      // Include all base mod fields
    pub thumbnail_path: Option<String>, // Path to preview image
    pub conflicts: Vec<String>,         // List of other mods this conflicts with
    pub files: Vec<ModFile>,            // Individual files included in this skin mod
}

/// Structure to track individual files within a mod for conflict resolution
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub relative_path: String,          // Path relative to game root
    pub original_path: String,          // Path in the original mod folder
    pub file_type: ModFileType,         // Type of file (PAK, natives, etc.)
    pub enabled: bool,                  // Whether this specific file is enabled
    pub size_bytes: u64,                // File size for information
}

/// Enum to categorize mod files
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ModFileType {
    PakFile,                            // .pak file
    NativesFile,                        // File inside natives directory
    Other,                              // Other files
}

/// The complete registry containing all mods
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModRegistry {
    pub mods: Vec<Mod>,                 // Regular mods (REFramework plugins/autorun)
    pub skin_mods: Vec<SkinMod>,        // Skin mods with additional metadata
    pub last_updated: i64,              // When registry was last updated (unix timestamp)
    pub format_version: u32,            // For future migration needs (start with 1)
}

/// Frontend-friendly view of a mod (for compatibility with existing frontend code)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub directory_name: String,         // Identifier for the mod
    pub name: Option<String>,           // Display name
    pub version: Option<String>,        // Version if available
    pub author: Option<String>,         // Author if available
    pub description: Option<String>,    // Description if available
    pub enabled: bool,                  // Whether enabled or not
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
                        info!("Successfully loaded mod registry with {} mods and {} skin mods", 
                            registry.mods.len(), registry.skin_mods.len());
                        Ok(registry)
                    },
                    Err(e) => {
                        // Handle legacy format
                        warn!("Failed to parse registry file as ModRegistry: {}", e);
                        Self::migrate_from_legacy(content, app_handle)
                    }
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Should never happen as we already checked existence
                Ok(Self::new())
            },
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
                info!("Found legacy ModListContainer format with {} mods and {} skins",
                      container.mods.len(), container.skins.len());
                
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
                        files: Vec::new(), // Will be populated on refresh
                    };
                    
                    registry.skin_mods.push(skin_mod);
                }
            },
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
                    },
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
        self.mods.iter()
            .filter(|m| m.mod_type == ModType::REFrameworkPlugin || m.mod_type == ModType::REFrameworkAutorun)
            .map(Self::to_mod_info)
            .collect()
    }

    /// Get skin mods as ModInfo objects
    pub fn get_skin_mod_info(&self) -> Vec<ModInfo> {
        self.skin_mods.iter()
            .map(Self::skin_to_mod_info)
            .collect()
    }

    /// Find a mod by directory name
    pub fn find_mod(&self, directory_name: &str) -> Option<&Mod> {
        self.mods.iter().find(|m| m.directory_name == directory_name)
    }

    /// Find a mod by directory name (mutable)
    pub fn find_mod_mut(&mut self, directory_name: &str) -> Option<&mut Mod> {
        self.mods.iter_mut().find(|m| m.directory_name == directory_name)
    }

    /// Find a skin mod by directory name
    pub fn find_skin_mod(&self, directory_name: &str) -> Option<&SkinMod> {
        self.skin_mods.iter().find(|m| m.base.directory_name == directory_name)
    }

    /// Find a skin mod by directory name (mutable)
    pub fn find_skin_mod_mut(&mut self, directory_name: &str) -> Option<&mut SkinMod> {
        self.skin_mods.iter_mut().find(|m| m.base.directory_name == directory_name)
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
                warn!("Mod '{}' has both enabled and disabled directories present! Assuming enabled.",
                     mod_entry.name);
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
        self.mods.retain(|m| m.directory_name != new_mod.directory_name);
        // Add the new mod
        self.mods.push(new_mod);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Add a new skin mod to the registry
    pub fn add_skin_mod(&mut self, new_skin_mod: SkinMod) {
        // Remove any existing skin mod with same directory name
        self.skin_mods.retain(|m| m.base.directory_name != new_skin_mod.base.directory_name);
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
        self.skin_mods.retain(|m| m.base.directory_name != directory_name);
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
    pub fn toggle_skin_mod_enabled(&mut self, directory_name: &str, enable: bool) -> Result<(), String> {
        // Find the skin mod
        if let Some(skin_mod) = self.find_skin_mod_mut(directory_name) {
            skin_mod.base.enabled = enable;
            self.last_updated = chrono::Utc::now().timestamp();
            Ok(())
        } else {
            Err(format!("Skin mod '{}' not found in registry", directory_name))
        }
    }
}

// Utility functions

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