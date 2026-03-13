#!/usr/bin/env python3
"""
PCA-based Image Compression - Python Prototype

This is a reference implementation of the PCA compression algorithm
for validating correctness before porting to Rust.
"""

import numpy as np
from PIL import Image, ExifTags
from typing import Tuple, Optional, List
from dataclasses import dataclass
from enum import Enum
import time
import os
import csv
from pathlib import Path


class PCAMode(Enum):
    """PCA processing modes"""
    PER_CHANNEL = "per-channel"
    JOINT_CHANNEL = "joint-channel"


class OrientationMode(Enum):
    """Orientation handling modes"""
    AUTO = "auto"
    EXIF = "exif"
    DISABLED = "disabled"


class OutputFormat(Enum):
    """Output image formats"""
    JPEG = "jpeg"
    PNG = "png"


@dataclass
class CompressionParams:
    """Compression parameters"""
    quality: float = 0.7  # 0.1-1.0
    mode: PCAMode = PCAMode.PER_CHANNEL
    retain_components: int = 1
    orientation: OrientationMode = OrientationMode.AUTO
    tile_size: Optional[int] = 1024
    max_memory_mb: Optional[int] = 1024
    output_format: OutputFormat = OutputFormat.JPEG
    strip_metadata: bool = False

    def validate(self):
        """Validate parameters"""
        if not 0.1 <= self.quality <= 1.0:
            raise ValueError(f"Quality must be between 0.1 and 1.0, got {self.quality}")
        if self.retain_components < 1:
            raise ValueError(f"Retain components must be >= 1, got {self.retain_components}")


@dataclass
class CompressionMetrics:
    """Compression result metrics"""
    original_size: int
    compressed_size: int
    compression_ratio: float
    ssim: float
    psnr: float
    processing_time_ms: int

    def format(self) -> str:
        return (f"Size: {self.original_size} -> {self.compressed_size} "
                f"(ratio: {self.compression_ratio:.2f}x) | "
                f"SSIM: {self.ssim:.3f} | PSNR: {self.psnr:.1f} dB")


class ImageData:
    """Image data wrapper"""

    def __init__(self, width: int, height: int, rgb_data: np.ndarray, alpha_data: Optional[np.ndarray] = None):
        self.width = width
        self.height = height
        self.rgb_data = rgb_data  # Shape: (height, width, 3), values in [0, 1]
        self.alpha_data = alpha_data  # Shape: (height, width), values in [0, 1] or None
        self.exif_orientation: Optional[int] = None

    @classmethod
    def from_file(cls, path: str) -> "ImageData":
        """Load image from file"""
        img = Image.open(path)

        # Get EXIF orientation
        exif_orientation = None
        try:
            exif = img._getexif()
            if exif:
                for tag, value in exif.items():
                    if ExifTags.TAGS.get(tag) == 'Orientation':
                        exif_orientation = value
                        break
        except:
            pass

        # Convert to RGB or RGBA
        if img.mode in ('RGBA', 'LA', 'P'):
            img = img.convert('RGBA')
            rgb_array = np.array(img).astype(np.float32) / 255.0
            rgb_data = rgb_array[:, :, :3]
            alpha_data = rgb_array[:, :, 3]
        else:
            img = img.convert('RGB')
            rgb_array = np.array(img).astype(np.float32) / 255.0
            rgb_data = rgb_array
            alpha_data = None

        image_data = cls(img.width, img.height, rgb_data, alpha_data)
        image_data.exif_orientation = exif_orientation
        return image_data

    def save(self, path: str, quality: int = 90):
        """Save image to file"""
        # Clamp values
        rgb = np.clip(self.rgb_data, 0, 1)

        if self.alpha_data is not None:
            # RGBA
            alpha = np.clip(self.alpha_data, 0, 1)
            rgba = np.dstack([rgb, alpha[:, :, np.newaxis]])
            img_array = (rgba * 255).astype(np.uint8)
            img = Image.fromarray(img_array, 'RGBA')
        else:
            # RGB
            img_array = (rgb * 255).astype(np.uint8)
            img = Image.fromarray(img_array, 'RGB')

        # Save
        if path.lower().endswith('.png'):
            img.save(path)
        else:
            img.save(path, quality=quality, optimize=True)

    def num_pixels(self) -> int:
        return self.width * self.height

    def size_bytes(self) -> int:
        size = self.rgb_data.nbytes
        if self.alpha_data is not None:
            size += self.alpha_data.nbytes
        return size


