//! Tile-based processing for large images

use crate::error::{CompressionError, Result};
use crate::image::ImageData;
use rayon::prelude::*;

/// Tile information
#[derive(Debug, Clone)]
pub struct Tile {
    /// Tile index in x direction
    pub tile_x: u32,
    /// Tile index in y direction
    pub tile_y: u32,
    /// Position in original image
    pub x: u32,
    pub y: u32,
    /// Tile dimensions (may be smaller at edges)
    pub width: u32,
    pub height: u32,
    /// Actual pixel data
    pub data: ImageData,
}

/// Split image into tiles
pub fn split_into_tiles(image: &ImageData, tile_size: u32) -> Vec<Tile> {
    let mut tiles = Vec::new();

    let tiles_x = (image.width + tile_size - 1) / tile_size;
    let tiles_y = (image.height + tile_size - 1) / tile_size;

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let x = tx * tile_size;
            let y = ty * tile_size;
            let width = (tile_size).min(image.width - x);
            let height = (tile_size).min(image.height - y);

            let tile_data = extract_tile(image, x, y, width, height);

            tiles.push(Tile {
                tile_x: tx,
                tile_y: ty,
                x,
                y,
                width,
                height,
                data: tile_data,
            });
        }
    }

    tiles
}

/// Extract a tile from an image
fn extract_tile(image: &ImageData, x: u32, y: u32, width: u32, height: u32) -> ImageData {
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    let mut alpha: Option<Vec<f32>> = image.alpha_data.as_ref()
        .map(|_| Vec::with_capacity((width * height) as usize));

    for row in y..(y + height) {
        for col in x..(x + width) {
            let src_idx = ((row * image.width + col) * 3) as usize;
            rgb.push(image.rgb_data[src_idx]);
            rgb.push(image.rgb_data[src_idx + 1]);
            rgb.push(image.rgb_data[src_idx + 2]);

            if let (Some(ref src_alpha), Some(ref mut dst_alpha)) = (&image.alpha_data, &mut alpha
            ) {
                let src_a_idx = (row * image.width + col) as usize;
                dst_alpha.push(src_alpha[src_a_idx]);
            }
        }
    }

    let mut tile = if let Some(a) = alpha {
        ImageData::with_alpha(width, height, rgb, a).unwrap()
    } else {
        ImageData::new(width, height, rgb).unwrap()
    };

    tile.color_space = image.color_space;
    tile
}

/// Stitch tiles back together into a single image
pub fn stitch_tiles(
    tiles: &[Tile],
    full_width: u32,
    full_height: u32,
    has_alpha: bool,
) -> Result<ImageData> {
    if tiles.is_empty() {
        return Err(CompressionError::InvalidParams {
            field: "tiles".to_string(),
            value: "empty tile list".to_string(),
        });
    }

    let mut full_rgb = vec![0.0f32; (full_width * full_height * 3) as usize];
    let mut full_alpha = if has_alpha {
        Some(vec![0.0f32; (full_width * full_height) as usize])
    } else {
        None
    };

    for tile in tiles {
        // Copy tile data to full image
        for row in 0..tile.height {
            for col in 0..tile.width {
                let src_x = col;
                let src_y = row;
                let dst_x = tile.x + col;
                let dst_y = tile.y + row;

                let src_idx = ((src_y * tile.width + src_x) * 3) as usize;
                let dst_idx = ((dst_y * full_width + dst_x) * 3) as usize;

                full_rgb[dst_idx] = tile.data.rgb_data[src_idx];
                full_rgb[dst_idx + 1] = tile.data.rgb_data[src_idx + 1];
                full_rgb[dst_idx + 2] = tile.data.rgb_data[src_idx + 2];

                if let (Some(ref tile_alpha), Some(ref mut full_a)) = (
                    &tile.data.alpha_data, &mut full_alpha
                ) {
                    let src_a_idx = (src_y * tile.width + src_x) as usize;
                    let dst_a_idx = (dst_y * full_width + dst_x) as usize;
                    full_a[dst_a_idx] = tile_alpha[src_a_idx];
                }
            }
        }
    }

    let mut result = if let Some(alpha) = full_alpha {
        ImageData::with_alpha(full_width, full_height, full_rgb, alpha)?
    } else {
        ImageData::new(full_width, full_height, full_rgb)?
    };

    // Propagate metadata from first tile
    result.color_space = tiles[0].data.color_space;

    Ok(result)
}

/// Process tiles in parallel
pub fn process_tiles_parallel<F, R>(
    tiles: Vec<Tile>,
    processor: F,
) -> Vec<R>
where
    F: Fn(Tile) -> R + Sync + Send,
    R: Send,
{
    tiles.into_par_iter().map(processor).collect()
}

/// Check if tile processing is needed for an image
pub fn needs_tiling(image: &ImageData, max_memory_mb: Option<usize>) -> bool {
    if let Some(max_mem) = max_memory_mb {
        let estimated_mb = (image.size_bytes() as f64 / (1024.0 * 1024.0)) * 3.0; // Working buffer multiplier
        estimated_mb > max_mem as f64
    } else {
        false
    }
}

