# Architecture

## System Overview

The PCA-based Image Compression tool uses a modular, layered architecture to separate concerns and enable maintainability. The system processes images through a pipeline of transformations, from raw input to compressed output.

## Architecture Layers

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

## Core Components

### 1. Image Loader

**Responsibility**: Load and validate image files

**Features**:
- JPEG/PNG decoding
- EXIF metadata extraction
- Color space conversion
- Alpha channel handling
- Size validation

**Implementation**:
```rust
pub struct ImageData {
    width: u32,
    height: u32,
    rgb_data: Vec<f32>,
    alpha_data: Option<Vec<f32>>,
    color_space: ColorSpace,
    exif_orientation: Option<u8>,
}
```

### 2. Preprocessing Module

**Responsibility**: Prepare image data for PCA

**Features**:
- Mean centering
- Channel splitting
- Tile segmentation
- Data normalization

**Implementation**:
```rust
pub struct Preprocessor {
    tile_size: Option<u32>,
    max_memory_mb: Option<usize>,
}

impl Preprocessor {
    pub fn process(&self, image: &ImageData) -> Result<PreprocessedData> {
        // Mean centering
        // Tile splitting if needed
        // Channel separation
    }
}
```

### 3. PCA Compression Engine

**Responsibility**: Perform PCA and compression

**Features**:
- Covariance matrix computation
- Eigen-decomposition
- Projection and reconstruction
- Per-channel and joint-channel modes

**Implementation**:
```rust
pub struct PCAAlgorithm {
    mode: PcaMode,
    retain_components: usize,
}

impl PCAAlgorithm {
    pub fn compress(&self, data: &PreprocessedData) -> Result<CompressedData> {
        // Covariance matrix
        // Eigen-decomposition
        // Projection
        // Reconstruction
    }
}
```

### 4. Tile Processing Engine

**Responsibility**: Handle large images efficiently

**Features**:
- Automatic tile splitting
- Seam-aware stitching
- Memory usage control
- Parallel processing

**Implementation**:
```rust
pub struct TileProcessor {
    tile_size: u32,
    max_memory_mb: usize,
}

impl TileProcessor {
    pub fn process_large_image(&self, image: &ImageData) -> Result<ImageData> {
        let tiles = self.split_into_tiles(image)?;
        let processed_tiles = tiles.par_iter()
            .map(|tile| self.process_tile(tile))
            .collect()?;
        let result = self.stitch_tiles(processed_tiles)?;
        Ok(result)
    }
}
```

### 5. Orientation Correction Engine

**Responsibility**: Correct image orientation

**Features**:
- PCA-based orientation detection
- EXIF metadata reading
- Confidence-based fallback
- Image rotation

**Implementation**:
```rust
pub struct OrientationEngine {
    mode: OrientationMode,
}

impl OrientationEngine {
    pub fn correct_orientation(&self, image: &ImageData, pca_data: &PcaData) -> Result<ImageData> {
        match self.mode {
            OrientationMode::Auto => self.auto_orientation(image, pca_data),
            OrientationMode::Exif => self.exif_orientation(image),
            OrientationMode::Disabled => Ok(image.clone()),
        }
    }
}
```

### 6. Quality Metrics Module

**Responsibility**: Calculate quality metrics

**Features**:
- SSIM calculation
- PSNR calculation
- Compression ratio
- Processing time

**Implementation**:
```rust
pub struct QualityMetrics {
    original: ImageData,
    compressed: ImageData,
}

impl QualityMetrics {
    pub fn calculate_all(&self) -> CompressionMetrics {
        let ssim = calculate_ssim(&self.original, &self.compressed);
        let psnr = calculate_psnr(&self.original, &self.compressed);
        let ratio = calculate_compression_ratio();
        let time = self.processing_time;

        CompressionMetrics {
            ssim,
            psnr,
            compression_ratio: ratio,
            processing_time_ms: time,
        }
    }
}
```

### 7. Encoding Module

**Responsibility**: Save compressed images

**Features**:
- JPEG/PNG encoding
- Quality control
- EXIF preservation
- Metadata handling

**Implementation**:
```rust
pub struct Encoder {
    format: OutputFormat,
    quality: u8,
    strip_metadata: bool,
}

impl Encoder {
    pub fn encode(&self, image: &ImageData, path: &Path) -> Result<() > {
        match self.format {
            OutputFormat::Jpeg => self.encode_jpeg(image, path),
            OutputFormat::Png => self.encode_png(image, path),
        }
    }
}
```

## Data Flow

### Single Image Processing

```
Input Image → Image Loader → Preprocessing → PCA Engine → Orientation → Quality → Encoding → Output
```

### Large Image Processing

```
Input Image → Image Loader → Tile Processor → [Per-Tile: Preprocessing → PCA → Quality] → Stitching → Orientation → Quality → Encoding → Output
```

