//! Eigenvalue and Eigenvector Analysis Module
//!
//! Provides eigenvalue decomposition analysis for PCA-based image compression
//! and orientation detection. Results include eigenvalues, eigenvectors,
//! variance explained, and principal axis visualization data.

use crate::error::{CompressionError, Result};
use crate::image::ImageData;
use nalgebra::{DMatrix, DVector, Matrix3, SymmetricEigen};

/// Results from eigenvalue decomposition analysis
#[derive(Debug, Clone)]
pub struct EigenAnalysisResult {
    /// Eigenvalues (λ₁ ≥ λ₂ ≥ λ₃ for 3D RGB data)
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors as 3x3 matrix (column-major order)
    pub eigenvectors: Vec<f64>,
    /// Variance explained by each component as percentage
    pub variance_explained: Vec<f64>,
    /// Cumulative variance explained as percentage
    pub cumulative_variance: Vec<f64>,
    /// Principal axis angle in degrees (from horizontal)
    pub principal_axis_angle: f64,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Recommended rotation in degrees (0, 90, 180, 270)
    pub recommended_rotation: f32,
}

impl Default for EigenAnalysisResult {
    fn default() -> Self {
        Self {
            eigenvalues: vec![0.0; 3],
            eigenvectors: vec![0.0; 9],
            variance_explained: vec![0.0; 3],
            cumulative_variance: vec![0.0; 3],
            principal_axis_angle: 0.0,
            confidence: 0.0,
            recommended_rotation: 0.0,
        }
    }
}

impl EigenAnalysisResult {
    /// Get primary eigenvalue (λ₁)
    pub fn primary_eigenvalue(&self) -> f64 {
        self.eigenvalues.first().copied().unwrap_or(0.0)
    }

    /// Get primary eigenvector as 3D vector
    pub fn primary_eigenvector(&self) -> [f64; 3] {
        if self.eigenvectors.len() >= 3 {
            [self.eigenvectors[0], self.eigenvectors[1], self.eigenvectors[2]]
        } else {
            [1.0, 0.0, 0.0]
        }
    }
}

/// Axis overlay data for visualization
#[derive(Debug, Clone)]
pub struct AxisOverlay {
    /// Start X coordinate (normalized 0.0-1.0)
    pub x1: f64,
    /// Start Y coordinate (normalized 0.0-1.0)
    pub y1: f64,
    /// End X coordinate (normalized 0.0-1.0)
    pub x2: f64,
    /// End Y coordinate (normalized 0.0-1.0)
    pub y2: f64,
    /// Principal axis angle in degrees
    pub angle: f64,
    /// Primary eigenvalue for display
    pub primary_eigenvalue: f64,
}

impl Default for AxisOverlay {
    fn default() -> Self {
        Self {
            x1: 0.0,
            y1: 0.5,
            x2: 1.0,
            y2: 0.5,
            angle: 0.0,
            primary_eigenvalue: 0.0,
        }
    }
}

/// Available colors for axis overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayColor {
    Red,
    Yellow,
    Cyan,
    Green,
    Magenta,
}

impl OverlayColor {
    /// Get RGB values (0-255)
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Red => (255, 0, 0),
            Self::Yellow => (255, 255, 0),
            Self::Cyan => (0, 255, 255),
            Self::Green => (0, 255, 0),
            Self::Magenta => (255, 0, 255),
        }
    }

    /// Get as hex string
    pub fn to_hex(&self) -> &'static str {
        match self {
            Self::Red => "#FF0000",
            Self::Yellow => "#FFFF00",
            Self::Cyan => "#00FFFF",
            Self::Green => "#00FF00",
            Self::Magenta => "#FF00FF",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "red" => Self::Red,
            "yellow" => Self::Yellow,
            "cyan" => Self::Cyan,
            "green" => Self::Green,
            "magenta" => Self::Magenta,
            _ => Self::Red,
        }
    }
}

