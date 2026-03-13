// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use pca_core::prelude::*;
use serde::{Deserialize, Serialize};

/// Compression parameters for GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiCompressionParams {
    pub quality: f32,
    pub mode: String,
    pub retain_components: usize,
    pub orientation: String,
    pub tile_size: Option<u32>,
    pub max_memory_mb: Option<usize>,
    pub output_format: String,
    pub strip_metadata: bool,
}

impl Default for GuiCompressionParams {
    fn default() -> Self {
        Self {
            quality: 0.7,
            mode: "per-channel".to_string(),
            retain_components: 1,
            orientation: "auto".to_string(),
            tile_size: Some(1024),
            max_memory_mb: Some(1024),
            output_format: "jpeg".to_string(),
            strip_metadata: false,
        }
    }
}

impl TryFrom<GuiCompressionParams> for CompressionParams {
    type Error = String;

    fn try_from(gui: GuiCompressionParams) -> Result<Self, Self::Error> {
        let mode = PcaMode::from_str(&gui.mode)
            .map_err(|e| format!("Invalid mode: {}", e))?;

        let orientation = OrientationMode::from_str(&gui.orientation)
            .map_err(|e| format!("Invalid orientation: {}", e))?;

        let output_format = OutputFormat::from_str(&gui.output_format)
            .map_err(|e| format!("Invalid output format: {}", e))?;

        Ok(CompressionParams {
            quality: gui.quality.clamp(0.1, 1.0),
            mode,
            retain_components: gui.retain_components,
            orientation,
            tile_size: gui.tile_size,
            max_memory_mb: gui.max_memory_mb,
            output_format,
            strip_metadata: gui.strip_metadata,
        })
    }
}

/// Compression result for GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiCompressionResult {
    pub success: bool,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
    pub ssim: f32,
    pub psnr: f32,
    pub processing_time_ms: u64,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

/// Compress an image
#[tauri::command]
async fn compress_image(
    input_path: String,
    output_path: String,
    params: GuiCompressionParams,
) -> Result<GuiCompressionResult, String> {
    // Convert params
    let params: CompressionParams = params.try_into()
        .map_err(|e| e.to_string())?;

    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_path);

    // Compress the image
    match pca_core::compression::compress_image(&input, &output, &params) {
        Ok(metrics) => {
            Ok(GuiCompressionResult {
                success: true,
                original_size: metrics.original_size,
                compressed_size: metrics.compressed_size,
                compression_ratio: metrics.compression_ratio,
                ssim: metrics.ssim,
                psnr: metrics.psnr,
                processing_time_ms: metrics.processing_time_ms,
                output_path: Some(output_path),
                error: None,
            })
        }
        Err(e) => {
            Ok(GuiCompressionResult {
                success: false,
                original_size: 0,
                compressed_size: 0,
                compression_ratio: 0.0,
                ssim: 0.0,
                psnr: 0.0,
                processing_time_ms: 0,
                output_path: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Get image information
#[tauri::command]
async fn get_image_info(path: String) -> Result<ImageInfo, String> {
    let image = pca_core::image::load_image(PathBuf::from(&path))
        .map_err(|e| e.to_string())?;

    let file_size = fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(ImageInfo {
        width: image.width,
        height: image.height,
        pixels: image.num_pixels(),
        has_alpha: image.has_alpha(),
        color_space: format!("{:?}", image.color_space),
        size_bytes: file_size,
        size_human: format_bytes(file_size),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub pixels: usize,
    pub has_alpha: bool,
    pub color_space: String,
    pub size_bytes: u64,
    pub size_human: String,
}

/// Get supported file extensions
#[tauri::command]
fn get_supported_extensions() -> Vec<&'static str> {
    pca_core::compression::supported_extensions()
}

/// Scan directory for images
#[tauri::command]
async fn scan_directory(path: String) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&path);
    let files = pca_core::compression::scan_directory(&dir);

    Ok(files.iter()
        .filter_map(|p| p.to_str())
        .map(|s| s.to_string())
        .collect())
}

/// Get app version
#[tauri::command]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_idx])
}

/// Handle deep link protocol
#[tauri::command]
fn handle_deep_link(url: String) -> Result<(), String> {
    println!("Deep link received: {}", url);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            compress_image,
            get_image_info,
            get_supported_extensions,
            scan_directory,
            get_version,
            handle_deep_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}