# API Documentation

## Core Library (Rust)

### Loading Images

```rust
use pca_core::image::{ImageData, load_image};

// Load from file
let image = load_image(Path::new("image.jpg"))?;

// Create from data
let data = vec![0.5f32; 100 * 100 * 3];
let image = ImageData::new(100, 100, data)?;

// Create with alpha channel
let rgb = vec![0.5f32; 100 * 100 * 3];
let alpha = vec![1.0f32; 100 * 100];
let image = ImageData::with_alpha(100, 100, rgb, alpha)?;
```

### Compression Parameters

```rust
use pca_core::pca::{CompressionParams, PcaMode, OrientationMode, OutputFormat};

// Default parameters
let params = CompressionParams::default();

// Builder pattern
let params = CompressionParams::default()
    .with_quality(0.8)
    .mode(PcaMode::JointChannel);

// Direct construction
let params = CompressionParams {
    quality: 0.7,
    mode: PcaMode::PerChannel,
    retain_components: 1,
    orientation: OrientationMode::Auto,
    tile_size: Some(1024),
    max_memory_mb: Some(1024),
    output_format: OutputFormat::Jpeg,
    strip_metadata: false,
};
```

### Compression

```rust
use pca_core::pca::compress;

let result = compress(&image, &params)?;

// Access results
println!("SSIM: {:.3}", result.ssim);
println!("PSNR: {:.1} dB", result.psnr);
println!("Processing time: {}ms", result.processing_time_ms);

// Get compressed image
let compressed_image = result.image;
```

### High-Level Compression Pipeline

```rust
use pca_core::compression::compress_image;

let metrics = compress_image(
    Path::new("input.jpg"),
    Path::new("output.jpg"),
    &params)?;

println!("Compressed {} -> {} (ratio: {:.2}x)",
    metrics.original_size,
    metrics.compressed_size,
    metrics.compression_ratio);
```

### Batch Processing

```rust
use pca_core::compression::{compress_batch, scan_directory};

// Scan directory
let input_files = scan_directory(Path::new("./input/"));

// Process batch
let results = compress_batch(&input_files, Path::new("./output/"), &params);

for result in &results {
    if result.success {
        println!("✓ {}: ratio={:.2}x", result.input_path, result.metrics.as_ref().unwrap().compression_ratio);
    } else {
        println!("✗ {}: {}", result.input_path, result.error.as_ref().unwrap());
    }
}
```

### CSV Reports

```rust
use std::path::Path;
use pca_core::compression::write_batch_report;

// Write CSV report
write_batch_report(&results, Path::new("report.csv"))?;
```

### Custom Error Handling

```rust
use pca_core::error::{CompressionError, Result};

fn compress_safe(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    // This will return a CompressionError if anything fails
    let result = compress(image, params)?;
    Ok(result.image)
}

match compress_safe(&image, &params) {
    Ok(compressed) => println!("Success!"),
    Err(e) => {
        if e.is_memory_error() {
            println!("Memory error: {}", e);
            // Try with tiling
        } else if e.needs_tiling() {
            println!("Image too large, enable tiling");
        } else {
            println!("Error: {}", e);
        }
    }
}
```

## Python Bindings

### Installation

```bash
pip install pca-compressor
```

### Basic Usage

```python
from pca_compressor import compress_file, CompressionParams

# Create parameters
params = CompressionParams(
    quality=0.7,
    mode='per-channel',
    retain_components=1,
    orientation='auto'
)

# Compress file
result = compress_file(
    'input.jpg',
    'output.jpg',
    params
)

print(f"SSIM: {result.ssim:.3f}")
print(f"Compression Ratio: {result.compression_ratio:.2f}x")
```

### Compressing from Bytes

```python
from pca_compressor import compress_bytes
from PIL import Image

# Load image as bytes
with open('input.jpg', 'rb') as f:
    image_bytes = f.read()

# Compress with parameters
params = CompressionParams(quality=0.7)
result = compress_bytes(image_bytes, params)

# Get compressed bytes
compressed_bytes = result.bytes

# Save with PIL
Image.open(io.BytesIO(compressed_bytes)).save('output.jpg')
```

