# PCA Image Compression - Implementation Plan

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Building a full-stack PCA-based image compression tool with:
- **Core Algorithm**: Principal Component Analysis on image pixel data
- **CLI**: Command-line interface for single and batch processing
- **GUI**: Desktop application with live preview
- **Library**: Reusable API with language bindings

---

## Architecture Overview

### System Layers

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

### Technology Stack

| Layer | Technology |
|-------|------------|
| Core Engine | Rust (with nalgebra, image crates) |
| Python Bindings | PyO3 |
| CLI | Rust (clap) |
| GUI | Tauri (Rust backend + Web frontend) |
| Image Processing | image crate, turbojpeg |
| Linear Algebra | nalgebra |
| Build System | Cargo |
| Testing | cargo test, pytest |
| CI/CD | GitHub Actions |

---

## Phase 1: Foundation - Core Algorithm

**Goal**: Working Rust implementation of PCA compression

### Tasks

#### 1.1 Project Structure
```
/pca-compressor
  /Cargo.toml           # Workspace root
  /crates
    /pca-core          # Core library
    /pca-cli           # CLI binary
    /pca-gui           # GUI binary
    /pca-py            # Python bindings
  /assets              # Test images
  /tests               # Integration tests
  /docs                # Documentation
```

#### 1.2 Core Library - pca-core
- **Image Loading Module**
  - JPEG/PNG decoding
  - EXIF metadata extraction
  - Color space conversion (CMYK → RGB)
  - Alpha channel separation

- **Preprocessing Module**
  - Mean centering of pixel data
  - Channel splitting (R, G, B)
  - Tile segmentation
  - Data normalization

- **PCA Module**
  - Covariance matrix computation
  - Eigen-decomposition (nalgebra)
  - Dominant eigenvector identification
  - Projection and reconstruction
  - Per-channel and joint-channel modes

- **Quality Metrics Module**
  - SSIM calculation
  - PSNR calculation
  - Compression ratio

- **Encoding Module**
  - JPEG encoding with quality control
  - PNG encoding
  - EXIF preservation/stripping

#### 1.3 Initial CLI Skeleton
- Argument parsing with clap
- Single image compression
- Basic output display

### Deliverables
- [ ] Rust workspace structure
- [ ] pca-core library with all modules
- [ ] Basic CLI that compresses single images
- [ ] Unit tests for PCA math
- [ ] Test images with verified outputs

---

## Phase 2: Advanced Features

**Goal**: Production-ready CLI with full feature set

### Tasks

#### 2.1 Enhanced CLI
**Command Structure:**
```bash
pca-compress single input.jpg -o output.jpg [OPTIONS]
pca-compress batch ./input/ -o ./output/ [OPTIONS]
```

**Full Flag Support:**
- `--quality` (0.1-1.0): Compression strength
- `--mode`: `per-channel` or `joint-channel`
- `--retain-components`: Number of components to keep
- `--orientation`: `auto`, `exif`, or `disabled`
- `--tile-size`: Tile dimensions (default 1024)
- `--max-ram`: Memory limit
- `--format`: Output format `jpeg` or `png`
- `--strip-metadata`: Remove EXIF
- `--verbose`: Detailed logging

#### 2.2 Batch Processing
- Folder scanning with file filtering
- Multi-threaded worker pool
- Progress bars with indicatif
- CSV report generation:
  ```csv
  filename,original_size,compressed_size,ratio,ssim,processing_time_ms,orientation_method
  ```

#### 2.3 Tile Processing Engine
- Automatic tile splitting for large images
- Configurable tile size
- Seam-aware stitching
- Memory usage control

#### 2.4 Orientation Correction
- Principal axis calculation from PCA eigenvectors
- EXIF orientation reading
- Auto mode with confidence-based fallback
- Image rotation implementation

#### 2.5 Error Handling
- Custom error types for each failure mode
- Graceful handling of:
  - Corrupted images
  - Unsupported formats
  - Memory limit exceeded
  - Images too small (<64x64)
  - Monochrome images

### Deliverables
- [ ] Full-featured CLI with all flags
- [ ] Batch processing with progress and CSV reports
- [ ] Tile processing for memory-constrained environments
- [ ] Comprehensive error handling with helpful messages

---

## Phase 3: Performance & Bindings

**Goal**: Optimized core with Python bindings

### Tasks

#### 3.1 Performance Optimization
- Parallel processing with rayon
- SIMD optimizations where applicable
- Streaming covariance for large images
- Memory pool management
- Benchmarking suite

#### 3.2 Python Bindings - pca-py
- PyO3 integration
- Pythonic API matching target signature:
  ```python
  from pca_compressor import compress

  result = compress(
      image_bytes,
      mode='per-channel',
      retain_components=1,
      quality=0.7,
      orientation='auto',
      tile_size=1024
  )
  ```

