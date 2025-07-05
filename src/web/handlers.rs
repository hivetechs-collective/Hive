use super::*;
use axum::extract::Query;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileQuery>,
) -> Result<Json<Vec<FileEntry>>, StatusCode> {
    let base_path = &state.workspace_path;
    let target_path = if let Some(path) = query.path {
        base_path.join(path)
    } else {
        base_path.clone()
    };

    if !target_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut entries = Vec::new();
    
    match fs::read_dir(&target_path) {
        Ok(dir_entries) => {
            for entry in dir_entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let metadata = entry.metadata().ok();
                    
                    let relative_path = path.strip_prefix(base_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    
                    let file_entry = FileEntry {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: relative_path,
                        is_dir: path.is_dir(),
                        size: metadata.as_ref().map(|m| m.len()),
                        modified: metadata.as_ref().and_then(|m| {
                            m.modified().ok().and_then(|t| {
                                DateTime::<Utc>::from(t).format("%Y-%m-%d %H:%M:%S").to_string().into()
                            })
                        }),
                        extension: path.extension().map(|e| e.to_string_lossy().to_string()),
                    };
                    
                    entries.push(file_entry);
                }
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Sort directories first, then files
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(Json(entries))
}

pub async fn get_file_content(
    State(state): State<Arc<AppState>>,
    Path(file_path): Path<String>,
) -> Result<String, StatusCode> {
    let full_path = state.workspace_path.join(&file_path);
    
    if !full_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    if full_path.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if file is binary
    if is_binary_file(&full_path) {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    match fs::read_to_string(&full_path) {
        Ok(content) => Ok(content),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WebRequest>,
) -> Result<Json<ChatMessage>, StatusCode> {
    // This is a placeholder - in a real implementation, this would
    // integrate with the consensus engine
    let response = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "assistant".to_string(),
        content: format!("Processing: {}", request.message),
        timestamp: Utc::now().to_rfc3339(),
        metadata: None,
    };

    // Send to WebSocket clients
    let _ = state.tx.send(serde_json::to_string(&response).unwrap_or_default());

    Ok(Json(response))
}

fn is_binary_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            matches!(ext.to_lowercase().as_str(), 
                "exe" | "bin" | "so" | "dll" | "dylib" | "a" | "o" | "obj" |
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "webp" |
                "mp3" | "wav" | "ogg" | "flac" | "mp4" | "avi" | "mkv" |
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" |
                "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx"
            )
        }
        None => {
            // Check first few bytes for binary markers
            if let Ok(bytes) = fs::read(path) {
                if bytes.len() > 8 {
                    let slice = &bytes[..8];
                    // Check for common binary file signatures
                    slice.starts_with(b"\x7fELF") || // ELF
                    slice.starts_with(b"MZ") || // PE
                    slice.starts_with(b"\xCA\xFE\xBA\xBE") || // Mach-O
                    slice.starts_with(b"\xFF\xD8\xFF") || // JPEG
                    slice.starts_with(b"\x89PNG") || // PNG
                    slice.contains(&0u8) // Contains null bytes
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}