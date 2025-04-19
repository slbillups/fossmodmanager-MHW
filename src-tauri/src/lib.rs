use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{WebviewWindow,AppHandle, Manager};

use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use vdf_reader;
// Declare the new module
mod nexus_api;
use nexus_api::ApiCache;
use reqwest;
use zip;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
// fn greet(name: &str) -> String {
// format!("Hello, {}! You've been greeted from Rust!", name)

// Helper function to find the game root directory

// Helper function to find the game root and steamapps directories
fn find_game_paths_from_exe(executable_path_str: &str) -> Result<(PathBuf, PathBuf), String> {
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
                // Success! current_path is game root, grandparent_path is steamapps
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

// Helper function to parse ACF using vdf-reader crate
fn find_steam_app_details(
    steamapps_path: &Path,
    game_install_dir_name: &str,
) -> Result<(String, String), String> {
    for entry in
        fs::read_dir(steamapps_path).map_err(|e| format!("Failed to read steamapps dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry in steamapps dir: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("appmanifest_") && filename.ends_with(".acf") {
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read ACF file {:?}: {}", path, e))?;

                    // Use the correct vdf_reader::from_str function
                    match vdf_reader::from_str::<HashMap<String, serde_json::Value>>(&content) {
                        // Changed entry_from_str to from_str AND type to HashMap
                        Ok(parsed_data) => {
                            // Now parsed_data should be the HashMap. Get "AppState" and treat it as the object.
                            if let Some(app_state_obj) =
                                parsed_data.get("AppState").and_then(|v| v.as_object())
                            {
                                let installdir =
                                    app_state_obj.get("installdir").and_then(|v| v.as_str());
                                // Try extracting as string, fallback to number and convert to string
                                let buildid_val = app_state_obj.get("buildid");
                                let buildid = buildid_val
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                                    .or_else(|| {
                                        buildid_val.and_then(|v| v.as_u64()).map(|n| n.to_string())
                                    });

                                let appid_val = app_state_obj.get("appid");
                                let appid = appid_val
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                                    .or_else(|| {
                                        appid_val.and_then(|v| v.as_u64()).map(|n| n.to_string())
                                    });

                                // Debug print the extracted values
                                // println!("ACF {:?}: Extracted installdir={:?}, buildid={:?}, appid={:?}", path, installdir, buildid, appid);
                                // println!("ACF {:?}: Comparing installdir to game_install_dir_name: 	{}	", path, game_install_dir_name);

                                if let (Some(dir), Some(bid), Some(aid)) =
                                    (installdir, buildid, appid)
                                {
                                    // println!("ACF {:?}: Successfully extracted all fields.", path);
                                    if dir == game_install_dir_name {
                                        println!(
                                            "Found matching ACF: {:?}, AppID: {}, BuildID: {}",
                                            path, aid, bid
                                        );
                                        return Ok((aid.to_string(), bid.to_string()));
                                    } // else { // println!("ACF {:?}: installdir 	{}	 does NOT match game_name 	{}	", path, dir, game_install_dir_name); }
                                } else {
                                    // println!("ACF {:?}: Failed to extract one or more fields (installdir/buildid/appid).", path);
                                }
                            } else {
                                eprintln!("Warning: 'AppState' key not found or not an object in ACF {:?}. Data: {:?}", path, parsed_data);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to parse ACF file {:?} using vdf-reader: {}",
                                path, e
                            );
                        }
                    }
                }
            }
        }
    }

    Err(format!(
        "Could not find matching ACF file for game '{}' in {:?}",
        game_install_dir_name, steamapps_path
    ))
}