def compress_per_channel(image: ImageData, retain_components: int) -> ImageData:
    """Compress using per-channel PCA"""
    result_rgb = np.zeros_like(image.rgb_data)

    # Process each channel independently
    for c in range(3):
        channel = image.rgb_data[:, :, c].flatten()

        # Simple compression: quantize based on variance
        # For a true 1D PCA, we'd reduce to mean, but let's use component analysis
        if retain_components >= 1:
            # For now, pass through with slight modification
            # Real implementation would project and reconstruct
            mean = np.mean(channel)
            std = np.std(channel)

            # Simple "compression": reduce variance slightly
            compressed = mean + (channel - mean) * 0.9
            result_rgb[:, :, c] = compressed.reshape(image.height, image.width)

    return ImageData(image.width, image.height, result_rgb, image.alpha_data)


def compress_joint_channel(image: ImageData, retain_components: int) -> ImageData:
    """Compress using joint-channel PCA (3D)"""
    h, w = image.height, image.width
    n_pixels = h * w

    # Reshape to (n_pixels, 3)
    pixels = image.rgb_data.reshape(-1, 3).astype(np.float64)

    # Mean center
    mean = np.mean(pixels, axis=0)
    centered = pixels - mean

    # Compute covariance (3x3)
    cov = np.cov(centered.T)

    # Eigen-decomposition
    eigenvalues, eigenvectors = np.linalg.eigh(cov)

    # Sort by eigenvalue (descending)
    idx = np.argsort(eigenvalues)[::-1]
    eigenvalues = eigenvalues[idx]
    eigenvectors = eigenvectors[:, idx]

    # Project onto principal components
    projected = centered @ eigenvectors

    # Zero out non-retained components
    n_retain = min(retain_components, 3)
    projected[:, n_retain:] = 0

    # Reconstruct
    reconstructed = projected @ eigenvectors.T + mean

    # Reshape back to image
    result_rgb = reconstructed.reshape(h, w, 3).astype(np.float32)

    return ImageData(image.width, image.height, result_rgb, image.alpha_data)


def compress_image(image: ImageData, params: CompressionParams) -> ImageData:
    """Compress an image using the specified mode"""
    if params.mode == PCAMode.PER_CHANNEL:
        return compress_per_channel(image, params.retain_components)
    else:
        return compress_joint_channel(image, params.retain_components)


def calculate_ssim(img1: ImageData, img2: ImageData) -> float:
    """Calculate SSIM (simplified)"""
    # Simplified SSIM - just structural comparison
    # Real SSIM uses sliding windows, etc.

    def channel_ssim(c1: np.ndarray, c2: np.ndarray) -> float:
        mu1 = np.mean(c1)
        mu2 = np.mean(c2)
        sigma1 = np.std(c1)
        sigma2 = np.std(c2)
        sigma12 = np.mean((c1 - mu1) * (c2 - mu2))

        c1_const = 0.01 ** 2
        c2_const = 0.03 ** 2

        ssim = ((2 * mu1 * mu2 + c1_const) * (2 * sigma12 + c2_const)) / \
               ((mu1 ** 2 + mu2 ** 2 + c1_const) * (sigma1 ** 2 + sigma2 ** 2 + c2_const))

        return float(ssim)

    ssim_r = channel_ssim(img1.rgb_data[:, :, 0], img2.rgb_data[:, :, 0])
    ssim_g = channel_ssim(img1.rgb_data[:, :, 1], img2.rgb_data[:, :, 1])
    ssim_b = channel_ssim(img1.rgb_data[:, :, 2], img2.rgb_data[:, :, 2])

    return (ssim_r + ssim_g + ssim_b) / 3.0


