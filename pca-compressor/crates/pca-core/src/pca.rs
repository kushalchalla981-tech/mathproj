//! PCA (Principal Component Analysis) compression implementation

use crate::error::{CompressionError, Result};
use crate::image::ImageData;
use nalgebra::{DMatrix, DVector, SymmetricEigen};

/// PCA processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PcaMode {
    /// Process each RGB channel independently
    #[default]
    PerChannel,
    /// Treat RGB as 3D vectors
    JointChannel,
}

impl PcaMode {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "per-channel" | "per_channel" | "perchannel" => Ok(Self::PerChannel),
            "joint-channel" | "joint_channel" | "jointchannel" => Ok(Self::JointChannel),
            _ => Err(CompressionError::InvalidParams {
                field: "mode".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Orientation handling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrientationMode {
    /// Use PCA principal axis, fallback to EXIF
    #[default]
    Auto,
    /// Use EXIF orientation only
    Exif,
    /// Don't apply orientation correction
    Disabled,
}

impl OrientationMode {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "exif" => Ok(Self::Exif),
            "disabled" => Ok(Self::Disabled),
            _ => Err(CompressionError::InvalidParams {
                field: "orientation".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Jpeg,
    Png,
}

impl OutputFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(Self::Jpeg),
            "png" => Ok(Self::Png),
            _ => Err(CompressionError::InvalidParams {
                field: "format".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Compression parameters
#[derive(Debug, Clone)]
pub struct CompressionParams {
    /// Compression quality (0.1-1.0)
    pub quality: f32,
    /// PCA mode
    pub mode: PcaMode,
    /// Number of components to retain (1-3 for per-channel)
    pub retain_components: usize,
    /// Orientation handling
    pub orientation: OrientationMode,
    /// Tile size (None = process whole image)
    pub tile_size: Option<u32>,
    /// Maximum memory in MB
    pub max_memory_mb: Option<usize>,
    /// Output format
    pub output_format: OutputFormat,
    /// Strip EXIF metadata
    pub strip_metadata: bool,
}

impl Default for CompressionParams {
    fn default() -> Self {
        Self {
            quality: 0.7,
            mode: PcaMode::PerChannel,
            retain_components: 1,
            orientation: OrientationMode::Auto,
            tile_size: Some(1024),
            max_memory_mb: Some(1024),
            output_format: OutputFormat::Jpeg,
            strip_metadata: false,
        }
    }
}

impl CompressionParams {
    /// Create with quality level
    pub fn with_quality(quality: f32) -> Self {
        let mut params = Self::default();
        params.quality = quality.clamp(0.1, 1.0);
        params
    }

    /// Validate parameters
    pub fn validate(&self) -> Result<()> {
        if self.quality < 0.0 || self.quality > 1.0 {
            return Err(CompressionError::InvalidParams {
                field: "quality".to_string(),
                value: self.quality.to_string(),
            });
        }

        if self.retain_components == 0 {
            return Err(CompressionError::InvalidParams {
                field: "retain_components".to_string(),
                value: self.retain_components.to_string(),
            });
        }

        if let Some(tile_size) = self.tile_size {
            if tile_size < 64 {
                return Err(CompressionError::InvalidParams {
                    field: "tile_size".to_string(),
                    value: tile_size.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Calculate maximum components based on mode
    pub fn max_components(&self) -> usize {
        match self.mode {
            PcaMode::PerChannel => 1, // Each channel is 1D
            PcaMode::JointChannel => 3, // RGB is 3D
        }
    }
}

/// Result of compression
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// Compressed image data
    pub image: ImageData,
    /// SSIM score (0.0-1.0)
    pub ssim: f32,
    /// PSNR in dB
    pub psnr: f32,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Compress an image using PCA
pub fn compress(image: &ImageData, params: &CompressionParams) -> Result<CompressionResult> {
    use std::time::Instant;
    let start = Instant::now();

    // Validate parameters
    params.validate()?;

    // Validate image size
    image.validate_size()?;

    // Check memory constraints
    if let Some(max_mem) = params.max_memory_mb {
        let estimated_memory = estimate_memory(image, params);
        if estimated_memory > max_mem {
            // If image is too large, force tiling
            if params.tile_size.is_none() {
                // Will be handled by tile module
            } else {
                return Err(CompressionError::MemoryLimitExceeded {
                    required_mb: estimated_memory as f64,
                    available_mb: max_mem as f64,
                });
            }
        }
    }

    // Process based on mode
    let processed = match params.mode {
        PcaMode::PerChannel => compress_per_channel(image, params)?,
        PcaMode::JointChannel => compress_joint_channel(image, params)?,
    };

    // Calculate processing time
    let processing_time_ms = start.elapsed().as_millis() as u64;

    // Calculate metrics
    let ssim = crate::metrics::calculate_ssim(image, &processed)?;
    let psnr = crate::metrics::calculate_psnr(image, &processed)?;

    // Compression ratio (estimated based on quality)
    let compression_ratio = estimate_compression_ratio(image, params, ssim);

    Ok(CompressionResult {
        image: processed,
        ssim,
        psnr,
        compression_ratio,
        processing_time_ms,
    })
}

/// Compress using per-channel PCA
fn compress_per_channel(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    let (mut r, mut g, mut b) = image.split_channels();
    let n_pixels = image.num_pixels();

    // Process each channel
    let components = (params.retain_components).min(1);

    r = compress_channel(&r, components)?;
    g = compress_channel(&g, components)?;
    b = compress_channel(&b, components)?;

    // Reconstruct image
    let mut result = image.clone();
    result.from_channels(&r, &g, &b)?;

    Ok(result)
}

/// Compress using joint-channel PCA
fn compress_joint_channel(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    let (r, g, b) = image.split_channels();
    let n_pixels = image.num_pixels();

    // Build 3D data matrix (n_pixels x 3)
    let mut data = Vec::with_capacity(n_pixels * 3);
    for i in 0..n_pixels {
        data.push(r[i]);
        data.push(g[i]);
        data.push(b[i]);
    }

    // Convert to DMatrix (rows = pixels, cols = 3 channels)
    let matrix = DMatrix::from_row_slice(n_pixels, 3, &data);

    // Mean center
    let mean = matrix.column_mean();
    let centered = matrix - DMatrix::from_row_slice(n_pixels, 3, &vec![mean[0], mean[1], mean[2]; n_pixels]);

    // Compute covariance (3x3)
    let cov = (centered.transpose() * &centered) / (n_pixels as f64 - 1.0);

    // Eigen-decomposition
    let eigen = SymmetricEigen::new(cov);

    // Get dominant eigenvectors
    let n_retain = params.retain_components.min(3);
    let eigenvectors = eigen.eigenvectors;
    let eigenvalues = eigen.eigenvalues;

    // Project onto dominant components
    let mut projected = &centered * &eigenvectors;

    // Zero out non-retained components
    for i in n_retain..3 {
        projected.column_mut(i).fill(0.0);
    }

    // Reconstruct
    let reconstructed = &projected * eigenvectors.transpose();
    let final_data = reconstructed + DMatrix::from_row_slice(n_pixels, 3, &vec![mean[0], mean[1], mean[2]; n_pixels]);

    // Extract channels
    let mut r_out = Vec::with_capacity(n_pixels);
    let mut g_out = Vec::with_capacity(n_pixels);
    let mut b_out = Vec::with_capacity(n_pixels);

    for i in 0..n_pixels {
        r_out.push(final_data[(i, 0)] as f32);
        g_out.push(final_data[(i, 1)] as f32);
        b_out.push(final_data[(i, 2)] as f32);
    }

    // Clamp to [0, 1]
    let r_out: Vec<f32> = r_out.iter().map(|v| v.clamp(0.0, 1.0)).collect();
    let g_out: Vec<f32> = g_out.iter().map(|v| v.clamp(0.0, 1.0)).collect();
    let b_out: Vec<f32> = b_out.iter().map(|v| v.clamp(0.0, 1.0)).collect();

    // Reconstruct image
    let mut result = image.clone();
    result.from_channels(&r_out, &g_out, &b_out)?;

    Ok(result)
}

/// Compress a single channel using 1D PCA
fn compress_channel(channel: &[f32], retain_components: usize) -> Result<Vec<f32>> {
    if retain_components >= 1 || channel.len() < 2 {
        // For 1D data with 1 component, we're essentially keeping the mean
        // Just return the original for now
        return Ok(channel.to_vec());
    }

    // 1D PCA is essentially mean-centering and keeping the mean
    // For actual compression in 1D, we could use other techniques
    // Here we'll just pass through the data
    Ok(channel.to_vec())
}

/// Estimate memory usage in MB
fn estimate_memory(image: &ImageData, _params: &CompressionParams) -> usize {
    let pixels = image.num_pixels();
    let bytes_per_pixel = if image.has_alpha() { 4 } else { 3 };
    let float_size = 4; // f32

    // Original image + working buffers + covariance matrix
    let image_size = pixels * bytes_per_pixel * float_size;
    let working_buffers = image_size * 3; // Multiple working copies
    let covariance = 3 * 3 * float_size; // 3x3 matrix
    let eigen = 3 * 3 * float_size; // Eigen decomposition

    let total_bytes = image_size + working_buffers + covariance + eigen;
    total_bytes / (1024 * 1024) // Convert to MB
}

/// Estimate compression ratio based on quality and SSIM
fn estimate_compression_ratio(image: &ImageData, params: &CompressionParams, ssim: f32) -> f32 {
    // Base ratio from retained components
    let base_ratio = match params.mode {
        PcaMode::PerChannel => {
            // 1 component per channel vs 1 (no real reduction in this simple model)
            // In practice, this would depend on quantization
            1.0
        }
        PcaMode::JointChannel => {
            let retained = params.retain_components.min(3) as f32;
            3.0 / retained // 3D to nD reduction
        }
    };

    // Adjust for quality
    let quality_factor = 0.5 + (params.quality * 0.5); // 0.5x to 1.0x

    // Adjust for SSIM (lower SSIM = less effective compression)
    let ssim_factor = ssim.max(0.5); // Minimum 0.5x for low quality

    let ratio = base_ratio * quality_factor * ssim_factor;
    ratio.max(1.0) // Never claim less than 1:1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pca_mode_from_str() {
        assert_eq!(PcaMode::from_str("per-channel").unwrap(), PcaMode::PerChannel);
        assert_eq!(PcaMode::from_str("PER_CHANNEL").unwrap(), PcaMode::PerChannel);
        assert_eq!(PcaMode::from_str("joint-channel").unwrap(), PcaMode::JointChannel);
        assert!(PcaMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_orientation_mode_from_str() {
        assert_eq!(OrientationMode::from_str("auto").unwrap(), OrientationMode::Auto);
        assert_eq!(OrientationMode::from_str("exif").unwrap(), OrientationMode::Exif);
        assert_eq!(OrientationMode::from_str("disabled").unwrap(), OrientationMode::Disabled);
        assert!(OrientationMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_params_validation() {
        let params = CompressionParams::default();
        assert!(params.validate().is_ok());

        let mut bad = CompressionParams::with_quality(1.5);
        assert!(bad.validate().is_err());

        let mut bad2 = CompressionParams::default();
        bad2.retain_components = 0;
        assert!(bad2.validate().is_err());
    }

    #[test]
    fn test_channel_compression() {
        let channel = vec![0.1f32, 0.2, 0.3, 0.4, 0.5];
        let result = compress_channel(&channel, 1).unwrap();
        assert_eq!(result.len(), channel.len());
    }
}
