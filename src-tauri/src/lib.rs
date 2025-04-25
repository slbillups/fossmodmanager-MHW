use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};
use zip::ZipArchive;

use tauri_plugin_opener::OpenerExt;
// Declare the new module
mod nexus_api;
use nexus_api::ApiCache;
use reqwest;
use zip; // For async mutex if needed later

mod utils;
use crate::utils::tempermission::ModOperationEvent;
use utils::config::{
    delete_config, load_game_config, save_game_config, validate_game_installation,
};
use utils::tempermission::with_game_dir_write_access;
use utils::modregistry::{Mod, ModRegistry, ModType, SkinMod, ModFile, ModFileType, ModInfo};


// Struct representing mod metadata read from modinfo.json
// #[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
// struct ModInfo {
//     directory_name: String, // The name of the folder the mod resides in
//     name: Option<String>,
//     version: Option<String>,
//     author: Option<String>,
//     description: Option<String>,
//     enabled: bool, // Derived from directory name (presence/absence of _DISABLED_)
// }

// Removed Nexus struct definitions - they are now in nexus_api/mod.rs

// --- Structs for GitHub API Response ---
#[derive(Deserialize, Debug)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    assets: Vec<GitHubReleaseAsset>,
    tag_name: String, // Useful for logging/display
    prerelease: bool, // Nightly might be marked as prerelease
}
// --- End GitHub Structs ---

// --- Abstraction for an installable package (like REFramework) ---
#[derive(Debug, Clone)] // Clone might be useful
struct Package {
    name: String, // e.g., "REFramework"
                  // Could add version, repo URL etc. later if needed
}

impl Package {
    // Helper to create a REFramework package instance
    fn reframework() -> Self {
        Package {
            name: "REFramework".to_string(),
        }
    }

    // Checks if the package seems present based on specific file/folder markers
    async fn is_present(&self, game_root_path: &str) -> Result<bool, String> {
        log::info!("Checking for {} presence in: {}", self.name, game_root_path);
        let root = PathBuf::from(game_root_path);

        // Specific checks for REFramework
        if self.name == "REFramework" {
            let dinput_path = root.join("dinput8.dll");
            let reframework_dir_path = root.join("reframework");

            let installed = dinput_path.exists() || reframework_dir_path.is_dir();
            log::info!(" -> {} installed status: {}", self.name, installed);
            Ok(installed)
        } else {
            // Handle other package types later if needed
            log::warn!("Presence check not implemented for package: {}", self.name);
            Err(format!("Presence check not implemented for {}", self.name))
        }
    }

    // Ensures the package is installed (downloads/extracts if needed)
    async fn ensure_installed(
        &self,
        game_root_path: &str,
        // app_handle: &AppHandle // Might need app_handle later for config paths etc.
    ) -> Result<(), String> {
        log::info!("Ensuring {} is installed in: {}", self.name, game_root_path);

        if self.is_present(game_root_path).await? {
            log::info!("{} is already present. Skipping installation.", self.name);
            return Ok(());
        }

        log::info!("{} not found. Proceeding with installation...", self.name);

        // Specific logic for REFramework
        if self.name == "REFramework" {
            let target_dir = PathBuf::from(game_root_path);
            if !target_dir.is_dir() {
                return Err(format!(
                    "Target game directory does not exist: {}",
                    game_root_path
                ));
            }

            // 1. Fetch release info (using a new helper)
            log::info!("Fetching latest {} release info...", self.name);
            let release_info = fetch_latest_release("praydog", "REFramework-nightly").await?;
            log::info!(
                "Latest release tag: {}, Prerelease: {}",
                release_info.tag_name,
                release_info.prerelease
            );

            // 2. Find the correct asset URL (MHWilds.zip for now)
            // TODO: Make asset name configurable or dynamically determined?
            let asset_name = "MHWilds.zip";
            let asset = release_info
                .assets
                .iter()
                .find(|a| a.name == asset_name)
                .ok_or_else(|| {
                    format!(
                        "{} not found in latest release ({})",
                        asset_name, release_info.tag_name
                    )
                })?;
            log::info!("Found asset URL: {}", asset.browser_download_url);

            // 3. Download the asset (using a new helper)
            log::info!("Downloading {}...", asset.name);
            let zip_data = download_bytes(&asset.browser_download_url).await?;
            log::info!("Download complete ({} bytes)", zip_data.len());

            // 4. Extract (using the existing helper)
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data))
                .map_err(|e| format!("Failed to open zip archive: {}", e))?;

            let extracted_count = extract_reframework_files(&mut archive, &target_dir)?;

            if extracted_count == 0 {
                log::error!(
                    "{} installation failed: No relevant files found in zip.",
                    self.name
                );
                return Err(format!(
                    "{} installation failed: No relevant files found in zip.",
                    self.name
                ));
            }

