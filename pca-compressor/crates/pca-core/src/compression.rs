//! High-level compression interface

use crate::error::{CompressionError, Result};
use crate::image::{ImageData, load_image, save_image};
use crate::metrics::{CompressionMetrics, calculate_ssim, calculate_psnr, calculate_compression_ratio};
use crate::orientation::{correct_orientation, OrientationMethod};
use crate::pca::{compress, CompressionParams, CompressionResult, PcaMode};
use crate::tile::{split_into_tiles, stitch_tiles, process_tiles_parallel, needs_tiling, calculate_optimal_tile_size};
use rayon::prelude::*;
use std::path::Path;
use std::time::Instant;

/// Complete compression pipeline for a single image
///
/// This is the main entry point for compression that:
/// 1. Loads the image
/// 2. Applies orientation correction
/// 3. Splits into tiles if needed
/// 4. Compresses each tile
/// 5. Stitches tiles back together
/// 6. Calculates metrics
/// 7. Saves the result
pub fn compress_image(
    input_path: &Path,
    output_path: &Path,
    params: &CompressionParams,
) -> Result<CompressionMetrics> {
    let start = Instant::now();

    // Load image
    let mut image = load_image(input_path)?;

    // Get original size
    let original_size = std::fs::metadata(input_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // Apply orientation correction
    let (image, _orientation_method) = if params.orientation != crate::pca::OrientationMode::Disabled {
        correct_orientation(&image, params.orientation)?
    } else {
        (image, OrientationMethod::None)
    };

    // Process based on tile settings
    let compressed_image = if should_use_tiling(&image, params) {
        compress_with_tiling(&image, params)?
    } else {
        compress_whole_image(&image, params)?
    };

    // Determine output quality based on params
    let output_quality = (params.quality * 100.0) as u8;

    // Save output
    save_image(output_path, &compressed_image, output_quality)?;

    // Get compressed size
    let compressed_size = std::fs::metadata(output_path)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // Calculate metrics
    let ssim = calculate_ssim(&image, &compressed_image)?;
    let psnr = calculate_psnr(&image, &compressed_image)?;
    let ratio = calculate_compression_ratio(original_size, compressed_size.max(1));

    let processing_time_ms = start.elapsed().as_millis() as u64;

    Ok(CompressionMetrics {
        original_size,
        compressed_size,
        compression_ratio: ratio,
        ssim,
        psnr,
        processing_time_ms,
    })
}

/// Check if tiling should be used
fn should_use_tiling(image: &ImageData, params: &CompressionParams) -> bool {
    if let Some(tile_size) = params.tile_size {
        if needs_tiling(image, params.max_memory_mb) {
            return true;
        }
        // Also tile if image is larger than 2x tile size
        if image.width > tile_size * 2 || image.height > tile_size * 2 {
            return true;
        }
    }
    false
}

/// Compress the entire image without tiling
fn compress_whole_image(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    let result = crate::pca::compress(image, params)?;
    Ok(result.image)
}

/// Compress using tile processing
fn compress_with_tiling(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    // Determine actual tile size
    let tile_size = if let Some(size) = params.tile_size {
        size
    } else {
        if let Some(max_mem) = params.max_memory_mb {
            calculate_optimal_tile_size(image, max_mem)
        } else {
            1024 // Default
        }
    };

    // Split into tiles
    let tiles = split_into_tiles(image, tile_size);

    // Process tiles in parallel
    let processed_tiles: Vec<_> = process_tiles_parallel(tiles, |tile| {
        match crate::pca::compress(&tile.data, params) {
            Ok(result) => {
                crate::tile::Tile {
                    data: result.image,
                    ..tile
                }
            }
            Err(_) => {
                // On error, return original tile data
                tile
            }
        }
    });

    // Stitch tiles back together
    let mut stitched = stitch_tiles(
        &processed_tiles,
        image.width,
        image.height,
        image.has_alpha(),
    )?;

    // Apply seam blending
    crate::tile::blend_tile_edges(&mut stitched, &processed_tiles, tile_size / 32);

    Ok(stitched)
}

/// Batch compression result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub input_path: String,
    pub output_path: String,
    pub success: bool,
    pub metrics: Option<CompressionMetrics>,
    pub error: Option<String>,
}

impl BatchResult {
    /// Create CSV header
    pub fn csv_header() -> String {
        "filename,original_size,compressed_size,compression_ratio,ssim,psnr,processing_time_ms,success,error".to_string()
    }

