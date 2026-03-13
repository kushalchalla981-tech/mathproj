//! Image I/O and data structures

use crate::error::{CompressionError, Result};
use image::{ImageBuffer, Rgb, Rgba, DynamicImage, ImageFormat};
use std::path::Path;

/// Supported color spaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgb,
    Rgba,
    Grayscale,
    /// CMYK converted to RGB
    ConvertedFromCmyk,
}

/// Image data wrapper with metadata
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// RGB pixel data (flattened: [R, G, B, R, G, B, ...])
    pub rgb_data: Vec<f32>,
    /// Optional alpha channel (flattened)
    pub alpha_data: Option<Vec<f32>>,
    /// Color space information
    pub color_space: ColorSpace,
    /// Original file path
    pub source_path: Option<String>,
    /// EXIF orientation (1-8, where 1 is normal)
    pub exif_orientation: Option<u8>,
}

impl ImageData {
    /// Create new image data from RGB buffer
    pub fn new(width: u32, height: u32, rgb_data: Vec<f32>) -> Result<Self> {
        let expected_size = (width * height * 3) as usize;
        if rgb_data.len() != expected_size {
            return Err(CompressionError::InvalidParams {
                field: "rgb_data".to_string(),
                value: format!("expected {} elements, got {}", expected_size, rgb_data.len()),
            });
        }

        Ok(Self {
            width,
            height,
            rgb_data,
            alpha_data: None,
            color_space: ColorSpace::Rgb,
            source_path: None,
            exif_orientation: None,
        })
    }

    /// Create with alpha channel
    pub fn with_alpha(width: u32, height: u32, rgb_data: Vec<f32>, alpha_data: Vec<f32>) -> Result<Self> {
        let expected_pixels = (width * height) as usize;
        if alpha_data.len() != expected_pixels {
            return Err(CompressionError::InvalidParams {
                field: "alpha_data".to_string(),
                value: format!("expected {} elements, got {}", expected_pixels, alpha_data.len()),
            });
        }

        let mut image = Self::new(width, height, rgb_data)?;
        image.alpha_data = Some(alpha_data);
        image.color_space = ColorSpace::Rgba;
        Ok(image)
    }

    /// Get total number of pixels
    pub fn num_pixels(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Get size in bytes (estimate)
    pub fn size_bytes(&self) -> usize {
        let rgb_size = self.rgb_data.len() * std::mem::size_of::<f32>();
        let alpha_size = self.alpha_data.as_ref()
            .map(|a| a.len() * std::mem::size_of::<f32>())
            .unwrap_or(0);
        rgb_size + alpha_size
    }

    /// Check if image has alpha channel
    pub fn has_alpha(&self) -> bool {
        self.alpha_data.is_some()
    }

    /// Validate minimum size requirements
    pub fn validate_size(&self) -> Result<()> {
        if self.width < 64 || self.height < 64 {
            return Err(CompressionError::ImageTooSmall {
                width: self.width,
                height: self.height,
            });
        }
        Ok(())
    }

    /// Get pixel at (x, y) as [R, G, B] floats
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[f32; 3]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 3) as usize;
        Some([
            self.rgb_data[idx],
            self.rgb_data[idx + 1],
            self.rgb_data[idx + 2],
        ])
    }

    /// Set pixel at (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, rgb: [f32; 3]) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(CompressionError::InvalidParams {
                field: "coordinates".to_string(),
                value: format!("({}, {}) out of bounds for {}x{}", x, y, self.width, self.height),
            });
        }
        let idx = ((y * self.width + x) * 3) as usize;
        self.rgb_data[idx] = rgb[0];
        self.rgb_data[idx + 1] = rgb[1];
        self.rgb_data[idx + 2] = rgb[2];
        Ok(())
    }

    /// Split into RGB channels
    pub fn split_channels(&self) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let n = self.num_pixels();
        let mut r = Vec::with_capacity(n);
        let mut g = Vec::with_capacity(n);
        let mut b = Vec::with_capacity(n);

        for i in 0..n {
            r.push(self.rgb_data[i * 3]);
            g.push(self.rgb_data[i * 3 + 1]);
            b.push(self.rgb_data[i * 3 + 2]);
        }

        (r, g, b)
    }

    /// Merge RGB channels back into image
    pub fn from_channels(&mut self, r: &[ f32], g: &[f32], b: &[f32]) -> Result<()> {
        let n = self.num_pixels();
        if r.len() != n || g.len() != n || b.len() != n {
            return Err(CompressionError::InvalidParams {
                field: "channels".to_string(),
                value: format!("expected {} elements per channel, got {}/{}/{}", n, r.len(), g.len(), b.len()),
            });
        }

        for i in 0..n {
            self.rgb_data[i * 3] = r[i];
            self.rgb_data[i * 3 + 1] = g[i];
            self.rgb_data[i * 3 + 2] = b[i];
        }
        Ok(())
    }
}

