use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use std::io::{self};
use zip::ZipArchive;

use tauri_plugin_opener::OpenerExt;
// Declare the new module
mod nexus_api;
use nexus_api::ApiCache;
use reqwest;
use zip;
use regex::Regex;
use once_cell::sync::Lazy;

mod utils;
use utils::config::{finalize_setup, load_game_config, delete_config};

// Helper function to find the game root and steamapps directories
// fn find_game_paths_from_exe(executable_path_str: &str) -> Result<(PathBuf, PathBuf), String> {
//     let executable_path = PathBuf::from(executable_path_str);

//     if !executable_path.is_file() {
//         return Err(format!(
//             "Provided path is not a file or does not exist: {}",
//             executable_path_str
//         ));
//     }

//     let mut current_path = executable_path.parent().ok_or_else(|| {
//         format!(
//             "Could not get parent directory of executable: {}",
//             executable_path_str
//         )
//     })?;

//     loop {
//         let parent_path = current_path.parent().ok_or_else(|| {
//             format!(
//                 "Reached filesystem root without finding 'steamapps/common' structure starting from: {}",
//                 executable_path_str
//             )
//         })?;

//         let parent_dir_name = parent_path
//             .file_name()
//             .and_then(|name| name.to_str())
//             .ok_or_else(|| format!("Could not get parent directory name for: {:?}", parent_path))?;

//         if parent_dir_name == "common" {
//             let grandparent_path = parent_path.parent().ok_or_else(|| {
//                 format!(
//                     "Found 'common' but no parent directory above it: {:?}",
//                     parent_path
//                 )
//             })?;

//             let grandparent_dir_name = grandparent_path
//                 .file_name()
//                 .and_then(|name| name.to_str())
//                 .ok_or_else(|| {
//                     format!(
//                         "Could not get grandparent directory name for: {:?}",
//                         grandparent_path
//                     )
//                 })?;

//             if grandparent_dir_name == "steamapps" {
//                 // Success! current_path is game root, grandparent_path is steamapps
//                 return Ok((current_path.to_path_buf(), grandparent_path.to_path_buf()));
//             }
//         }

//         if current_path == parent_path {
//             return Err(format!(
//                 "Path resolution stopped unexpectedly at: {:?}. Could not find 'steamapps/common' structure.",
//                 current_path
//             ));
//         }
//         current_path = parent_path;
//     }
// }

// Helper function to parse ACF using vdf-reader crate

// Helper function to get cover art data URL - Updated Logic

// Command to finalize setup, write config, and potentially handle window logic
// #[tauri::command]
// async fn finalize_setup(
//     window: WebviewWindow, // Keep window arg if still needed for closing setup window
//     app_handle: AppHandle,
//     executable_path: String,
// ) -> Result<(), String> {
//     // Use find_game_paths_from_exe to get game root and exe path buf
//     let (game_root_path_buf, _) = find_game_paths_from_exe(&executable_path)?; // Keep _ if exe path buf not needed here
//     let game_root_path_str = game_root_path_buf.to_str().ok_or("Game root path contains invalid UTF-8")?.to_string();

//     println!("Selected Executable: {}", executable_path);
//     println!("Determined Game Root: {}", game_root_path_str);

//     // --- Create necessary directories ---
//     let fossmodmanager_path = game_root_path_buf.join("fossmodmanager");
//     let mods_path = fossmodmanager_path.join("mods");
//     fs::create_dir_all(&mods_path)
//         .map_err(|e| format!("Failed to create mods directory {:?}: {}", mods_path, e))?;
//     println!("Ensured directory exists: {:?}", mods_path);
//     // ------------------------------------


//     // --- Persist the game data to userconfig.json ---
//     let config_dir = app_handle.path()
//         .app_config_dir()
//         .map_err(|e| format!("Failed to get app config dir: {}", e))?;

//     // Ensure the config directory exists
//     fs::create_dir_all(&config_dir)
//         .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;

//     let config_path = config_dir.join("userconfig.json");

//     // Create the simplified game data entry
//     let game_data = GameData {
//         game_root_path: game_root_path_str.clone(), // Keep clone if path is used later
//         game_executable_path: executable_path.clone(),
//     };

//     // Serialize the data to a JSON string
//     let json_string = serde_json::to_string_pretty(&game_data)
//         .map_err(|e| format!("Failed to serialize game data to JSON: {}", e))?;

//     // Write the JSON string to userconfig.json
//     fs::write(&config_path, &json_string) // Pass json_string by reference if possible, or clone if needed
//         .map_err(|e| format!("Failed to write userconfig.json to {:?}: {}", config_path, e))?;

//     // Print after successful write
//     println!("userconfig.json saved to {:?}:\n{}", config_path, json_string);
//     // --------------------------------------

