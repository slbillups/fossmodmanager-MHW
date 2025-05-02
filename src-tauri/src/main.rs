// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Conditionally set env var for Wayland in release builds
    if cfg!(not(debug_assertions)) {
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(session_type) if session_type.eq_ignore_ascii_case("wayland") => {
                std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
                // Optional: Add a log message if needed, e.g., using the log crate
                // log::info!("Wayland detected in release build, setting WEBKIT_DISABLE_COMPOSITING_MODE=1");
            }
            _ => {} // Not Wayland or variable not set/error
        }
    }

    // Call the library's run function directly
    fossmodmanager_lib::run();
}