            log::info!(
                "{} installation successful. Extracted {} items.",
                self.name,
                extracted_count
            );
            Ok(())
        } else {
            log::error!(
                "Installation logic not implemented for package: {}",
                self.name
            );
            Err(format!(
                "Installation logic not implemented for {}",
                self.name
            ))
        }
    }
}
// --- End Package Abstraction ---

// --- Placeholder Helper Functions ---
// TODO: Implement fetch_latest_release using reqwest and GitHub API
async fn fetch_latest_release(owner: &str, repo: &str) -> Result<GitHubRelease, String> {
    log::info!("Fetching latest release for {}/{}...", owner, repo);
    // Adapted from get_latest_reframework_url
    let client = reqwest::Client::builder()
        .user_agent("FossModManager/0.1.0") // GitHub requires a User-Agent
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
    log::debug!("Fetching releases from URL: {}", url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases from {}: {}", url, e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(format!(
            "GitHub API request failed for {}: Status {} - {}",
            url, status, text
        ));
    }

    log::debug!("Successfully fetched releases list for {}/{}.", owner, repo);

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub releases JSON from {}: {}", url, e))?;

    // Find the latest release (prefer non-prerelease, but take first if none)
    // This logic might need refinement depending on tagging conventions
    let mut releases_iter = releases.into_iter();
    let latest_release = releases_iter
        .find(|r| !r.prerelease)
        .or_else(|| releases_iter.next()) // Fallback to first if no non-prerelease
        .ok_or_else(|| format!("No releases found for {}/{}", owner, repo))?;

    log::info!(
        "Found latest suitable release for {}/{}: Tag {}, Prerelease: {}",
        owner,
        repo,
        latest_release.tag_name,
        latest_release.prerelease
    );
    Ok(latest_release)
}

// TODO: Implement download_bytes using reqwest
async fn download_bytes(url: &str) -> Result<bytes::Bytes, String> {
    log::info!("Downloading bytes from: {}", url);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download from {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download request failed from {}: Status {}",
            url,
            response.status()
        ));
    }

    let data = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download bytes from {}: {}", url, e))?;

    log::info!("Successfully downloaded {} bytes from {}", data.len(), url);
    Ok(data)
}
// --- End Placeholder Helpers ---

// --- Existing Helper: REFramework Selective Extraction ---
fn extract_reframework_files(
    archive: &mut zip::ZipArchive<std::io::Cursor<bytes::Bytes>>, // Take archive by mutable ref
    target_dir: &PathBuf,
) -> Result<usize, String> {
    // Return count of extracted files/dirs
    log::info!(
        "Starting REFramework selective extraction to {}",
        target_dir.display()
    );
    let mut extracted_count = 0;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("Error reading zip entry {}: {}. Skipping.", i, e);
                continue;
            }
        };
        // Use owned path for manipulation
        let entry_path = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => {
                log::warn!("Skipping potentially unsafe zip entry: {}", file.name());
                continue;
            }
        };

        // Filter logic: Must be dinput8.dll at root OR inside reframework/ directory
        let is_dinput = entry_path == PathBuf::from("dinput8.dll");
        let is_in_reframework_dir = entry_path.starts_with("reframework/");

        if !is_dinput && !is_in_reframework_dir {
            log::debug!(
                "Skipping entry (not dinput8.dll or in reframework/): {:?}",
                entry_path
            );
            continue; // Skip this file
        }

        // Determine the final output path relative to target_dir
        let outpath = target_dir.join(&entry_path);

        log::debug!("Processing entry: {:?} -> {:?}", entry_path, outpath);

        if file.name().ends_with('/') {
            log::debug!("Creating directory {}", outpath.display());
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory {}: {}", outpath.display(), e))?;
        } else {
            log::debug!("Extracting file {}", outpath.display());
            // Ensure parent directory exists
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| {
                        format!("Failed to create parent directory {}: {}", p.display(), e)
                    })?;
                }
            }
            // Overwrite strategy: remove existing first
            if outpath.exists() {
                log::warn!("Overwriting existing path: {}", outpath.display());
                if outpath.is_dir() {
                    fs::remove_dir_all(&outpath).map_err(|e| {
                        format!(
                            "Failed to remove existing directory before overwrite {}: {}",
                            outpath.display(),
                            e
                        )
                    })?;
                } else {
                    fs::remove_file(&outpath).map_err(|e| {
                        format!(
                            "Failed to remove existing file before overwrite {}: {}",
                            outpath.display(),
                            e
                        )
                    })?;
                }
            }

            let mut outfile = fs::File::create(&outpath).map_err(|e| {
                format!("Failed to create output file {}: {}", outpath.display(), e)
            })?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to copy content to {}: {}", outpath.display(), e))?;
            extracted_count += 1;
        }

        // Set permissions (optional)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                if let Err(e) = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)) {
                    log::warn!("Failed to set permissions on {}: {}", outpath.display(), e);
                }
            }
        }
    }

    log::info!(
        "REFramework selective extraction complete. {} files/dirs extracted.",
        extracted_count
    );
    Ok(extracted_count)
}
// --- End Helper ---