    /// Convert to CSV row
    pub fn to_csv(&self) -> String {
        if let Some(ref m) = self.metrics {
            format!(
                "{},{},{},{:.2},{:.3},{:.1},{},{},{}",
                self.input_path,
                m.original_size,
                m.compressed_size,
                m.compression_ratio,
                m.ssim,
                m.psnr,
                m.processing_time_ms,
                self.success,
                self.error.as_ref().unwrap_or(&"".to_string())
            )
        } else {
            format!(
                "{},{},{},{},{},{},{},{},{}",
                self.input_path, 0, 0, 0.0, 0.0, 0.0, 0,
                self.success,
                self.error.as_ref().unwrap_or(&"unknown error".to_string())
            )
        }
    }
}

/// Compress multiple images in batch
pub fn compress_batch(
    input_paths: &[PathBuf],
    output_dir: &Path,
    params: &CompressionParams,
) -> Vec<BatchResult> {
    use rayon::prelude::*;
    use std::path::PathBuf;

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        return vec![BatchResult {
            input_path: "batch".to_string(),
            output_path: output_dir.to_string_lossy().to_string(),
            success: false,
            metrics: None,
            error: Some(format!("Failed to create output directory: {}", e)),
        }];
    }

    // Process images in parallel
    let results: Vec<BatchResult> = input_paths
        .par_iter()
        .map(|input_path| {
            let filename = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let extension = input_path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg");

            let output_filename = format!("{}_compressed.{}", filename, extension);
            let output_path = output_dir.join(&output_filename);

            match compress_image(input_path, &output_path, params) {
                Ok(metrics) => BatchResult {
                    input_path: input_path.to_string_lossy().to_string(),
                    output_path: output_path.to_string_lossy().to_string(),
                    success: true,
                    metrics: Some(metrics),
                    error: None,
                },
                Err(e) => BatchResult {
                    input_path: input_path.to_string_lossy().to_string(),
                    output_path: output_path.to_string_lossy().to_string(),
                    success: false,
                    metrics: None,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect();

    results
}

/// Write batch results to CSV file
pub fn write_batch_report(results: &[BatchResult], report_path: &Path) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(report_path)
        .map_err(|e| CompressionError::Io(e))?;

    // Write header
    writeln!(file, "{}", BatchResult::csv_header())
        .map_err(|e| CompressionError::Io(e))?;

    // Write rows
    for result in results {
        writeln!(file, "{}", result.to_csv())
            .map_err(|e| CompressionError::Io(e))?;
    }

    Ok(())
}

/// Analyze eigen decomposition of an image file
pub fn analyze_eigen_file(input_path: &Path) -> Result<crate::eigen_analysis::EigenAnalysisResult> {
    let image = load_image(input_path)?;
    crate::eigen_analysis::analyze_image(&image)
}

/// Get axis overlay data for an image file
pub fn get_axis_overlay_file(input_path: &Path) -> Result<crate::eigen_analysis::AxisOverlay> {
    let eigen_result = analyze_eigen_file(input_path)?;
    Ok(crate::eigen_analysis::get_axis_overlay(&eigen_result))
}

/// Get supported input extensions
pub fn supported_extensions() -> Vec<&'static str> {
    vec!["jpg", "jpeg", "png"]
}

/// Check if a file has supported extension
pub fn is_supported_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_ascii_lowercase();
        let ext_str = ext.to_str().unwrap_or("");
        supported_extensions().contains(&ext_str)
    } else {
        false
    }
}

/// Scan directory for supported image files
pub fn scan_directory(dir: &Path) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_supported_file(&path) {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_supported_extensions() {
        let exts = supported_extensions();
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"png"));
    }

    #[test]
    fn test_is_supported_file() {
        assert!(is_supported_file(Path::new("test.jpg")));
        assert!(is_supported_file(Path::new("test.png")));
        assert!(!is_supported_file(Path::new("test.gif")));
        assert!(!is_supported_file(Path::new("test")));
    }

    #[test]
    fn test_batch_result_csv() {
        let result = BatchResult {
            input_path: "input.jpg".to_string(),
            output_path: "output.jpg".to_string(),
            success: true,
            metrics: Some(CompressionMetrics {
                original_size: 1000,
                compressed_size: 500,
                compression_ratio: 2.0,
                ssim: 0.95,
                psnr: 40.0,
                processing_time_ms: 100,
            }),
            error: None,
        };

        let csv = result.to_csv();
        assert!(csv.contains("input.jpg"));
        assert!(csv.contains("2.00"));
        assert!(csv.contains("0.950"));
    }

    #[test]
    fn test_batch_result_csv_header() {
        assert!(BatchResult::csv_header().contains("filename"));
        assert!(BatchResult::csv_header().contains("ssim"));
    }
}