## Error Handling Architecture

### Error Types

```rust
pub enum CompressionError {
    Io(#[from] std::io::Error),
    ImageFormat(String),
    InvalidParams { field: String, value: String },
    MemoryLimitExceeded { required_mb: f64, available_mb: f64 },
    PcaComputationFailed(String),
    // ... other error types
}
```

### Error Propagation

```rust
pub type Result<T> = std::result::Result<T, CompressionError>;

pub fn compress_image(path: &Path, params: &CompressionParams) -> Result<CompressionMetrics> {
    let image = load_image(path)?;  // Propagate errors
    let preprocessed = preprocess(&image, params)?;
    let pca_result = run_pca(&preprocessed, params)?;
    let oriented = correct_orientation(&pca_result, params)?;
    let metrics = calculate_metrics(&oriented, &image);
    Ok(metrics)
}
```

## Configuration Management

### Parameters

```rust
#[derive(Debug, Clone)]
pub struct CompressionParams {
    quality: f32,
    mode: PcaMode,
    retain_components: usize,
    orientation: OrientationMode,
    tile_size: Option<u32>,
    max_memory_mb: Option<usize>,
    output_format: OutputFormat,
    strip_metadata: bool,
}
```

### Default Values

```rust
impl Default for CompressionParams {
    fn default() -> Self {
        Self {
            quality: 0.7,
            mode: PcaMode::PerChannel,
            retain_components: 1,
            orientation: OrientationMode::Auto,
            tile_size: Some(1024),
            max_memory_mb: Some(1024),
            output_format: OutputFormat::Jpeg,
            strip_metadata: false,
        }
    }
}
```

## Concurrency Model

### Parallel Processing

```rust
use rayon::prelude::*;

pub fn batch_process(files: Vec<PathBuf>, params: CompressionParams) -> Vec<BatchResult> {
    // Set thread count
    let num_threads = (num_cpus::get() - 1).max(1);

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .ok();

    // Process files in parallel
    files.par_iter()
        .map(|file| process_file(file, &params))
        .collect()
}
```

### Thread Safety

```rust
use std::sync::Arc;

pub struct ThreadSafeCompressor {
    core: Arc<Mutex<CoreCompressor>>,
}

impl ThreadSafeCompressor {
    pub fn new() -> Self {
        Self {
            core: Arc::new(Mutex::new(CoreCompressor::new())),
        }
    }

    pub fn compress(&self, image: &ImageData, params: &CompressionParams) -> Result<CompressedData> {
        let mut core = self.core.lock().unwrap();
        core.compress(image, params)
    }
}
```

## Memory Management

### Buffer Management

```rust
pub struct MemoryManager {
    max_memory_mb: usize,
    current_usage_mb: usize,
}

impl MemoryManager {
    pub fn allocate(&mut self, size_mb: usize) -> Result<() > {
        if self.current_usage_mb + size_mb > self.max_memory_mb {
            return Err(CompressionError::MemoryLimitExceeded {
                required_mb: self.current_usage_mb + size_mb,
                available_mb: self.max_memory_mb,
            });
        }
        self.current_usage_mb += size_mb;
        Ok(())
    }

    pub fn deallocate(&mut self, size_mb: usize) {
        self.current_usage_mb = self.current_usage_mb.saturating_sub(size_mb);
    }
}
```

### Resource Cleanup

```rust
impl Drop for CoreCompressor {
    fn drop(&mut self) {
        // Cleanup resources
        self.cleanup_buffers();
        self.release_memory();
        self.close_files();
    }
}

impl Drop for TileProcessor {
    fn drop(&mut self) {
        // Ensure all tiles are processed
        self.finalize_processing();
    }
}
```

## Testing Architecture

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_covariance_matrix() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let cov = compute_covariance(&data);
        assert_approx_eq!(cov, expected_covariance);
    }

    #[test]
    fn test_eigen_decomposition() {
        let matrix = DMatrix::from_row_slice(2, 2, &[1.0, 2.0, 2.0, 1.0]);
        let (eigenvalues, eigenvectors) = eigen_decompose(&matrix);
        assert_approx_eq!(eigenvalues, expected_eigenvalues);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        let image = load_test_image("test.jpg").unwrap();
        let params = CompressionParams::default();
        let result = compress_image(&image, &params).unwrap();

        assert!(result.ssim > 0.8);
        assert!(result.compression_ratio > 1.0);
    }

    #[test]
    fn test_large_image_processing() {
        let image = load_test_image("large.jpg").unwrap();
        let params = CompressionParams::default().tile_size(1024);
        let result = compress_image(&image, &params).unwrap();

        assert!(result.ssim > 0.7); // Lower threshold for tiled processing
    }
}
```

## Extensibility

### Plugin Architecture

```rust
pub trait CompressionPlugin {
    fn name(&self) -> &str;
    fn compress(&self, image: &ImageData, params: &CompressionParams) -> Result<ImageData>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn CompressionPlugin>>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn CompressionPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn compress_with_plugins(
        &self,
        image: &ImageData,
        params: &CompressionParams,
    ) -> Result<ImageData> {
        for plugin in &self.plugins {
            if let Ok(result) = plugin.compress(image, params) {
                return Ok(result);
            }
        }
        Err(CompressionError::NoPluginAvailable)
    }
}
```

### Configuration System

```rust
#[derive(Deserialize)]
pub struct AppConfig {
    compression: CompressionConfig,
    quality: QualityConfig,
    logging: LoggingConfig,
}