//     // Get the main window using AppHandle obtained from args
//     if let Some(main_window) = app_handle.get_webview_window("main") {
//         let _ = main_window.show();
//         let _ = main_window.set_focus();
//     } else {
//         eprintln!("Error: Could not find main window after setup."); // Modified message slightly
//     }

//     // Close the setup window itself (if it exists and we are in that flow)
//     // Frontend might handle view switching instead of closing windows now.
//     // Consider if this window closing logic is still relevant.
//     if window.label() == "setup" { // Check if this label is still used/relevant
//          println!("Closing setup window (label: {}).", window.label());
//          // We might not need to close a window if setup is an overlay/view
//          // window.close().map_err(|e| e.to_string())?;
//     }

//     Ok(())
// }

// Struct to hold game data - Simplified
// #[derive(Serialize, Deserialize, Clone, Debug)]
// struct GameData {
//     game_root_path: String,
//     game_executable_path: String,
//     // Removed: appid, game_name, game_slug, version, cover_art_data_url
// }

// // Command to load the single game configuration from userconfig.json
// #[tauri::command]
// async fn load_game_config(app_handle: AppHandle) -> Result<Option<GameData>, String> {
//     let config_dir = app_handle.path()
//         .app_config_dir()
//         .map_err(|e| format!("Failed to get app config dir: {}", e))?;
//     let config_path = config_dir.join("userconfig.json");

//     println!("Attempting to load config from: {:?}", config_path);

//     match fs::read_to_string(&config_path) {
//         Ok(json_string) => {
//             println!("Successfully read userconfig.json. Contents:\n{}", json_string);
//             // Attempt to deserialize the JSON into GameData
//             // Make sure this matches the simplified GameData struct
//             serde_json::from_str::<GameData>(&json_string)
//                 .map_err(|e| format!("Failed to parse userconfig.json from {:?}: {}", config_path, e))
//                 .map(|data| Some(data))
//         }
//         Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
//             println!("No 'userconfig.json' found at {:?}. Assuming first run.", config_path);
//             Ok(None)
//         }
//         Err(e) => {
//             Err(format!("Failed to read userconfig.json from {:?}: {}", config_path, e))
//         }
//     }
// }

// // Command to delete the user configuration file
// #[tauri::command]
// async fn delete_config(app_handle: AppHandle) -> Result<(), String> {
//     let config_dir = app_handle.path()
//         .app_config_dir()
//         .map_err(|e| format!("Failed to get app config dir: {}", e))?;
//     let config_path = config_dir.join("userconfig.json");

//     println!("Attempting to delete config file: {:?}", config_path);

//     match fs::remove_file(&config_path) {
//         Ok(_) => {
//             println!("Successfully deleted config file: {:?}", config_path);
//             Ok(())
//         }
//         Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
//             println!("Config file not found at {:?}, nothing to delete.", config_path);
//             Ok(()) // Not an error if the file doesn't exist
//         }
//         Err(e) => {
//             eprintln!("Error deleting config file {:?}: {}", config_path, e);
//             Err(format!("Failed to delete config file: {}", e))
//         }
//     }
// }

// Struct representing mod metadata read from modinfo.json
#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
struct ModInfo {
    directory_name: String, // The name of the folder the mod resides in
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    description: Option<String>,
    enabled: bool, // Derived from directory name (presence/absence of _DISABLED_)
}

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

// --- Helper Function to get Latest REFramework URL ---
async fn get_latest_reframework_url() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("FossModManager/0.1.0") // GitHub requires a User-Agent
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = "https://api.github.com/repos/praydog/REFramework-nightly/releases";
    log::info!("Fetching releases from: {}", url); // Use log crate

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        return Err(format!("GitHub API request failed: Status {} - {}", status, text));
    }

     log::info!("Successfully fetched releases list.");

    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub releases JSON: {}", e))?;

    // Find the latest release (GitHub API usually returns latest first, but let's be sure)
    // We might need more sophisticated logic if tags aren't easily sortable or if we want to avoid pre-releases explicitly
    // For now, assume the first one is the latest suitable one.
    let latest_release = releases.into_iter().next()
        .ok_or_else(|| "No releases found for REFramework-nightly".to_string())?;

     log::info!("Found latest release: {}", latest_release.tag_name);

    // Find the MHWilds.zip asset
    let asset = latest_release.assets.into_iter()
        .find(|a| a.name == "MHWilds.zip")
        .ok_or_else(|| format!("MHWilds.zip not found in latest release ({})", latest_release.tag_name))?;

     log::info!("Found MHWilds.zip asset URL: {}", asset.browser_download_url);
    Ok(asset.browser_download_url)
}


