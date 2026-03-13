# Documentation

## Getting Started

### Prerequisites

- Rust (for building CLI and core library)
- Python 3.8+ (for Python bindings)
- Cargo (Rust package manager)

### Installation

#### From Source

```bash
cd pca-compressor
git clone <repository-url>
cd pca-compressor
cargo build --release
```

#### Python Package

```bash
pip install pca-compressor
```

## API Reference

### Rust Library

```rust
use pca_core::compress;
use pca_core::CompressionParams;
use pca_core::ImageData;

// Load image
let image = ImageData::from_file("input.jpg")?;

// Set parameters
let params = CompressionParams::default()
    .with_quality(0.7)
    .mode(PcaMode::PerChannel);

// Compress
let result = compress(&image, &params)?;

// Save result
ImageData::save_image("output.jpg", &result.image, 90)?;
```

### Python Bindings

```python
from pca_compressor import compress
from pca_compressor import CompressionParams

# Load image
from PIL import Image
img = Image.open('input.jpg')

# Compress
result = compress(
    img,
    quality=0.7,
    mode='per-channel',
    retain_components=1,
    orientation='auto'
)

# Save result
img.save('output.jpg', quality=int(result.quality * 100))
```

## Configuration Options

### CompressionParams

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `quality` | float | 0.7 | Compression strength (0.1-1.0) |
| `mode` | PcaMode | PerChannel | Processing mode |
| `retain_components` | int | 1 | Number of PCA components to keep |
| `orientation` | OrientationMode | Auto | Orientation handling |
| `tile_size` | int | 1024 | Tile size for large images |
| `max_memory_mb` | int | 1024 | Memory limit in MB |
| `output_format` | OutputFormat | Jpeg | Output format |
| `strip_metadata` | bool | False | Strip EXIF metadata |

### PcaMode

- `PerChannel`: Process each RGB channel independently
- `JointChannel`: Treat RGB as 3D vectors

### OrientationMode

- `Auto`: Use PCA principal axis, fallback to EXIF
- `Exif`: Use EXIF orientation only
- `Disabled`: No orientation correction

## Error Handling

### Rust Errors

```rust
use pca_core::error::CompressionError;

try {
    let result = compress(&image, &params)?;
    Ok(result)
} catch CompressionError::MemoryLimitExceeded { |e| {
    eprintln!("Memory limit exceeded: {}", e);
    // Handle by enabling tiling or reducing quality
} catch CompressionError::ImageTooSmall { |e| {
    eprintln!("Image too small: {}", e);
    // Return original or use lossless mode
}
```

### Python Errors

```python
try:
    result = compress(image, quality=0.7)
except ValueError as e:
    print(f"Error: {e}")
    # Handle specific cases
    if "too small" in str(e):
        # Handle small image
        pass
    elif "memory" in str(e):
        # Handle memory issues
        pass
```

## Performance Tips

### Memory Management

```rust
// Set memory limit
let params = CompressionParams::default()
    .max_memory_mb(512); // 512MB limit

// Enable tiling for large images
let params = params.tile_size(2048);
```

### Batch Processing

```rust
use rayon::prelude::*;

// Multi-threaded processing
let results: Vec<_> = input_files.par_iter()
    .map(|file| process_file(file, &params))
    .collect();
```

### Python

```python
# Batch processing
from concurrent.futures import ThreadPoolExecutor

def process_file(file):
    # Process single file
    pass

with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(process_file, files))
```

## Quality Metrics

### SSIM Calculation

```rust
// Calculate SSIM
let ssim = calculate_ssim(&original, &compressed)?;
println!("SSIM: {:.3}", ssim);

// Check quality
if ssim < 0.9 {
    println!("Warning: Low quality output");
}
```

### PSNR Calculation

```rust
let psnr = calculate_psnr(&original, &compressed)?;
println!("PSNR: {:.1} dB", psnr);

// PSNR interpretation
// > 40 dB: Excellent quality
// 30-40 dB: Good quality
// 20-30 dB: Fair quality
// < 20 dB: Poor quality
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pca_math() {
        // Test PCA implementation
    }

    #[test]
    fn test_ssim_calculation() {
        // Test SSIM implementation
    }

    #[test]
    fn test_image_processing() {
        // Test full pipeline
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_pipeline() {
    let image = ImageData::from_file("test.jpg")?;
    let params = CompressionParams::default();
    let result = compress(&image, &params)?;

    // Verify output
    assert!(result.ssim > 0.8);
    assert!(result.compression_ratio > 1.0);
}
```

## Troubleshooting

### Common Issues

1. **Memory Errors**
   - Enable tiling: `params.tile_size(1024)`
   - Reduce quality: `params.quality(0.5)`
   - Increase memory limit: `params.max_memory_mb(2048)`

2. **Quality Issues**
   - Increase retain components: `params.retain_components(2)`
   - Use joint-channel mode for color images
   - Adjust quality parameter

3. **Orientation Issues**
   - Force EXIF: `params.orientation(OrientationMode::Exif)`
   - Disable correction: `params.orientation(OrientationMode::Disabled)`

### Performance Issues

1. **Slow Processing**
   - Check CPU usage
   - Enable multi-threading
   - Use smaller tile size

2. **High Memory Usage**
   - Reduce tile size
   - Lower quality
   - Enable memory limit

## Best Practices

1. **Always validate input**
   ```rust
   if image.width < 64 || image.height < 64 {
       // Handle small images
   }
   ```

2. **Handle errors gracefully**
   ```rust
   match compress(&image, &params) {
       Ok(result) => { /* success */ }
       Err(e) => {
           eprintln!("Error: {}", e);
           // Fallback or retry
       }
   }
   ```

3. **Use appropriate parameters**
   - For photos: `PerChannel` mode
   - For graphics: `JointChannel` mode
   - For web: Higher quality (0.8-1.0)
   - For archival: Lower quality (0.3-0.5)

4. **Test with representative images**
   - Include various sizes
   - Test different color spaces
   - Test edge cases (small, large, monochrome)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Style

- Follow Rust conventions
- Use meaningful variable names
- Add documentation comments
- Write tests for all new functionality

### Performance Guidelines

- Profile before optimizing
- Use appropriate data structures
- Minimize memory allocations
- Consider parallel processing for batch operations