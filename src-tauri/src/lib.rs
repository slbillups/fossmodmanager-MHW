use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, WebviewWindow};
use serde_json::json;
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use vdf_reader;
use std::collections::HashMap;
use base64::{engine::general_purpose::STANDARD, Engine as _};

// Declare the new module
mod nexus_api;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// #[tauri::command]
// fn greet(name: &str) -> String {
// format!("Hello, {}! You've been greeted from Rust!", name)

// Helper function to find the game root directory
fn find_game_root_from_exe(executable_path_str: &str) -> Result<PathBuf, String> {
    let executable_path = PathBuf::from(executable_path_str);

    // Basic check: Ensure the provided path exists and is a file
    if !executable_path.is_file() {
        return Err(format!(
            "Provided path is not a file or does not exist: {}",
            executable_path_str
        ));
    }

    // Start from the directory containing the executable
    let mut current_path = executable_path.parent().ok_or_else(|| {
        format!(
            "Could not get parent directory of executable: {}",
            executable_path_str
        )
    })?;

    loop {
        // Get the name of the current directory
        // let current_dir_name = current_path
        //     .file_name()
        //     .and_then(|name| name.to_str())
        //     .ok_or_else(|| format!("Could not get directory name for: {:?}", current_path))?;
        // We don't actually need the current dir name itself for the check

        // Get the parent directory
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

        // Check if the parent directory is named "common"
        if parent_dir_name == "common" {
            // Check the grandparent directory
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

            // Check if the grandparent directory is named "steamapps"
            if grandparent_dir_name == "steamapps" {
                // Success! The current_path is the game root directory.
                // We return it as owned PathBuf
                return Ok(current_path.to_path_buf());
            }
        }

        // If we haven't found the structure, move up one level
        // Check if we've reached the root to prevent infinite loops on weird paths
        if current_path == parent_path {
            return Err(format!(
                "Path resolution stopped unexpectedly at: {:?}. Could not find 'steamapps/common' structure.",
                current_path
            ));
        }
        current_path = parent_path;
    }
}

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
fn find_steam_app_details(steamapps_path: &Path, game_install_dir_name: &str) -> Result<(String, String), String> {
    for entry in fs::read_dir(steamapps_path).map_err(|e| format!("Failed to read steamapps dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry in steamapps dir: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("appmanifest_") && filename.ends_with(".acf") {
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read ACF file {:?}: {}", path, e))?;

                    // Use the correct vdf_reader::from_str function
                    match vdf_reader::from_str::<HashMap<String, serde_json::Value>>(&content) { // Changed entry_from_str to from_str AND type to HashMap
                        Ok(parsed_data) => {
                            // Now parsed_data should be the HashMap. Get "AppState" and treat it as the object.
                            if let Some(app_state_obj) = parsed_data
                                    .get("AppState")
                                    .and_then(|v| v.as_object()) {
                                let installdir = app_state_obj.get("installdir").and_then(|v| v.as_str());
                                // Try extracting as string, fallback to number and convert to string
                                let buildid_val = app_state_obj.get("buildid");
                                let buildid = buildid_val
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                                    .or_else(|| buildid_val.and_then(|v| v.as_u64()).map(|n| n.to_string()));

                                let appid_val = app_state_obj.get("appid");
                                let appid = appid_val
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                                    .or_else(|| appid_val.and_then(|v| v.as_u64()).map(|n| n.to_string()));

                                // Debug print the extracted values
                                // println!("ACF {:?}: Extracted installdir={:?}, buildid={:?}, appid={:?}", path, installdir, buildid, appid);
                                // println!("ACF {:?}: Comparing installdir to game_install_dir_name: 	{}	", path, game_install_dir_name);

                                if let (Some(dir), Some(bid), Some(aid)) = (installdir, buildid, appid) {
                                    // println!("ACF {:?}: Successfully extracted all fields.", path);
                                    if dir == game_install_dir_name {
                                        println!("Found matching ACF: {:?}, AppID: {}, BuildID: {}", path, aid, bid);
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
                            eprintln!("Warning: Failed to parse ACF file {:?} using vdf-reader: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    Err(format!("Could not find matching ACF file for game '{}' in {:?}", game_install_dir_name, steamapps_path))
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

    println!("[Cover Art Debug] Checking for JPEG in base path: {:?}", base_path);
    if let Some(image_path) = find_first_jpeg_in_dir(&base_path) {
        println!("[Cover Art Debug] Found JPEG directly in base path.");
        return read_and_encode_image(&image_path);
    }

    println!("[Cover Art Debug] No JPEG found directly. Checking subdirectories in: {:?}", base_path);
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
                 eprintln!("Error reading base directory entries {:?}: {}", base_path, e);
            }
        }
    }
     else {
          println!("[Cover Art Debug] Base path {:?} does not exist or is not a directory.", base_path);
     }


    println!("[Cover Art Debug] Cover art JPEG not found for appid {} after checking base and subdirs.
", appid);
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
        .ok_or_else(|| format!("Could not extract game name from path: {:?}", game_root_path))?
        .to_string();

    // Convert game root path to string
    let game_root_path_str = game_root_path
        .to_str()
        .ok_or_else(|| format!("Failed to convert game root path {:?} to string", game_root_path))?
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
        version: buildid, // Use buildid as version
        cover_art_data_url, // Restore original field
    };

    // Save it as a list containing just this one game
    let games_list = vec![game_data];
    store.set("games_list".to_string(), json!(games_list));

    // Save the store to disk
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

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
    println!("Attempting to add game with executable: {}", executable_path);

    // Find game root and steamapps paths
    let (game_root_path, steamapps_path) = find_game_paths_from_exe(&executable_path)?;

    // Extract the game name (install dir name)
    let game_name = game_root_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Could not extract game name from path: {:?}", game_root_path))?
        .to_string();

    // Convert game root path to string
    let game_root_path_str = game_root_path
        .to_str()
        .ok_or_else(|| format!("Failed to convert game root path {:?} to string", game_root_path))?
        .to_string();

    // Find AppID and BuildID from ACF files
    let (appid, buildid) = find_steam_app_details(&steamapps_path, &game_name)?;
    println!("Found AppID: {}, BuildID: {} for {}", appid, buildid, game_name);

    // Get cover art
    let cover_art_data_url = get_cover_art_data_url(&appid);

    // --- Load existing games and add the new one ---
    let store = app_handle
        .store("settings.dat")
        .map_err(|e| format!("Failed to load/create store: {}", e))?;

    // Load the current list, default to empty if not found or parse error
    let mut games_list: Vec<GameData> = match store.get("games_list") {
        Some(games_list_json) => serde_json::from_value(games_list_json.clone())
                                    .unwrap_or_else(|e| {
                                        eprintln!("Failed to parse existing games_list, starting fresh: {}", e);
                                        Vec::new()
                                    }),
        None => Vec::new(),
    };

    // Check if game already exists (by appid or root path)
    if games_list.iter().any(|g| g.appid == appid || g.game_root_path == game_root_path_str) {
        return Err(format!("Game '{}' (AppID: {}) already exists in the list.", game_name, appid));
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
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

    println!("Successfully added game (AppID: {}) and saved updated list.", appid);

    Ok(new_game_data) // Return the data for the newly added game
}

// Removed Nexus struct definitions - they are now in nexus_api/mod.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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
                                Some(list_val) => serde_json::from_value::<Vec<GameData>>(list_val.clone()).ok().map_or(true, |list| list.is_empty()),
                                None => true, // Show setup if key doesn't exist
                            }
                        }
                        Err(_) => true // Show setup if store fails to load
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
                    let _setup_window = WebviewWindowBuilder::new(
                        app.handle(),
                        "setup".to_string(), /* the unique window label */
                        WebviewUrl::App("setup.html".into()), // Use WebviewUrl
                    )
                    .title("Initial Setup - Select Game Executable")
                    .inner_size(600.0, 400.0) // Slightly smaller maybe?
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
            // Add the command from the nexus_api module
            nexus_api::fetch_trending_mods
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