// --- REMOVE OLD Helper Function to get Latest REFramework URL ---
/*
async fn get_latest_reframework_url() -> Result<String, String> {
    // ... old implementation ...
}
*/

#[tauri::command]
async fn check_reframework_installed(game_root_path: String) -> Result<bool, String> {
    // Use the Package abstraction
    let reframework_pkg = Package::reframework();
    reframework_pkg.is_present(&game_root_path).await
}

// Rename this command to match todo.md and its behaviour
#[tauri::command]
async fn ensure_reframework(_app_handle: AppHandle, game_root_path: String) -> Result<(), String> {
    // Use the Package abstraction
    let reframework_pkg = Package::reframework();
    // Pass app_handle if needed by ensure_installed later (currently not needed)
    reframework_pkg.ensure_installed(&game_root_path).await
}

// Command to ensure the fossmodmanager/mods directory exists AND open it
#[tauri::command]
async fn open_mods_folder(app_handle: AppHandle, game_root_path: String) -> Result<(), String> {
    // Renamed, changed signature
    println!(
        "Ensuring and opening mod directory for path: {}",
        game_root_path
    );

    // Construct the mod directory path
    let mut mod_manager_dir = PathBuf::from(&game_root_path);
    mod_manager_dir.push("fossmodmanager");
    mod_manager_dir.push("mods"); // Ensure we target the 'mods' subdirectory

    let mods_path_str = mod_manager_dir
        .to_str()
        .ok_or_else(|| format!("Failed to convert mod path {:?} to string", mod_manager_dir))?;

    // Check and create if it doesn't exist
    if !mod_manager_dir.exists() {
        println!(
            "Mod directory does not exist, creating: {:?}\n",
            mod_manager_dir
        );
        fs::create_dir_all(&mod_manager_dir) // Use create_dir_all for robustness
            .map_err(|e| {
                format!(
                    "Failed to create fossmodmanager/mods directory at {:?}: {}",
                    mod_manager_dir, e
                )
            })?;
        println!(
            "Successfully created mod directory: {:?}\n",
            mod_manager_dir
        );
    } else {
        println!("Mod directory already exists: {:?}\n", mod_manager_dir);
    }

    // Open the directory
    println!("Attempting to open directory: {}\n", mods_path_str);
    app_handle
        .opener()
        .open_path(mods_path_str, None::<&str>)
        .map_err(|e| format!("Failed to open mod directory '{}': {}", mods_path_str, e))?;

    println!(
        "Successfully ensured and requested to open mod directory for path: {}",
        game_root_path
    );
    Ok(())
}

