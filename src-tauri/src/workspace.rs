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

#[command]
pub fn add_file_to_workspace(workspace_name: String, file_path: String) -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");
    let workspace_path = hackx_dir.join(&workspace_name);

    if !workspace_path.exists() {
        return Err("Workspace does not exist".to_string());
    }

    let source_path = std::path::Path::new(&file_path);
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    let file_name = source_path
        .file_name()
        .ok_or("Invalid source file name")?
        .to_str()
        .ok_or("Invalid characters in file name")?;

    let dest_path = workspace_path.join(file_name);

    fs::copy(source_path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(dest_path.to_string_lossy().to_string())
}

#[command]
pub fn get_workspace_files(workspace_name: String) -> Result<Vec<String>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");
    let workspace_path = hackx_dir.join(&workspace_name);

    if !workspace_path.exists() {
        return Err("Workspace does not exist".to_string());
    }

    let mut files = Vec::new();
    let entries = fs::read_dir(workspace_path).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Filter out hidden files or system files if needed, e.g. .DS_Store
                    if !name.starts_with('.') {
                        files.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn get_vault_path(workspace_name: &str) -> Result<std::path::PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");
    let workspace_path = hackx_dir.join(workspace_name);
    let vault_path = workspace_path.join(".vault");

    if !vault_path.exists() {
        fs::create_dir_all(&vault_path).map_err(|e| e.to_string())?;
    }

    Ok(vault_path)
}

#[command]
pub fn save_annotations(
    workspace_name: String,
    file_name: String,
    document: crate::document::Document,
) -> Result<(), String> {
    let vault_path = get_vault_path(&workspace_name)?;
    // Create a safe filename for the json sidecar, e.g. "document.pdf.json"
    let sidecar_name = format!("{}.json", file_name);
    let file_path = vault_path.join(sidecar_name);

    let json = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    fs::write(file_path, json).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn load_annotations(
    workspace_name: String,
    file_name: String,
) -> Result<crate::document::Document, String> {
    let vault_path = get_vault_path(&workspace_name)?;
    let sidecar_name = format!("{}.json", file_name);
    let file_path = vault_path.join(sidecar_name);

    if !file_path.exists() {
        // Return a new empty document if no annotations exist yet
        return Ok(crate::document::Document::new());
    }

    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let document: crate::document::Document =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    Ok(document)
}

#[command]
pub fn read_workspace_file(workspace_name: String, file_name: String) -> Result<Vec<u8>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let hackx_dir = home_dir.join("hackxindia26");
    let workspace_path = hackx_dir.join(&workspace_name);

    if !workspace_path.exists() {
        return Err("Workspace does not exist".to_string());
    }

    let file_path = workspace_path.join(&file_name);

    // basic security check to prevent directory traversal
    if !file_path.starts_with(&workspace_path) {
        return Err("Invalid file path".to_string());
    }

    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    fs::read(file_path).map_err(|e| e.to_string())
}