#### 3.3 Library Testing
- Python binding tests
- Integration tests between Python and Rust
- Performance comparison

### Deliverables
- [ ] Optimized core library
- [ ] Python package with bindings
- [ ] Performance benchmarks meeting targets
- [ ] Published to crates.io and PyPI

---

## Phase 4: GUI Application

**Goal**: Cross-platform desktop GUI with live preview

### Tasks

#### 4.1 Framework Setup
- Tauri configuration
- Frontend: React or Vue with TypeScript
- Image preview component (WebGL for performance)

#### 4.2 Home Screen
- Drag-and-drop upload zone
- File picker button
- Recent files list (thumbnails, names, timestamps)
- Supported formats note
- Error display for invalid uploads

#### 4.3 Image Workspace
- Split-pane layout:
  - Left: Original image
  - Right: Compressed preview
- Pan and zoom controls
- Sync zoom option
- Compression summary strip:
  - Original size
  - Estimated output size
  - Compression ratio
  - SSIM score
  - Processing time

#### 4.4 Compression Controls Panel
- **Compression Strength**: Slider (0.1-1.0)
- **PCA Mode**: Radio buttons (per-channel/joint)
- **Retained Components**: Number input
- **Orientation**: Dropdown (auto/EXIF/disabled)
- **Tile Processing**: Toggle + size input
- **Live Preview**: Updates on parameter change (<500ms target)

#### 4.5 Export Dialog
- Output format selection
- Metadata handling options
- File destination picker
- Export summary
- Success/error feedback

#### 4.6 Settings Screen
- Max RAM per process
- Thread count
- Default quality metric
- Default export format
- Auto-recompute toggle
- Settings persistence

#### 4.7 Batch Report Viewer
- Sortable data table
- Row selection
- Preview panel
- CSV export button

### Deliverables
- [ ] Cross-platform GUI (Windows, macOS, Linux)
- [ ] Live preview with <500ms refresh
- [ ] All screens implemented
- [ ] Settings persistence

---

## Phase 5: Integration & Delivery

**Goal**: Complete product with testing, docs, and distribution

### Tasks

#### 5.1 Test Suite
**Unit Tests:**
- PCA math correctness
- Covariance computation
- Eigen-decomposition
- Quality metrics

**Integration Tests:**
- Full compression pipeline
- CLI commands
- Batch processing
- Error scenarios

**Edge Cases:**
- Small images (<64x64)
- Monochrome images
- Images with alpha channel
- CMYK images
- Corrupted images
- Very large images (tiling)

**Performance Tests:**
- 12MP image processing
- Memory profiling
- Batch throughput

#### 5.2 Documentation
- **API Documentation**: Rust docs, Python docs
- **CLI Guide**: Usage examples, flag reference
- **GUI Manual**: User guide with screenshots
- **Algorithm Documentation**: Math explanation
- **Troubleshooting**: Common issues and solutions

#### 5.3 Packaging & Distribution

**CLI:**
- Binary releases for Windows, macOS, Linux
- Install via cargo install
- Homebrew formula (macOS)
- Chocolatey/scoop (Windows)

**Python Library:**
- PyPI package with wheels
- maturin build system
- pip install pca-compressor

**GUI:**
- Windows: MSI installer
- macOS: DMG bundle
- Linux: AppImage and deb/rpm packages

#### 5.4 CI/CD Pipeline
- GitHub Actions workflows:
  - Test on push
  - Build binaries on release
  - Multi-platform testing
  - Automated releases

#### 5.5 Optional HTTP API
- FastAPI service (separate crate)
- Single image endpoint
- Batch job submission
- Job status endpoint
- API key authentication

### Deliverables
- [ ] Comprehensive test suite
- [ ] Complete documentation
- [ ] Distributed packages for all platforms
- [ ] CI/CD pipeline
- [ ] Optional HTTP API

---

## Implementation Details

### Core Algorithm Flow

```rust
// Pseudo-code for compression
fn compress(image: &Image, params: &Params) -> Result<CompressedImage> {
    // 1. Load and validate
    let rgb = image.to_rgb()?;

    // 2. Split into tiles if needed
    let tiles = rgb.split_tiles(params.tile_size)?;

    // 3. Process each tile
    let compressed_tiles: Vec<Tile> = tiles.par_iter()
        .map(|tile| process_tile(tile, params))
        .collect()?;

    // 4. Stitch tiles
    let compressed = stitch_tiles(compressed_tiles)?;

    // 5. Orientation correction
    let oriented = correct_orientation(compressed, &params.orientation)?;

    // 6. Calculate metrics
    let ssim = calculate_ssim(&rgb, &oriented)?;

    // 7. Encode output
    let output = encode(&oriented, params.format, params.quality)?;

    Ok(CompressedImage {
        bytes: output,
        metrics: Metrics { ssim, ratio: ..., time: ... }
    })
}

fn process_tile(tile: &Tile, params: &Params) -> Result<Tile> {
    // 1. Mean center
    let centered = mean_center(tile)?;

    // 2. Compute covariance
    let cov = compute_covariance(&centered, params.mode)?;

    // 3. Eigen-decomposition
    let (eigenvalues, eigenvectors) = eigendecompose(&cov)?;

    // 4. Project onto dominant eigenvectors
    let projected = project(&centered, &eigenvectors, params.retain_components)?;

    // 5. Reconstruct
    let reconstructed = reconstruct(&projected, &eigenvectors)?;

    Ok(reconstructed)
}
```