// --- Mod List Structs (Define BEFORE list_mods) ---
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModMetadata {
    parsed_name: String,
    original_zip_name: String,
    // installed_files: Vec<String>, // List of relative paths within <game_root> added/overwritten by this mod
    installed_directory: String, // Relative path from game_root to the mod's specific folder (e.g., "reframework/plugins/MyMod")
    source: String,              // e.g., "local_zip"
    version: Option<String>,     // Optional: Maybe parsed from filename later
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SkinMetadata {
    name: String,
    path: String,
    enabled: bool,
    thumbnail_path: Option<String>,
    author: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
struct ModListContainer {
    mods: Vec<ModMetadata>,
    skins: Vec<SkinMetadata>,
}

// For legacy compatibility
// type ModList = Vec<ModMetadata>;

//replacement for list_mods
#[tauri::command]
async fn list_mods(app_handle: AppHandle, game_root_path: String) -> Result<Vec<ModInfo>, String> {
    log::info!("Listing mods based on registry for game root: {}", game_root_path);

    let mut registry = ModRegistry::load(&app_handle)?;

    //update registry based on fs
    let game_root = PathBuf::from(&game_root_path);
    registry.update_mod_enabled_status(&game_root)?;

    //get all mod info
    let mods_info = registry.get_reframework_mod_info();

    log::info!("Finished processing mod list. Returning {} mods to frontend.", mods_info.len());
    Ok(mods_info)
}

// Command to list installed mods by reading modlist.json and checking file status
// #[tauri::command]
// async fn list_mods(app_handle: AppHandle, game_root_path: String) -> Result<Vec<ModInfo>, String> {
//     log::info!(
//         "Listing mods based on modlist.json for game root: {}",
//         game_root_path
//     );
//     let game_root = PathBuf::from(&game_root_path);

//     // --- 1. Load Mod List ---
//     let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;
//     log::debug!("Reading mod list from: {:?}", modlist_path);

//     let mods_metadata: Vec<ModMetadata> = match fs::read_to_string(&modlist_path) {
//         Ok(content) => {
//             if content.is_empty() {
//                 log::info!("modlist.json is empty. No mods tracked.");
//                 Vec::new()
//             } else {
//                 // First try to parse as ModListContainer
//                 let result: Result<ModListContainer, _> = serde_json::from_str(&content);
//                 match result {
//                     Ok(container) => {
//                         log::info!("Successfully parsed modlist.json as container with {} mods and {} skins", 
//                                   container.mods.len(), container.skins.len());
//                         container.mods
//                     },
//                     Err(_) => {
//                         // Fall back to legacy format
//                         serde_json::from_str(&content).map_err(|e| {
//                             format!("Failed to parse modlist.json: {}. Content: {}", e, content)
//                         })?
//                     }
//                 }
//             }
//         }
//         Err(e) if e.kind() == io::ErrorKind::NotFound => {
//             log::info!("modlist.json not found. No mods tracked.");
//             Vec::new()
//         }
//         Err(e) => return Err(format!("Failed to read modlist.json: {}", e)),
//     };

//     log::info!("Found {} entries in modlist.json", mods_metadata.len());

//     // --- 2. Determine Status and Transform ---
//     let mut mods_info_list: Vec<ModInfo> = Vec::new();

//     for metadata in mods_metadata {
//         // --- Determine Enabled Status based on Directory ---
//         let mod_dir_rel = PathBuf::from(&metadata.installed_directory);
//         let mod_dir_abs = game_root.join(&mod_dir_rel);
//         let disabled_dir_str = format!("{}.disabled", metadata.installed_directory);
//         let disabled_dir_abs = game_root.join(PathBuf::from(&disabled_dir_str));

//         let is_enabled = mod_dir_abs.is_dir(); // Enabled if the directory exists without .disabled

//         log::debug!(
//             "Checking status for mod '{}': Directory {:?} exists? {}. Disabled path: {:?}",
//             metadata.parsed_name,
//             mod_dir_abs,
//             is_enabled,
//             disabled_dir_abs
//         );

//         // Optional: Add a check/warning if BOTH the normal and .disabled directories exist, or if NEITHER exist.
//         if is_enabled && disabled_dir_abs.exists() {
//             log::warn!("Mod '{}' has both enabled ({:?}) and disabled ({:?}) directories present! Assuming enabled.", metadata.parsed_name, mod_dir_abs, disabled_dir_abs);
//         } else if !is_enabled && !disabled_dir_abs.exists() {
//             log::warn!("Mod '{}' directory not found at either {:?} or {:?}. Mod may be corrupted or partially deleted. Assuming disabled.", metadata.parsed_name, mod_dir_abs, disabled_dir_abs);
//         }
//         // --- End Status Check ---

//         log::info!(
//             "Mod '{}' final enabled status: {}",
//             metadata.parsed_name,
//             is_enabled
//         );

//         // Transform to ModInfo (using the existing definition) for the frontend
//         let info = ModInfo {
//             directory_name: metadata.parsed_name.clone(), // Use parsed_name as the identifier
//             name: Some(metadata.parsed_name),             // Use parsed_name as display name for now
//             enabled: is_enabled,
//             version: metadata.version, // Pass along if it exists (currently None)
//             author: None,              // Not tracked yet
//             description: None,         // Not tracked yet
//         };
//         mods_info_list.push(info);
//     }

//     log::info!(
//         "Finished processing mod list. Returning {} mods to frontend.",
//         mods_info_list.len()
//     );
//     Ok(mods_info_list)
// }

#[tauri::command]
async fn install_mod_from_zip(
    app_handle: AppHandle,
    game_root_path: String,
    zip_path_str: String,
    on_event: Channel<ModOperationEvent>,
) -> Result<(), String> {
    let game_root = PathBuf::from(&game_root_path);
    let zip_path = PathBuf::from(&zip_path_str);

    // Get mod name from zip filename
    let original_zip_name = zip_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid zip filename".to_string())?
        .to_string();

    let parsed_name = zip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.split('-').next().unwrap_or(s).trim().to_string())
        .ok_or_else(|| "Couldn't determine mod name".to_string())?;

    // Use secure access wrapper
    with_game_dir_write_access(
        &app_handle,
        &game_root,
        &on_event,
        "install",
        &parsed_name,
        |_channel| {
            // Open the zip
            let file =
                fs::File::open(&zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
            let mut archive =
                ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;

            // Scan once to detect if it's a plugins or autorun mod
            let mut is_autorun = false;
            for i in 0..archive.len() {
                if let Ok(entry) = archive.by_index(i) {
                    if entry.name().contains("autorun/") {
                        is_autorun = true;
                        break;
                    }
                }
            }

            // Create the mod directory
            let mod_type = if is_autorun { 
                "autorun" 
            } else { 
                "plugins" 
            };
            
            let mod_type_enum = if is_autorun {
                ModType::REFrameworkAutorun
            } else {
                ModType::REFrameworkPlugin
            };
            
            let rf_path = game_root.join("reframework");
            let mod_dir = rf_path.join(mod_type).join(&parsed_name);

            // Clean up existing mod
            if mod_dir.exists() {
                fs::remove_dir_all(&mod_dir)
                    .map_err(|e| format!("Failed to remove existing mod: {}", e))?;
            }
            fs::create_dir_all(&mod_dir)
                .map_err(|e| format!("Failed to create mod directory: {}", e))?;

            // Track if we extracted anything
            let mut extracted = 0;

            // Extract files - this part remains largely the same
            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read zip entry: {}", e))?;

                // Skip directories
                if file.is_dir() {
                    continue;
                }

                let name = file.name();

                // Root fallback - single lua or dll files
                if !name.contains('/') {
                    if name.ends_with(".lua") && mod_type == "autorun" {
                        let target = mod_dir.join(name);
                        let mut outfile = fs::File::create(&target)
                            .map_err(|e| format!("Failed to create file: {}", e))?;
                        io::copy(&mut file, &mut outfile)
                            .map_err(|e| format!("Failed to write file: {}", e))?;
                        extracted += 1;
                    } else if name.ends_with(".dll")
                        && name != "dinput8.dll"
                        && mod_type == "plugins"
                    {
                        let target = mod_dir.join(name);
                        let mut outfile = fs::File::create(&target)
                            .map_err(|e| format!("Failed to create file: {}", e))?;
                        io::copy(&mut file, &mut outfile)
                            .map_err(|e| format!("Failed to write file: {}", e))?;
                        extracted += 1;
                    }
                    continue;
                }

                // Extract files from reframework/plugins or reframework/autorun
                let path = PathBuf::from(name);
                if let Some(rel_path) = path
                    .components()
                    .skip_while(|c| c.as_os_str() != mod_type)
                    .skip(1) // Skip the mod_type component itself
                    .collect::<PathBuf>()
                    .to_str()
                {
                    let target = mod_dir.join(rel_path);

                    // Create parent directories
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                    }

                    // Extract the file
                    let mut outfile = fs::File::create(&target)
                        .map_err(|e| format!("Failed to create file: {}", e))?;
                    io::copy(&mut file, &mut outfile)
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                    extracted += 1;
                }
            }

            if extracted == 0 {
                return Err("No valid mod files found in zip".to_string());
            }

            // This part changes to use ModRegistry
            let rel_path = format!("reframework/{}/{}", mod_type, parsed_name);
            
            // Load registry instead of modlist.json
            let mut registry = utils::modregistry::ModRegistry::load(&app_handle)?;
            
            // Create new mod entry
            let new_mod = utils::modregistry::Mod {
                name: parsed_name.clone(),
                directory_name: parsed_name.clone(),
                path: zip_path_str.clone(),
                enabled: true, // Newly installed mods start enabled
                author: None,
                version: None,
                description: None,
                source: Some("local_zip".to_string()),
                installed_timestamp: chrono::Utc::now().timestamp(),
                installed_directory: rel_path,
                mod_type: mod_type_enum,
            };
            
            // Add to registry and save
            registry.add_mod(new_mod);
            registry.save(&app_handle)?;
            
            log::info!("Successfully installed mod '{}' and updated registry", parsed_name);
            Ok(())
        },
    )
    .await
}
// #[tauri::command]
// async fn install_mod_from_zip(
//     app_handle: AppHandle,
//     game_root_path: String,
//     zip_path_str: String,
//     on_event: Channel<ModOperationEvent>,
// ) -> Result<(), String> {
//     let game_root = PathBuf::from(&game_root_path);
//     let zip_path = PathBuf::from(&zip_path_str);

