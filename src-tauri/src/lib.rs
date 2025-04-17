// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_wine_prefix() -> String {
    // Set the wineprefix to default for both Linux and Windows
    let wine_prefix = if cfg!(target_os = "Linux") {
        "$HOME/.wine"
    } else {
        "$HOME\\AppData\\Local\\Wine"
    };
    wine_prefix.to_string()
}

#[tauri::command]
fn get_proton_path() -> String {
    let proton_path = if cfg!(target_os = "Linux") {
        "$HOME/.steam/steam/steamapps/common/(protonprefix_name)"
        // windows will not have a proton path
    };
    proton_path.to_string()
}

#[tauri::command]
fn get_steam_game_dir(game_id: String) -> String {
    // search for games in the filesystem securely, these could be in several locations and across block devices
    let game_dir = if cfg!(target_os = "Linux") {
        // linux paths will have the directory SteamLibrary/common/(game_id)
        // may not be in $HOME, so we need to search the filesystem
        let steam_library = std::env::var("STEAM_LIBRARY").unwrap_or_default();
        format!("{}/SteamLibrary/common/{}", steam_library, game_id)
    } else {
        // windows will not have a steam path
        // we need to search the filesystem for the game_id
        let game_dir = std::env::var("GAME_DIR").unwrap_or_default();
        format!("{}/{}", game_dir, game_id)
    };
    game_dir.to_string()
}
