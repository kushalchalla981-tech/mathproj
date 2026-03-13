//! Quality metrics: SSIM and PSNR

use crate::error::{CompressionError, Result};
use crate::image::ImageData;

/// Compression metrics result
#[derive(Debug, Clone, Copy, Default)]
pub struct CompressionMetrics {
    /// Original file size in bytes
    pub original_size: usize,
    /// Compressed file size in bytes
    pub compressed_size: usize,
    /// Compression ratio
    pub compression_ratio: f32,
    /// SSIM score (0.0-1.0, higher is better)
    pub ssim: f32,
    /// PSNR in dB (higher is better)
    pub psnr: f32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

impl CompressionMetrics {
    /// Create new metrics
    pub fn new(original_size: usize, compressed_size: usize, ssim: f32, psnr: f32) -> Self {
        let ratio = if compressed_size > 0 {
            original_size as f32 / compressed_size as f32
        } else {
            1.0
        };

        Self {
            original_size,
            compressed_size,
            compression_ratio: ratio,
            ssim,
            psnr,
            processing_time_ms: 0,
        }
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Size: {} → {} (ratio: {:.2}x) | SSIM: {:.3} | PSNR: {:.1} dB",
            format_bytes(self.original_size),
            format_bytes(self.compressed_size),
            self.compression_ratio,
            self.ssim,
            self.psnr
        )
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[char] = &['B', 'K', 'M', 'G'];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_idx])
}

/// Calculate SSIM (Structural Similarity Index) between two images
pub fn calculate_ssim(original: &ImageData, compressed: &ImageData) -> Result<f32> {
    // Validate dimensions match
    if original.width != compressed.width || original.height != compressed.height {
        return Err(CompressionError::InvalidParams {
            field: "images".to_string(),
            value: format!(
                "Dimension mismatch: {}x{} vs {}x{}",
                original.width, original.height,
                compressed.width, compressed.height
            ),
        });
    }

    // SSIM constants
    const C1: f32 = 0.01 * 0.01; // (0.01*L)^2, L=1 for normalized data
    const C2: f32 = 0.03 * 0.03; // (0.03*L)^2

    let n_pixels = original.num_pixels();
    let window_size: usize = 11;
    let window_sigma: f32 = 1.5;

    // Calculate Gaussian window
    let gaussian_window = create_gaussian_window(window_size, window_sigma);

    // Process each channel separately and average
    let (r1, g1, b1) = original.split_channels();
    let (r2, g2, b2) = compressed.split_channels();

    let ssim_r = calculate_ssim_channel(&r1, &r2, n_pixels, original.width, original.height, &gaussian_window, C1, C2)?;
    let ssim_g = calculate_ssim_channel(&g1, &g2, n_pixels, original.width, original.height, &gaussian_window, C1, C2)?;
    let ssim_b = calculate_ssim_channel(&b1, &b2, n_pixels, original.width, original.height, &gaussian_window, C1, C2)?;

    // Average SSIM across channels
    let ssim = (ssim_r + ssim_g + ssim_b) / 3.0;
    Ok(ssim.clamp(0.0, 1.0))
}

/// Calculate SSIM for a single channel using sliding window
fn calculate_ssim_channel(
    img1: &[f32],
    img2: &[f32],
    _n_pixels: usize,
    width: u32,
    height: u32,
    gaussian_window: &[Vec<f32>],
    c1: f32,
    c2: f32,
) -> Result<f32> {
    let window_size = gaussian_window.len();
    let half_window = window_size / 2;
    let w = width as usize;
    let h = height as usize;

    let mut ssim_sum = 0.0;
    let mut window_count = 0;

    // Slide window across image
    for y in half_window..(h.saturating_sub(half_window)) {
        for x in half_window..(w.saturating_sub(half_window)) {
            // Extract window
            let mut mu1 = 0.0;
            let mut mu2 = 0.0;
            let mut sigma1_sq = 0.0;
            let mut sigma2_sq = 0.0;
            let mut sigma12 = 0.0;

            for wy in 0..window_size {
                for wx in 0..window_size {
                    let px = x + wx - half_window;
                    let py = y + wy - half_window;
                    let idx = py * w + px;

                    let v1 = img1[idx];
                    let v2 = img2[idx];
                    let weight = gaussian_window[wy][wx];

                    mu1 += v1 * weight;
                    mu2 += v2 * weight;
                }
            }

            // Calculate variances and covariance
            for wy in 0..window_size {
                for wx in 0..window_size {
                    let px = x + wx - half_window;
                    let py = y + wy - half_window;
                    let idx = py * w + px;

                    let v1 = img1[idx];
                    let v2 = img2[idx];
                    let weight = gaussian_window[wy][wx];

                    let diff1 = v1 - mu1;
                    let diff2 = v2 - mu2;

                    sigma1_sq += diff1 * diff1 * weight;
                    sigma2_sq += diff2 * diff2 * weight;
                    sigma12 += diff1 * diff2 * weight;
                }
            }

            // SSIM formula
            let numerator = (2.0 * mu1 * mu2 + c1) * (2.0 * sigma12 + c2);
            let denominator = (mu1 * mu1 + mu2 * mu2 + c1) * (sigma1_sq + sigma2_sq + c2);
            let ssim = numerator / denominator;

            ssim_sum += ssim;
            window_count += 1;
        }
    }

    if window_count == 0 {
        return Ok(1.0); // Image too small for windowing, assume perfect
    }

    Ok(ssim_sum / window_count as f32)
}

