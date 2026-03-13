// Integration tests for PCA compression library

use pca_core::prelude::*;
use std::fs;
use std::path::PathBuf;

fn setup_test_image_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("test_image.jpg");
    path
}

#[test]
fn test_full_compression_pipeline() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        // Skip test if no test image available
        return;
    }

    // Load image
    let image = load_image(&test_image_path).expect("Failed to load image");

    // Check dimensions
    assert!(image.width >= 64);
    assert!(image.height >= 64);

    // Test compression
    let params = CompressionParams::default();

    let result = compress(&image, &params).expect("Compression failed");

    // Check result
    assert!(result.ssim > 0.0);
    assert!(result.psnr >= 0.0);
    assert!(result.compression_ratio >= 1.0);
}

#[test]
fn test_per_channel_vs_joint_channel() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    let per_channel_params = CompressionParams {
        mode: PcaMode::PerChannel,
        ..Default::default()
    };

    let joint_channel_params = CompressionParams {
        mode: PcaMode::JointChannel,
        ..Default::default()
    };

    let per_channel_result = compress(&image, &per_channel_params).expect("Per-channel failed");
    let joint_channel_result = compress(&image, &joint_channel_params).expect("Joint-channel failed");

    // Both should produce valid results
    assert!(per_channel_result.ssim > 0.0);
    assert!(joint_channel_result.ssim > 0.0);

    // Results may differ
    assert_ne!(per_channel_result.ssim, joint_channel_result.ssim);
}

#[test]
fn test_tile_processing() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Test tiling
    let tiles = crate::pca_core::tile::split_into_tiles(&image, 256);

    assert!(!tiles.is_empty());

    // Stitch back
    let stitched = crate::pca_core::tile::stitch_tiles(&tiles, image.width, image.height, false)
        .expect("Stitching failed");

    assert_eq!(stitched.width, image.width);
    assert_eq!(stitched.height, image.height);
}

#[test]
fn test_quality_scaling() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Test different quality levels
    let qualities = [0.3, 0.5, 0.7, 0.9];

    for quality in qualities {
        let params = CompressionParams::with_quality(quality);
        let result = compress(&image, &params).expect("Compression failed");

        assert!(result.ssim > 0.0);
        assert!(result.psnr >= 0.0);
    }
}

#[test]
fn test_error_handling() {
    // Test loading non-existent file
    let nonexistent = PathBuf::from("/nonexistent/image.jpg");
    let result = load_image(&nonexistent);

    assert!(result.is_err());

    // Test invalid parameters
    let test_image_path = setup_test_image_path();
    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    let invalid_params = CompressionParams {
        quality: -0.5, // Invalid
        ..Default::default()
    };

    assert!(invalid_params.validate().is_err());
}

#[test]
fn test_orientation_correction() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Test different orientation modes
    let modes = [
        crate::pca::OrientationMode::Disabled,
        crate::pca::OrientationMode::Exif,
        crate::pca::OrientationMode::Auto,
    ];

    for mode in modes {
        let params = CompressionParams {
            orientation: mode,
            ..Default::default()
        };

        let result = compress(&image, &params).expect("Compression failed");
        assert!(result.ssim > 0.0);
    }
}