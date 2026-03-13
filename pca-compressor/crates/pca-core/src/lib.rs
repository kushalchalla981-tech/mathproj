//! PCA-based Image Compression Core Library
//!
//! This library provides PCA-based image compression with:
//! - Per-channel and joint-channel PCA modes
//! - Orientation correction
//! - Tile processing for large images
//! - Quality metrics (SSIM, PSNR)

pub mod error;
pub mod image;
pub mod pca;
pub mod metrics;
pub mod orientation;
pub mod tile;
pub mod compression;

pub use error::{CompressionError, Result};
pub use image::{ImageData, ColorSpace, load_image, save_image};
pub use pca::{PcaMode, CompressionParams, compress};
pub use metrics::CompressionMetrics;
pub use orientation::OrientationMethod;

pub mod prelude {
    pub use crate::{
        CompressionError, Result, ImageData, ColorSpace,
        PcaMode, CompressionParams, compress,
        CompressionMetrics, OrientationMethod,
    };
}