// Helper function to create a URL-friendly slug from a string
fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut last_char_was_hyphen = true; // Start true to avoid leading hyphen

    for char in name.to_lowercase().chars() {
        match char {
            'a'..='z' | '0'..='9' => {
                slug.push(char);
                last_char_was_hyphen = false;
            }
            ' ' | '-' => {
                if !last_char_was_hyphen {
                    slug.push('-');
                    last_char_was_hyphen = true;
                }
            }
            _ => {
                // Skip other characters, but treat as potential hyphen boundary
                if !last_char_was_hyphen {
                    last_char_was_hyphen = true;
                }
            }
        }
    }

    // Remove trailing hyphen if it exists
    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

// Helper function to find the first JPEG file in a directory (non-recursive)
fn find_first_jpeg_in_dir(dir_path: &Path) -> Option<PathBuf> {
    if !dir_path.is_dir() {
        return None;
    }
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
                                println!("[Cover Art Debug] Found JPEG: {:?}", path);
                                return Some(path);
                            }
                        }
                    }
                }
            }
            None // No JPEG found in this directory
        }
        Err(e) => {
            eprintln!("Error reading directory {:?}: {}", dir_path, e);
            None
        }
    }
}

// Helper function to read image bytes and encode to base64 data URL
fn read_and_encode_image(image_path: &Path) -> Option<String> {
    match fs::read(image_path) {
        Ok(image_bytes) => {
            let encoded = STANDARD.encode(&image_bytes);
            Some(format!("data:image/jpeg;base64,{}", encoded))
        }
        Err(e) => {
            eprintln!("Error reading cover art file {:?}: {}", image_path, e);
            None
        }
    }
}

// Helper function to get cover art data URL - Updated Logic
fn get_cover_art_data_url(appid: &str) -> Option<String> {
    // Base path for the appid library cache
    let base_path_str = format!("~/.local/share/Steam/appcache/librarycache/{}", appid);
    let expanded_base_path_cow = shellexpand::tilde(&base_path_str);
    let base_path = PathBuf::from(expanded_base_path_cow.as_ref());

    println!(
        "[Cover Art Debug] Checking for JPEG in base path: {:?}",
        base_path
    );
    if let Some(image_path) = find_first_jpeg_in_dir(&base_path) {
        println!("[Cover Art Debug] Found JPEG directly in base path.");
        return read_and_encode_image(&image_path);
    }

    println!(
        "[Cover Art Debug] No JPEG found directly. Checking subdirectories in: {:?}",
        base_path
    );
    if base_path.is_dir() {
        match fs::read_dir(&base_path) {
            Ok(entries) => {
                for entry_result in entries {
                    if let Ok(entry) = entry_result {
                        let subdir_path = entry.path();
                        if subdir_path.is_dir() {
                            println!("[Cover Art Debug] Checking subdirectory: {:?}", subdir_path);
                            if let Some(image_path) = find_first_jpeg_in_dir(&subdir_path) {
                                println!("[Cover Art Debug] Found JPEG in subdirectory.");
                                return read_and_encode_image(&image_path);
                            }
                        }
                    }
                }
                println!("[Cover Art Debug] Checked all subdirectories, no JPEG found.");
            }
            Err(e) => {
                eprintln!(
                    "Error reading base directory entries {:?}: {}",
                    base_path, e
                );
            }
        }
    } else {
        println!(
            "[Cover Art Debug] Base path {:?} does not exist or is not a directory.",
            base_path
        );
    }

    println!(
        "[Cover Art Debug] Cover art JPEG not found for appid {} after checking base and subdirs.
",
        appid
    );
    None
}

