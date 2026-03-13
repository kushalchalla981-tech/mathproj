# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **design and planning repository** for a PCA-based Image Compression and Orientation Correction tool (V1). The project is currently in the specification phase with PRD, TRD, and UI design documents. No implementation code exists yet.

### Core Algorithm
The compression tool uses Principal Component Analysis on image pixel data:
1. Compute covariance matrix of mean-centered pixel data
2. Perform eigen-decomposition to identify principal axes of variance
3. Project multi-dimensional image data onto dominant eigenvector for dimensionality reduction
4. Reconstruct image while preserving maximum visual information

### Target Deliverables (V1)
- **CLI**: Single-image and batch compression with CSV reporting
- **GUI**: Minimal desktop app with side-by-side preview and compression slider
- **Library**: C++ or Rust core with language bindings
- **API**: Optional HTTP service mode (local only by default)

## Document Reference
Key specification files in this repository:

| File | Purpose |
|------|---------|
| `PRD.txt` | Product Requirements Document - user flows, functional requirements, acceptance criteria |
| `TRD.txt` | Technical Requirements Document - system architecture, module design, API spec |
| `UI.txt` | UI/UX Design - screen layouts, component specs, interaction flows |
| `problem.txt` | Core problem statement and algorithm description |

## Architecture Summary

### Technology Stack (Planned)

| Layer | Technology |
|-------|------------|
| Core engine | C++ or Rust |
| Prototype | Python (NumPy, Pillow, scikit-image) |
| GUI | Qt (preferred) or Electron |
| CLI | Native binary |
| Image I/O | OpenCV, libjpeg-turbo |
| Linear Algebra | Eigen or LAPACK |
| Storage | SQLite (optional, for batch reports) |

### System Layers
1. **Client Layer**: CLI, GUI, external library consumers
2. **Service Layer**: Compression service wrapper, orientation detection, quality metrics
3. **Core Compute Layer**: PCA compression engine, eigen-decomposition, tile processing
4. **Utility Layer**: Image loader/decoder, metadata processor, logging

### Processing Pipeline
```
Image Load → EXIF Read → RGB Conversion → Tile Splitting → Mean Centering →
Covariance Matrix → Eigen-decomposition → Dominant Eigenvector Projection →
Reconstruction → Orientation Correction → Quality Metrics (SSIM/PSNR) → Encode Output
```

## Key Technical Constraints

### PCA Modes
- **Per-channel**: RGB processed independently
- **Joint-channel**: Color vector treated as single multi-dimensional data

### Orientation Correction
- Primary: Principal axis from PCA
- Fallback: EXIF orientation metadata
- Override: User can force EXIF mode

### Tile Processing
- Default tile size: 1024x1024
- Prevents memory spikes on large images
- Essential for images >12MP on memory-constrained systems

### Supported Formats
- Input: JPEG, PNG
- Output: JPEG (default for lossy), PNG
- Color spaces: RGB (CMYK converted to RGB with warning)
- Alpha channel: Preserved separately (PCA applied to RGB only)

## API Signature (Target)

```python
compress(
    image_bytes,
    mode='per-channel',           # 'per-channel' | 'joint-channel'
    retain_components=1,          # Integer component count
    quality=0.7,                  # 0.1 - 1.0 compression strength
    orientation='auto',           # 'auto' | 'exif' | 'disabled'
    tile_size=1024
) -> {
    bytes: compressed_bytes,
    size_before: int,
    size_after: int,
    compression_ratio: float,
    ssim: float,
    orientation_used: 'pca' | 'exif'
}
```

## CLI Interface (Target)

### Single Image
```bash
compress input.jpg --output output.jpg --quality 0.7 --mode per-channel --orientation auto
```

### Batch Processing
```bash
compress --batch /input/folder --output /output/folder --quality 0.7 --report csv
```

### Key Flags
- `--quality`: Compression strength 0.1-1.0
- `--mode`: `per-channel` or `joint-channel`
- `--retain-components`: Number of PCA components to keep
- `--tile-size`: Tile dimensions (default 1024)
- `--max-ram`: Memory limit override
- `--orientation`: `auto`, `exif`, or `disabled`

## Edge Cases to Handle
- Images <64x64: Return original or use lossless mode
- Monochrome/single-color: Detect low-rank data, minimal components
- Transparent images: Preserve alpha separately
- Corrupted images: Validate and reject with clear error
- CMYK images: Convert to RGB, document conversion loss
- Animated GIFs: Out of scope for V1 (error or first-frame only)
- EXIF/PCA conflict: Show both options, default to PCA for correctness
- Extremely large images: Auto-tile with seam-aware blending

## Quality Metrics
- **SSIM**: Primary perceptual quality metric (target >0.90 at default settings)
- **PSNR**: Secondary technical metric
- Compression ratio reported for all outputs

## Performance Targets
- Process 12MP image within desktop hardware budget
- GUI preview refresh <500ms for usability
- Memory limit configurable per process
- Multi-threaded batch processing (worker pool = CPU cores - 1)

## Testing Requirements
- Unit tests for PCA math correctness
- Integration tests for full compression pipeline
- Stress tests for large images (12MP+)
- Batch pipeline tests
- Alpha channel handling tests
- EXIF vs PCA orientation tests

## Implementation Phases (Roadmap)

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 0 | Weeks 0-4 | Algorithm prototype, CLI single-image, basic tests |
| Phase 1 | Weeks 4-8 | GUI preview, batch CLI, library API, tile processing |
| Phase 2 | Weeks 8-12 | Performance optimization, platform builds, documentation |
| Phase 3 | Post-V1 | Hybrid modes, advanced heuristics, plugin integrations |

## Important Notes

- **Local-only by default** - no cloud dependency
- **Deterministic processing** - identical inputs produce identical outputs
- **Security**: Local processing only; API mode requires explicit authentication
- **Positioning**: Experimental/explainable compressor, not a replacement for JPEG/WebP

## Implementation Status

**Phase 1: Core Algorithm Foundation** - ✓ Completed
- Core Rust library (pca-core) with all modules
- Image loading/saving (JPEG/PNG support)
- PCA compression algorithm (per-channel and joint-channel)
- Quality metrics (SSIM, PSNR)
- Orientation correction (PCA + EXIF)
- Tile processing for large images
- CLI implementation (pca-cli)

**Phase 2: Advanced Features** - ✓ Completed
- Enhanced CLI with all command-line flags
- Batch processing with multi-threading
- CSV report generation
- Comprehensive error handling
- Test infrastructure (unit, integration, performance)
- Python bindings (pca-py) via PyO3
- GUI application (pca-gui) via Tauri

**Build Commands:**
```bash
# Build all components
cd pca-compressor
./build.sh --release --all

# Build specific components
./build.sh --release --cli      # CLI tool
./build.sh --release --python   # Python bindings
./build.sh --release --gui      # Desktop GUI

# Run tests
cargo test

# Build Python package
cd crates/pca-py
maturin build --release
pip install target/wheels/*.whl
```

**Directory Structure:**
```
pca-compressor/
├── crates/
│   ├── pca-core/         # Core library
│   ├── pca-cli/          # CLI application
│   ├── pca-py/           # Python bindings
│   └── pca-gui/          # Desktop GUI
├── tests/                # Test suite
├── docs/                 # Documentation
└── python-proto/         # Python reference implementation
```