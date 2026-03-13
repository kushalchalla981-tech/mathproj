# Quick Start Guide

This guide will help you get started with PCA Compressor quickly.

## Installation

### Option 1: Pre-built Binary

Download the appropriate binary for your platform:

- **Linux**: `pca-compress-x86_64-unknown-linux-gnu`
- **Windows**: `pca-compress-x86_64-pc-windows-msvc.exe`
- **macOS**: `pca-compress-x86_64-apple-darwin`

Move the binary to a directory in your PATH:

```bash
# Linux/macOS
sudo mv pca-compress /usr/local/bin/

# Windows (PowerShell)
copy pca-compress.exe C:\Windows\System32\
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/pca-compressor.git
cd pca-compressor

# Build using the build script
./build.sh --release

# Or use cargo directly
cargo build --release
```

The binary will be available at `target/release/pca-compress`.

### Option 3: Python Package

```bash
pip install pca-compressor
```

## CLI Usage

### Basic Compression

```bash
# Compress a single image with default settings
pca-compress single input.jpg

# Compress with custom quality
pca-compress single input.jpg --quality 0.8

# Specify output file
pca-compress single input.jpg -o output.jpg
```

### Advanced Options

```bash
# Use joint-channel mode
pca-compress single input.jpg --mode joint-channel

# Keep more PCA components
pca-compress single input.jpg --retain-components 2

// Use EXIF orientation only
pca-compress single input.jpg --orientation exif

// Enable tile processing
pca-compress single input.jpg --tile-size 512
```

### Batch Processing

```bash
// Compress all images in a folder
pca-compress batch ./input/ -o ./output/

// Generate CSV report
pca-compress batch ./input/ -o ./output/ --report report.csv

// Use multiple worker threads
pca-compress batch ./input/ -o ./output/ --threads 8
```

### Validation

```bash
// Validate image without compressing
pca-compress validate image.jpg
```

## Python Usage

```python
from pca_compressor import CompressionParams, compress_file

# Basic compression
params = CompressionParams(
    quality=0.7,
    mode='per-channel',
    orientation='auto'
)

result = compress_file(
    'input.jpg',
    'output.jpg',
    params
)

print(f"SSIM: {result.ssim:.3f}")
print(f"Compression Ratio: {result.compression_ratio:.2f}x")
```

## Library (Rust) Usage

```rust
use pca_core::prelude::*;

// Load and compress
let image = load_image("input.jpg")?;
let params = CompressionParams::default()
    .with_quality(0.7)
    .mode(PcaMode::PerChannel);

let result = compress(&image, &params)?;

// Save result
save_image("output.jpg", &result.image, 90)?;

// Access metrics
println!("SSIM: {:.3}", result.ssim);
println!("PSNR: {:.1} dB", result.psnr);
```

## Common Use Cases

### Reduce File Size for Web

```bash
pca-compress single photo.jpg -o web-photo.jpg --quality 0.6
```

### Archive with Good Quality

```bash
pca-compress single photo.jpg -o archived.jpg --quality 0.85 --mode joint-channel
```

### Batch Process Photos

```bash
pca-compress batch ./photos/ -o ./compressed/ --quality 0.7 --report results.csv
```

### Preview Before Export (GUI)

Launch the GUI application and use the compression slider to find the right balance:

```bash
# Build GUI first
./build.sh --gui

# Then run the GUI
./target/release/pca-gui
```

## Parameters Guide

### Quality (0.1-1.0)

- **0.1-0.4**: Highest compression, lowest quality
- **0.5-0.7**: Balanced (recommended for most use cases)
- **0.8-1.0**: Best quality, minimal compression

### PCA Mode

- **per-channel**: Process RGB channels independently (faster, good for photos)
- **joint-channel**: Treat RGB as 3D vectors (better for graphics, colorful images)

### Orientation

- **auto**: Use PCA principal axis, fall back to EXIF (default)
- **exif**: Use EXIF metadata only
- **disabled**: No orientation correction

### Memory Management

- **--max-memory**: Set memory limit in MB (useful for large images)
- **--tile-size**: Set tile size for parallel processing (default: 1024)

## Tips

1. **Start with default settings**: `quality=0.7` and `per-channel` mode work well for most images

2. **Use batch mode for folders**: `--report` flag gives you detailed statistics

3. **Enable tiling for large images**: Images over 10MP benefit from `--tile-size 512` or `--tile-size 1024`

4. **Check SSIM values**: Values above 0.90 indicate good visual quality

5. **Use GUI for experimentation**: The GUI lets you quickly preview different settings

## Troubleshooting

### Memory Issues

```bash
# Reduce memory usage
pca-compress single large.jpg --tile-size 512 --max-memory 512
```

### Quality Issues

```bash
# Increase quality or components
pca-compress single image.jpg --quality 0.8 --retain-components 2
```

### Slow Processing

```bash
# Use more threads (batch mode only)
pca-compress batch ./input/ -o ./output/ --threads 8
```

## Performance Benchmarks

On a typical modern laptop (i7 CPU, 16GB RAM):

- **Small images (<1MP)**: ~100ms
- **Medium images (1-4MP)**: ~500ms-2s
- **Large images (4-12MP)**: ~2-5s
- **Very large images (>12MP)**: Varies (use tiling)

## Next Steps

- Read the [API Documentation](./api.md) for detailed API reference
- Check [Performance Guide](./performance.md) for optimization tips
- See [Architecture](./architecture.md) for understanding the internals
- Visit [Troubleshooting](./troubleshooting.md) for common issues

## Support

- GitHub Issues: https://github.com/yourusername/pca-compressor/issues
- Documentation: https://github.com/yourusername/pca-compressor/tree/main/docs
- Discussions: https://github.com/yourusername/pca-compressor/discussions