def calculate_psnr(img1: ImageData, img2: ImageData) -> float:
    """Calculate PSNR"""
    mse = np.mean((img1.rgb_data - img2.rgb_data) ** 2)
    if mse == 0:
        return float('inf')
    return 10 * np.log10(1.0 / mse)


def compress_file(input_path: str, output_path: str, params: CompressionParams) -> CompressionMetrics:
    """Compress a single image file"""
    start_time = time.time()

    # Load
    original = ImageData.from_file(input_path)
    original_size = os.path.getsize(input_path)

    # Validate size
    if original.width < 64 or original.height < 64:
        raise ValueError(f"Image too small: {original.width}x{original.height}")

    # Compress
    compressed = compress_image(original, params)

    # Calculate metrics
    ssim = calculate_ssim(original, compressed)
    psnr = calculate_psnr(original, compressed)

    # Save
    quality = int(params.quality * 100)
    compressed.save(output_path, quality=quality)
    compressed_size = os.path.getsize(output_path)

    # Calculate ratio
    ratio = original_size / compressed_size if compressed_size > 0 else 1.0

    processing_time_ms = int((time.time() - start_time) * 1000)

    return CompressionMetrics(
        original_size=original_size,
        compressed_size=compressed_size,
        compression_ratio=ratio,
        ssim=ssim,
        psnr=psnr,
        processing_time_ms=processing_time_ms
    )


def compress_batch(input_dir: str, output_dir: str, params: CompressionParams) -> List[dict]:
    """Compress all images in a directory"""
    results = []

    os.makedirs(output_dir, exist_ok=True)

    # Find images
    image_extensions = {'.jpg', '.jpeg', '.png'}
    files = [f for f in os.listdir(input_dir)
             if any(f.lower().endswith(ext) for ext in image_extensions)]

    print(f"Found {len(files)} images")

    for filename in sorted(files):
        input_path = os.path.join(input_dir, filename)
        name, ext = os.path.splitext(filename)
        output_path = os.path.join(output_dir, f"{name}_compressed.jpg")

        try:
            metrics = compress_file(input_path, output_path, params)
            results.append({
                'input': filename,
                'success': True,
                'original_size': metrics.original_size,
                'compressed_size': metrics.compressed_size,
                'ratio': metrics.compression_ratio,
                'ssim': metrics.ssim,
                'psnr': metrics.psnr,
                'time_ms': metrics.processing_time_ms,
                'error': None
            })
            print(f"✓ {filename}: {metrics.format()}")
        except Exception as e:
            results.append({
                'input': filename,
                'success': False,
                'error': str(e)
            })
            print(f"✗ {filename}: {e}")

    return results


def write_csv_report(results: List[dict], report_path: str):
    """Write batch results to CSV"""
    with open(report_path, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['filename', 'original_size', 'compressed_size',
                        'compression_ratio', 'ssim', 'psnr', 'processing_time_ms',
                        'success', 'error'])

        for r in results:
            if r.get('success'):
                writer.writerow([
                    r['input'], r['original_size'], r['compressed_size'],
                    f"{r['ratio']:.2f}", f"{r['ssim']:.3f}",
                    f"{r['psnr']:.1f}", r['time_ms'], 'yes', ''
                ])
            else:
                writer.writerow([r['input'], '', '', '', '', '', '', 'no', r.get('error', '')])


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python pca_compressor.py <input> <output> [options]")
        print("       python pca_compressor.py --batch <input_dir> <output_dir>")
        sys.exit(1)

    if sys.argv[1] == '--batch':
        input_dir = sys.argv[2]
        output_dir = sys.argv[3]
        params = CompressionParams(quality=0.7, mode=PCAMode.JOINT_CHANNEL)
        results = compress_batch(input_dir, output_dir, params)
        write_csv_report(results, os.path.join(output_dir, 'report.csv'))
    else:
        input_path = sys.argv[1]
        output_path = sys.argv[2]
        params = CompressionParams(quality=0.7, mode=PCAMode.JOINT_CHANNEL)
        metrics = compress_file(input_path, output_path, params)
        print(metrics.format())
