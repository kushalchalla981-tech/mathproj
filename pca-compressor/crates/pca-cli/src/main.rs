//! PCA Image Compression CLI

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pca_core::compression::{BatchResult, compress_batch, compress_image, scan_directory, write_batch_report};
use pca_core::pca::{CompressionParams, OrientationMode, OutputFormat, PcaMode};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "pca-compress")]
#[command(about = "PCA-based image compression tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a single image
    Single {
        /// Input image path
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output image path (default: INPUT.compressed.jpg)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compression quality (0.1-1.0, higher = better quality)
        #[arg(short, long, default_value = "0.7")]
        quality: f32,

        /// PCA mode
        #[arg(short, long, value_enum, default_value = "per-channel")]
        mode: ModeArg,

        /// Number of components to retain
        #[arg(short = 'n', long, default_value = "1")]
        retain_components: usize,

        /// Orientation handling
        #[arg(short, long, value_enum, default_value = "auto")]
        orientation: OrientationArg,

        /// Tile size (default: 1024, use 0 for no tiling)
        #[arg(short, long)]
        tile_size: Option<u32>,

        /// Maximum memory in MB
        #[arg(long)]
        max_memory: Option<usize>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "jpeg")]
        format: FormatArg,

        /// Strip EXIF metadata
        #[arg(long)]
        strip_metadata: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compress multiple images in batch
    Batch {
        /// Input directory
        #[arg(short, long, value_name = "DIR")]
        input: PathBuf,

        /// Output directory
        #[arg(short, long, value_name = "DIR")]
        output: PathBuf,

        /// Compression quality
        #[arg(short, long, default_value = "0.7")]
        quality: f32,

        /// PCA mode
        #[arg(short, long, value_enum, default_value = "per-channel")]
        mode: ModeArg,

        /// Number of components to retain
        #[arg(short = 'n', long, default_value = "1")]
        retain_components: usize,

        /// Orientation handling
        #[arg(short, long, value_enum, default_value = "auto")]
        orientation: OrientationArg,

        /// Tile size
        #[arg(short, long)]
        tile_size: Option<u32>,

        /// Maximum memory in MB
        #[arg(long)]
        max_memory: Option<usize>,

        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "jpeg")]
        format: FormatArg,

        /// Strip EXIF metadata
        #[arg(long)]
        strip_metadata: bool,

        /// CSV report path
        #[arg(short, long)]
        report: Option<PathBuf>,

        /// Number of worker threads (default: num CPUs - 1)
        #[arg(short = 'j', long)]
        threads: Option<usize>,
    },

    /// Validate an image without compressing
    Validate {
        /// Image path
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum ModeArg {
    /// Process each RGB channel independently
    PerChannel,
    /// Treat RGB as 3D vectors
    JointChannel,
}

impl From<ModeArg> for PcaMode {
    fn from(arg: ModeArg) -> Self {
        match arg {
            ModeArg::PerChannel => PcaMode::PerChannel,
            ModeArg::JointChannel => PcaMode::JointChannel,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum OrientationArg {
    /// Use PCA principal axis, fallback to EXIF
    Auto,
    /// Use EXIF orientation only
    Exif,
    /// Don't apply orientation correction
    Disabled,
}

impl From<OrientationArg> for OrientationMode {
    fn from(arg: OrientationArg) -> Self {
        match arg {
            OrientationArg::Auto => OrientationMode::Auto,
            OrientationArg::Exif => OrientationMode::Exif,
            OrientationArg::Disabled => OrientationMode::Disabled,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum FormatArg {
    /// JPEG format
    Jpeg,
    /// PNG format
    Png,
}

impl From<FormatArg> for OutputFormat {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Jpeg => OutputFormat::Jpeg,
            FormatArg::Png => OutputFormat::Png,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Single {
            input,
            output,
            quality,
            mode,
            retain_components,
            orientation,
            tile_size,
            max_memory,
            format,
            strip_metadata,
            verbose,
        } => {
            run_single(
                &input,
                output.as_deref(),
                quality,
                mode.into(),
                retain_components,
                orientation.into(),
                tile_size,
                max_memory,
                format.into(),
                strip_metadata,
                verbose,
            )?;
        }
        Commands::Batch {
            input,
            output,
            quality,
            mode,
            retain_components,
            orientation,
            tile_size,
            max_memory,
            format,
            strip_metadata,
            report,
            threads,
        } => {
            run_batch(
                &input,
                &output,
                quality,
                mode.into(),
                retain_components,
                orientation.into(),
                tile_size,
                max_memory,
                format.into(),
                strip_metadata,
                report.as_deref(),
                threads,
            )?;
        }
        Commands::Validate { input, verbose } => {
            run_validate(&input, verbose)?;
        }
    }

    Ok(())
}

fn run_single(
    input: &Path,
    output: Option<&Path>,
    quality: f32,
    mode: PcaMode,
    retain_components: usize,
    orientation: OrientationMode,
    tile_size: Option<u32>,
    max_memory: Option<usize>,
    format: OutputFormat,
    strip_metadata: bool,
    verbose: bool,
) -> Result<()> {
    // Validate input exists
    if !input.exists() {
        anyhow::bail!("Input file does not exist: {}", input.display());
    }

    // Determine output path
    let output_path = if let Some(out) = output {
        out.to_path_buf()
    } else {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = match format {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png",
        };
        input.with_file_name(format!("{}_compressed.{}", stem, ext))
    };

    // Build parameters
    let params = CompressionParams {
        quality,
        mode,
        retain_components,
        orientation,
        tile_size: tile_size.or(Some(1024)),
        max_memory_mb: max_memory.or(Some(1024)),
        output_format: format,
        strip_metadata,
    };

    if verbose {
        eprintln!("Input: {}", input.display());
        eprintln!("Output: {}", output_path.display());
        eprintln!("Quality: {}", quality);
        eprintln!("Mode: {:?}", mode);
        eprintln!("Retain components: {}", retain_components);
        eprintln!("Orientation: {:?}", orientation);
        eprintln!("Tile size: {:?}", tile_size);
        eprintln!("Max memory: {:?} MB", max_memory);
        eprintln!();
    }

    // Run compression
    let start = Instant::now();

    let metrics = compress_image(input, &output_path, &params)
        .context(format!("Failed to compress {}", input.display()))?;

    let elapsed = start.elapsed();

    // Print results
    println!("✓ Compressed: {} → {}", input.display(), output_path.display());
    println!("  Size: {} → {} ({:.2}x ratio)",
        format_bytes(metrics.original_size),
        format_bytes(metrics.compressed_size),
        metrics.compression_ratio
    );
    println!("  Quality: SSIM = {:.3}, PSNR = {:.1} dB",
        metrics.ssim, metrics.psnr
    );
    println!("  Time: {:?}", elapsed);

    Ok(())
}

fn run_batch(
    input_dir: &Path,
    output_dir: &Path,
    quality: f32,
    mode: PcaMode,
    retain_components: usize,
    orientation: OrientationMode,
    tile_size: Option<u32>,
    max_memory: Option<usize>,
    format: OutputFormat,
    strip_metadata: bool,
    report_path: Option<&Path>,
    threads: Option<usize>,
) -> Result<()> {
    // Validate input directory
    if !input_dir.exists() {
        anyhow::bail!("Input directory does not exist: {}", input_dir.display());
    }
    if !input_dir.is_dir() {
        anyhow::bail!("Input path is not a directory: {}", input_dir.display());
    }

    // Set thread count
    let num_threads = threads.unwrap_or_else(|| {
        (std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(2)
            - 1)
            .max(1)
    });

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .ok(); // May fail if already initialized, that's fine

    // Scan for images
    eprintln!("Scanning {} for images...", input_dir.display());
    let input_files = scan_directory(input_dir);

    if input_files.is_empty() {
        anyhow::bail!("No supported images found in {}", input_dir.display());
    }

    eprintln!("Found {} images", input_files.len());
    eprintln!("Using {} worker threads", num_threads);
    eprintln!();

    // Build parameters
    let params = CompressionParams {
        quality,
        mode,
        retain_components,
        orientation,
        tile_size: tile_size.or(Some(1024)),
        max_memory_mb: max_memory.or(Some(1024)),
        output_format: format,
        strip_metadata,
    };

    // Run batch compression
    let start = Instant::now();

    let results = compress_batch(&input_files, output_dir, &params);

    let elapsed = start.elapsed();

    // Summary
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    // Calculate aggregate metrics
    let total_original: usize = results
        .iter()
        .filter_map(|r| r.metrics.as_ref())
        .map(|m| m.original_size)
        .sum();
    let total_compressed: usize = results
        .iter()
        .filter_map(|r| r.metrics.as_ref())
        .map(|m| m.compressed_size)
        .sum();
    let avg_ssim: f32 = results
        .iter()
        .filter_map(|r| r.metrics.as_ref())
        .map(|m| m.ssim)
        .sum::<f32>()
        / successful.max(1) as f32;

    println!();
    println!("╔════════════════════════════════════════════════╗");
    println!("║          Batch Compression Complete            ║");
    println!("╠════════════════════════════════════════════════╣");
    println!("║  Processed:    {:>30}  ║", results.len());
    println!("║  Successful:   {:>30}  ║", successful);
    println!("║  Failed:       {:>30}  ║", failed);
    println!("╠════════════════════════════════════════════════╣");
    if total_original > 0 {
        let ratio = total_original as f32 / total_compressed.max(1) as f32;
        println!("║  Total Size:   {:>15} → {:>10}  ║",
            format_bytes(total_original), format_bytes(total_compressed));
        println!("║  Ratio:        {:>30.2}x ║", ratio);
    }
    println!("║  Avg SSIM:     {:>30.3} ║", avg_ssim);
    println!("║  Time:         {:>30} ║", format_duration(elapsed));
    println!("╚════════════════════════════════════════════════╝");

    // Write report if requested
    if let Some(report) = report_path {
        write_batch_report(&results, report)
            .context("Failed to write CSV report")?;
        println!();
        println!("✓ Report saved to: {}", report.display());
    }

    // Show errors if any
    if failed > 0 {
        eprintln!();
        eprintln!("Errors:");
        for result in &results {
            if !result.success {
                if let Some(ref error) = result.error {
                    eprintln!("  ✗ {}: {}", result.input_path, error);
                }
            }
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_validate(input: &Path, verbose: bool) -> Result<()> {
    if !input.exists() {
        anyhow::bail!("File does not exist: {}", input.display());
    }

    use pca_core::image::load_image;

    match load_image(input) {
        Ok(image) => {
            println!("✓ Valid image: {}", input.display());
            println!("  Dimensions: {}x{}", image.width, image.height);
            println!("  Pixels: {}", image.num_pixels());
            println!("  Color space: {:?}", image.color_space);
            println!("  Has alpha: {}", image.has_alpha());

            if let Some(orientation) = image.exif_orientation {
                println!("  EXIF orientation: {}", orientation);
            } else {
                println!("  EXIF orientation: not found");
            }

            if verbose {
                println!();
                println!("  Size in memory: {}", format_bytes(image.size_bytes()));
            }
        }
        Err(e) => {
            anyhow::bail!("Invalid image: {}", e);
        }
    }

    Ok(())
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: &[char] = &['B', 'K', 'M', 'G'];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_idx])
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}.{:03}s", secs, duration.subsec_millis())
    } else {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {:02}s", mins, secs)
    }
}