/// Analyze image and perform eigenvalue decomposition
pub fn analyze_image(image: &ImageData) -> Result<EigenAnalysisResult> {
    let (r, g, b) = image.split_channels();
    let n_pixels = image.num_pixels();

    if n_pixels == 0 {
        return Err(CompressionError::InvalidParams {
            field: "image".to_string(),
            value: "Empty image".to_string(),
        });
    }

    // Build 3D data matrix (pixels x 3 channels)
    let mut data = Vec::with_capacity(n_pixels * 3);
    for i in 0..n_pixels {
        data.push(r[i] as f64);
        data.push(g[i] as f64);
        data.push(b[i] as f64);
    }

    let matrix = DMatrix::from_row_slice(n_pixels, 3, &data);

    // Mean centering
    let mean = matrix.column_mean();
    let centered = &matrix - DMatrix::from_row_slice(n_pixels, 3, &vec![mean[0], mean[1], mean[2]; n_pixels]);

    // Compute covariance matrix (3x3)
    let cov = (&centered.transpose() * &centered) / (n_pixels as f64 - 1.0);

    // Eigen decomposition
    let eigen = SymmetricEigen::new(cov);

    // Extract eigenvalues (already sorted descending by nalgebra)
    let eigenvalues: Vec<f64> = eigen.eigenvalues.iter().map(|&v| v.abs().max(1e-10)).collect();
    let eigenvectors = eigen.eigenvectors.data.as_slice().to_vec();

    // Calculate variance explained
    let total_variance: f64 = eigenvalues.iter().sum();
    let mut variance_explained = Vec::with_capacity(3);
    let mut cumulative_variance = Vec::with_capacity(3);
    let mut cum_sum = 0.0;

    for &ev in &eigenvalues {
        let pct = if total_variance > 0.0 { (ev / total_variance) * 100.0 } else { 0.0 };
        cum_sum += pct;
        variance_explained.push(pct);
        cumulative_variance.push(cum_sum);
    }

    // Calculate principal axis angle from first eigenvector
    let primary_ev = if eigenvectors.len() >= 3 {
        [eigenvectors[0], eigenvectors[1], eigenvectors[2]]
    } else {
        [1.0, 0.0, 0.0]
    };

    let principal_angle = primary_ev[1].atan2(primary_ev[0]).to_degrees();

    // Normalize angle to 0-360
    let normalized_angle = ((principal_angle % 360.0) + 360.0) % 360.0;

    // Calculate confidence from eigenvalue ratio
    let confidence = if eigenvalues[0] + eigenvalues[1] + eigenvalues[2] > 0.0 {
        let ratio = eigenvalues[0] / (eigenvalues[0] + eigenvalues[1] + eigenvalues[2]);
        (ratio * 2.0 - 1.0).clamp(0.0, 1.0) as f64
    } else {
        0.0
    };

    // Determine recommended rotation
    let recommended_rotation = normalize_to_standard_rotation(normalized_angle);

    Ok(EigenAnalysisResult {
        eigenvalues,
        eigenvectors,
        variance_explained,
        cumulative_variance,
        principal_axis_angle: normalized_angle,
        confidence,
        recommended_rotation,
    })
}

/// Generate axis overlay data from eigen analysis result
pub fn get_axis_overlay(result: &EigenAnalysisResult) -> AxisOverlay {
    let primary_ev = result.primary_eigenvector();
    let angle_rad = primary_ev[1].atan2(primary_ev[0]);

    let center_x = 0.5;
    let center_y = 0.5;
    let length = 0.4; // Half-length of the axis line

    let dx = angle_rad.cos() * length;
    let dy = angle_rad.sin() * length;

    AxisOverlay {
        x1: (center_x - dx).clamp(0.0, 1.0),
        y1: (center_y - dy).clamp(0.0, 1.0),
        x2: (center_x + dx).clamp(0.0, 1.0),
        y2: (center_y + dy).clamp(0.0, 1.0),
        angle: result.principal_axis_angle,
        primary_eigenvalue: result.primary_eigenvalue(),
    }
}