### Batch Processing

```python
import os
from pathlib import Path
from pca_compressor import compress_file, CompressionParams

def batch_compress(input_dir, output_dir, params):
    input_path = Path(input_dir)
    output_path = Path(output_dir)

    output_path.mkdir(parents=True, exist_ok=True)

    results = []

    for file in input_path.iterdir():
        if file.suffix.lower() in ['.jpg', '.jpeg', '.png']:
            output_file = output_path / f"{file.stem}_compressed.jpg"

            try:
                result = compress_file(str(file), str(output_file), params)
                results.append({
                    'file': file.name,
                    'success': True,
                    'ssim': result.ssim,
                    'ratio': result.compression_ratio
                })
            except Exception as e:
                results.append({
                    'file': file.name,
                    'success': False,
                    'error': str(e)
                })

    return results

# Usage
params = CompressionParams(quality=0.7)
results = batch_compress('./photos/', './compressed/', params)

for r in results:
    status = "✓" if r['success'] else "✗"
    print(f"{status} {r['file']}")
```

### Custom Quality Metric Callbacks

```python
def compress_with_callback(input_path, params, callback):
    """Compress with progress callback"""
    result = compress_file(input_path, 'temp.jpg', params)

    # Call user callback with results
    callback('progress', {
        'ssim': result.ssim,
        'psnr': result.psnr,
        'ratio': result.compression_ratio
    })

    return result

# Usage
def my_callback(event, data):
    if event == 'progress':
        print(f"Progress: SSIM={data['ssim']:.3f}")

result = compress_with_callback('input.jpg', params, my_callback)
```

## CLI API

### Command Structure

```
pca-compress <command> [options]

Commands:
  single      Compress a single image
  batch       Compress multiple images in batch
  validate    Validate an image without compressing
  -h, --help Show help
```

### Single Image Command

```bash
pca-compress single <INPUT> [OPTIONS]

Options:
  -o, --output <OUTPUT>            Output file path
  -q, --quality <QUALITY>          Compression quality [0.1-1.0] [default: 0.7]
  -m, --mode <MODE>                PCA mode [per-channel|joint-channel] [default: per-channel]
  -n, --retain-components <N>      Number of components to keep [default: 1]
  --orientation <MODE>             Orientation mode [auto|exif|disabled] [default: auto]
  --tile-size <SIZE>               Tile size (0 for no tiling)
  --max-memory <MB>                Maximum memory in MB
  -f, --format <FORMAT>            Output format [jpeg|png] [default: jpeg]
      --strip-metadata             Strip EXIF metadata
  -v, --verbose                    Verbose output
```

### Batch Command

```bash
pca-compress batch <INPUT_DIR> <OUTPUT_DIR> [OPTIONS]

Options:
  -q, --quality <QUALITY>          Compression quality [default: 0.7]
  -m, --mode <MODE>                PCA mode [default: per-channel]
  -n, --retain-components <N>      Number of components [default: 1]
  --orientation <MODE>             Orientation mode [default: auto]
  --tile-size <SIZE>               Tile size
  --max-memory <MB>                Maximum memory in MB
  -f, --format <FORMAT>            Output format [default: jpeg]
      --strip-metadata             Strip EXIF metadata
  -r, --report <PATH>              CSV report path
  -j, --threads <N>                Number of worker threads [default: CPU cores - 1]
```

### Validate Command

```bash
pca-compress validate <INPUT> [OPTIONS]

Options:
  -v, --verbose                    Verbose output
```

## Error Handling

### Rust Error Types

```rust
use pca_core::error::CompressionError;

match compress(&image, &params) {
    Ok(result) => { /* success */ },
    Err(CompressionError::ImageTooSmall { width, height }) => {
        eprintln!("Image too small: {}x{}", width, height);
    },
    Err(CompressionError::MemoryLimitExceeded { required_mb, available_mb }) => {
        eprintln!("Memory exceeded: need {}MB, have {}MB", required_mb, available_mb);
    },
    Err(CompressionError::InvalidParams { field, value }) => {
        eprintln!("Invalid {}: {}", field, value);
    },
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Python Error Handling

```python
try:
    result = compress_file('input.jpg', 'output.jpg', params)
