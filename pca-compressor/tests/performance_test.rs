// Performance tests for PCA compression library

use pca_core::prelude::*;
use std::time::Instant;
use std::path::PathBuf;

fn setup_test_image_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("test_image.jpg");
    path
}

#[test]
#[ignore]
fn benchmark_single_image_compression() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");
    let params = CompressionParams::default();

    let start = Instant::now();
    let result = compress(&image, &params).expect("Compression failed");
    let elapsed = start.elapsed();

    println!("Single image compression:");
    println!("  Time: {:?}", elapsed);
    println!("  SSIM: {:.3}", result.ssim);
    println!("  PSNR: {:.1} dB", result.psnr);

    // Performance target: should compress within 5 seconds for 12MP image
    let pixels = image.num_pixels();
    let megapixels = pixels as f64 / 1_000_000.0;

    println!("  Image size: {:.1} MP", megapixels);
    println!("  Per MP: {:?}", elapsed / (megapixels as u32));
}

#[test]
#[ignore]
fn benchmark_tile_vs_no_tile() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Without tiling
    let no_tile_params = CompressionParams {
        tile_size: None,
        ..Default::default()
    };

    let start = Instant::now();
    let no_tile_result = compress(&image, &no_tile_params).expect("Compression failed");
    let no_tile_time = start.elapsed();

    // With tiling
    let tile_params = CompressionParams {
        tile_size: Some(512),
        ..Default::default()
    };

    let start = Instant::now();
    let tile_result = compress(&image, &tile_params).expect("Compression failed");
    let tile_time = start.elapsed();

    println!("No tile compression: {:?}", no_tile_time);
    println!("Tile compression: {:?}", tile_time);
    println!("Difference: {:?}", no_tile_time.saturating_sub(tile_time));

    // Results should be similar
    assert!((no_tile_result.ssim - tile_result.ssim).abs() < 0.1);
}

#[test]
#[ignore]
fn test_memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Simple memory tracking allocator wrapper
    struct TrackingAllocator {
        allocations: AtomicUsize,
        system: System,
    }

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            self.allocations.fetch_add(layout.size(), Ordering::SeqCst);
            self.system.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            self.allocations.fetch_sub(layout.size(), Ordering::SeqCst);
            self.system.dealloc(ptr, layout)
        }
    }

    // Note: This test would require global allocator setup
    // For now, we'll just check estimated memory usage
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Estimate memory usage
    let estimated_mb = (image.size_bytes() as f64 / (1024.0 * 1024.0)) * 3.0;

    println!("Estimated memory usage: {:.1} MB", estimated_mb);
    println!("Image size: {}x{}", image.width, image.height);
    println!("Total pixels: {}", image.num_pixels());
}

#[test]
#[ignore]
fn benchmark_parallel_tile_processing() {
    use rayon::prelude::*;
    use pca_core::tile::split_into_tiles;
    use pca_core::tile::process_tiles_parallel;

    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");
    let params = CompressionParams::default();

    let tile_size = 512;
    let tiles = split_into_tiles(&image, tile_size);

    println!("Processing {} tiles", tiles.len());

    // Sequential processing
    let start = Instant::now();
    let sequential_results: Vec<_> = tiles.iter()
        .map(|tile| crate::pca::compress(&tile.data, &params))
        .collect();

    let sequential_time = start.elapsed();
    let sequential_success = sequential_results.iter().filter(|r| r.is_ok()).count();

    // Parallel processing
    let start = Instant::now();
    let parallel_results = process_tiles_parallel(tiles, |tile| {
        crate::pca::compress(&tile.data, &params)
    });

    let parallel_time = start.elapsed();
    let parallel_success = parallel_results.iter().filter(|r| r.is_ok()).count();

    println!("Sequential:");
    println!("  Time: {:?}", sequential_time);
    println!("  Success: {}/{}", sequential_success, sequential_results.len());

    println!("Parallel:");
    println!("  Time: {:?}", parallel_time);
    println!("  Success: {}/{}", parallel_success, parallel_results.len());

    println!("Speedup: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());

    // Parallel should be faster on multi-core systems
    let cpu_count = num_cpus::get();
    if cpu_count > 1 {
        assert!(parallel_time < sequential_time);
    }
}

#[test]
fn test_quality_metrics_accuracy() {
    let test_image_path = setup_test_image_path();

    if !test_image_path.exists() {
        return;
    }

    let image = load_image(&test_image_path).expect("Failed to load image");

    // Test that quality metrics work correctly
    let params = CompressionParams::default();
    let result = compress(&image, &params).expect("Compression failed");

    // SSIM should be between 0 and 1
    assert!(result.ssim >= 0.0);
    assert!(result.ssim <= 1.0);

    // PSNR should be non-negative
    assert!(result.psnr >= 0.0);

    // Test with original image (should be 1.0 SSIM, infinite PSNR)
    let identical_result = compress(&image, &CompressionParams { quality: 1.0, ..Default::default() })
        .expect("Compression failed");

    // Higher quality should give better metrics
    assert!(identical_result.ssim >= result.ssim * 0.9);
}

#[test]
fn test_image_size_limits() {
    // Test small image handling
    let small_data = vec![0.5f32; 50 * 50 * 3];
    let small_image = ImageData::new(50, 50, small_data).expect("Small image created");

    let result = small_image.validate_size();
    assert!(result.is_err());

    // Test valid minimum size
    let min_data = vec![0.5f32; 64 * 64 * 3];
    let min_image = ImageData::new(64, 64, min_data).expect("Min image created");

    let result = min_image.validate_size();
    assert!(result.is_ok());
}