#[derive(Deserialize)]
pub struct CompressionConfig {
    default_quality: f32,
    default_mode: String,
    max_memory_mb: usize,
}

impl From<AppConfig> for CompressionParams {
    fn from(config: AppConfig) -> Self {
        Self {
            quality: config.compression.default_quality,
            mode: PcaMode::from_str(&config.compression.default_mode).unwrap(),
            max_memory_mb: Some(config.compression.max_memory_mb),
            // ... other fields
        }
    }
}
```

## Documentation Architecture

### API Documentation

```rust
/// Compress an image using PCA
///
/// This function performs PCA-based compression on the input image.
/// It supports both per-channel and joint-channel processing modes.
///
/// # Arguments
/// * `image` - The input image to compress
/// * `params` - Compression parameters
///
/// # Returns
/// * `Result` containing the compressed image data
///
/// # Errors
/// * `CompressionError` if compression fails
///
/// # Examples
///
/// ```
/// let image = ImageData::from_file("input.jpg")?;
/// let params = CompressionParams::default();
/// let compressed = compress(&image, &params)?;
/// ```
pub fn compress(
    image: &ImageData,
    params: &CompressionParams,
) -> Result<ImageData> {
    // Implementation
}
```

### Architecture Documentation

```rust
/// ## Architecture Overview
///
/// The system uses a layered architecture with clear separation of concerns:
///
/// 1. **Client Layer**: CLI, GUI, and external interfaces
/// 2. **Service Layer**: High-level compression service and metrics
/// 3. **Core Compute Layer**: PCA engine and tile processing
/// 4. **Utility Layer**: Image I/O and encoding
///
/// ## Data Flow
///
/// ```
/// Input Image → Image Loader → Preprocessing → PCA Engine → Orientation → Quality → Encoding → Output
/// ```
///
/// ## Error Handling
///
/// The system uses a Result-based error handling approach with custom error types:
///
/// ```rust
/// pub enum CompressionError {
///     Io(#[from] std::io::Error),
///     ImageFormat(String),
///     InvalidParams { field: String, value: String },
///     // ... other error types
/// }
/// ```
```

## Security Considerations

### Input Validation

```rust
pub fn validate_image(image: &ImageData) -> Result<() > {
    // Check for malicious content
    if image.width > MAX_WIDTH || image.height > MAX_HEIGHT {
        return Err(CompressionError::ImageTooLarge {
            width: image.width,
            height: image.height,
        });
    }

    // Check for corrupted data
    if !is_valid_image_data(&image.rgb_data) {
        return Err(CompressionError::CorruptedImage {
            path: image.source_path.clone().unwrap_or_default(),
            reason: "Invalid pixel data".to_string(),
        });
    }

    Ok(())
}
```

### Memory Safety

```rust
// Use safe Rust for memory safety
pub fn process_image_safe(image: &ImageData) -> Result<ImageData> {
    // Safe Rust operations
    let processed = process_image_logic(image)?;
    Ok(processed)
}

// Use unsafe only when necessary
pub unsafe fn process_image_unsafe(image: *const ImageData) -> Result<ImageData> {
    // Performance-critical operations
    // Must be carefully reviewed for safety
}
```

### Resource Management

```rust
impl Drop for CoreCompressor {
    fn drop(&mut self) {
        // Ensure all resources are cleaned up
        self.cleanup_resources();
        self.release_memory();
        self.close_files();
    }
}

impl Drop for ImageLoader {
    fn drop(&mut self) {
        // Ensure file handles are closed
        self.close_file_handle();
    }
}
```

## Deployment Architecture

### Binary Distribution

```rust
// Build for different platforms
fn build_distributions() {
    // Windows
    cargo build --release --target x86_64-pc-windows-gnu

    // macOS
    cargo build --release --target x86_64-apple-darwin

    // Linux
    cargo build --release --target x86_64-unknown-linux-gnu
}
```

### Python Package

```rust
// Build Python package
fn build_python_package() {
    maturin develop --release
    maturin build --release
}
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /app/target/release/pca-compress /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/pca-compress"]
```

This architecture provides a solid foundation for building a reliable, maintainable, and extensible PCA-based image compression tool.