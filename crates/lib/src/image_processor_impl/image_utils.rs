use image::ImageDecoder;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageReader, Limits};
use crate::ffi::bridging::{ImageResult};

pub fn decode_image(path: &str) -> Result<(DynamicImage, u32, u32, String), String> {
    let bytes = fs::read(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    if bytes.is_empty() {
        return Err("File is empty".to_string());
    }

    let reader = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?;

    let mut decoder = reader
        .into_decoder()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    decoder
        .set_limits(Limits::no_limits())
        .map_err(|e| format!("Failed to set limits: {}", e))?;

    let (width, height) = decoder.dimensions();

    let img = DynamicImage::from_decoder(decoder)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let format = path
        .rsplit('.')
        .next()
        .unwrap_or("png")
        .to_lowercase()
        .replace('\0', "");

    Ok((img, width, height, format))
}

pub fn save_bytes(bytes: &[u8], file_name: &str, format: &str) -> Result<ImageResult, craby::prelude::String> {
    let stem = Path::new(file_name)
        .file_stem()                        // "portrait" — no extension
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = match format {
        "jpeg" => "jpg",
        other  => other,
    };
    let out_path = std::env::temp_dir()
        .join(format!("rust_compressed_{}.{}", stem, ext));
    let out_path_str = out_path.to_string_lossy().to_string();

    fs::write(&out_path, bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    let size = fs::metadata(&out_path)
        .map(|m| m.len() as f64)
        .unwrap_or(0.0);

    // We need dimensions from the bytes — re-open briefly just for dimensions
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read output dimensions: {}", e))?;
    let (width, height) = reader.into_dimensions()
        .unwrap_or((0, 0));

    Ok(ImageResult {
        uri: format!("file://{}", out_path_str),
        width : f64::from(width),
        height: f64::from(height),
        format: format.to_string(),
        size,
        compression_ratio: 0.0, // filled in by caller
    })
}


pub fn resolve_dimensions(orig_w: u32, orig_h: u32, tw: f64, th: f64) -> (u32, u32) {
    match (tw > 0.0, th > 0.0) {
        (true, true)  => (tw as u32, th as u32),
        (true, false) => {
            let ratio = tw / orig_w as f64;
            (tw as u32, (orig_h as f64 * ratio).round() as u32)
        }
        (false, true) => {
            let ratio = th / orig_h as f64;
            ((orig_w as f64 * ratio).round() as u32, th as u32)
        }
        (false, false) => (orig_w, orig_h),
    }
}

pub fn original_size(path: &str) -> f64 {
    fs::metadata(path).map(|m| m.len() as f64).unwrap_or(0.0)
}

pub fn file_name_from_path(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.jpg")
}