//     // Get mod name from zip filename
//     let original_zip_name = zip_path
//         .file_name()
//         .and_then(|n| n.to_str())
//         .ok_or_else(|| "Invalid zip filename".to_string())?
//         .to_string();

//     let parsed_name = zip_path
//         .file_stem()
//         .and_then(|s| s.to_str())
//         .map(|s| s.split('-').next().unwrap_or(s).trim().to_string())
//         .ok_or_else(|| "Couldn't determine mod name".to_string())?;

//     // Use secure access wrapper
//     with_game_dir_write_access(
//         &app_handle,
//         &game_root,
//         &on_event,
//         "install",
//         &parsed_name,
//         |_channel| {
//             // Open the zip
//             let file =
//                 fs::File::open(&zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
//             let mut archive =
//                 ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;

//             // Scan once to detect if it's a plugins or autorun mod
//             let mut is_autorun = false;
//             for i in 0..archive.len() {
//                 if let Ok(entry) = archive.by_index(i) {
//                     if entry.name().contains("autorun/") {
//                         is_autorun = true;
//                         break;
//                     }
//                 }
//             }

//             // Create the mod directory
//             let mod_type = if is_autorun { "autorun" } else { "plugins" };
//             let rf_path = game_root.join("reframework");
//             let mod_dir = rf_path.join(mod_type).join(&parsed_name);