/// Create 2D Gaussian window
fn create_gaussian_window(size: usize, sigma: f32) -> Vec<Vec<f32>> {
    let mut window = vec![vec![0.0f32; size]; size];
    let center = (size / 2) as f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    let mut sum = 0.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let value = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
            window[y][x] = value;
            sum += value;
        }
    }

    // Normalize
    for y in 0..size {
        for x in 0..size {
            window[y][x] /= sum;
        }
    }

    window
}

/// Calculate PSNR (Peak Signal-to-Noise Ratio)
pub fn calculate_psnr(original: &ImageData, compressed: &ImageData) -> Result<f32> {
    // Validate dimensions match
    if original.width != compressed.width || original.height != compressed.height {
        return Err(CompressionError::InvalidParams {
            field: "images".to_string(),
            value: format!(
                "Dimension mismatch: {}x{} vs {}x{}",
                original.width, original.height,
                compressed.width, compressed.height
            ),
        });
    }

    let (r1, g1, b1) = original.split_channels();
    let (r2, g2, b2) = compressed.split_channels();

    // Calculate MSE for each channel
    let mse_r = calculate_mse(&r1, &r2);
    let mse_g = calculate_mse(&g1, &g2);
    let mse_b = calculate_mse(&b1, &b2);

    // Average MSE
    let mse = (mse_r + mse_g + mse_b) / 3.0;

    if mse == 0.0 {
        return Ok(f32::INFINITY); // Perfect match
    }

    // PSNR = 10 * log10((MAX^2) / MSE)
    // For normalized [0,1] data, MAX = 1
    let max_val = 1.0f32;
    let psnr = 10.0 * ((max_val * max_val) / mse).log10();

    Ok(psnr)
}

/// Calculate Mean Squared Error
fn calculate_mse(img1: &[f32], img2: &[f32]) -> f32 {
    let n = img1.len().min(img2.len());
    if n == 0 {
        return 0.0;
    }

    let sum_sq_diff: f32 = img1.iter()
        .zip(img2.iter())
        .map(|(a, b)| {
            let diff = a - b;
            diff * diff
        })
        .sum();

    sum_sq_diff / n as f32
}

/// Calculate compression ratio from file sizes
pub fn calculate_compression_ratio(original_bytes: usize, compressed_bytes: usize) -> f32 {
    if compressed_bytes == 0 {
        return 1.0;
    }
    original_bytes as f32 / compressed_bytes as f32
}

/// Estimate file size from quality parameter (rough approximation)
pub fn estimate_compressed_size(original_size: usize, quality: f32, ssim: f32) -> usize {
    // Quality factor: 0.1 = high compression, 1.0 = low compression
    let compression_factor = (1.1 - quality).clamp(0.1, 1.0);
    let ssim_penalty = (1.0 - ssim).max(0.0) * 0.5; // Penalty for low quality

    let estimated = (original_size as f32 * compression_factor * (1.0 + ssim_penalty)) as usize;
    estimated.max(1) // At least 1 byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse() {
        let a = vec![0.5f32, 0.5, 0.5];
        let b = vec![0.6f32, 0.6, 0.6];
        let mse = calculate_mse(&a, &b);
        assert_approx_eq::assert_approx_eq!(mse, 0.01, 1e-6);
    }

    #[test]
    fn test_mse_identical() {
        let a = vec![0.5f32, 0.3, 0.7];
        let mse = calculate_mse(&a, &a);
        assert_eq!(mse, 0.0);
    }

    #[test]
    fn test_gaussian_window() {
        let window = create_gaussian_window(11, 1.5);
        assert_eq!(window.len(), 11);
        assert_eq!(window[0].len(), 11);

        // Check normalization
        let sum: f32 = window.iter().flat_map(|row| row.iter()).sum();
        assert_approx_eq::assert_approx_eq!(sum, 1.0, 1e-5);

        // Center should be highest
        assert!(window[5][5] > window[0][0]);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.0B");
        assert_eq!(format_bytes(1024), "1.0K");
        assert_eq!(format_bytes(1024 * 1024), "1.0M");
    }

    #[test]
    fn test_compression_ratio() {
        assert_eq!(calculate_compression_ratio(1000, 500), 2.0);
        assert_eq!(calculate_compression_ratio(1000, 1000), 1.0);
        assert_eq!(calculate_compression_ratio(1000, 0), 1.0);
    }
}