/// Load image from file path
pub fn load_image(path: &Path) -> Result<ImageData> {
    // Check file exists
    if !path.exists() {
        return Err(CompressionError::InvalidFormat {
            path: path.to_string_lossy().to_string(),
            reason: "File does not exist".to_string(),
        });
    }

    // Load with image crate
    let img = image::open(path).map_err(|e| {
        CompressionError::DecodingFailed(format!("{}: {}", path.display(), e))
    })?;

    // Read EXIF orientation
    let exif_orientation = read_exif_orientation(path).ok();

    // Convert to RGB/RGBA
    let (rgb_data, alpha_data, color_space) = match img {
        DynamicImage::ImageRgb8(rgb) => {
            let data: Vec<f32> = rgb.pixels()
                .flat_map(|p| vec![p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0])
                .collect();
            (data, None, ColorSpace::Rgb)
        }
        DynamicImage::ImageRgba8(rgba) => {
            let mut rgb = Vec::with_capacity((rgba.width() * rgba.height() * 3) as usize);
            let mut alpha = Vec::with_capacity((rgba.width() * rgba.height()) as usize);

            for pixel in rgba.pixels() {
                rgb.push(pixel[0] as f32 / 255.0);
                rgb.push(pixel[1] as f32 / 255.0);
                rgb.push(pixel[2] as f32 / 255.0);
                alpha.push(pixel[3] as f32 / 255.0);
            }
            (rgb, Some(alpha), ColorSpace::Rgba)
        }
        DynamicImage::ImageLuma8(gray) => {
            let data: Vec<f32> = gray.pixels()
                .flat_map(|p| vec![p[0] as f32 / 255.0, p[0] as f32 / 255.0, p[0] as f32 / 255.0])
                .collect();
            (data, None, ColorSpace::Grayscale)
        }
        DynamicImage::ImageLumaA8(gray_alpha) => {
            let mut rgb = Vec::with_capacity((gray_alpha.width() * gray_alpha.height() * 3) as usize);
            let mut alpha = Vec::with_capacity((gray_alpha.width() * gray_alpha.height()) as usize);

            for pixel in gray_alpha.pixels() {
                let v = pixel[0] as f32 / 255.0;
                rgb.push(v);
                rgb.push(v);
                rgb.push(v);
                alpha.push(pixel[1] as f32 / 255.0);
            }
            (rgb, Some(alpha), ColorSpace::Grayscale)
        }
        _ => {
            // Convert to RGB8 first
            let rgb = img.to_rgb8();
            let data: Vec<f32> = rgb.pixels()
                .flat_map(|p| vec![p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0])
                .collect();
            (data, None, ColorSpace::Rgb)
        }
    };

    let mut image_data = if let Some(alpha) = alpha_data {
        ImageData::with_alpha(img.width(), img.height(), rgb_data, alpha)?
    } else {
        ImageData::new(img.width(), img.height(), rgb_data)?
    };

    image_data.color_space = color_space;
    image_data.source_path = Some(path.to_string_lossy().to_string());
    image_data.exif_orientation = exif_orientation;

    Ok(image_data)
}