### Error Types

```rust
enum CompressionError {
    InvalidFormat { path: String },
    CorruptedImage { path: String },
    ImageTooSmall { width: u32, height: u32 },
    MemoryLimitExceeded { required: usize, available: usize },
    OrientationConflict { pca: f32, exif: u8 },
    InvalidParams { field: String, value: String },
}
```

### Key Data Structures

```rust
struct CompressionParams {
    quality: f32,                    // 0.1-1.0
    mode: PCAMode,                   // PerChannel | JointChannel
    retain_components: usize,
    orientation: OrientationMode,    // Auto | Exif | Disabled
    tile_size: Option<u32>,
    max_memory: Option<usize>,
    output_format: OutputFormat,     // Jpeg | Png
    strip_metadata: bool,
}

struct CompressionResult {
    image_data: Vec<u8>,
    metrics: CompressionMetrics,
    orientation_used: OrientationMethod,
}

struct CompressionMetrics {
    original_size: usize,
    compressed_size: usize,
    ratio: f32,
    ssim: f32,
    processing_time_ms: u64,
}
```

---

## Testing Strategy

### Test Categories

1. **Unit Tests**: Individual functions, edge cases
2. **Integration Tests**: Full pipeline scenarios
3. **Property Tests**: Invariants (SSIM always between 0-1)
4. **Performance Tests**: Benchmarks against targets
5. **Visual Tests**: SSIM validation on sample images

### Test Images

| Category | Size | Purpose |
|----------|------|---------|
| Small | 32x32 | Edge case: too small |
| Standard | 1920x1080 | Typical use case |
| Large | 12MP (4032x3024) | Performance target |
| Very Large | 50MP | Tile processing test |
| Monochrome | Various | Special handling |
| With Alpha | Various | Alpha preservation |
| CMYK | Various | Color space conversion |

---

## Success Criteria

### Algorithm
- [ ] SSIM > 0.90 at default compression (quality=0.7)
- [ ] Deterministic: same input produces same output
- [ ] Per-channel and joint-channel modes both functional

### Performance
- [ ] 12MP image processed within desktop budget (<5 seconds)
- [ ] Batch processing utilizes all CPU cores efficiently
- [ ] Memory usage stays within configured limits

### Quality
- [ ] All edge cases handled gracefully
- [ ] Error messages are clear and actionable
- [ ] Visual quality acceptable on test panel

### Product
- [ ] CLI installs and runs on Windows, macOS, Linux
- [ ] GUI builds and runs on all platforms
- [ ] Python package installable via pip
- [ ] Documentation complete and clear

---

## Dependencies

### Core (Rust)
- `image` - Image I/O
- `nalgebra` - Linear algebra
- `nshare` - ndarray/nalgebra interop
- `kamadak-exif` - EXIF handling
- `turbojpeg` - Fast JPEG encoding
- `rayon` - Parallel processing
- `thiserror` - Error handling

### CLI
- `clap` - Argument parsing
- `indicatif` - Progress bars
- `csv` - CSV report generation
- `serde` + `serde_yaml` - Config files
- `anyhow` - Error handling

### GUI (Tauri)
- `tauri` - Desktop framework
- `tokio` - Async runtime

### Python Bindings
- `pyo3` - Rust/Python interop
- `maturin` - Build tool
- `numpy` - Python arrays

---

## Notes for Future Claude Instances

1. **Start with Phase 1**: Always begin with the Rust core implementation
2. **Test frequently**: Run tests after every significant change
3. **Benchmark**: Track performance from day one
4. **Document as you go**: Don't leave docs for the end
5. **Handle edge cases early**: Small images, CMYK, etc.
6. **Use feature flags**: For optional components like HTTP API
7. **Keep CLI simple**: GUI is where complexity lives
8. **Validate outputs**: Always check SSIM on test images

## Files to Reference

- `PRD.txt` - Product requirements
- `TRD.txt` - Technical architecture
- `UI.txt` - Interface specifications
- `problem.txt` - Core algorithm description
- `CLAUDE.md` - Project context (this file)

---

*Generated for full-stack implementation of PCA Image Compression tool.*
