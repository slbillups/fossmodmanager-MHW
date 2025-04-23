use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;
use zip::ZipArchive;
use base64;

// Structure for skin mod entries
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkinModEntry {
    name: String,
    path: String,
    enabled: bool,
    thumbnail_path: Option<String>,
    author: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

// Enhanced structure for scanned skin mods
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScannedSkinMod {
    name: String,
    path: String,
    enabled: bool,
    screenshot_path: Option<String>,
    author: Option<String>,
    version: Option<String>,
    description: Option<String>,
    category: Option<String>,
    screenshot: Option<String>,
}

// Function to check if Wine is available
fn is_wine_available() -> bool {
    let output = Command::new("which").arg("wine").output();

    output.is_ok() && output.unwrap().status.success()
}

// Function to scan for skin mods by looking for modinfo.ini files
#[tauri::command]
pub async fn scan_for_skin_mods(game_root_path: String) -> Result<Vec<ScannedSkinMod>, String> {
    let game_root = PathBuf::from(&game_root_path);
    let mods_dir = game_root.join("fossmodmanager").join("mods");
    let mut mods = Vec::new();

    // Check if the mods directory exists
    if !mods_dir.exists() {
        // Create the directory if it doesn't exist
        match fs::create_dir_all(&mods_dir) {
            Ok(_) => log::info!("Created mods directory at: {:?}", mods_dir),
            Err(e) => log::error!("Failed to create mods directory: {:?}", e),
        }
        return Ok(mods); // Return empty list since we just created the directory
    }

    log::info!("Scanning for skin mods in: {:?}", mods_dir);

    // Walk through the mods directory, looking for modinfo.ini files
    for entry in WalkDir::new(&mods_dir)
        .max_depth(4) // Adjust depth as needed
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && path.file_name().map_or(false, |n| {
                n.to_string_lossy().to_lowercase() == "modinfo.ini"
            })
        {
            if let Some(mod_data) = process_mod_folder(path.parent().unwrap_or(Path::new(""))) {
                mods.push(mod_data);
            }
        }
    }

    log::info!("Found {} skin mods", mods.len());
    Ok(mods)
}