/// Save image to file
pub fn save_image(path: &Path, image: &ImageData, quality: u8) -> Result<()> {
    // Clamp RGB values to [0, 1]
    let clamped: Vec<u8> = image.rgb_data.iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    // Determine format from extension
    let format = if let Some(ext) = path.extension() {
        match ext.to_ascii_lowercase().to_str() {
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            _ => return Err(CompressionError::InvalidFormat {
                path: path.to_string_lossy().to_string(),
                reason: "Unsupported extension".to_string(),
            }),
        }
    } else {
        return Err(CompressionError::InvalidFormat {
            path: path.to_string_lossy().to_string(),
            reason: "No file extension".to_string(),
        });
    };

    // Create image buffer with or without alpha
    if let Some(ref alpha) = image.alpha_data {
        // RGBA
        let mut rgba_data = Vec::with_capacity(clamped.len() / 3 * 4);
        for (i, rgb) in clamped.chunks(3).enumerate() {
            rgba_data.push(rgb[0]);
            rgba_data.push(rgb[1]);
            rgba_data.push(rgb[2]);
            rgba_data.push((alpha[i].clamp(0.0, 1.0) * 255.0) as u8);
        }

        let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            image.width,
            image.height,
            rgba_data,
        ).ok_or_else(|| CompressionError::EncodingFailed("Failed to create RGBA buffer".to_string()))?;

        buffer.save(path).map_err(|e| {
            CompressionError::EncodingFailed(format!("{}: {}", path.display(), e))
        })?;
    } else {
        // RGB
        let buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(
            image.width,
            image.height,
            clamped,
        ).ok_or_else(|| CompressionError::EncodingFailed("Failed to create RGB buffer".to_string()))?;

        // Handle JPEG quality
        if format == ImageFormat::Jpeg {
            let file = std::fs::File::create(path).map_err(CompressionError::Io)?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
            encoder.encode_image(&DynamicImage::ImageRgb8(buffer)
            ).map_err(|e| CompressionError::EncodingFailed(format!("{}: {}", path.display(), e)))?;
        } else {
            buffer.save(path).map_err(|e| {
                CompressionError::EncodingFailed(format!("{}: {}", path.display(), e))
            })?;
        }
    }

    Ok(())
}

/// Read EXIF orientation from image file
fn read_exif_orientation(path: &Path) -> Result<u8> {
    let file = std::fs::File::open(path).map_err(|e| {
        CompressionError::ExifReadFailed(format!("Cannot open file: {}", e))
    })?;

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = kamadak_exif::Reader::new();

    match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => {
            if let Some(orientation) = exif.get_field(kamadak_exif::Tag::Orientation, kamadak_exif::In::PRIMARY) {
                if let kamadak_exif::Value::Short(v) = &orientation.value {
                    return Ok(v[0]);
                }
            }
            Ok(1) // Default orientation
        }
        Err(_) => Ok(1), // No EXIF or can't read, default to normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let data = vec![0.5f32; 300 * 200 * 3];
        let img = ImageData::new(300, 200, data).unwrap();
        assert_eq!(img.width, 300);
        assert_eq!(img.height, 200);
        assert_eq!(img.num_pixels(), 300 * 200);
    }

    #[test]
    fn test_invalid_size() {
        let data = vec![0.5f32; 100]; // Wrong size
        let result = ImageData::new(300, 200, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_small() {
        let data = vec![0.5f32; 32 * 32 * 3];
        let img = ImageData::new(32, 32, data).unwrap();
        assert!(img.validate_size().is_err());
    }

    #[test]
    fn test_pixel_access() {
        let mut data = vec![0.0f32; 100 * 100 * 3];
        data[0] = 1.0; // Top-left pixel R
        data[1] = 0.5; // Top-left pixel G
        data[2] = 0.0; // Top-left pixel B

        let img = ImageData::new(100, 100, data).unwrap();
        let pixel = img.get_pixel(0, 0).unwrap();
        assert_eq!(pixel, [1.0, 0.5, 0.0]);
    }

    #[test]
    fn test_channel_splitting() {
        let mut data = Vec::new();
        for i in 0..100 {
            data.push(i as f32); // R
            data.push((i + 100) as f32); // G
            data.push((i + 200) as f32); // B
        }

        let img = ImageData::new(10, 10, data).unwrap();
        let (r, g, b) = img.split_channels();
        assert_eq!(r.len(), 100);
        assert_eq!(g.len(), 100);
        assert_eq!(b.len(), 100);
        assert_eq!(r[0], 0.0);
        assert_eq!(g[0], 100.0);
        assert_eq!(b[0], 200.0);
    }
}
