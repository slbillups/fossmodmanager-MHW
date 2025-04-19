use dotenvy::dotenv;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// --- Cache Structures ---

#[derive(Clone, Debug)]
pub struct CacheEntry {
    data: Vec<NexusMod>,
    timestamp: Instant,
}

// Wrapper struct for the cache state to be managed by Tauri
#[derive(Default)] // Add default derive for easy initialization
pub struct ApiCache {
    // The Mutex is now inside the struct
    pub cache: Mutex<HashMap<String, CacheEntry>>,
}

const CACHE_DURATION: Duration = Duration::from_secs(3600);

// --- Nexus Mods API Structures (V1 REST API) ---

// Represents mod info from the Nexus V1 REST API (Trending Endpoint)
// NOTE: This structure is based on guessing the V1 /trending.json format.
// It might need adjustment after seeing the actual API response.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NexusMod {
    // Common fields likely present
    pub mod_id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub version: Option<String>,
    pub picture_url: Option<String>,     // Often the main image
    pub updated_timestamp: Option<u64>,  // V1 might use timestamps
    pub endorsements_count: Option<i64>, // Different naming convention?
    pub total_downloads: Option<i64>,    // Different naming convention?
    pub total_unique_downloads: Option<i64>,
    pub author: Option<String>,
    pub uploaded_timestamp: Option<u64>,
    pub external_virus_scan_url: Option<String>,
    // Fields from GraphQL that might map differently or not exist in V1 trending:
    // pub domain_name: String, // Likely not in mod details in V1 trending
    // pub thumbnail_url: Option<String>, // Might be same as picture_url or absent

    // Fields specific to trending endpoint structure (if any)
    // Example: pub trend_position: Option<i32>,
}

// --- End Nexus Mods API Structures ---

// Constants
// const NEXUS_API_URL_GRAPHQL: &str = "https://api.nexusmods.com/v2/graphql"; // Keep if needed later
const NEXUS_API_URL_V1_BASE: &str = "https://api.nexusmods.com/v1";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "fossmodmanager";

// Removed execute_query as it was for GraphQL

#[tauri::command]
pub async fn fetch_trending_mods(
    game_domain_name: String,
    state: tauri::State<'_, ApiCache>,
    // count: Option<u32>, // V1 trending doesn't seem to support count directly
) -> Result<Vec<NexusMod>, String> {
    let now = Instant::now();

    // --- Cache Check ---
    {
        let cache_map = state.cache.lock().await;
        if let Some(entry) = cache_map.get(&game_domain_name) {
            if now.duration_since(entry.timestamp) < CACHE_DURATION {
                println!(
                    "Cache hit for game: '{}'. Returning cached data.",
                    game_domain_name
                );
                return Ok(entry.data.clone());
            }
            println!(
                "Cache expired for game: '{}'. Fetching fresh data.",
                game_domain_name
            );
        } else {
            println!(
                "Cache miss for game: '{}'. Fetching data.",
                game_domain_name
            );
        }
    }

    // --- API Fetch (if cache miss or expired) ---
    println!("Proceeding with API fetch for game: '{}'", game_domain_name);

    // Load environment variables from .env file
    dotenv().ok(); // Ignore error if .env is not found, API key might be set elsewhere

    // Get API key from environment
    let api_key = env::var("NEXUS_API_KEY")
        .map_err(|_| "NEXUS_API_KEY not found in environment variables or .env file".to_string())?;

    let client = reqwest::Client::new();

    // Construct the V1 API URL
    let request_url = format!(
        "{}/games/{}/mods/trending.json",
        NEXUS_API_URL_V1_BASE, game_domain_name
    );
    println!("Fetching trending mods from: {}", request_url);

    // Construct headers for V1
    let mut headers = HeaderMap::new();
    let user_agent_string = format!("{}/{} (Rust; reqwest)", APP_NAME, APP_VERSION);
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&user_agent_string)
            .map_err(|e| format!("Invalid User-Agent header value: {}", e))?,
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    // Use HeaderName for the custom API key header
    headers.insert(
        HeaderName::from_static("apikey"),
        HeaderValue::from_str(&api_key).map_err(|_| "Invalid API Key format".to_string())?,
    );

    // Send request
    let response = client
        .get(&request_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Nexus API V1 request failed: {}", e))?;

    // Check status and parse response
    if response.status().is_success() {
        let mods = response.json::<Vec<NexusMod>>().await.map_err(|e| {
            format!(
                "Failed to parse Nexus API V1 response into Vec<NexusMod>: {}. URL: {}",
                e, request_url
            )
        })?;

        // --- Cache Update ---
        {
            let mut cache_map = state.cache.lock().await;
            println!("Updating cache for game: '{}'", game_domain_name);
            let new_entry = CacheEntry {
                data: mods.clone(),
                timestamp: Instant::now(),
            };
            cache_map.insert(game_domain_name.clone(), new_entry);
        }

        Ok(mods)
    } else {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        Err(format!(
            "Nexus API V1 request failed with status {} at URL {}: {}",
            status, request_url, error_body
        ))
    }
}
// Removed GraphQL related TODOs
