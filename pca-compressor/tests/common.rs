// Common test utilities

use std::path::PathBuf;

/// Get path to test assets directory
pub fn test_assets_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("assets");
    path
}

/// Get path to a specific test asset
pub fn test_asset_path(asset_name: &str) -> PathBuf {
    let mut path = test_assets_dir();
    path.push(asset_name);
    path
}

/// Create a test image with the given dimensions and color pattern
pub fn create_test_image(width: u32, height: u32, pattern: TestPattern) -> Vec<f32> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = pattern.get_color(x, y, width, height);
            data.extend_from_slice(&[r, g, b]);
        }
    }

    data
}

#[derive(Debug, Clone, Copy)]
pub enum TestPattern {
    /// Grayscale gradient from black to white
    Gradient,
    /// Checkerboard pattern
    Checkerboard,
    /// Random noise
    Random,
    /// Solid color
    Solid { r: f32, g: f32, b: f32 },
    /// Color stripes
    Stripes,
}

impl TestPattern {
    pub fn get_color(&self, x: u32, y: u32, width: u32, height: u32) -> [f32; 3] {
        match self {
            TestPattern::Gradient => {
                let intensity = (x as f32 / width as f32).max(y as f32 / height as f32);
                [intensity, intensity, intensity]
            }
            TestPattern::Checkerboard => {
                let checker_size = 8;
                let is_white = ((x / checker_size) + (y / checker_size)) % 2 == 0;
                if is_white { [1.0, 1.0, 1.0] } else { [0.0, 0.0, 0.0] }
            }
            TestPattern::Random => {
                let x = (x.wrapping_mul(1103515245) + 12345) & 0x7fffffff;
                let y = (y.wrapping_mul(1103515245) + 12345) & 0x7fffffff;
                let combined = (x + y) as f32;
                [
                    (combined % 256.0) / 255.0,
                    ((combined * 3.0) % 256.0) / 255.0,
                    ((combined * 7.0) % 256.0) / 255.0,
                ]
            }
            TestPattern::Solid { r, g, b } => [*r, *g, *b],
            TestPattern::Stripes => {
                let stripe_width = 32;
                let stripe_index = x / stripe_width;
                match stripe_index % 3 {
                    0 => [1.0, 0.0, 0.0], // Red
                    1 => [0.0, 1.0, 0.0], // Green
                    _ => [0.0, 0.0, 1.0], // Blue
                }
            }
        }
    }
}

/// Assert that two floating point values are approximately equal
pub fn assert_approx_eq(a: f32, b: f32, epsilon: f32) {
    let diff = (a - b).abs();
    assert!(
        diff <= epsilon,
        "Values differ by {}: {} vs {} (epsilon: {})",
        diff, a, b, epsilon
    );
}

/// Assert that two vectors are approximately equal
pub fn assert_vec_approx_eq(a: &[f32], b: &[f32], epsilon: f32) {
    assert_eq!(a.len(), b.len(), "Vectors have different lengths");
    for (i, (&a_val, &b_val)) in a.iter().zip(b.iter()).enumerate() {
        assert_approx_eq(a_val, b_val, epsilon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_pattern() {
        let (r, g, b) = TestPattern::Gradient.get_color(0, 0, 100, 100);
        assert_approx_eq(r, 0.0, 0.001);
        assert_approx_eq(g, 0.0, 0.001);
        assert_approx_eq(b, 0.0, 0.001);

        let (r, g, b) = TestPattern::Gradient.get_color(99, 99, 100, 100);
        assert_approx_eq(r, 0.99, 0.01);
        assert_approx_eq(g, 0.99, 0.01);
        assert_approx_eq(b, 0.99, 0.01);
    }

    #[test]
    fn test_solid_pattern() {
        let pattern = TestPattern::Solid { r: 0.5, g: 0.75, b: 0.25 };
        let (r, g, b) = pattern.get_color(10, 20, 100, 100);
        assert_approx_eq(r, 0.5, 0.001);
        assert_approx_eq(g, 0.75, 0.001);
        assert_approx_eq(b, 0.25, 0.001);
    }

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(1.0, 1.0 + 0.001, 0.01);
        assert_approx_eq(1.0, 1.0 - 0.001, 0.01);

        let result = std::panic::catch_unwind(|| {
            assert_approx_eq(1.0, 1.1, 0.01);
        });
        assert!(result.is_err());
    }
}