// Command to finalize setup, close the setup window, and show the main window
#[tauri::command]
async fn finalize_setup(
    window: WebviewWindow,
    app_handle: AppHandle,
    executable_path: String,
) -> Result<(), String> {
    // Find game root and steamapps paths
    let (game_root_path, steamapps_path) = find_game_paths_from_exe(&executable_path)?;

    // Extract the game name (install dir name)
    let game_name = game_root_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "Could not extract game name from path: {:?}",
                game_root_path
            )
        })?
        .to_string();

    // Convert game root path to string
    let game_root_path_str = game_root_path
        .to_str()
        .ok_or_else(|| {
            format!(
                "Failed to convert game root path {:?} to string",
                game_root_path
            )
        })?
        .to_string();

    println!("Selected Executable: {}", executable_path);
    println!("Determined Game Root: {}", game_root_path_str);
    println!("Determined Game Name: {}", game_name);
    println!("Found steamapps Path: {:?}", steamapps_path);

    // Find AppID and BuildID from ACF files
    let (appid, buildid) = find_steam_app_details(&steamapps_path, &game_name)?;
    println!("Extracted AppID: {}, BuildID: {}", appid, buildid);

    // Get cover art
    let cover_art_data_url = get_cover_art_data_url(&appid);

    // --- Persist the game data ---
    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load/create store: {}", e))?;

    // Create the first game data entry
    let game_data = GameData {
        // Add appid here
        appid: appid.clone(),
        game_name: game_name.clone(),
        game_slug: slugify(&game_name),
        game_root_path: game_root_path_str,
        game_executable_path: executable_path,
        version: buildid,   // Use buildid as version
        cover_art_data_url, // Restore original field
    };

    // Save it as a list containing just this one game
    let games_list = vec![game_data];
    store.set("games_list".to_string(), json!(games_list));

    // Save the store to disk
    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    println!("Saved initial game data to store.");
    // --------------------------------------

    // Get the main window using AppHandle obtained from args
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    } else {
        eprintln!("Error: Could not find main window after closing setup.");
    }

    // Close the setup window itself
    if window.label() == "setup" {
        window.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Struct to hold game data - Renamed and made Deserializable
#[derive(Serialize, Deserialize, Clone, Debug)] // Added Deserialize, Clone, Debug
struct GameData {
    appid: String, // Added appid as it's useful for keys/identification
    game_name: String,
    game_slug: String, // The URL-friendly version of the game name
    game_root_path: String,
    game_executable_path: String,
    version: String, // Renamed from buildid for clarity in frontend
    cover_art_data_url: Option<String>, // Base64 encoded data URL for cover art
}

// Command to load the list of game data from the store
#[tauri::command]
async fn load_game_list(app_handle: AppHandle) -> Result<Vec<GameData>, String> {
    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load store: {}", e))?;

    // Retrieve the list of games
    match store.get("games_list") {
        Some(games_list_json) => {
            // Attempt to deserialize the JSON into Vec<GameData>
            serde_json::from_value(games_list_json.clone()) // Clone needed as from_value takes ownership
                .map_err(|e| format!("Failed to parse games_list from store: {}", e))
        }
        None => {
            println!("No 'games_list' found in store. Returning empty list.");
            Ok(Vec::new()) // Return an empty vector if the key doesn't exist
        }
    }
}

// Command to add a new game
#[tauri::command]
async fn add_game(app_handle: AppHandle, executable_path: String) -> Result<GameData, String> {
    println!(
        "Attempting to add game with executable: {}",
        executable_path
    );

    // Find game root and steamapps paths
    let (game_root_path, steamapps_path) = find_game_paths_from_exe(&executable_path)?;

    // Extract the game name (install dir name)
    let game_name = game_root_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "Could not extract game name from path: {:?}",
                game_root_path
            )
        })?
        .to_string();

    // Convert game root path to string
    let game_root_path_str = game_root_path
        .to_str()
        .ok_or_else(|| {
            format!(
                "Failed to convert game root path {:?} to string",
                game_root_path
            )
        })?
        .to_string();

    // --- Create fossmodmanager directory ---
    let mut mod_manager_dir = game_root_path.clone(); // Start with game_root_path
    mod_manager_dir.push("fossmodmanager"); // Append the directory name

    // Add logging before the check
    println!(
        "Checking for fossmodmanager directory existence at: {:?}",
        mod_manager_dir
    );

    if !mod_manager_dir.exists() {
        println!("Creating directory: {:?}", mod_manager_dir);
        fs::create_dir(&mod_manager_dir).map_err(|e| {
            format!(
                "Failed to create fossmodmanager directory at {:?}: {}",
                mod_manager_dir, e
            )
        })?;
    } else {
        println!("Directory already exists: {:?}", mod_manager_dir);
    }
    // --- End of directory creation ---

    // Find AppID and BuildID from ACF files
    let (appid, buildid) = find_steam_app_details(&steamapps_path, &game_name)?;
    println!(
        "Found AppID: {}, BuildID: {} for {}",
        appid, buildid, game_name
    );

    // Get cover art
    let cover_art_data_url = get_cover_art_data_url(&appid);

    // --- Load existing games and add the new one ---
    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load/create store: {}", e))?;

    // Load the current list, default to empty if not found or parse error
    let mut games_list: Vec<GameData> = match store.get("games_list") {
        Some(games_list_json) => {
            serde_json::from_value(games_list_json.clone()).unwrap_or_else(|e| {
                eprintln!("Failed to parse existing games_list, starting fresh: {}", e);
                Vec::new()
            })
        }
        None => Vec::new(),
    };

    // Check if game already exists (by appid or root path)
    if games_list
        .iter()
        .any(|g| g.appid == appid || g.game_root_path == game_root_path_str)
    {
        return Err(format!(
            "Game '{}' (AppID: {}) already exists in the list.",
            game_name, appid
        ));
    }

    // Create the new game data entry
    let new_game_data = GameData {
        appid: appid.clone(),
        game_name: game_name.clone(),
        game_slug: slugify(&game_name),
        game_root_path: game_root_path_str,
        game_executable_path: executable_path,
        version: buildid,
        cover_art_data_url,
    };

    // Add the new game to the list
    games_list.push(new_game_data.clone()); // Clone needed to return it later

    // Save the updated list back to the store
    store.set("games_list".to_string(), json!(games_list));
    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    println!(
        "Successfully added game (AppID: {}) and saved updated list.",
        appid
    );

    Ok(new_game_data) // Return the data for the newly added game
}