/// Calculate optimal tile size
pub fn calculate_optimal_tile_size(
    image: &ImageData,
    max_memory_mb: usize,
) -> u32 {
    let bytes_per_pixel: u64 = if image.has_alpha() { 16 } else { 12 }; // f32 * channels
    let max_pixels = (max_memory_mb as u64 * 1024 * 1024) / bytes_per_pixel;
    let tile_pixels = (max_pixels as f64).sqrt() as u32;

    // Round to multiple of 8 for alignment
    let tile_size = ((tile_pixels / 8) * 8).max(64).min(2048);

    tile_size
}

/// Seam-aware blending between tiles (for smooth transitions)
pub fn blend_tile_edges(
    image: &mut ImageData,
    tiles: &[Tile],
    blend_width: u32,
) {
    if blend_width == 0 || tiles.len() <= 1 {
        return;
    }

    // Find tile boundaries
    let mut vertical_boundaries: Vec<u32> = tiles.iter().map(|t| t.x + t.width).collect();
    let mut horizontal_boundaries: Vec<u32> = tiles.iter().map(|t| t.y + t.height).collect();

    vertical_boundaries.sort_unstable();
    vertical_boundaries.dedup();
    horizontal_boundaries.sort_unstable();
    horizontal_boundaries.dedup();

    // Blend vertical boundaries
    for &boundary in &vertical_boundaries {
        if boundary >= image.width || boundary < blend_width {
            continue;
        }

        let start_x = boundary.saturating_sub(blend_width);
        let end_x = boundary.min(image.width);

        for y in 0..image.height {
            for x in start_x..end_x {
                let dist_from_boundary = if x < boundary {
                    boundary - x
                } else {
                    x - boundary + blend_width
                };
                let alpha = (dist_from_boundary as f32) / (blend_width as f32);
                let alpha = alpha.clamp(0.0, 1.0);

                let idx = ((y * image.width + x) * 3) as usize;

                // Apply slight blur at boundary
                image.rgb_data[idx] *= 0.95 + alpha * 0.05;
                image.rgb_data[idx + 1] *= 0.95 + alpha * 0.05;
                image.rgb_data[idx + 2] *= 0.95 + alpha * 0.05;
            }
        }
    }

    // Blend horizontal boundaries
    for &boundary in &horizontal_boundaries {
        if boundary >= image.height || boundary < blend_width {
            continue;
        }

        let start_y = boundary.saturating_sub(blend_width);
        let end_y = boundary.min(image.height);

        for y in start_y..end_y {
            for x in 0..image.width {
                let dist_from_boundary = if y < boundary {
                    boundary - y
                } else {
                    y - boundary + blend_width
                };
                let alpha = (dist_from_boundary as f32) / (blend_width as f32);
                let alpha = alpha.clamp(0.0, 1.0);

                let idx = ((y * image.width + x) * 3) as usize;

                image.rgb_data[idx] *= 0.95 + alpha * 0.05;
                image.rgb_data[idx + 1] *= 0.95 + alpha * 0.05;
                image.rgb_data[idx + 2] *= 0.95 + alpha * 0.05;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data: Vec<f32> = (0..width * height * 3)
            .map(|i| ((i % 256) as f32 / 255.0))
            .collect();
        ImageData::new(width, height, data).unwrap()
    }

    #[test]
    fn test_split_and_stitch() {
        let img = create_test_image(200, 200);
        let tiles = split_into_tiles(&img, 64);

        // Should create multiple tiles
        assert!(tiles.len() > 1);

        // Stitch back together
        let stitched = stitch_tiles(&tiles, img.width, img.height, false).unwrap();
        assert_eq!(stitched.width, img.width);
        assert_eq!(stitched.height, img.height);
    }

    #[test]
    fn test_tile_dimensions() {
        // Non-evenly divisible dimensions
        let img = create_test_image(150, 150);
        let tiles = split_into_tiles(&img, 64);

        // Last row/column tiles should be smaller
        let last_tile = tiles.last().unwrap();
        assert!(last_tile.width <= 64);
        assert!(last_tile.height <= 64);
    }

    #[test]
    fn test_calculate_tile_size() {
        let img = create_test_image(1000, 1000);
        let size = calculate_optimal_tile_size(&img, 100);
        assert!(size >= 64);
        assert!(size <= 2048);
    }

    #[test]
    fn test_needs_tiling() {
        let img = create_test_image(100, 100);
        assert!(!needs_tiling(&img, Some(1024)));

        let large_img = create_test_image(5000, 5000);
        assert!(needs_tiling(&large_img, Some(100)));
    }

    #[test]
    fn test_parallel_processing() {
        let img = create_test_image(200, 200);
        let tiles = split_into_tiles(&img, 64);

        let results: Vec<usize> = process_tiles_parallel(tiles, |tile| {
            tile.data.num_pixels()
        });

        assert_eq!(results.len(), 16); // Should have 16 tiles
    }
}
