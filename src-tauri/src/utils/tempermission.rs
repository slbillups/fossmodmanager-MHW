use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{ipc::Channel, AppHandle};
// Event types for file operations
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ModOperationEvent {
    #[serde(rename_all = "camelCase")]
    Started { operation: String, mod_name: String },
    #[serde(rename_all = "camelCase")]
    Progress {
        operation: String,
        mod_name: String,
        progress: f32, // 0.0 to 1.0
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    Finished {
        operation: String,
        mod_name: String,
        success: bool,
        message: String,
    },
}

// Security wrapper combined with event notifications
// This is not a Tauri command, it's a helper function
pub async fn with_game_dir_write_access<F, R>(
    app_handle: &AppHandle,
    game_root: &PathBuf,
    on_event: &Channel<ModOperationEvent>,
    operation: &str,
    mod_name: &str,
    action: F,
) -> Result<R, String>
where
    F: FnOnce(&Channel<ModOperationEvent>) -> Result<R, String>,
{
    // 1. Verify game_root matches configured path
    let config = crate::utils::config::load_game_config(app_handle.clone()).await?;
    if let Some(config_data) = config {
        let config_game_root = PathBuf::from(&config_data.game_root_path);
        if config_game_root != *game_root {
            return Err(format!(
                "Security error: Requested game path {} doesn't match configured path {}",
                game_root.display(),
                config_game_root.display()
            ));
        }
    } else {
        return Err("Game configuration not found. Please complete setup first.".to_string());
    }

    // 2. Notify start of operation
    on_event
        .send(ModOperationEvent::Started {
            operation: operation.to_string(),
            mod_name: mod_name.to_string(),
        })
        .map_err(|e| format!("Failed to send start event: {}", e))?;

    // 3. Execute the action
    let result = action(on_event);

    // 4. Notify completion
    match &result {
        Ok(_) => {
            log::info!(
                "Successfully completed '{}' operation for '{}'",
                operation,
                mod_name
            );
            on_event
                .send(ModOperationEvent::Finished {
                    operation: operation.to_string(),
                    mod_name: mod_name.to_string(),
                    success: true,
                    message: format!("Successfully {} mod '{}'", operation, mod_name),
                })
                .map_err(|e| format!("Failed to send finish event: {}", e))?;
        }
        Err(e) => {
            log::error!(
                "Failed during '{}' operation for '{}': {}",
                operation,
                mod_name,
                e
            );
            on_event
                .send(ModOperationEvent::Finished {
                    operation: operation.to_string(),
                    mod_name: mod_name.to_string(),
                    success: false,
                    message: format!("Failed to {} mod '{}': {}", operation, mod_name, e),
                })
                .map_err(|e| format!("Failed to send error event: {}", e))?;
        }
    }

    result
}
