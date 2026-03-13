//! Error types for the PCA compression library

use thiserror::Error;

/// Result type alias with CompressionError
pub type Result<T> = std::result::Result<T, CompressionError>;

/// Errors that can occur during image compression
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image format error: {0}")]
    ImageFormat(String),

    #[error("Invalid image format at {path}: {reason}")]
    InvalidFormat { path: String, reason: String },

    #[error("Corrupted image at {path}: {reason}")]
    CorruptedImage { path: String, reason: String },

    #[error("Image too small: {width}x{height} (minimum 64x64 required)")]
    ImageTooSmall { width: u32, height: u32 },

    #[error("Image too large: {width}x{height} (use tile processing)")]
    ImageTooLarge { width: u32, height: u32 },

    #[error("Memory limit exceeded: required {required_mb:.1}MB, available {available_mb:.1}MB")]
    MemoryLimitExceeded { required_mb: f64, available_mb: f64 },

    #[error("Unsupported color space: {0}")]
    UnsupportedColorSpace(String),

    #[error("Invalid parameter '{field}': {value}")]
    InvalidParams { field: String, value: String },

    #[error("Orientation conflict: PCA suggests {pca_degrees}°, EXIF is {exif}")]
    OrientationConflict { pca_degrees: f32, exif: u8 },

    #[error("PCA computation failed: {0}")]
    PcaComputationFailed(String),

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Decoding failed: {0}")]
    DecodingFailed(String),

    #[error("EXIF read failed: {0}")]
    ExifReadFailed(String),

    #[error("Batch processing failed: {0} succeeded, {1} failed")]
    BatchPartialFailure { succeeded: usize, failed: usize },

    #[error("Transparency not supported in {mode} mode")]
    TransparencyNotSupported { mode: String },

    #[error("Monochrome image detected - compression may have limited effect")]
    MonochromeImage,
}

impl CompressionError {
    /// Returns true if this error is related to memory limits
    pub fn is_memory_error(&self) -> bool {
        matches!(self, Self::MemoryLimitExceeded { .. } | Self::ImageTooLarge { .. })
    }

    /// Returns true if this error indicates the image should be tiled
    pub fn needs_tiling(&self) -> bool {
        matches!(self, Self::ImageTooLarge { .. } | Self::MemoryLimitExceeded { .. })
    }
}
