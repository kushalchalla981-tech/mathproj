# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**PCA Compressor** is a research-grade image compression tool using Principal Component Analysis. It provides transparent, explainable compression with tunable parameters and scientific quality metrics.

### Core Algorithm

The compression tool uses Principal Component Analysis on image pixel data:
1. Compute covariance matrix of mean-centered pixel data
2. Perform eigen-decomposition to identify principal axes of variance
3. Project multi-dimensional image data onto dominant eigenvector for dimensionality reduction
4. Reconstruct image while preserving maximum visual information

### Eigen Analysis Feature

The project includes eigenvalue/eigenvector analysis with:
- Real-time visualization of principal axis on images
- User-selectable overlay colors (Red, Yellow, Cyan, Green, Magenta)
- Scientific notation for eigenvalues
- Variance explained calculations

### Target Deliverables

| Component | Description |
|-----------|-------------|
| CLI | Single-image and batch compression with CSV reporting |
| GUI | Minimal desktop app with side-by-side preview and compression slider |
| Web | Browser-based interface with eigen analysis panel |
| Library | Rust core with Python bindings |

## Architecture

```
Client Layer     → CLI, GUI, Python Bindings
Service Layer    → Compression, Orientation, Metrics
Core Compute     → PCA Engine, Eigen Analysis, Tile Processing
Utility Layer    → Image I/O, EXIF, Encoding
```

### Technology Stack

| Layer | Technology |
|-------|------------|
| Core engine | Rust |
| Python bindings | PyO3 |
| Desktop GUI | Tauri |
| Linear Algebra | nalgebra |
| Web | JavaScript, Canvas API |

## Key Technical Constraints

### PCA Modes
- **Per-channel**: RGB processed independently
- **Joint-channel**: Color vector treated as single multi-dimensional data

### Orientation Correction
- Primary: Principal axis from PCA
- Fallback: EXIF orientation metadata
- Override: User can force EXIF mode

### Tile Processing
- Default tile size: 1024×1024
- Prevents memory spikes on large images
- Essential for images >12MP on memory-constrained systems

## Processing Pipeline

```
Image Load → EXIF Read → RGB Conversion → Tile Splitting → Mean Centering →
Covariance Matrix → Eigen-decomposition → Dominant Eigenvector Projection →
Reconstruction → Orientation Correction → Quality Metrics (SSIM/PSNR) → Encode Output
```

## Build Commands

```bash
# Build all components
cd pca-compressor
cargo build --release --all

# Build specific components
cargo build --release -p pca-core    # Core library
cargo build --release -p pca-cli     # CLI tool
cargo build --release -p pca-py     # Python bindings
cargo build --release -p pca-gui    # Desktop GUI

# Run tests
cargo test --all

# Build Python package
cd crates/pca-py
maturin build --release
pip install target/wheels/*.whl
```

## Directory Structure

```
pca-compressor/
├── crates/
│   ├── pca-core/         # Core library
│   ├── pca-cli/          # CLI application
│   ├── pca-py/           # Python bindings
│   └── pca-gui/          # Desktop GUI
├── tests/                # Test suite
├── docs/                 # Documentation
├── web/                  # Web interface
└── python-proto/         # Python reference implementation
```

## Implementation Status

**Phase 1: Core Algorithm Foundation** - ✓ Complete
**Phase 2: Advanced Features** - ✓ Complete
**Phase 2b: Eigen Analysis Feature** - ✓ Complete

## Important Notes

- **Local-only by default** - no cloud dependency
- **Deterministic processing** - identical inputs produce identical outputs
- **Security**: Local processing only
- **Positioning**: Experimental/explainable compressor, not a replacement for JPEG/WebP
