#[tauri::command]
pub fn create_file(path: String, name: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let parent_path = Path::new(&path);

    if !parent_path.exists() {
        return Err("Parent directory does not exist".to_string());
    }

    if !parent_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let file_path = parent_path.join(&name);

    if file_path.exists() {
        return Err("File already exists".to_string());
    }

    fs::write(&file_path, "").map_err(|e| format!("Failed to create file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let parent_path = Path::new(&path);

    if !parent_path.exists() {
        return Err("Parent directory does not exist".to_string());
    }

    if !parent_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let folder_path = parent_path.join(&name);

    if folder_path.exists() {
        return Err("Folder already exists".to_string());
    }

    fs::create_dir(&folder_path).map_err(|e| format!("Failed to create folder: {}", e))?;

    Ok(folder_path.to_string_lossy().to_string())
}

// File/Folder Deletion Commands

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    if file_path.is_dir() {
        return Err("Path is a directory, use delete_folder instead".to_string());
    }

    fs::remove_file(file_path).map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(())
}

// File Content Commands

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    if file_path.is_dir() {
        return Err("Path is a directory".to_string());
    }

    // Check if file is binary
    let content_bytes = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Try to convert to UTF-8 string
    match String::from_utf8(content_bytes) {
        Ok(content) => Ok(content),
        Err(_) => Err("File is binary and cannot be displayed as text".to_string()),
    }
}

#[tauri::command]
pub fn write_file_content(path: String, content: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    if file_path.is_dir() {
        return Err("Path is a directory".to_string());
    }

    fs::write(file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn write_binary_file(path: String, content: Vec<u8>) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    if file_path.is_dir() {
        return Err("Path is a directory".to_string());
    }

    fs::write(file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn is_file_binary(path: String) -> Result<bool, String> {
    use std::fs;
    use std::path::Path;

    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    if file_path.is_dir() {
        return Err("Path is a directory".to_string());
    }

    // Read first 8KB to check if binary
    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    use std::io::Read;
    let mut buffer = Vec::new();
    let bytes_read = file
        .take(8192)
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if bytes_read == 0 {
        return Ok(false); // Empty file is not binary
    }

    // Check for null bytes or high percentage of non-printable characters
    let null_bytes = buffer.iter().filter(|&&b| b == 0).count();
    if null_bytes > 0 {
        return Ok(true);
    }

    // Count non-printable ASCII characters (excluding common whitespace)
    let non_printable = buffer
        .iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();

    // If more than 30% non-printable, consider it binary
    Ok((non_printable as f32 / bytes_read as f32) > 0.3)
}

#[tauri::command]
pub fn delete_folder(path: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let folder_path = Path::new(&path);

    if !folder_path.exists() {
        return Err("Folder does not exist".to_string());
    }

    if !folder_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    fs::remove_dir_all(folder_path).map_err(|e| format!("Failed to delete folder: {}", e))?;

    Ok(())
}

// Rename Commands

#[tauri::command]
pub fn rename_entry(old_path: String, new_name: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let old_path_obj = Path::new(&old_path);

    if !old_path_obj.exists() {
        return Err("File or folder does not exist".to_string());
    }

    let parent = old_path_obj
        .parent()
        .ok_or_else(|| "Cannot get parent directory".to_string())?;

    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return Err("A file or folder with that name already exists".to_string());
    }

    fs::rename(old_path_obj, &new_path).map_err(|e| format!("Failed to rename: {}", e))?;

    Ok(new_path.to_string_lossy().to_string())
}

// Copy/Move Commands

#[tauri::command]
pub fn copy_entry(source_path: String, dest_path: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let source = Path::new(&source_path);
    let dest_dir = Path::new(&dest_path);

    if !source.exists() {
        return Err("Source does not exist".to_string());
    }

    if !dest_dir.exists() || !dest_dir.is_dir() {
        return Err("Destination directory does not exist".to_string());
    }

    let source_name = source
        .file_name()
        .ok_or_else(|| "Cannot get source name".to_string())?;

    let mut dest_path_final = dest_dir.join(source_name);

    // Handle name conflicts by appending a number
    if dest_path_final.exists() {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("copy");
        let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut counter = 1;
        loop {
            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            };
            dest_path_final = dest_dir.join(&new_name);
            if !dest_path_final.exists() {
                break;
            }
            counter += 1;
        }
    }

    if source.is_dir() {
        copy_dir_recursive(source, &dest_path_final)?;
    } else {
        fs::copy(source, &dest_path_final).map_err(|e| format!("Failed to copy file: {}", e))?;
    }

    Ok(dest_path_final.to_string_lossy().to_string())
}

#[tauri::command]
pub fn move_entry(source_path: String, dest_path: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;

    let source = Path::new(&source_path);
    let dest_dir = Path::new(&dest_path);

    if !source.exists() {
        return Err("Source does not exist".to_string());
    }

    if !dest_dir.exists() || !dest_dir.is_dir() {
        return Err("Destination directory does not exist".to_string());
    }

    let source_name = source
        .file_name()
        .ok_or_else(|| "Cannot get source name".to_string())?;

    let mut dest_path_final = dest_dir.join(source_name);

    // Handle name conflicts by appending a number
    if dest_path_final.exists() {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("moved");
        let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut counter = 1;
        loop {
            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            };
            dest_path_final = dest_dir.join(&new_name);
            if !dest_path_final.exists() {
                break;
            }
            counter += 1;
        }
    }

    fs::rename(source, &dest_path_final).map_err(|e| format!("Failed to move: {}", e))?;

    Ok(dest_path_final.to_string_lossy().to_string())
}

// Helper function to copy directory recursively
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    use std::fs;

    if !dst.exists() {
        fs::create_dir(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}
