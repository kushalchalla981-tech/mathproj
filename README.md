# PCA Compressor

<p align="center">
  <a href="https://github.com/kushalchalla981-tech/mathproj/actions">
    <img src="https://github.com/kushalchalla981-tech/mathproj/workflows/CI/badge.svg" alt="Build Status">
  </a>
  <a href="https://crates.io/crates/pca-core">
    <img src="https://img.shields.io/crates/v/pca-core?style=flat-square" alt="Crate Version">
  </a>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License">
</p>

<p align="center">
  <code>Eigenvalue · Eigenvector · Principal Component Analysis</code>
</p>

---

## About

**PCA Compressor** is a research-grade image compression tool using Principal Component Analysis. Unlike traditional codecs, it provides transparent, explainable compression with tunable parameters and scientific quality metrics.

```
Image → Mean Centering → Covariance Matrix → Eigen-Decomposition → Projection → Reconstruction
```

### Key Features

| Feature | Description |
|---------|-------------|
| **PCA Compression** | Dimensionality reduction via eigen-decomposition |
| **Eigen Analysis** | Real-time eigenvalue/eigenvector visualization |
| **Auto Orientation** | Smart correction using principal axis detection |
| **Quality Metrics** | SSIM and PSNR for perceptual assessment |
| **Tile Processing** | Memory-safe handling of large images |
| **Multi-Platform** | CLI, GUI, Python bindings, and Library API |

---

## Quick Start

### Web Interface

Visit **[mathproj-ecru.vercel.app](https://mathproj-ecru.vercel.app)** to use the browser-based compression tool.

### CLI

```bash
# Clone and build
git clone https://github.com/kushalchalla981-tech/mathproj.git
cd mathproj/pca-compressor
cargo build --release

# Compress an image
./target/release/pca-compress single input.jpg --output output.jpg --quality 0.7
```

### Python

```bash
pip install pca-compressor
```

```python
from pca_compressor import compress_file, CompressionParams

params = CompressionParams(quality=0.7, mode='per-channel')
result = compress_file('input.jpg', 'output.jpg', params)
print(f"SSIM: {result.ssim:.3f}, Compression: {result.compression_ratio:.2f}x")
```

---

## Mathematical Foundation

### The PCA Algorithm

Principal Component Analysis transforms data into a coordinate system where the greatest variance lies along the first principal component.

**Covariance Matrix:**
```
C = (X - μ)ᵀ(X - μ) / (n-1)
```

**Eigen-Decomposition:**
```
C·v = λ·v
```

**Projection:**
```
Y = X·Vₖ
```

**Reconstruction:**
```
X̂ = Y·Vₖᵀ + μ
```

### Variance Explained

The eigenvalues quantify how much variance each principal component captures:

```
λ₁ ≥ λ₂ ≥ λ₃ ≥ ... ≥ 0

σ²ᵢ = λᵢ / Σλⱼ × 100%
```

---

## Usage Examples

### Single Image

```bash
pca-compress single input.jpg --output output.jpg --quality 0.7
```

### Batch Processing

```bash
pca-compress batch ./photos/ -o ./compressed/ --report results.csv
```

### With Eigen Analysis

```bash
# View eigenvalue decomposition
pca-compress analyze input.jpg --format json
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Client Layer     │  CLI  │  GUI  │  Python Bindings       │
├─────────────────────────────────────────────────────────────┤
│  Service Layer    │  Compression  │  Orientation  │ Metrics │
├─────────────────────────────────────────────────────────────┤
│  Core Compute     │  PCA Engine  │  Eigen Analysis │ Tiles  │
├─────────────────────────────────────────────────────────────┤
│  Utility Layer    │  Image I/O  │  EXIF  │  Encoding        │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
pca-compressor/
├── crates/
│   ├── pca-core/         # Core compression library (Rust)
│   ├── pca-cli/          # Command-line interface
│   ├── pca-py/           # Python bindings (PyO3)
│   └── pca-gui/          # Desktop GUI (Tauri)
├── web/                  # Browser-based interface
├── python-proto/         # Python reference implementation
├── tests/                # Test suite
└── docs/                 # Documentation
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [PRD.txt](PRD.txt) | Product Requirements - feature specifications |
| [TRD.txt](TRD.txt) | Technical Requirements - architecture details |
| [UI.txt](UI.txt) | UI Design - interface specifications |
| [problem.txt](problem.txt) | Algorithm description |

---

## Performance

| Image Size | Processing Time | Memory | Quality Target |
|------------|-----------------|--------|----------------|
| Small (<1MP) | <100ms | ~50MB | SSIM > 0.95 |
| Medium (1-4MP) | 500ms-2s | ~200MB | SSIM > 0.90 |
| Large (4-12MP) | 2-5s | ~500MB | SSIM > 0.85 |
| Very Large (>12MP) | Variable | Configurable | SSIM > 0.80 |

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with **Rust**, **Python**, and **Linear Algebra**
