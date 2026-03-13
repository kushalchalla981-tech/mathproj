# Troubleshooting

## Common Issues

### 1. Memory Errors

**Error:** "Memory limit exceeded" or "Image too large"

**Cause:** Processing very large images without tiling

**Solutions:**

```rust
// Enable tiling
let params = CompressionParams::default()
    .tile_size(1024) // Process in 1024x1024 tiles
    .max_memory_mb(512); // 512MB limit

// Reduce quality to save memory
let params = params.quality(0.5);
```

**Python:**

```python
params = CompressionParams(
    tile_size=1024,
    max_memory_mb=512,
    quality=0.5
)
```

### 2. Quality Issues

**Problem:** Poor visual quality or artifacts

**Solutions:**

```rust
// Increase retained components
let params = params.retain_components(2); // Keep more principal components

// Use joint-channel mode for color images
let params = params.mode(PcaMode::JointChannel);

// Increase quality
let params = params.quality(0.8); // Higher quality
```

**Python:**

```python
params = CompressionParams(
    retain_components=2,
    mode='joint-channel',
    quality=0.8
)
```

### 3. Orientation Issues

**Problem:** Images appear rotated incorrectly

**Solutions:**

```rust
// Force EXIF orientation
let params = params.orientation(OrientationMode::Exif);

// Disable orientation correction
let params = params.orientation(OrientationMode::Disabled);

// Use auto mode (default)
let params = params.orientation(OrientationMode::Auto);
```

**Python:**

```python
params = CompressionParams(
    orientation='exif',  # Force EXIF
    # or orientation='disabled',  # No correction
    # or orientation='auto'  # Default (PCA with EXIF fallback)
)
```

### 4. Performance Issues

**Problem:** Slow processing or high CPU usage

**Solutions:**

```rust
// Enable multi-threading for batch processing
use rayon::prelude::*;

// Process files in parallel
let results: Vec<_> = input_files.par_iter()
    .map(|file| process_file(file, &params))
    .collect();

// Reduce tile size for better performance
let params = params.tile_size(512);

// Lower quality for faster processing
let params = params.quality(0.6);
```

**Python:**

```python
from concurrent.futures import ThreadPoolExecutor

# Multi-threaded batch processing
with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))

# Adjust parameters for speed
params = CompressionParams(
    quality=0.6,
    tile_size=512
)
```

## Error Messages

### Memory Errors

```
Memory limit exceeded: required 2048MB, available 1024MB
```

**Fix:**
- Enable tiling
- Reduce quality
- Increase memory limit
- Process smaller batches

### Image Size Errors

```
Image too small: 32x32 (minimum 64x64 required)
```

**Fix:**
- Return original image
- Use lossless compression
- Skip small images in batch processing

### Format Errors

```
Unsupported format: .bmp
```

**Fix:**
- Convert to JPEG/PNG first
- Use different library for unsupported formats
- Add format support to the tool

### EXIF Errors

```
EXIF read failed: Cannot open file
```

**Fix:**
- Check file permissions
- Use different EXIF library
- Skip EXIF processing

## Performance Optimization

### Memory Usage

```rust
// Monitor memory
let estimated_memory = estimate_memory(&image, &params);
if estimated_memory > max_memory_mb {
    // Take action
}

// Optimize for memory
let params = params
    .tile_size(512)  // Smaller tiles = less memory
    .max_memory_mb(256);  // Strict limit
```

### Processing Speed

```rust
// Profile bottlenecks
use std::time::Instant;
let start = Instant::now();
// ... processing ...
let elapsed = start.elapsed();
println!("Processing time: {:?}", elapsed);

// Optimize hot paths
// Use parallel processing for independent operations
// Cache expensive computations
// Use appropriate data structures
```

### Quality vs Speed Trade-offs