#[tauri::command]
async fn check_reframework_installed(game_root_path: String) -> Result<bool, String> {
     log::info!("Checking for REFramework in: {}", game_root_path);
    let root = PathBuf::from(game_root_path);
    let dinput_path = root.join("dinput8.dll");
    let reframework_dir_path = root.join("reframework");

    // Check if either dinput8.dll exists OR the reframework directory exists
    let installed = dinput_path.exists() || reframework_dir_path.is_dir();
     log::info!("REFramework installed status: {}", installed);
    Ok(installed)
}


#[tauri::command]
async fn install_reframework(app_handle: AppHandle, game_root_path: String) -> Result<(), String> {
     log::info!("Starting REFramework installation for path: {}", game_root_path);
    let target_dir = PathBuf::from(&game_root_path);

    if !target_dir.is_dir() {
        return Err(format!("Target game directory does not exist: {}", game_root_path));
    }

    // --- Get Download URL ---
     log::info!("Fetching latest REFramework download URL...");
    let download_url = get_latest_reframework_url().await?;
     log::info!("Using download URL: {}", download_url);


    // --- Download ---
     log::info!("Downloading REFramework...");
    let client = reqwest::Client::new(); // Create a new client for download
    let response = client.get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download request failed: Status {}", response.status()));
    }

    let zip_data = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download bytes: {}", e))?;
     log::info!("Download complete ({} bytes)", zip_data.len());

    // --- Selective Extraction ---
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data)) // Wrap zip_data in Cursor
        .map_err(|e| format!("Failed to open zip archive: {}", e))?;

     log::info!("Starting selective extraction to {}", target_dir.display());
    let mut extracted_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Error reading zip entry {}: {}", i, e))?;
        let entry_path_str = file.name().to_string();

        // Check if the entry should be extracted
        let should_extract = entry_path_str == "dinput8.dll" || entry_path_str.starts_with("reframework/");

        if !should_extract {
            continue; // Skip this file
        }

        let outpath = match file.enclosed_name() {
             Some(path) => target_dir.join(path),
             None => {
                  log::warn!("Skipping potentially unsafe zip entry: {}", entry_path_str);
                 continue;
             }
         };

         log::debug!("Processing entry: {}", entry_path_str); // More detailed log

        if file.name().ends_with('/') {
             log::debug!("Creating directory {}", outpath.display());
            fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create directory {}: {}", outpath.display(), e))?;
        } else {
             log::debug!("Extracting file {}", outpath.display());
            // Ensure parent directory exists
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| format!("Failed to create parent directory {}: {}", p.display(), e))?;
                }
            }
            // Check if file exists and delete if it does (overwrite)
            // Be cautious with this in a real app, maybe offer options?
             if outpath.exists() {
                  log::warn!("Overwriting existing file: {}", outpath.display());
                 if outpath.is_dir() {
                     fs::remove_dir_all(&outpath).map_err(|e| format!("Failed to remove existing directory before overwrite {}: {}", outpath.display(), e))?;
                 } else {
                     fs::remove_file(&outpath).map_err(|e| format!("Failed to remove existing file before overwrite {}: {}", outpath.display(), e))?;
                 }
             }

            let mut outfile = fs::File::create(&outpath).map_err(|e| format!("Failed to create output file {}: {}", outpath.display(), e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy content to {}: {}", outpath.display(), e))?;
             extracted_count += 1;
        }

        // Set permissions (optional, might be needed on Linux/macOS)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                // Be careful applying zip permissions directly, could be too restrictive/permissive
                 log::debug!("Attempting to set permissions {:o} on {}", mode, outpath.display());
                if let Err(e) = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)) {
                    // Log warning instead of hard error for permissions
                     log::warn!("Failed to set permissions on {}: {}", outpath.display(), e);
                 }
            }
        }
    }

     log::info!("Selective extraction complete. {} files/folders processed for extraction.", extracted_count);
     log::info!("REFramework installation successful for {}", game_root_path);
    Ok(())
}

