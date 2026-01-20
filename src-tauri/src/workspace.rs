use std::fs;
use tauri::command;

#[derive(serde::Serialize)]
pub struct Workspace {
    name: String,
    path: String,
}

#[command]
pub fn get_recent_workspaces() -> Result<Vec<Workspace>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");

    if !hackx_dir.exists() {
        return Ok(Vec::new());
    }

    let mut workspaces = Vec::new();

    let entries = fs::read_dir(hackx_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                // Check for .vault folder inside
                let vault_path = path.join(".vault");
                if vault_path.exists() && vault_path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        workspaces.push(Workspace {
                            name: name.to_string(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(workspaces)
}

#[command]
pub fn create_workspace(name: String) -> Result<String, String> {
    if name.is_empty() {
        return Err("Workspace name cannot be empty".to_string());
    }

    // Basic validation for folder name characters could be added here
    if name.contains('/') || name.contains('\\') || name.contains(':') {
        return Err("Invalid workspace name".to_string());
    }

    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");

    if !hackx_dir.exists() {
        fs::create_dir(&hackx_dir).map_err(|e| format!("Failed to create main folder: {}", e))?;
    }

    let workspace_path = hackx_dir.join(&name);
    if workspace_path.exists() {
        return Err("Workspace already exists".to_string());
    }

    fs::create_dir(&workspace_path)
        .map_err(|e| format!("Failed to create workspace folder: {}", e))?;

    let vault_path = workspace_path.join(".vault");
    fs::create_dir(&vault_path).map_err(|e| format!("Failed to create .vault folder: {}", e))?;

    Ok(workspace_path.to_string_lossy().to_string())
}
