//! PCA-based Image Compression Core Library
//!
//! This library provides PCA-based image compression with:
//! - Per-channel and joint-channel PCA modes
//! - Orientation correction
//! - Eigenvalue/eigenvector analysis
//! - Tile processing for large images
//! - Quality metrics (SSIM, PSNR)

pub mod error;
pub mod image;
pub mod pca;
pub mod metrics;
pub mod orientation;
pub mod tile;
pub mod compression;
pub mod eigen_analysis;

pub use error::{CompressionError, Result};
pub use image::{ImageData, ColorSpace, load_image, save_image};
pub use pca::{PcaMode, CompressionParams, compress};
pub use metrics::CompressionMetrics;
pub use orientation::OrientationMethod;
pub use eigen_analysis::{EigenAnalysisResult, AxisOverlay, OverlayColor, analyze_image, get_axis_overlay, format_eigenvalue_scientific};

pub mod prelude {
    pub use crate::{
        CompressionError, Result, ImageData, ColorSpace,
        PcaMode, CompressionParams, compress,
        CompressionMetrics, OrientationMethod,
        EigenAnalysisResult, AxisOverlay, OverlayColor,
        analyze_image, get_axis_overlay, format_eigenvalue_scientific,
    };
}
