use std::env;

fn main() {
    tauri_build::build();
    
    // Only relevant for Linux
    #[cfg(target_os = "linux")]
    {
        // Check if running in Wayland
        let wayland_display = env::var("WAYLAND_DISPLAY").is_ok();
        let xdg_session_type = env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false);
            
        if wayland_display || xdg_session_type {
            println!("cargo:rustc-env=WEBKIT_DISABLE_COMPOSITING_MODE=1");
        }
    }
}