// Command to ensure the fossmodmanager/mods directory exists AND open it
#[tauri::command]
async fn open_mods_folder(app_handle: AppHandle, game_root_path: String) -> Result<(), String> { // Renamed, changed signature
    println!("Ensuring and opening mod directory for path: {}", game_root_path);

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
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ModMetadata {
    parsed_name: String,
    original_zip_name: String,
    // installed_files: Vec<String>, // List of relative paths within <game_root> added/overwritten by this mod
    installed_directory: String, // Relative path from game_root to the mod's specific folder (e.g., "reframework/plugins/MyMod")
    source: String,              // e.g., "local_zip"
    version: Option<String>,     // Optional: Maybe parsed from filename later
}

// Using a type alias for simplicity
type ModList = Vec<ModMetadata>;

// Command to list installed mods by reading modlist.json and checking file status
#[tauri::command]
async fn list_mods(app_handle: AppHandle, game_root_path: String) -> Result<Vec<ModInfo>, String> {
    log::info!("Listing mods based on modlist.json for game root: {}", game_root_path);
    let game_root = PathBuf::from(&game_root_path);

    // --- 1. Load Mod List --- 
    let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;
    log::debug!("Reading mod list from: {:?}", modlist_path);

    let mods_metadata: ModList = match fs::read_to_string(&modlist_path) {
        Ok(content) => {
            if content.is_empty() {
                log::info!("modlist.json is empty. No mods tracked.");
                Vec::new()
            } else {
                serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse modlist.json: {}. Content: {}", e, content))?
            }
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
             log::info!("modlist.json not found. No mods tracked.");
             Vec::new()
        },
        Err(e) => return Err(format!("Failed to read modlist.json: {}", e)),
    };

    log::info!("Found {} entries in modlist.json", mods_metadata.len());

    // --- 2. Determine Status and Transform --- 
    let mut mods_info_list: Vec<ModInfo> = Vec::new();

    for metadata in mods_metadata {
        // --- Determine Enabled Status based on Directory --- 
        let mod_dir_rel = PathBuf::from(&metadata.installed_directory);
        let mod_dir_abs = game_root.join(&mod_dir_rel);
        let disabled_dir_str = format!("{}.disabled", metadata.installed_directory);
        let disabled_dir_abs = game_root.join(PathBuf::from(&disabled_dir_str));

        let is_enabled = mod_dir_abs.is_dir(); // Enabled if the directory exists without .disabled

        log::debug!(
            "Checking status for mod '{}': Directory {:?} exists? {}. Disabled path: {:?}",
            metadata.parsed_name,
            mod_dir_abs,
            is_enabled,
            disabled_dir_abs
        );

        // Optional: Add a check/warning if BOTH the normal and .disabled directories exist, or if NEITHER exist.
        if is_enabled && disabled_dir_abs.exists() {
             log::warn!("Mod '{}' has both enabled ({:?}) and disabled ({:?}) directories present! Assuming enabled.", metadata.parsed_name, mod_dir_abs, disabled_dir_abs);
         } else if !is_enabled && !disabled_dir_abs.exists() {
             log::warn!("Mod '{}' directory not found at either {:?} or {:?}. Mod may be corrupted or partially deleted. Assuming disabled.", metadata.parsed_name, mod_dir_abs, disabled_dir_abs);
         }
        // --- End Status Check ---

         log::info!("Mod '{}' final enabled status: {}", metadata.parsed_name, is_enabled);

        // Transform to ModInfo (using the existing definition) for the frontend
        let info = ModInfo {
            directory_name: metadata.parsed_name.clone(), // Use parsed_name as the identifier
            name: Some(metadata.parsed_name), // Use parsed_name as display name for now
            enabled: is_enabled,
            version: metadata.version, // Pass along if it exists (currently None)
            author: None, // Not tracked yet
            description: None, // Not tracked yet
        };
        mods_info_list.push(info);
    }

    log::info!("Finished processing mod list. Returning {} mods to frontend.", mods_info_list.len());
    Ok(mods_info_list)
}

// --- Static Regex Compilation --- 
// Compile the regex once for efficiency
static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Matches patterns like "Mod Name-123-4-5.zip" or "Another Mod-Complex-Name-1-0-12345.zip"
    // Captures the part before the version/identifier numbers
    Regex::new(r"^(.+?)-(\d+(?:[.-]\d+)*)-(\d+)$")
        .expect("Invalid Regex pattern")
});