//             // Clean up existing mod
//             if mod_dir.exists() {
//                 fs::remove_dir_all(&mod_dir)
//                     .map_err(|e| format!("Failed to remove existing mod: {}", e))?;
//             }
//             fs::create_dir_all(&mod_dir)
//                 .map_err(|e| format!("Failed to create mod directory: {}", e))?;

//             // Track if we extracted anything
//             let mut extracted = 0;

//             // Extract files
//             for i in 0..archive.len() {
//                 let mut file = archive
//                     .by_index(i)
//                     .map_err(|e| format!("Failed to read zip entry: {}", e))?;

//                 // Skip directories
//                 if file.is_dir() {
//                     continue;
//                 }

//                 let name = file.name();

//                 // Root fallback - single lua or dll files
//                 if !name.contains('/') {
//                     if name.ends_with(".lua") && mod_type == "autorun" {
//                         let target = mod_dir.join(name);
//                         let mut outfile = fs::File::create(&target)
//                             .map_err(|e| format!("Failed to create file: {}", e))?;
//                         io::copy(&mut file, &mut outfile)
//                             .map_err(|e| format!("Failed to write file: {}", e))?;
//                         extracted += 1;
//                     } else if name.ends_with(".dll")
//                         && name != "dinput8.dll"
//                         && mod_type == "plugins"
//                     {
//                         let target = mod_dir.join(name);
//                         let mut outfile = fs::File::create(&target)
//                             .map_err(|e| format!("Failed to create file: {}", e))?;
//                         io::copy(&mut file, &mut outfile)
//                             .map_err(|e| format!("Failed to write file: {}", e))?;
//                         extracted += 1;
//                     }
//                     continue;
//                 }

//                 // Extract files from reframework/plugins or reframework/autorun
//                 let path = PathBuf::from(name);
//                 if let Some(rel_path) = path
//                     .components()
//                     .skip_while(|c| c.as_os_str() != mod_type)
//                     .skip(1) // Skip the mod_type component itself
//                     .collect::<PathBuf>()
//                     .to_str()
//                 {
//                     let target = mod_dir.join(rel_path);

//                     // Create parent directories
//                     if let Some(parent) = target.parent() {
//                         fs::create_dir_all(parent)
//                             .map_err(|e| format!("Failed to create directory: {}", e))?;
//                     }

//                     // Extract the file
//                     let mut outfile = fs::File::create(&target)
//                         .map_err(|e| format!("Failed to create file: {}", e))?;
//                     io::copy(&mut file, &mut outfile)
//                         .map_err(|e| format!("Failed to write file: {}", e))?;
//                     extracted += 1;
//                 }
//             }

//             if extracted == 0 {
//                 return Err("No valid mod files found in zip".to_string());
//             }

//             // Update modlist.json
//             let rel_path = format!("reframework/{}/{}", mod_type, parsed_name);
//             let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;

//             // Read existing or create new container
//             let mut mod_container = match fs::read_to_string(&modlist_path) {
//                 Ok(content) if !content.is_empty() => {
//                     // Try to parse as container first
//                     let container_result: Result<ModListContainer, _> = serde_json::from_str(&content);
//                     match container_result {
//                         Ok(container) => container,
//                         Err(_) => {
//                             // Fall back to legacy format and migrate
//                             let legacy_mods: Result<ModList, _> = serde_json::from_str(&content);
//                             match legacy_mods {
//                                 Ok(mods) => ModListContainer {
//                                     mods,
//                                     skins: Vec::new(),
//                                 },
//                                 Err(e) => return Err(format!("Failed to parse modlist.json: {}", e))
//                             }
//                         }
//                     }
//                 },
//                 _ => ModListContainer {
//                     mods: Vec::new(),
//                     skins: Vec::new(),
//                 },
//             };

//             // Update list
//             let new_mod = ModMetadata {
//                 parsed_name: parsed_name.clone(),
//                 original_zip_name,
//                 installed_directory: rel_path,
//                 source: "local_zip".to_string(),
//                 version: None,
//             };

