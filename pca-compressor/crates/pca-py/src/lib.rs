//! Python bindings for PCA image compression library
//!
//! This module provides Python bindings using PyO3 to expose the
//! compression functionality to Python code.

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::fs;
use std::path::Path;
use crate::pca_core::prelude::*;

/// Compression parameters
#[pyclass(name = "CompressionParams")]
#[derive(Clone)]
pub struct PyCompressionParams {
    #[pyo3(get, set)]
    quality: f32,
    #[pyo3(get, set)]
    mode: String,
    #[pyo3(get, set)]
    retain_components: usize,
    #[pyo3(get, set)]
    orientation: String,
    #[pyo3(get, set)]
    tile_size: Option<u32>,
    #[pyo3(get, set)]
    max_memory_mb: Option<usize>,
    #[pyo3(get, set)]
    output_format: String,
    #[pyo3(get, set)]
    strip_metadata: bool,
}

#[pymethods]
impl PyCompressionParams {
    #[new]
    #[pyo3(signature = (
        quality=0.7,
        mode="per-channel".to_string(),
        retain_components=1,
        orientation="auto".to_string(),
        tile_size=None,
        max_memory_mb=None,
        output_format="jpeg".to_string(),
        strip_metadata=false
    ))]
    fn new(
        quality: f32,
        mode: String,
        retain_components: usize,
        orientation: String,
        tile_size: Option<u32>,
        max_memory_mb: Option<usize>,
        output_format: String,
        strip_metadata: bool,
    ) -> Self {
        Self {
            quality,
            mode,
            retain_components,
            orientation,
            tile_size,
            max_memory_mb,
            output_format,
            strip_metadata,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "CompressionParams(quality={}, mode={}, retain_components={}, orientation={})",
            self.quality, self.mode, self.retain_components, self.orientation
        )
    }
}

impl TryFrom<PyCompressionParams> for CompressionParams {
    type Error = PyErr;

    fn try_from(py_params: PyCompressionParams) -> PyResult<Self> {
        let mode = PcaMode::from_str(&py_params.mode)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let orientation = OrientationMode::from_str(&py_params.orientation)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let output_format = OutputFormat::from_str(&py_params.output_format)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(CompressionParams {
            quality: py_params.quality.clamp(0.1, 1.0),
            mode,
            retain_components: py_params.retain_components,
            orientation,
            tile_size: py_params.tile_size,
            max_memory_mb: py_params.max_memory_mb,
            output_format,
            strip_metadata: py_params.strip_metadata,
        })
    }
}

/// Compression result metrics
#[pyclass(name = "CompressionResult")]
#[derive(Clone)]
pub struct PyCompressionResult {
    /// Compressed image bytes
    #[pyo3(get)]
    bytes: PyObject,
    /// Original size in bytes
    #[pyo3(get)]
    original_size: usize,
    /// Compressed size in bytes
    #[pyo3(get)]
    compressed_size: usize,
    /// Compression ratio
    #[pyo3(get)]
    compression_ratio: f32,
    /// SSIM score (0.0-1.0)
    #[pyo3(get)]
    ssim: f32,
    /// PSNR in dB
    #[pyo3(get)]
    psnr: f32,
    /// Processing time in milliseconds
    #[pyo3(get)]
    processing_time_ms: u64,
    /// Orientation method used
    #[pyo3(get)]
    orientation_method: String,
}

#[pymethods]
impl PyCompressionResult {
    fn __repr__(&self) -> String {
        format!(
            "CompressionResult(original_size={}, compressed_size={}, ratio={:.2}x, ssim={:.3}, psnr={:.1}, time={}ms)",
            self.original_size, self.compressed_size, self.compression_ratio,
            self.ssim, self.psnr, self.processing_time_ms
        )
    }

    /// Convert to dictionary
    pub fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        use pyo3::types::PyDict;

        let dict = PyDict::new_bound(py);
        dict.set_item("original_size", self.original_size)?;
        dict.set_item("compressed_size", self.compressed_size)?;
        dict.set_item("compression_ratio", self.compression_ratio)?;
        dict.set_item("ssim", self.ssim)?;
        dict.set_item("psnr", self.psnr)?;
        dict.set_item("processing_time_ms", self.processing_time_ms)?;
        dict.set_item("orientation_method", self.orientation_method.clone())?;

        Ok(dict.to_object(py))
    }
}

/// Compress image from bytes
#[pyfunction]
#[pyo3(signature = (image_bytes, params=None))]
pub fn compress_bytes(
    py: Python,
    image_bytes: &Bound<PyBytes>,
    params: Option<PyCompressionParams>,
) -> PyResult<PyCompressionResult> {
    let py_params = params.unwrap_or_default();
    let params: CompressionParams = PyCompressionParams::try_from(py_params)?;

    // Create temp file for input
    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join("pca_compress_input_temp.jpg");
    let temp_output = temp_dir.join("pca_compress_output_temp.jpg");

    // Write input bytes
    fs::write(&temp_input, image_bytes.as_bytes())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    // Get original size
    let original_size = fs::metadata(&temp_input)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // Compress
    let metrics = crate::pca_core::compression::compress_image(
        &temp_input,
        &temp_output,
        &params,
    ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // Read output bytes
    let output_bytes = fs::read(&temp_output)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    // Clean up temp files
    let _ = fs::remove_file(&temp_input);
    let _ = fs::remove_file(&temp_output);

    // Create result
    Ok(PyCompressionResult {
        bytes: PyBytes::new_bound(py, &output_bytes).to_object(py),
        original_size,
        compressed_size: metrics.compressed_size,
        compression_ratio: metrics.compression_ratio,
        ssim: metrics.ssim,
        psnr: metrics.psnr,
        processing_time_ms: metrics.processing_time_ms,
        orientation_method: "unknown".to_string(),
    })
}

/// Compress image from file path
#[pyfunction]
#[pyo3(signature = (input_path, output_path, params=None))]
pub fn compress_file(
    py: Python,
    input_path: String,
    output_path: String,
    params: Option<PyCompressionParams>,
) -> PyResult<PyCompressionResult> {
    let py_params = params.unwrap_or_default();
    let params: CompressionParams = PyCompressionParams::try_from(py_params)?;

    let input = Path::new(&input_path);
    let output = Path::new(&output_path);

    // Get original size
    let original_size = fs::metadata(input)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    // Compress
    let metrics = crate::pca_core::compression::compress_image(
        input,
        output,
        &params,
    ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // Read output bytes
    let output_bytes = fs::read(output)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    // Create result
    Ok(PyCompressionResult {
        bytes: PyBytes::new_bound(py, &output_bytes).to_object(py),
        original_size,
        compressed_size: metrics.compressed_size,
        compression_ratio: metrics.compression_ratio,
        ssim: metrics.ssim,
        psnr: metrics.psnr,
        processing_time_ms: metrics.processing_time_ms,
        orientation_method: "unknown".to_string(),
    })
}

/// Get library version
#[pyfunction]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Python module
#[pymodule]
#[pyo3(name = "pca_compressor")]
fn pca_compressor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add classes
    m.add_class::<PyCompressionParams>()?;
    m.add_class::<PyCompressionResult>()?;

    // Add functions
    m.add_function(wrap_pyfunction!(compress_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(compress_file, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    // Add module-level constants
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}