#[tauri::command]
async fn install_mod_from_zip(
    app_handle: AppHandle,
    game_root_path: String,
    zip_path_str: String,
) -> Result<(), String> {
    println!("Starting install_mod_from_zip for: {}", zip_path_str);
    println!("Game Root Path: {}", game_root_path);

    let game_root = PathBuf::from(&game_root_path);
    let zip_path = PathBuf::from(&zip_path_str);

    if !game_root.is_dir() {
        return Err(format!("Game root path does not exist or is not a directory: {}", game_root_path));
    }
    if !zip_path.is_file() {
        return Err(format!("Zip path does not exist or is not a file: {}", zip_path_str));
    }

    // --- 1. Parse Filename --- 
    let original_zip_name = zip_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Could not get filename from zip path: {}", zip_path_str))?
        .to_string();

    let file_stem = zip_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(""); // Get stem, default to empty if fails

    // Use the compiled Regex to extract the base name
    let parsed_name = if !file_stem.is_empty() {
        match FILENAME_REGEX.captures(file_stem) {
            Some(caps) => {
                 // Use the first capture group (the part before the version/ID)
                caps.get(1).map_or(file_stem, |m| m.as_str()).trim_end_matches(|c: char| c == '-' || c == '.').to_string()
            }
            None => {
                // Fallback: If regex doesn't match, use the whole stem
                println!("Regex did not match filename stem: '{}'. Using full stem as parsed name.", file_stem);
                file_stem.to_string()
            }
        }
    } else {
        // If stem is empty (e.g., ".zip"), use a default
        "unknown_mod".to_string()
    };

    // Ensure parsed_name is not empty after potential trimming
    let parsed_name = if parsed_name.is_empty() { 
        println!("Warning: Parsed name became empty. Falling back to 'unknown_mod'. Original stem: '{}'", file_stem);
        "unknown_mod".to_string()
    } else {
        parsed_name
    };


    println!("Parsed mod name: '{}', Original zip: '{}'", parsed_name, original_zip_name);

    // --- 2. Determine Target Base Dir & Create Mod Dir --- 
    let target_root_path = PathBuf::from(&game_root_path);
    let source_zip_path = PathBuf::from(&zip_path_str);

    if !source_zip_path.is_file() {
        return Err(format!("Source zip file not found: {:?}", source_zip_path));
    }
    let target_reframework_path = target_root_path.join("reframework");
    fs::create_dir_all(&target_reframework_path)
        .map_err(|e| format!("Failed to create target reframework directory {:?}: {}", target_reframework_path, e))?;

    // Determine if mod goes in plugins or autorun (default to plugins)
    // TODO: This logic might need refinement. How to reliably detect autorun scripts?
    // Maybe check if the zip ONLY contains .lua files directly under autorun?
    // For now, assume plugins unless proven otherwise (needs better check).
    let mut base_target_dir_name = "plugins";
    // A simple check: if the zip contains *any* path starting with "autorun/"
    let mut is_autorun_mod = false;
    {
        let file_peek = fs::File::open(&source_zip_path).map_err(|e| format!("Peek failed: {}", e))?;
        let mut archive_peek = ZipArchive::new(file_peek).map_err(|e| format!("Peek archive failed: {}", e))?;
        for i in 0..archive_peek.len() {
            if let Ok(file_peek) = archive_peek.by_index(i) {
                if let Some(path) = file_peek.enclosed_name() {
                    // Check if any component path contains reframework/autorun
                    let path_str = path.to_string_lossy();
                    if path_str.contains("reframework/autorun") {
                         // More robust check - look for the pattern anywhere
                        is_autorun_mod = true;
                        log::debug!("Peek: Found 'reframework/autorun' pattern in '{}', classifying as autorun mod.", path_str);
                        break;
                    } else if path.components().any(|comp| comp.as_os_str() == "autorun") {
                        // Less specific check: If "autorun" exists as a directory component *anywhere*,
                        // assume it's an autorun mod. This might be too broad?
                        // Consider if we only want top-level or reframework/ level?
                        // Let's stick to the contains check for now, it's less ambiguous.
                        // is_autorun_mod = true;
                        // log::debug!("Peek: Found 'autorun' component in '{}', classifying as autorun mod.", path_str);
                        // break;
                    }
                }
            }
        }
    }
    if is_autorun_mod {
        base_target_dir_name = "autorun";
    }

    let target_base_path = target_reframework_path.join(base_target_dir_name);
    let mod_target_dir_abs = target_base_path.join(&parsed_name);
    let mod_target_dir_disabled_abs = target_base_path.join(format!("{}.disabled", parsed_name));
    let mod_target_dir_rel = PathBuf::from("reframework").join(base_target_dir_name).join(&parsed_name);

    log::info!("Target directory for '{}': {:?}", parsed_name, mod_target_dir_abs);

    // Handle existing directories
    if mod_target_dir_abs.exists() {
        log::warn!("Mod directory {:?} already exists. Removing before extraction.", mod_target_dir_abs);
        fs::remove_dir_all(&mod_target_dir_abs)
            .map_err(|e| format!("Failed to remove existing mod directory {:?}: {}", mod_target_dir_abs, e))?;
    }
    if mod_target_dir_disabled_abs.exists() {
        log::warn!("Disabled mod directory {:?} already exists. Removing before extraction.", mod_target_dir_disabled_abs);
        fs::remove_dir_all(&mod_target_dir_disabled_abs)
            .map_err(|e| format!("Failed to remove existing disabled mod directory {:?}: {}", mod_target_dir_disabled_abs, e))?;
    }

    // Create the final mod directory
    fs::create_dir_all(&mod_target_dir_abs)
         .map_err(|e| format!("Failed to create mod directory {:?}: {}", mod_target_dir_abs, e))?;

    // --- 3. Process Zip Archive & Extract --- 
    let file = fs::File::open(&source_zip_path)
        .map_err(|e| format!("Failed to open zip file {:?}: {}", source_zip_path, e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive {:?}: {}", source_zip_path, e))?;

    let mut extraction_count = 0;
    let mut determined_base_dir: Option<&str> = None; // Track if we are in plugins or autorun

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to get entry {} from zip: {}", i, e))?;

        let original_entry_path = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => {
                log::warn!("Skipping zip entry with potentially unsafe path: {}", file.name());
                continue;
            }
        };

        log::debug!("Processing zip entry: {:?}", original_entry_path);

        // --- Determine target path within the mod directory --- 
        let mut relative_path_for_extraction: Option<PathBuf> = None;
        let mut current_entry_base: Option<&str> = None;

        let components: Vec<_> = original_entry_path.components().collect();

        // Case 1: Starts directly with "plugins" or "autorun"
        if components.len() > 1 {
             if let Some(first_comp_str) = components[0].as_os_str().to_str() {
                 if first_comp_str == "plugins" || first_comp_str == "autorun" {
                     current_entry_base = Some(first_comp_str);
                     relative_path_for_extraction = Some(original_entry_path.iter().skip(1).collect());
                     log::debug!("  -> Case 1: Direct match '{}', relative: {:?}", first_comp_str, relative_path_for_extraction);
                 }
             }
         }

        // Case 2: Starts with "reframework/plugins" or "reframework/autorun"
        if relative_path_for_extraction.is_none() && components.len() > 2 {
             if let Some(first_comp_str) = components[0].as_os_str().to_str() {
                 if first_comp_str == "reframework" {
                     if let Some(second_comp_str) = components[1].as_os_str().to_str() {
                         if second_comp_str == "plugins" || second_comp_str == "autorun" {
                             current_entry_base = Some(second_comp_str);
                             relative_path_for_extraction = Some(original_entry_path.iter().skip(2).collect());
                              log::debug!("  -> Case 2: Nested match 'reframework/{}', relative: {:?}", second_comp_str, relative_path_for_extraction);
                         }
                     }
                 }
             }
         }

        // Case 3: Starts with <FolderName>/plugins or <FolderName>/autorun
         if relative_path_for_extraction.is_none() && components.len() > 2 {
             // No need to check first component name
             if let Some(second_comp_str) = components[1].as_os_str().to_str() {
                 if second_comp_str == "plugins" || second_comp_str == "autorun" {
                     current_entry_base = Some(second_comp_str);
                     relative_path_for_extraction = Some(original_entry_path.iter().skip(2).collect());
                      log::debug!("  -> Case 3: Nested match '{}/{}', relative: {:?}", components[0].as_os_str().to_string_lossy(), second_comp_str, relative_path_for_extraction);
                 }
             }
         }

        // Case 4: Starts with <FolderName>/reframework/plugins or <FolderName>/reframework/autorun
        if relative_path_for_extraction.is_none() && components.len() > 3 {
            if let Some(second_comp_str) = components[1].as_os_str().to_str() {
                if second_comp_str == "reframework" {
                    if let Some(third_comp_str) = components[2].as_os_str().to_str() {
                        if third_comp_str == "plugins" || third_comp_str == "autorun" {
                            current_entry_base = Some(third_comp_str);
                            relative_path_for_extraction = Some(original_entry_path.iter().skip(3).collect());
                            log::debug!("  -> Case 4: Nested match '{}/reframework/{}', relative: {:?}", components[0].as_os_str().to_string_lossy(), third_comp_str, relative_path_for_extraction);
                        }
                    }
                }
            }
        }

        // --- Validate and Finalize Path ---
        if relative_path_for_extraction.is_none() || current_entry_base.is_none(){
            log::debug!("  -> Skipping entry, does not match expected structure (plugins/autorun).");
            continue;
        }

        let path_inside_mod_dir = relative_path_for_extraction.unwrap();
        let entry_base = current_entry_base.unwrap();

        // Ensure consistency: If the zip contains both plugins/ and autorun/ files,
        // we should probably fail or handle it more explicitly.
        // For now, we use the base dir determined during the initial peek.
        if entry_base != base_target_dir_name {
             log::warn!(
                 "Skipping entry {:?}: Belongs to '{}', but mod was classified as '{}' based on initial zip scan.",
                 original_entry_path, entry_base, base_target_dir_name
             );
             continue;
         }

        // Prevent empty paths after stripping prefixes
         if path_inside_mod_dir.as_os_str().is_empty() {
             log::debug!("  -> Skipping entry {:?}: Resulting path inside mod dir is empty.", original_entry_path);
             continue;
         }

        let final_target_path = mod_target_dir_abs.join(&path_inside_mod_dir);

        // --- Perform Extraction ---
        if file.is_dir() {
             log::debug!("  -> Creating directory: {:?}", final_target_path);
            fs::create_dir_all(&final_target_path)
                .map_err(|e| format!("Failed to create directory {:?}: {}", final_target_path, e))?;
        } else {
             log::debug!("  -> Extracting file: {:?}", final_target_path);
            if let Some(parent_dir) = final_target_path.parent() {
                if !parent_dir.exists() {
                     log::debug!("    -> Creating parent directory: {:?}", parent_dir);
                    fs::create_dir_all(parent_dir)
                         .map_err(|e| format!("Failed to create parent directory {:?}: {}", parent_dir, e))?;
                }
            }

            // Overwrite strategy: Remove existing file/dir first if present
            if final_target_path.exists() {
                log::warn!("  -> Target path {:?} exists. Removing before extraction.", final_target_path);
                if final_target_path.is_dir() {
                    fs::remove_dir_all(&final_target_path)
                        .map_err(|e| format!("Failed to remove existing target directory {:?}: {}", final_target_path, e))?;
                } else {
                     fs::remove_file(&final_target_path)
                         .map_err(|e| format!("Failed to remove existing target file {:?}: {}", final_target_path, e))?;
                }
            }

            let mut outfile = fs::File::create(&final_target_path)
                .map_err(|e| format!("Failed to create target file {:?}: {}", final_target_path, e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to copy content to {:?}: {}", final_target_path, e))?;
             extraction_count += 1;
        }
    }

    // --- 4. Post Extraction & Metadata Update ---
    if extraction_count > 0 {
        log::info!("Successfully extracted {} files from {} into {:?}.", extraction_count, source_zip_path.display(), mod_target_dir_abs);

        // --- Update modlist.json ---
        log::info!("Updating modlist.json...");
        let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;

        // Read existing list or create new
        let mut mods_list: ModList = match fs::read_to_string(&modlist_path) {
            Ok(content) => {
                if content.is_empty() {
                    log::info!("modlist.json is empty. Creating a new list.");
                    Vec::new()
                } else {
                    serde_json::from_str(&content)
                        .map_err(|e| format!("Failed to parse modlist.json: {}. Content: {}", e, content))?
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::info!("modlist.json not found. Creating a new list.");
                Vec::new()
            },
            Err(e) => return Err(format!("Failed to read modlist.json: {}", e)),
        };

        // Convert the relative directory path to string for storage
        let installed_dir_str = mod_target_dir_rel.to_str()
            .ok_or_else(|| format!("Failed to convert mod directory path {:?} to string", mod_target_dir_rel))?
            .replace("\\", "/"); // Ensure forward slashes

        // Create new metadata entry
        let new_mod = ModMetadata {
            parsed_name: parsed_name.clone(),
            original_zip_name: original_zip_name.clone(),
            installed_directory: installed_dir_str, // Store relative path to the mod's directory
            source: "local_zip".to_string(),
            version: None, // TODO: Potentially parse version from filename if pattern allows
        };

        // Remove existing entry if found (simple overwrite based on parsed_name)
        mods_list.retain(|m| m.parsed_name != parsed_name);
        mods_list.push(new_mod);
        log::debug!("Added/Updated mod metadata for: '{}'", parsed_name);

        // Serialize and write back
        let json_string = serde_json::to_string_pretty(&mods_list)
            .map_err(|e| format!("Failed to serialize mod list to JSON: {}", e))?;

        fs::write(&modlist_path, &json_string)
            .map_err(|e| format!("Failed to write updated modlist.json to {:?}: {}", modlist_path, e))?;

        log::info!("Successfully updated modlist.json at {:?}", modlist_path);
        // --- End of modlist.json update ---

         // TODO: Optionally delete source_zip_path on success?
         // fs::remove_file(&source_zip_path).log_warn...?
         Ok(())
    } else {
         log::warn!("No relevant files (inside reframework/plugins/autorun) found for extraction in zip: {}", source_zip_path.display());
         // Return an error because the zip didn't contain anything useful for installation
         Err(format!("Zip file {:?} did not contain expected mod structure (reframework/plugins/autorun folders).", source_zip_path))
    }
}

// --- Helper Function --- 
// Function to get the full path to a file within the app's config directory
fn get_app_config_path(app_handle: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let config_dir = app_handle.path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get app config dir: {}", e))?;
    // Ensure the directory exists before returning path
     fs::create_dir_all(&config_dir)
         .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;
    Ok(config_dir.join(filename))
}

// --- New Command: Toggle Mod Enabled State ---
#[tauri::command]
async fn toggle_mod_enabled_state(
    app_handle: AppHandle,
    game_root_path: String,
    mod_name: String, // The parsed_name from ModMetadata
    enable: bool,      // true to enable, false to disable
) -> Result<(), String> {
    log::info!(
        "Toggling mod '{}' to enabled={} in game root: {}",
        mod_name,
        enable,
        game_root_path
    );
    let game_root = PathBuf::from(&game_root_path);

    // --- 1. Load Mod List --- 
    let modlist_path = get_app_config_path(&app_handle, "modlist.json")?;
    log::debug!("Reading mod list from: {:?}", modlist_path);

    let mods_metadata: ModList = match fs::read_to_string(&modlist_path) {
        Ok(content) => {
            if content.is_empty() {
                return Err(format!(
                    "Cannot toggle mod '{}': modlist.json is empty.",
                    mod_name
                ));
            } else {
                serde_json::from_str(&content).map_err(|e| {
                    format!(
                        "Failed to parse modlist.json: {}. Content: {}",
                        e,
                        content
                    )
                })?
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(format!(
                "Cannot toggle mod '{}': modlist.json not found.",
                mod_name
            ));
        }
        Err(e) => return Err(format!("Failed to read modlist.json: {}", e)),
    };

    // --- 2. Find the Mod Metadata --- 
    let mod_meta = mods_metadata
        .iter()
        .find(|m| m.parsed_name == mod_name)
        .ok_or_else(|| format!("Mod '{}' not found in modlist.json", mod_name))?;

    log::debug!(
        "Found metadata for mod '{}'. Installed directory: {}",
        mod_name,
        mod_meta.installed_directory
    );

    // --- 3. Rename Directory --- 
    let installed_dir_rel = PathBuf::from(&mod_meta.installed_directory);
    let installed_dir_abs = game_root.join(&installed_dir_rel);
    let disabled_dir_str = format!("{}.disabled", mod_meta.installed_directory);
    let disabled_dir_abs = game_root.join(PathBuf::from(&disabled_dir_str));

    if enable {
        // Enable: Rename *.disabled to * (if it exists)
        if disabled_dir_abs.exists() {
            log::info!("Enabling mod '{}': Renaming {:?} -> {:?}", mod_name, disabled_dir_abs, installed_dir_abs);
            match fs::rename(&disabled_dir_abs, &installed_dir_abs) {
                Ok(_) => {
                    log::info!("Successfully enabled mod '{}'", mod_name);
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Failed to rename {:?} to {:?}: {}", disabled_dir_abs, installed_dir_abs, e);
                    log::error!("{}", err_msg);
                    Err(err_msg)
                }
            }
        } else if installed_dir_abs.exists() {
            log::info!("Mod '{}' is already enabled (directory {:?} exists).", mod_name, installed_dir_abs);
            Ok(()) // Already in desired state
        } else {
            let err_msg = format!("Cannot enable mod '{}': Neither directory {:?} nor {:?} found.", mod_name, installed_dir_abs, disabled_dir_abs);
            log::error!("{}", err_msg);
            Err(err_msg)
        }
    } else {
        // Disable: Rename * to *.disabled (if it exists)
        if installed_dir_abs.exists() {
            log::info!("Disabling mod '{}': Renaming {:?} -> {:?}", mod_name, installed_dir_abs, disabled_dir_abs);
            match fs::rename(&installed_dir_abs, &disabled_dir_abs) {
                Ok(_) => {
                    log::info!("Successfully disabled mod '{}'", mod_name);
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Failed to rename {:?} to {:?}: {}", installed_dir_abs, disabled_dir_abs, e);
                    log::error!("{}", err_msg);
                    Err(err_msg)
                }
            }
        } else if disabled_dir_abs.exists() {
            log::info!("Mod '{}' is already disabled (directory {:?} exists).", mod_name, disabled_dir_abs);
            Ok(()) // Already in desired state
        } else {
             let err_msg = format!("Cannot disable mod '{}': Neither directory {:?} nor {:?} found.", mod_name, installed_dir_abs, disabled_dir_abs);
             log::error!("{}", err_msg);
             Err(err_msg)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize devtools only in debug builds
    // #[cfg(debug_assertions)] let devtools = tauri_plugin_devtools::init();

    // --- Create Cache State ---
    let api_cache = ApiCache::default();

    // Start the builder
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_log::Builder::new().build());

    // // Add devtools plugin conditionally
    // #[cfg(debug_assertions)]
    // {
    //     builder = builder.plugin(devtools);
    // }

    // Initialize essential plugins
    builder = builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init()); // Keep shell plugin for other potential uses (if any)


    // Continue with the rest of the builder configuration
    // --- Add Cache State ---
    builder
        .manage(api_cache) // Manage the ApiCache instance
        .invoke_handler(tauri::generate_handler![
            finalize_setup,
            // Add the command from the nexus_api module
            nexus_api::fetch_trending_mods,
            // Added new commands
            check_reframework_installed,
            install_reframework,
            // Command to load the single game config
            load_game_config,
            // Command to open the mods folder
            open_mods_folder,
            // Command to delete the user configuration file
            delete_config,
            // Command to list installed mods
            list_mods,
            // Command to install mod from zip
            install_mod_from_zip,
            // Command to toggle mod enabled state
            toggle_mod_enabled_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