/// Format eigenvalue in scientific notation
pub fn format_eigenvalue_scientific(value: f64) -> String {
    if value == 0.0 {
        return "0.00e0".to_string();
    }

    let sign = if value < 0.0 { "-" } else { "" };
    let abs_value = value.abs();

    let exponent = (abs_value.log10().floor()) as i32;
    let mantissa = abs_value / 10.0_f64.powi(exponent);

    format!("{}{:.2}e{}", sign, mantissa, exponent)
}

/// Normalize angle to standard rotation (0, 90, 180, 270)
fn normalize_to_standard_rotation(angle: f64) -> f32 {
    let rotations = [0.0f64, 90.0, 180.0, 270.0];
    let mut closest = 0.0f64;
    let mut min_diff = f64::MAX;

    for &rot in &rotations {
        let diff = (angle - rot).abs();
        let diff = diff.min(360.0 - diff);
        if diff < min_diff {
            min_diff = diff;
            closest = rot;
        }
    }

    closest as f32
}

/// Analyze image pixels weighted by intensity for orientation detection
pub fn analyze_orientation(image: &ImageData) -> Result<EigenAnalysisResult> {
    analyze_image(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for y in 0..height {
            for x in 0..width {
                let intensity = ((x + y) as f32 / ((width + height) as f32)) * 0.5 + 0.25;
                data.push(intensity);
                data.push(intensity);
                data.push(intensity);
            }
        }
        ImageData::new(width, height, data).unwrap()
    }

    #[test]
    fn test_analyze_image() {
        let img = create_test_image(100, 100);
        let result = analyze_image(&img).unwrap();

        assert_eq!(result.eigenvalues.len(), 3);
        assert_eq!(result.eigenvectors.len(), 9);
        assert_eq!(result.variance_explained.len(), 3);
        assert_eq!(result.cumulative_variance.len(), 3);

        // Variance should sum close to 100%
        let sum: f64 = result.variance_explained.iter().sum();
        assert!((sum - 100.0).abs() < 0.1);

        // Cumulative should end at 100%
        assert!((result.cumulative_variance[2] - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_axis_overlay() {
        let img = create_test_image(100, 100);
        let result = analyze_image(&img).unwrap();
        let overlay = get_axis_overlay(&result);

        // Check coordinates are normalized
        assert!(overlay.x1 >= 0.0 && overlay.x1 <= 1.0);
        assert!(overlay.y1 >= 0.0 && overlay.y1 <= 1.0);
        assert!(overlay.x2 >= 0.0 && overlay.x2 <= 1.0);
        assert!(overlay.y2 >= 0.0 && overlay.y2 <= 1.0);
    }

    #[test]
    fn test_scientific_notation() {
        assert_eq!(format_eigenvalue_scientific(847.23), "8.47e2");
        assert_eq!(format_eigenvalue_scientific(42.15), "4.22e1");
        assert_eq!(format_eigenvalue_scientific(1.06), "1.06e0");
        assert_eq!(format_eigenvalue_scientific(0.003), "3.00e-3");
        assert_eq!(format_eigenvalue_scientific(0.0), "0.00e0");
    }

    #[test]
    fn test_overlay_color() {
        assert_eq!(OverlayColor::Red.to_hex(), "#FF0000");
        assert_eq!(OverlayColor::Cyan.to_rgb(), (0, 255, 255));
        assert_eq!(OverlayColor::from_str("cyan"), OverlayColor::Cyan);
    }

    #[test]
    fn test_normalize_rotation() {
        assert_eq!(normalize_to_standard_rotation(5.0), 0.0);
        assert_eq!(normalize_to_standard_rotation(85.0), 90.0);
        assert_eq!(normalize_to_standard_rotation(175.0), 180.0);
        assert_eq!(normalize_to_standard_rotation(350.0), 0.0);
    }
}