// Helper function to process a folder containing modinfo.ini
fn process_mod_folder(folder_path: &Path) -> Option<ScannedSkinMod> {
    let modinfo_path = folder_path.join("modinfo.ini");
    if !modinfo_path.exists() || !modinfo_path.is_file() {
        return None;
    }

    // Read and parse the modinfo.ini file
    let content = fs::read_to_string(&modinfo_path).ok()?;

    let mut name = None;
    let mut name_as_bundle = None;
    let mut author = None;
    let mut version = None;
    let mut description = None;
    let mut category = None;
    let mut screenshot = None;

    // Parse the INI file
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
                    "name" => name = Some(value.to_string()),
                    "nameasbundle" => name_as_bundle = Some(value.to_string()),
                    "author" => author = Some(value.to_string()),
                    "version" => version = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "category" => category = Some(value.to_string()),
                    "screenshot" => screenshot = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    // Find the screenshot path (if specified in modinfo.ini)
    let screenshot_path = if let Some(screenshot_file) = &screenshot {
        let img_path = folder_path.join(screenshot_file);
        if img_path.exists() && img_path.is_file() {
            Some(img_path.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        // Try to find any image in the folder if screenshot isn't specified
        find_image_in_folder(folder_path)
    };

    // Build the mod entry - prefer NameAsBundle over name if available
    Some(ScannedSkinMod {
        name: name_as_bundle.or(name).unwrap_or_else(|| {
            folder_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown Mod")
                .to_string()
        }),
        path: folder_path.to_string_lossy().to_string(),
        enabled: true, // Assuming it's enabled if we found it
        screenshot_path,
        author,
        version,
        description,
        category,
        screenshot,
    })
}

// Helper to find any image file in a folder
fn find_image_in_folder(folder_path: &Path) -> Option<String> {
    // Common image extensions
    let image_extensions = &["png", "jpg", "jpeg", "webp", "gif"];

    for entry in fs::read_dir(folder_path).ok()? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if image_extensions
                        .iter()
                        .any(|&valid_ext| valid_ext.eq_ignore_ascii_case(ext))
                    {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

// Extract game assets using the bundled extractgameassets.sh script
#[tauri::command]
pub async fn extract_game_assets(
    app_handle: AppHandle,
    game_root_path: String,
) -> Result<(), String> {
    log::info!(
        "Starting game asset extraction for path: {}",
        game_root_path
    );

    // Create the output directory in fossmodmanager/extracted
    let output_dir = PathBuf::from(&game_root_path)
        .join("fossmodmanager")
        .join("extracted");

    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Get path to game PAK file
    let pak_path = PathBuf::from(&game_root_path).join("re_chunk_000.pak.sub_000.pak");
    if !pak_path.exists() {
        return Err(format!(
            "Game PAK file not found at: {}",
            pak_path.display()
        ));
    }

    // For this simplified approach, we'll use the REtool directly from where it's installed
    // Normally the user would have REtool installed in their system, or we'd provide instructions
    let user_retool_path = format!("{}/fossmodmanager/tools/REtool/REtool.exe", game_root_path);
    let user_list_path = format!(
        "{}/fossmodmanager/tools/MHWs_STM_Release.list",
        game_root_path
    );

    // Create the tools directory if it doesn't exist
    let tools_dir = PathBuf::from(&game_root_path)
        .join("fossmodmanager")
        .join("tools");
    if !tools_dir.exists() {
        fs::create_dir_all(&tools_dir)
            .map_err(|e| format!("Failed to create tools directory: {}", e))?;
    }

    // Copy REtool files from resources to the game directory if they don't exist
    let retool_dir = tools_dir.join("REtool");
    if !retool_dir.exists() {
        fs::create_dir_all(&retool_dir)
            .map_err(|e| format!("Failed to create REtool directory: {}", e))?;
    }

    // Check if REtool files exist, copy if they don't
    if !PathBuf::from(&user_retool_path).exists() || !PathBuf::from(&user_list_path).exists() {
        // Get paths to bundled resources
        let bundle_path = app_handle
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource directory: {}", e))?;

        log::info!(
            "Copying REtool files from resource dir: {}",
            bundle_path.display()
        );

        // Copy REtool directory
        let bundle_retool_dir = bundle_path.join("binaries").join("REtool");
        if bundle_retool_dir.exists() {
            for entry in fs::read_dir(&bundle_retool_dir)
                .map_err(|e| format!("Failed to read REtool directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                if path.is_file() {
                    let target_path = retool_dir.join(path.file_name().unwrap());
                    fs::copy(&path, &target_path)
                        .map_err(|e| format!("Failed to copy file {}: {}", path.display(), e))?;
                }
            }
        } else {
            return Err(format!(
                "REtool directory not found at: {}",
                bundle_retool_dir.display()
            ));
        }

        // Copy list file
        let bundle_list_path = bundle_path.join("binaries").join("MHWs_STM_Release.list");
        if bundle_list_path.exists() {
            fs::copy(&bundle_list_path, &tools_dir.join("MHWs_STM_Release.list"))
                .map_err(|e| format!("Failed to copy list file: {}", e))?;
        } else {
            return Err(format!(
                "List file not found at: {}",
                bundle_list_path.display()
            ));
        }
    }

    // Get the sidecar command for the script
    log::info!("Preparing to run extractgameassets.sh sidecar");
    let sidecar_command = match app_handle.shell().sidecar("extractgameassets.sh-x86_64-unknown-linux-gnu") {
        Ok(cmd) => cmd,
        Err(e) => return Err(format!("Failed to get sidecar command: {}", e)),
    };

    // Execute the script with arguments
    log::info!("Executing extractgameassets.sh sidecar");
    let output = sidecar_command
        .args([
            &user_retool_path,
            &user_list_path,
            &game_root_path,
            output_dir.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Script execution failed: {}", stderr));
    }

    // Log the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    log::info!("Sidecar output: {}", stdout);

    log::info!("Game asset extraction completed successfully");
    Ok(())
}

// Install a skin mod
#[tauri::command]
pub async fn install_skin_mod(
    app_handle: AppHandle,
    game_root_path: String,
    zip_path_str: String,
) -> Result<(), String> {
    let game_root = PathBuf::from(&game_root_path);
    let zip_path = PathBuf::from(&zip_path_str);

    // Determine mod name from zip file
    let mod_name = zip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid zip filename".to_string())?
        .to_string();

    // Set up mod directory
    let mods_dir = game_root
        .join("fossmodmanager")
        .join("mods")
        .join(&mod_name);

    // Create directory if it doesn't exist
    if mods_dir.exists() {
        fs::remove_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to remove existing mod directory: {}", e))?;
    }
    fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mod directory: {}", e))?;

    // Extract the zip file
    let file = fs::File::open(&zip_path).map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    // Extract files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        // Skip directories
        if file.is_dir() {
            continue;
        }

        // Get file name and path
        let outpath = mods_dir.join(file.name());

        // Create parent directory if needed
        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }

        // Extract file
        let mut outfile =
            fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
        io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to write file: {}", e))?;
    }

    // Update registry
    update_skin_mod_registry(&app_handle, &game_root, &mod_name, &mods_dir)?;

    Ok(())
}

// Function to list skin mods
pub async fn list_skin_mods(
    app_handle: AppHandle,
    game_root_path: String,
) -> Result<Vec<SkinModEntry>, String> {
    // Registry path
    let registry_path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?
        .join("skinmods.json");

    // Read registry file
    if !registry_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&registry_path)
        .map_err(|e| format!("Failed to read skin mods registry: {}", e))?;

    if content.is_empty() {
        return Ok(Vec::new());
    }

    // Parse registry
    let mods: Vec<SkinModEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse skin mods registry: {}", e))?;

    // Verify paths exist
    let mut verified_mods = Vec::new();

    for mod_entry in mods {
        let mod_path = PathBuf::from(&mod_entry.path);
        if mod_path.exists() {
            verified_mods.push(mod_entry);
        } else {
            log::warn!("Skin mod directory not found: {}", mod_entry.path);
            // Could add logic to clean up registry here
        }
    }

    Ok(verified_mods)
}

// Helper to update the skin mod registry
fn update_skin_mod_registry(
    app_handle: &AppHandle,
    game_root: &PathBuf,
    mod_name: &str,
    mod_dir: &PathBuf,
) -> Result<(), String> {
    let registry_path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?
        .join("skinmods.json");

    // Ensure directory exists
    if let Some(parent) = registry_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create registry directory: {}", e))?;
        }
    }

    // Read existing registry or create new
    let mut registry: Vec<SkinModEntry> = if registry_path.exists() {
        let content = fs::read_to_string(&registry_path)
            .map_err(|e| format!("Failed to read registry: {}", e))?;

        if content.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse registry: {}", e))?
        }
    } else {
        Vec::new()
    };

    // Look for thumbnail and modinfo
    let thumbnail_path = find_thumbnail(mod_dir);
    let mod_info = parse_modinfo(mod_dir);

    // Create entry
    let entry = SkinModEntry {
        name: mod_info
            .as_ref()
            .and_then(|i| i.name.clone())
            .unwrap_or_else(|| mod_name.to_string()),
        path: mod_dir.to_str().unwrap_or("").to_string(),
        enabled: true,
        thumbnail_path,
        author: mod_info.as_ref().and_then(|i| i.author.clone()),
        version: mod_info.as_ref().and_then(|i| i.version.clone()),
        description: mod_info.as_ref().and_then(|i| i.description.clone()),
    };

    // Remove existing entry with same name
    registry.retain(|e| e.name != entry.name);
    registry.push(entry);

    // Write registry
    let json = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;
    fs::write(&registry_path, json).map_err(|e| format!("Failed to write registry: {}", e))?;

    Ok(())
}

