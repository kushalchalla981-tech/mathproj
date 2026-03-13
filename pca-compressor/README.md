# PCA Compressor

A powerful, efficient image compression tool based on Principal Component Analysis (PCA) with automatic orientation correction.

## Features

- **PCA-based Compression**: Reduce file size while preserving visual quality
- **Orientation Correction**: Automatic orientation detection using PCA and EXIF metadata
- **Tile Processing**: Efficiently handle large images with memory-safe tiling
- **Quality Metrics**: SSIM and PSNR metrics for quality assessment
- **Multiple Interfaces**: CLI, Python bindings, and GUI
- **Cross-platform**: Works on Linux, macOS, and Windows

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/yourusername/pca-compressor.git
cd pca-compressor
cargo build --release

# Python package
pip install pca-compressor
```

### CLI Usage

```bash
# Compress single image
cargo run --release --bin pca-compress single input.jpg --quality 0.7

# Batch processing
cargo run --release --bin pca-compress batch ./photos/ -o ./compressed/ --report results.csv

# Validate image
cargo run --release --bin pca-compress validate image.jpg
```

### Python Usage

```python
from pca_compressor import compress_file, CompressionParams

params = CompressionParams(quality=0.7)
result = compress_file('input.jpg', 'output.jpg', params)
print(f"SSIM: {result.ssim:.3f}, Compression: {result.compression_ratio:.2f}x")
```

## Architecture

The project uses a modular workspace structure:

```
pca-compressor/
├── crates/
│   ├── pca-core/        # Core compression library (Rust)
│   ├── pca-cli/         # Command line interface (Rust)
│   ├── pca-py/          # Python bindings (PyO3)
│   └── pca-gui/         # Desktop application (Tauri)
├── tests/               # Integration and performance tests
├── docs/                # Documentation
└── python-proto/        # Python reference implementation
```

### Core Modules

- **Image Loader**: JPEG/PNG decoding, EXIF extraction, color space conversion
- **Preprocessing**: Mean centering, channel splitting, tile segmentation
- **PCA Engine**: Covariance computation, eigen-decomposition, projection/reconstruction
- **Tile Processing**: Parallel tile processing for large images
- **Orientation Correction**: PCA-based and EXIF-based orientation detection
- **Quality Metrics**: SSIM and PSNR calculation
- **Encoding**: JPEG/PNG output generation

## Compression Modes

### Per-Channel Mode

Processes each RGB channel independently using 1D PCA:
- Faster processing
- Good for natural photos
- Lower memory usage

### Joint-Channel Mode

Treats RGB color vectors as 3D data:
- Better color preservation
- Higher compression potential
- Slightly higher computational cost

## Parameters

| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| `quality` | 0.1-1.0 | 0.7 | Compression strength (higher = better quality) |
| `mode` | per-channel, joint-channel | per-channel | PCA processing mode |
| `retain_components` | 1-3 | 1 | Number of PCA components to keep |
| `orientation` | auto, exif, disabled | auto | Orientation correction method |
| `tile_size` | 64-2048 | 1024 | Tile size for large images |
| `max_memory` | 128-4096 MB | 1024 MB | Memory limit per process |

## Performance

- **Small images** (<1MP): <100ms
- **Medium images** (1-4MP): 500ms-2s
- **Large images** (4-12MP): 2-5s
- **Very large images** (>12MP): Variable (use tiling)

Quality targets:
- SSIM > 0.90 at default quality (0.7)
- Deterministic processing: same input → same output

## Error Handling

```rust
use pca_core::error::CompressionError;

match compress(&image, &params) {
    Ok(result) => { /* success */ },
    Err(CompressionError::MemoryLimitExceeded { required, available }) => {
        // Enable tiling or reduce quality
    },
    Err(CompressionError::ImageTooSmall { width, height }) => {
        // Return original or use lossless mode
    },
    // ... other error types
}
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test integration_test

# Run benchmarks (requires --ignore flag or nightly)
cargo test --ignored

# Test Python bindings
cd crates/pca-py && maturin develop && pytest
```

## Development

### Building

```bash
# Build all components
./build.sh --all

# Build with release optimizations
./build.sh --release

# Build specific component
./build.sh --cli
./build.sh --python
./build.sh --gui
```

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Performance tests
cargo test --test performance_test -- --ignored

# With verbose output
cargo test -- --nocapture
```

### Documentation

```bash
# Generate documentation
cargo doc --open

# Documentation for specific module
cargo doc --open -p pca-core
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run tests (`cargo test`)
6. Format code (`cargo fmt`)
7. Run linter (`cargo clippy`)
8. Commit changes (`git commit -m 'Add amazing feature'`)
9. Push to branch (`git push origin feature/amazing-feature`)
10. Create a Pull Request

## Code Style

- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Add documentation comments (`///`) for public APIs
- Keep functions focused and composable
- Handle errors explicitly with `Result`

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust, NumPy, Pillow, PCA mathematics
- Inspired by research in dimensionality reduction for image processing
- Uses quality metrics (SSIM, PSNR) from research literature

## Resources

- [Documentation](https://github.com/yourusername/pca-compressor/tree/main/docs)
- [API Reference](https://github.com/yourusername/pca-compressor/tree/main/docs/API.md)
- [Quick Start](https://github.com/yourusername/pca-compressor/tree/main/docs/QUICKSTART.md)
- [Performance Guide](https://github.com/yourusername/pca-compressor/tree/main/docs/performance.md)
- [Architecture](https://github.com/yourusername/pca-compressor/tree/main/docs/architecture.md)

## Citation

If you use PCA Compressor in research, please cite:

```bibtex
@software{pca_compressor,
  title = {PCA Compressor: An Efficient Image Compression and Orientation Correction Tool},
  author = {PCA Compressor Team},
  year = {2025},
  url = {https://github.com/yourusername/pca-compressor}
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.