| Quality Level | Speed | Memory | Use Case |
|---------------|-------|--------|----------|
| High (0.8-1.0) | Slow | High | Final output |
| Medium (0.5-0.7) | Medium | Medium | General use |
| Low (0.1-0.4) | Fast | Low | Preview/archival |

## Debugging Tips

### Enable Verbose Output

```rust
let params = params.verbose(true);

// Print detailed information
println!("Input: {}", input.display());
eprintln!("Quality: {}", quality);
eprintln!("Mode: {:?}", mode);
```

### Log Processing Steps

```rust
use log::{info, warn, error};

info!("Loading image: {}", input.display());
info!("Processing with params: {:?}", params);
info!("Estimated memory: {}MB", estimated_memory);

// Log errors with context
if let Err(e) = compress(&image, &params) {
    error!("Compression failed for {}: {}", input.display(), e);
}
```

### Validate Intermediate Results

```rust
// Check PCA computation
let cov = compute_covariance(&centered);
assert!(cov.is_symmetric(), "Covariance matrix not symmetric");

// Verify reconstruction
let reconstructed = project_and_reconstruct(&centered, &eigenvectors);
let error = calculate_mse(&centered, &reconstructed);
assert!(error < tolerance, "Reconstruction error too high");
```

## Common Workflows

### Batch Processing Large Datasets

```rust
// Process in batches to control memory
let batch_size = 10;
let mut batches = input_files.chunks(batch_size);

for batch in batches {
    let results: Vec<_> = batch.par_iter()
        .map(|file| process_file(file, &params))
        .collect();

    // Process batch, then clear memory
    process_batch_results(&results);
    clear_memory();
}
```

### Progressive Quality

```rust
// Start with low quality, increase if needed
let mut quality = 0.3;
let mut best_result = None;

for _ in 0..3 {
    let result = compress_with_quality(quality);
    if result.ssim > 0.9 {
        best_result = Some(result);
        break;
    }
    quality += 0.2; // Increase quality
}

// Use best result found
```

## Compatibility Notes

### Platform Differences

- **Windows**: May have different memory limits
- **macOS**: Different EXIF handling
- **Linux**: May need additional dependencies

### Library Dependencies

Ensure required libraries are installed:

```bash
# Rust dependencies
cargo install image nalgebra rayon

# Python dependencies
pip install numpy pillow scikit-image

# System libraries (Linux)
sudo apt-get install libjpeg-turbo libpng
```

### Version Compatibility

- Check for breaking changes in dependencies
- Test with different library versions
- Use version pinning for production

## Reporting Issues

When reporting issues, include:

1. **Version information**
   - Tool version
   - Library versions
   - Platform/OS version

2. **Reproducible example**
   - Sample image (if possible)
   - Configuration used
   - Expected vs actual behavior

3. **Error details**
   - Full error message
   - Stack trace if available
   - Context about when error occurred

4. **Performance data**
   - Processing times
   - Memory usage
   - Hardware specifications

## Advanced Topics

### Custom PCA Implementation

```rust
// Custom eigen-decomposition
pub fn custom_eigendecompose(matrix: &DMatrix<f64>) -> Result<(DVector<f64>, DMatrix<f64>)> {
    // Custom implementation for specific needs
    // May be faster or more stable for certain cases
}
```

### Hardware Acceleration

```rust
// Use SIMD instructions
use std::arch::x86_64::*;

// Vectorized operations
#[target_feature(enable = "avx2")]
unsafe fn process_batch_simd(data: &[f32]) -> Vec<f32> {
    // SIMD-optimized processing
}
```

### Custom Metrics

```rust
// Implement custom quality metric
pub fn custom_quality_metric(original: &ImageData, compressed: &ImageData) -> f32 {
    // Custom metric combining multiple factors
    let ssim = calculate_ssim(original, compressed);
    let perceptual_score = calculate_perceptual_score(original, compressed);
    (ssim * 0.7 + perceptual_score * 0.3) // Weighted combination
}
```