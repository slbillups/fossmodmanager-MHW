// src-tauri/src/utils/cachethumbs.rs
use base64::{engine::general_purpose, Engine};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
// Image cache entry metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub original_path: String,        // Original image path
    pub timestamp: i64,               // When cached (unix timestamp)
}

/// Get the image cache directory path
pub fn get_image_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get app cache dir: {}", e))?
        .join("fossmodmanager")
        .join("images");

    // Ensure the cache directory exists
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create image cache directory: {}", e))?;

    Ok(cache_dir)
}

/// Generate a cache key for an image path
pub fn get_image_cache_key(image_path: &str) -> String {
    // Use a simple hash to ensure the filename is valid for filesystem
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Function to read mod image files and return as base64
#[tauri::command]
pub fn read_mod_image(image_path: String) -> Result<String, String> {
    info!("Reading mod image from: {}", image_path);

    let path = PathBuf::from(&image_path);
    if !path.exists() {
        return Err(format!("Image file does not exist: {}", image_path));
    }

    // Read the image file
    let img_data = fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?;

    // Convert to base64
    let base64_encoded = general_purpose::STANDARD.encode(&img_data);

    info!("Successfully read image: {} ({} bytes)", image_path, img_data.len());
    Ok(base64_encoded)
}

/// Function to cache a mod image
#[tauri::command]
pub async fn cache_mod_image(
    app_handle: AppHandle,
    image_path: String,
    image_data: String,
) -> Result<(), String> {
    debug!("Caching image: {}", image_path);

    // Create a unique cache key
    let cache_key = get_image_cache_key(&image_path);

    // Get the cache directory
    let cache_dir = get_image_cache_dir(&app_handle)?;
    let cache_file_path = cache_dir.join(format!("{}.cache", cache_key));

    // Store the cache entry info
    let cache_info = CacheEntry {
        original_path: image_path.clone(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    let cache_info_json = serde_json::to_string(&cache_info)
        .map_err(|e| format!("Failed to serialize cache info: {}", e))?;

    let cache_info_path = cache_dir.join(format!("{}.json", cache_key));
    fs::write(&cache_info_path, cache_info_json)
        .map_err(|e| format!("Failed to write cache info: {}", e))?;

    // Write the image data
    match general_purpose::STANDARD.decode(&image_data) {
        Ok(decoded_data) => {
            fs::write(&cache_file_path, decoded_data)
                .map_err(|e| format!("Failed to write image cache file: {}", e))?;
            debug!("Successfully cached image at {:?}", cache_file_path);
            Ok(())
        }
        Err(e) => Err(format!("Failed to decode image data: {}", e)),
    }
}

/// Function to get cached mod images
#[tauri::command]
pub async fn get_cached_mod_images(
    app_handle: AppHandle,
    image_paths: Vec<String>,
) -> Result<HashMap<String, String>, String> {
    let mut result = HashMap::new();
    let cache_dir = get_image_cache_dir(&app_handle)?;

    let image_paths_count = image_paths.len();

    // For each requested path
    for path in image_paths {
        let cache_key = get_image_cache_key(&path);
        let cache_file_path = cache_dir.join(format!("{}.cache", cache_key));
        let cache_info_path = cache_dir.join(format!("{}.json", cache_key));

        // Check if both the cache file and info exist
        if cache_file_path.exists() && cache_info_path.exists() {
            // Read and validate cache info
            match fs::read_to_string(&cache_info_path) {
                Ok(info_json) => {
                    match serde_json::from_str::<CacheEntry>(&info_json) {
                        Ok(cache_info) => {
                            // Verify it's for the right path (in case of hash collision)
                            if cache_info.original_path != path {
                                warn!("Cache key collision: {} vs {}", cache_info.original_path, path);
                                continue;
                            }

                            // Check if cache is not too old (e.g., older than 7 days)
                            // I am not sure why we are checking the cache age...doesn't seem to be useful - users are not going to be installing hundreds of skins
                            let now = chrono::Utc::now().timestamp();
                            let age = now - cache_info.timestamp;
                            if age > 7 * 24 * 60 * 60 {
                                // 7 days in seconds
                                debug!("Cache entry too old ({}), will reload: {}", age, path);
                                continue;
                            }

                            // Read and return the cached image
                            match fs::read(&cache_file_path) {
                                Ok(data) => {
                                    let base64_data = general_purpose::STANDARD.encode(data);
                                    result.insert(path.clone(), base64_data);
                                    debug!("Retrieved image from cache: {}", path);
                                }
                                Err(e) => {
                                    warn!("Failed to read cached image data: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse cache info: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read cache info: {}", e);
                }
            }
        } else {
            debug!("No cache found for: {}", path);
        }
    }

    info!("Retrieved {} cached images out of {} requested", result.len(), image_paths_count);
    Ok(result)
}