except ValueError as e:
    print(f"Invalid parameter: {e}")
except RuntimeError as e:
    print(f"Compression failed: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Quality Metrics

### Manual SSIM Calculation

```rust
use pca_core::metrics::calculate_ssim;

let ssim = calculate_ssim(&original, &compressed)?;
println!("SSIM: {:.3}", ssim);

// SSIM interpretation:
// > 0.95: Excellent quality
// 0.90-0.95: Good quality
// 0.80-0.90: Acceptable quality
// < 0.80: Poor quality
```

### Manual PSNR Calculation

```rust
use pca_core::metrics::calculate_psnr;

let psnr = calculate_psnr(&original, &compressed)?;
println!("PSNR: {:.1} dB", psnr);

// PSNR interpretation:
// > 40 dB: Excellent quality
// 30-40 dB: Good quality
// 20-30 dB: Fair quality
// < 20 dB: Poor quality
```

### Custom Metrics

```rust
use std::time::Instant;

struct CustomMetrics {
    ssim: f32,
    psnr: f32,
    time_ms: u64,
    size_ratio: f32,
}

impl CustomMetrics {
    fn calculate(
        original: &ImageData,
        compressed: &ImageData,
        original_size: usize,
        compressed_size: usize,
    ) -> Result<Self> {
        let start = Instant::now();

        let ssim = calculate_ssim(original, compressed)?;
        let psnr = calculate_psnr(original, compressed)?;

        let time_ms = start.elapsed().as_millis() as u64;
        let size_ratio = original_size as f32 / compressed_size.max(1) as f32;

        Ok(Self {
            ssim,
            psnr,
            time_ms,
            size_ratio,
        })
    }

    fn is_acceptable(&self) -> bool {
        self.ssim > 0.85 && self.psnr > 25.0
    }

    fn grade(&self) -> &'static str {
        if self.is_acceptable() {
            if self.ssim > 0.95 { "Excellent" }
            else { "Good" }
        } else {
            "Poor"
        }
    }
}
```

## Configuration Examples

### High Quality Archival

```rust
let params = CompressionParams {
    quality: 0.9,
    mode: PcaMode::JointChannel,
    retain_components: 2,
    orientation: OrientationMode::Auto,
    tile_size: None,
    max_memory_mb: Some(2048),
    output_format: OutputFormat::Jpeg,
    strip_metadata: false,
};
```

### Fast Preview

```rust
let params = CompressionParams {
    quality: 0.3,
    mode: PcaMode::PerChannel,
    retain_components: 1,
    orientation: OrientationMode::Disabled, // Skip for speed
    tile_size: Some(256),
    max_memory_mb: Some(256),
    output_format: OutputFormat::Jpeg,
    strip_metadata: true, // Save space
};
```

### Balanced Web Use

```rust
let params = CompressionParams::default()
    .with_quality(0.7)
    .mode(PcaMode::PerChannel);
```

## Advanced Patterns

### Adaptive Quality

```rust
fn adaptive_quality(image: &ImageData) -> f32 {
    // Analyze image complexity
    let complexity = estimate_complexity(image);

    // Higher complexity = higher quality
    match complexity {
        Complexity::Low => 0.5,
        Complexity::Medium => 0.7,
        Complexity::High => 0.9,
    }
}

enum Complexity {
    Low, Medium, High,
}

fn estimate_complexity(image: &ImageData) -> Complexity {
    // Simple heuristic based on edge count
    let edge_count = count_edges(image);
    match edge_count {
        0..1000 => Complexity::Low,
        1000..5000 => Complexity::Medium,
        _ => Complexity::High,
    }
}
```

### Progressive Compression

```rust
fn progressive_compress(image: &ImageData, max_ratio: f32) -> ImageData {
    let mut quality = 0.9;
    let mut compressed = image.clone();

    while quality >= 0.3 {
        let params = CompressionParams::with_quality(quality);
        if let Ok(result) = compress(&compressed, &params) {
            if result.compression_ratio >= max_ratio {
                return result.image;
            }
            compressed = result.image;
        }
        quality -= 0.1;
    }

    compressed
}
```