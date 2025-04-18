use serde::{Deserialize, Serialize};
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::json;
use std::collections::HashMap;

// --- Nexus Mods API Structures ---

// Represents basic game info from the Nexus GraphQL API
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NexusGame {
    #[serde(rename = "domainName")] // Matches GraphQL schema field name
    pub domain_name: String,
    pub id: i64, // Using i64 for potentially large IDs
    pub name: String,
    #[serde(rename = "modCount")]
    pub mod_count: Option<i64>, // Make optional as it might not always be present/needed
    #[serde(rename = "tileImageUrl")]
    pub tile_image_url: Option<String>,
}

// Represents basic mod info from the Nexus GraphQL API
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NexusMod {
    #[serde(rename = "modId")] // Matches GraphQL schema field name
    pub mod_id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "pictureUrl")]
    pub picture_url: Option<String>, // URL for the main mod image
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>, // ISO 8601 DateTime String
    pub endorsements: Option<i64>,
    pub downloads: Option<i64>,
    // We might need the game association later if fetching mods without game context
    // game: Option<NexusGame>,
}

// Structure to hold the paginated response for mods (specifically the 'mods' query result structure)
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ModsQueryResult {
    nodes: Vec<NexusMod>,
    // We might need PageInfo later for actual pagination
    // pageInfo: PageInfo,
    #[serde(rename = "totalCount")]
    total_count: i64,
}

// Structure representing the top-level 'data' object in the GraphQL response when querying mods
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ModsResponseData {
    mods: ModsQueryResult,
}

// --- End Nexus Mods API Structures ---

// Constants
const NEXUS_API_URL: &str = "https://api.nexusmods.com/v2/graphql";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION"); // Get version from Cargo.toml
const APP_NAME: &str = "fossmodmanager"; // Replace with your actual app name if different

// Helper function to execute a GraphQL query
async fn execute_query(query: String, variables: Option<HashMap<String, serde_json::Value>>) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();

    // Construct headers
    let mut headers = HeaderMap::new();
    // Construct the User-Agent string first
    let user_agent_string = format!("{}/{} (Rust; reqwest)", APP_NAME, APP_VERSION);
    // Use from_str for non-static strings and handle the Result
    headers.insert(
        USER_AGENT, 
        HeaderValue::from_str(&user_agent_string)
            .map_err(|e| format!("Invalid User-Agent header value: {}", e))?
    );
    headers.insert("Application-Name", HeaderValue::from_static(APP_NAME));
    headers.insert("Application-Version", HeaderValue::from_static(APP_VERSION));
    // TODO: Add API key header if/when needed

    // Construct request body with optional variables
    let mut body_map = HashMap::new();
    body_map.insert("query".to_string(), json!(query));
    if let Some(vars) = variables {
        body_map.insert("variables".to_string(), json!(vars));
    }
    let body = json!(body_map);

    // Send request
    let response = client
        .post(NEXUS_API_URL)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Nexus API request failed: {}", e))?;

    // Check status and parse response
    if response.status().is_success() {
        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse Nexus API response: {}", e))
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
        Err(format!(
            "Nexus API request failed with status {}: {}",
            status,
            error_body
        ))
    }
}


#[tauri::command]
pub async fn fetch_trending_mods(game_domain_name: String, count: Option<u32>) -> Result<Vec<NexusMod>, String> {
    let mod_count = count.unwrap_or(30); // Default to 30 mods

    // Define the GraphQL query string with placeholders
    let query = r#"
        query GetTrendingMods($gameDomain: String!, $count: Int!) {
            mods(filter: { gameDomainName: { value: $gameDomain, op: EQUALS } }, sort: [{ endorsements: { direction: DESC } }], count: $count) {
                nodes {
                    modId
                    name
                    summary
                    version
                    pictureUrl
                    thumbnailUrl
                    updatedAt
                    endorsements
                    downloads
                }
                totalCount
            }
        }
    "#.to_string();

    // Define the variables map
    let mut variables = HashMap::new();
    variables.insert("gameDomain".to_string(), json!(game_domain_name));
    variables.insert("count".to_string(), json!(mod_count));

    // Execute the query
    let response_json = execute_query(query, Some(variables)).await?;

    // Deserialize the response
    // We expect the structure { "data": { "mods": { "nodes": [...], "totalCount": ... } } }
    let response_data: ModsResponseData = serde_json::from_value(response_json["data"].clone())
        .map_err(|e| format!("Failed to deserialize Nexus Mods response: {}. Response JSON: {}", e, response_json))?;

    Ok(response_data.mods.nodes)
}

// TODO: Add execute_query helper function
// TODO: Add fetch_trending_mods command function 