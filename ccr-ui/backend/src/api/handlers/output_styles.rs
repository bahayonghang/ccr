use crate::models::api::ApiResponse;
use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputStyle {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOutputStyleRequest {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOutputStyleRequest {
    pub content: String,
}

fn get_output_styles_dir() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
    Ok(PathBuf::from(home).join(".claude").join("output-styles"))
}

pub async fn list_output_styles() -> impl IntoResponse {
    let dir = match get_output_styles_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    if !dir.exists() {
        return ApiResponse::success(Vec::<OutputStyle>::new());
    }

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let mut styles = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        styles.push(OutputStyle {
            name: name.to_string(),
            content,
        });
    }

    styles.sort_by(|a, b| a.name.cmp(&b.name));
    ApiResponse::success(styles)
}

pub async fn get_output_style(Path(name): Path<String>) -> impl IntoResponse {
    let dir = match get_output_styles_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let path = dir.join(format!("{}.md", name));
    match fs::read_to_string(&path) {
        Ok(content) => ApiResponse::success(OutputStyle { name, content }),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn create_output_style(
    Json(payload): Json<CreateOutputStyleRequest>,
) -> impl IntoResponse {
    let dir = match get_output_styles_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    if let Err(e) = fs::create_dir_all(&dir) {
        return ApiResponse::error(e.to_string());
    }

    let path = dir.join(format!("{}.md", payload.name));
    if path.exists() {
        return ApiResponse::error(format!("Output style '{}' already exists", payload.name));
    }

    match fs::write(&path, &payload.content) {
        Ok(_) => ApiResponse::success("Output style created successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn update_output_style(
    Path(name): Path<String>,
    Json(payload): Json<UpdateOutputStyleRequest>,
) -> impl IntoResponse {
    let dir = match get_output_styles_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let path = dir.join(format!("{}.md", name));
    if !path.exists() {
        return ApiResponse::error(format!("Output style '{}' not found", name));
    }

    match fs::write(&path, &payload.content) {
        Ok(_) => ApiResponse::success("Output style updated successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

pub async fn delete_output_style(Path(name): Path<String>) -> impl IntoResponse {
    let dir = match get_output_styles_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::error(e.to_string()),
    };

    let path = dir.join(format!("{}.md", name));
    if !path.exists() {
        return ApiResponse::error(format!("Output style '{}' not found", name));
    }

    match fs::remove_file(&path) {
        Ok(_) => ApiResponse::success("Output style deleted successfully".to_string()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}