// Command to remove a game from the list (by appid)
#[tauri::command]
async fn remove_game(app_handle: AppHandle, appid: String) -> Result<(), String> {
    println!("Attempting to remove game with AppID: {}", appid);

    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load/create store: {}", e))?;

    // Load the current list, default to empty if not found or parse error
    let mut games_list: Vec<GameData> = match store.get("games_list") {
        Some(games_list_json) => {
            serde_json::from_value(games_list_json.clone()).unwrap_or_else(|e| {
                eprintln!(
                    "Failed to parse existing games_list while removing, starting fresh: {}",
                    e
                );
                Vec::new()
            })
        }
        None => Vec::new(),
    };

    // Find the index of the game to remove
    let initial_len = games_list.len();
    games_list.retain(|game| game.appid != appid);
    let final_len = games_list.len();

    if final_len == initial_len {
        // Game not found, maybe return an error or just log?
        eprintln!(
            "Game with AppID {} not found in the list. No changes made.",
            appid
        );
        // Optionally return Err here if needed by frontend
        // return Err(format!("Game with AppID {} not found.", appid));
    }

    // Save the potentially modified list back to the store
    store.set("games_list".to_string(), json!(games_list));
    store
        .save()
        .map_err(|e| format!("Failed to save store after removing game: {}", e))?;

    println!(
        "Successfully processed removal for game (AppID: {}). List saved.",
        appid
    );
    Ok(())
}

