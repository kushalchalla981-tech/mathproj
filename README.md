# PCA Image Compression Tool

A PCA-based image compression and orientation correction tool with CLI, GUI, and library interfaces.

## Overview

This tool uses Principal Component Analysis on image pixel data to compress images while preserving visual information. It automatically corrects image orientation using either the principal axis from PCA or EXIF metadata.

## Features

- **PCA Compression**: Per-channel and joint-channel modes
- **Orientation Correction**: PCA-based or EXIF metadata
- **Tile Processing**: Handles large images efficiently
- **Quality Metrics**: SSIM and PSNR reporting
- **Multi-Platform**: CLI and GUI available
- **Python Bindings**: For easy integration

## Getting Started

### Prerequisites

- Rust (for building CLI and core library)
- Python 3.8+ (for Python bindings)
- Cargo (Rust package manager)

### Installation

#### Option 1: From Source

```bash
# Clone and build
cd pca-compressor
git clone <repository-url>
cd pca-compressor
cargo build --release
```

#### Option 2: Python Package

```bash
pip install pca-compressor
```

## Usage

### CLI

```bash
# Compress single image
pca-compress single input.jpg -o output.jpg --quality 0.7 --mode per-channel

# Batch processing
pca-compress batch ./input/ -o ./output/ --report csv

# Validate image
pca-compress validate image.jpg
```

### Python

```python
from pca_compressor import compress

# Compress image
result = compress(
    image_bytes,
    mode='per-channel',
    quality=0.7,
    retain_components=1,
    orientation='auto'
)

print(f"SSIM: {result.ssim:.3f}, Compression: {result.compression_ratio:.2f}x")
```

## Configuration

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `quality` | float | 0.7 | Compression strength (0.1-1.0) |
| `mode` | enum | per-channel | PCA processing mode |
| `retain_components` | int | 1 | Number of PCA components to keep |
| `orientation` | enum | auto | Orientation handling method |
| `tile_size` | int | 1024 | Tile dimensions for large images |

### Modes

- **Per-channel**: Process R, G, B independently
- **Joint-channel**: Treat RGB as 3D vectors

## Performance

- Processes 12MP images in <5 seconds
- Memory usage configurable (default: 1GB)
- Multi-threaded batch processing
- Tile-based processing for very large images

## Quality

- Target SSIM > 0.90 at default settings
- PSNR calculated for technical quality assessment
- Visual quality preserved in most cases

## Development

### Building

```bash
# Build CLI
cargo build --release

# Build Python bindings
cd crates/pca-py
maturin develop
```

### Testing

```bash
# Run tests
cargo test

# Run Python tests
pytest
```

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Client Layer    │  CLI │  GUI │  External Applications     │
├─────────────────────────────────────────────────────────────┤
│  Service Layer   │  Compression Service │ Quality Metrics    │
├─────────────────────────────────────────────────────────────┤
│  Core Compute    │  PCA Engine │ Eigen-decomp │ Tile Proc    │
├─────────────────────────────────────────────────────────────┤
│  Utility Layer   │  Image I/O │ EXIF │ Logging │ Encoding  │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Support

- [Documentation](docs/README.md)
- [Issues](https://github.com/yourusername/pca-compressor/issues)
- [Discussions](https://github.com/yourusername/pca-compressor/discussions)

## Related Projects

- [scikit-image](https://scikit-image.org/) - Image processing in Python
- [OpenCV](https://opencv.org/) - Computer vision library
- [Eigen](http://eigen.tuxfamily.org/) - C++ template library for linear algebra