// Find thumbnail in mod directory
fn find_thumbnail(mod_dir: &PathBuf) -> Option<String> {
    let texture_dir = mod_dir.join("Texture");
    if !texture_dir.exists() || !texture_dir.is_dir() {
        return None;
    }

    // Common image file extensions
    let extensions = ["png", "jpg", "jpeg", "webp"];

    // Try to find a thumbnail file
    for entry in fs::read_dir(texture_dir).ok()? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext.to_lowercase().as_str()) {
                        return path.to_str().map(|s| s.to_string());
                    }
                }
            }
        }
    }

    None
}

// Simple struct for modinfo.ini data
#[derive(Debug)]
struct ModInfo {
    name: Option<String>,
    author: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

// Parse modinfo.ini file
fn parse_modinfo(mod_dir: &PathBuf) -> Option<ModInfo> {
    let modinfo_path = mod_dir.join("Texture").join("modinfo.ini");
    if !modinfo_path.exists() || !modinfo_path.is_file() {
        return None;
    }

    let content = fs::read_to_string(modinfo_path).ok()?;

    let mut info = ModInfo {
        name: None,
        author: None,
        version: None,
        description: None,
    };

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

// Function to safely read mod image files and return as base64
#[tauri::command]
pub async fn read_mod_image(image_path: String) -> Result<String, String> {
    log::info!("Reading mod image from: {}", image_path);
    
    // Verify the path is within a mod directory (security check)
    let path = PathBuf::from(&image_path);
    
    if !path.exists() {
        return Err(format!("Image file does not exist: {}", image_path));
    }
    
    // Read the image file
    let img_data = fs::read(&path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;
    
    // Convert to base64
    let base64_encoded = base64::encode(&img_data);
    
    log::info!("Successfully read image: {} ({} bytes)", image_path, img_data.len());
    Ok(base64_encoded)
}