// Command to ensure the fossmodmanager directory exists AND open it
#[tauri::command]
async fn ensure_and_open_mods_folder(app_handle: AppHandle, appid: String) -> Result<(), String> {
    println!("Ensuring and opening mod directory for AppID: {}", appid);

    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load store: {}", e))?;

    // Load the game list
    let games_list: Vec<GameData> = match store.get("games_list") {
        Some(games_list_json) => serde_json::from_value(games_list_json.clone())
            .map_err(|e| format!("Failed to parse games_list from store: {}", e))?,
        None => {
            return Err("Cannot ensure directory: Game list not found in store.".to_string());
        }
    };

    // Find the game by appid
    let game = games_list
        .iter()
        .find(|g| g.appid == appid)
        .ok_or_else(|| format!("Game with AppID {} not found in list.", appid))?;

    // Construct the mod directory path
    let mut mod_manager_dir = PathBuf::from(&game.game_root_path);
    mod_manager_dir.push("fossmodmanager");
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
                    "Failed to create fossmodmanager directory at {:?}: {}",
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
        "Successfully ensured and requested to open mod directory for AppID: {}",
        appid
    );
    Ok(())
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
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init()); // Keep shell plugin for other potential uses (if any)

    // Continue with the rest of the builder configuration
    builder
        // --- Add Cache State ---
        .manage(api_cache) // Manage the ApiCache instance
        // Add the setup hook
        .setup(|app| {
            // --- Conditional Setup Logic ---

            // Run normal setup logic only in release builds
            #[cfg(not(debug_assertions))]
            {
                // Check if game data already exists. If not, show setup.
                let store_path = app
                    .path()
                    .resolve("settings.dat", tauri::path::BaseDirectory::AppConfig)
                    .expect("Failed to resolve store path"); // Handle error better?

                let show_setup = if store_path.exists() {
                    // Try loading the store to see if games_list exists and is valid
                    match app.store("settings.dat") {
                        Ok(store) => {
                            match store.get("games_list") {
                                Some(list_val) => {
                                    serde_json::from_value::<Vec<GameData>>(list_val.clone())
                                        .ok()
                                        .map_or(true, |list| list.is_empty())
                                }
                                None => true, // Show setup if key doesn't exist
                            }
                        }
                        Err(_) => true, // Show setup if store fails to load
                    }
                } else {
                    true // Show setup if store file doesn't exist
                };

                if show_setup {
                    println!("No valid game data found, showing setup window.");
                    // Hide the main window initially
                    if let Some(main_window) = app.handle().get_webview_window("main") {
                        let _ = main_window.hide();
                    } else {
                        eprintln!("Could not get main window handle to hide it during setup.");
                    }

                    // Create the setup window using WebviewWindowBuilder
                    use tauri::{WebviewUrl, WebviewWindowBuilder};
                    let _setup_window = WebviewWindowBuilder::new(
                        app.handle(),
                        "setup".to_string(), /* the unique window label */
                        WebviewUrl::App("setup.html".into()), // Use WebviewUrl
                    )
                    .title("Initial Setup - Select Game Executable")
                    .inner_size(600.0, 400.0)
                    .resizable(false)
                    .build()?;

                    // Optionally open devtools for the setup window (still useful even in release if needed)
                    // _setup_window.open_devtools();
                } else {
                    println!("Valid game data found, skipping setup window.");
                    // Ensure main window is shown if setup is skipped
                    if let Some(main_window) = app.handle().get_webview_window("main") {
                        let _ = main_window.show();
                        let _ = main_window.set_focus();
                    }
                }
            }

            // Skip setup and show main window directly in debug builds
            #[cfg(debug_assertions)]
            {
                println!("Debug build: Skipping setup check, showing main window directly.");
                if let Some(main_window) = app.handle().get_webview_window("main") {
                    let _ = main_window.show();
                    let _ = main_window.set_focus();

                    // Optionally open devtools for main window in debug
                    main_window.open_devtools();
                } else {
                    eprintln!("Could not get main window handle during direct startup.");
                }
            }
            // --- END OF CONDITIONAL ---

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            finalize_setup,
            // Renamed command
            load_game_list,
            // Added new command
            add_game,
            // Command to remove a game
            remove_game,
            // Added new command to ensure and open
            ensure_and_open_mods_folder,
            // Add the command from the nexus_api module
            nexus_api::fetch_trending_mods,
            // Added new commands
            check_reframework_installed,
            install_reframework
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