//             mod_container.mods.retain(|m| m.parsed_name != parsed_name);
//             mod_container.mods.push(new_mod);

//             // Write back
//             let json = serde_json::to_string_pretty(&mod_container)
//                 .map_err(|e| format!("Failed to serialize modlist: {}", e))?;
//             fs::write(&modlist_path, &json)
//                 .map_err(|e| format!("Failed to write modlist.json: {}", e))?;

//             Ok(())
//         },
//     )
//     .await
// }

// --- Helper Function ---
// Function to get the full path to a file within the app's config directory
fn get_app_config_path(app_handle: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    // Ensure the directory exists before returning path
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;
    Ok(config_dir.join(filename))
}
#[tauri::command]
async fn toggle_mod_enabled_state(
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
    
    // Load the registry instead of modlist.json
    let mut registry = utils::modregistry::ModRegistry::load(&app_handle)?;
    
    // Find the mod
    let mod_entry = match registry.find_mod(&mod_name) {
        Some(m) => m.clone(), // Clone to avoid borrow issues
        None => {
            // Try to find it as a skin mod
            if let Some(_) = registry.find_skin_mod(&mod_name) {
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
            fs::rename(&disabled_dir_abs, &installed_dir_abs)
                .map_err(|e| {
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
            fs::rename(&installed_dir_abs, &disabled_dir_abs)
                .map_err(|e| {
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
    
    log::info!("Successfully toggled mod '{}' to enabled={}", mod_name, enable);
    Ok(())
}
// --- New Command: Toggle Mod Enabled State ---
// #[tauri::command]
// async fn toggle_mod_enabled_state(
//     app_handle: AppHandle,
//     game_root_path: String,
//     mod_name: String, // The parsed_name from ModMetadata
//     enable: bool,     // true to enable, false to disable
// ) -> Result<(), String> {
//     log::info!(
//         "Toggling mod '{}' to enabled={} in game root: {}",
//         mod_name,
//         enable,
//         game_root_path
//     );
//     let game_root = PathBuf::from(&game_root_path);

//     // --- 1. Load Mod List ---
//     let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;
//     log::debug!("Reading mod list from: {:?}", modlist_path);

//     let content = match fs::read_to_string(&modlist_path) {
//         Ok(content) => {
//             if content.is_empty() {
//                 return Err(format!(
//                     "Cannot toggle mod '{}': modlist.json is empty.",
//                     mod_name
//                 ));
//             }
//             content
//         }
//         Err(e) if e.kind() == io::ErrorKind::NotFound => {
//             return Err(format!(
//                 "Cannot toggle mod '{}': modlist.json not found.",
//                 mod_name
//             ));
//         }
//         Err(e) => return Err(format!("Failed to read modlist.json: {}", e)),
//     };

//     // Try to parse as container first
//     let container_result: Result<ModListContainer, _> = serde_json::from_str(&content);
//     let mod_meta = match container_result {
//         Ok(container) => {
//             // Clone the found mod to avoid borrowing issues
//             container.mods.iter()
//                 .find(|m| m.parsed_name == mod_name)
//                 .cloned()
//                 .ok_or_else(|| format!("Mod '{}' not found in modlist.json", mod_name))?
//         },
//         Err(_) => {
//             // Fall back to legacy format
//             let mods_metadata: ModList = serde_json::from_str(&content)
//                 .map_err(|e| format!("Failed to parse modlist.json: {}. Content: {}", e, content))?;
            
//             // Clone the found mod to avoid borrowing issues
//             mods_metadata.iter()
//                 .find(|m| m.parsed_name == mod_name)
//                 .cloned()
//                 .ok_or_else(|| format!("Mod '{}' not found in modlist.json", mod_name))?
//         }
//     };

//     log::debug!(
//         "Found metadata for mod '{}'. Installed directory: {}",
//         mod_name,
//         mod_meta.installed_directory
//     );

//     // --- 3. Rename Directory ---
//     let installed_dir_rel = PathBuf::from(&mod_meta.installed_directory);
//     let installed_dir_abs = game_root.join(&installed_dir_rel);
//     let disabled_dir_str = format!("{}.disabled", mod_meta.installed_directory);
//     let disabled_dir_abs = game_root.join(PathBuf::from(&disabled_dir_str));

//     if enable {
//         // Enable: Rename *.disabled to * (if it exists)
//         if disabled_dir_abs.exists() {
//             log::info!(
//                 "Enabling mod '{}': Renaming {:?} -> {:?}",
//                 mod_name,
//                 disabled_dir_abs,
//                 installed_dir_abs
//             );
//             match fs::rename(&disabled_dir_abs, &installed_dir_abs) {
//                 Ok(_) => {
//                     log::info!("Successfully enabled mod '{}'", mod_name);
//                     Ok(())
//                 }
//                 Err(e) => {
//                     let err_msg = format!(
//                         "Failed to rename {:?} to {:?}: {}",
//                         disabled_dir_abs, installed_dir_abs, e
//                     );
//                     log::error!("{}", err_msg);
//                     Err(err_msg)
//                 }
//             }
//         } else if installed_dir_abs.exists() {
//             log::info!(
//                 "Mod '{}' is already enabled (directory {:?} exists).",
//                 mod_name,
//                 installed_dir_abs
//             );
//             Ok(()) // Already in desired state
//         } else {
//             let err_msg = format!(
//                 "Cannot enable mod '{}': Neither directory {:?} nor {:?} found.",
//                 mod_name, installed_dir_abs, disabled_dir_abs
//             );
//             log::error!("{}", err_msg);
//             Err(err_msg)
//         }
//     } else {
//         // Disable: Rename * to *.disabled (if it exists)
//         if installed_dir_abs.exists() {
//             log::info!(
//                 "Disabling mod '{}': Renaming {:?} -> {:?}",
//                 mod_name,
//                 installed_dir_abs,
//                 disabled_dir_abs
//             );
//             match fs::rename(&installed_dir_abs, &disabled_dir_abs) {
//                 Ok(_) => {
//                     log::info!("Successfully disabled mod '{}'", mod_name);
//                     Ok(())
//                 }
//                 Err(e) => {
//                     let err_msg = format!(
//                         "Failed to rename {:?} to {:?}: {}",
//                         installed_dir_abs, disabled_dir_abs, e
//                     );
//                     log::error!("{}", err_msg);
//                     Err(err_msg)
//                 }
//             }
//         } else if disabled_dir_abs.exists() {
//             log::info!(
//                 "Mod '{}' is already disabled (directory {:?} exists).",
//                 mod_name,
//                 disabled_dir_abs
//             );
//             Ok(()) // Already in desired state
//         } else {
//             let err_msg = format!(
//                 "Cannot disable mod '{}': Neither directory {:?} nor {:?} found.",
//                 mod_name, installed_dir_abs, disabled_dir_abs
//             );
//             log::error!("{}", err_msg);
//             Err(err_msg)
//         }
//     }
// }

// --- New Command: Preload Mod Assets ---
#[tauri::command]
async fn preload_mod_assets(
    app_handle: AppHandle,
    mods: Vec<String>
) -> Result<(), String> {
    log::info!("Preloading assets for {} mods", mods.len());
    
    // Get the cache directory where we'll store mod assets
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get app cache dir: {}", e))?
        .join("fossmodmanager")
        .join("assets");
    
    // Ensure the cache directory exists
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create mod assets cache directory: {}", e))?;
    
    // For each mod, check if there are assets to preload
    // This could include thumbnails, preview images, etc.
    for mod_name in mods {
        log::debug!("Preparing assets for mod: {}", mod_name);
        
        // Create a mod-specific cache directory
        let mod_cache_dir = cache_dir.join(&mod_name);
        if !mod_cache_dir.exists() {
            fs::create_dir_all(&mod_cache_dir)
                .map_err(|e| format!("Failed to create cache directory for mod {}: {}", mod_name, e))?;
            log::debug!("Created cache directory for mod: {}", mod_name);
        }
        
        // In the future, we could add code to preload specific assets:
        // - Check if the mod has thumbnails/screenshots
        // - Check for readme files or documentation
        // - Process and optimize images
        // - Extract essential metadata
    }
    
    log::info!("Mod assets preloading completed successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting Foss Mod Manager");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("Another instance tried to start: {:?} in {:?}", argv, cwd);
            app.emit_to("main", "single-instance", ()).unwrap();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_mods,
            save_game_config,
            load_game_config,
            validate_game_installation,
            delete_config,
            check_reframework_installed,
            ensure_reframework,
            install_mod_from_zip,
            toggle_mod_enabled_state,
            open_mods_folder,
            preload_mod_assets,
            // Nexus API commands
            nexus_api::fetch_trending_mods,
            // Skin extraction utilities
            utils::skinextract::scan_for_skin_mods,
            utils::skinextract::read_mod_image,
            utils::skinextract::cache_mod_image,
            utils::skinextract::get_cached_mod_images,
            utils::skinextract::enable_skin_mod,
            utils::skinextract::disable_skin_mod,
            utils::skinextract::list_installed_skin_mods,
            
        ])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let window = app.get_window("main").expect("Main window not found");
                window.set_decorations(false)?;
            }

            // Ensure API cache system is initialized
            let cache = ApiCache::new(app.handle().clone());
            app.manage(cache);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
