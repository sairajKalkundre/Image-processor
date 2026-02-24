use craby::prelude::*;
use crate::ffi::bridging::*;
use crate::generated::*;
use image::imageops::FilterType;
use parking_lot::RwLock;

mod image_utils;
mod compress_fns;

pub struct ImageProcessor {
    ctx: Context,
}
enum Operation {
    Resize { width: f64, height: f64, fit: String },
    Compress { quality: u8, format: String },
    Crop { x: f64, y: f64, width: f64, height: f64 },
    Flip { horizontal: bool },
    Rotate { degrees: f64 },
}

struct EncodeOptions {
    quality: u8,
    format: String,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        EncodeOptions {
            quality: 85,       // sensible default
            format: String::new(), // empty = use input format
        }
    }
}

struct PipelineState {
    path: String,
    operations: Vec<Operation>,
    encode: EncodeOptions,  // tracks the final format across the pipeline
}
static PIPELINE: RwLock<Option<PipelineState>> = RwLock::new(None);
#[craby_module]
impl ImageProcessorSpec for ImageProcessor {
    fn set_file_path(&mut self, path: &str) -> Void {
        *PIPELINE.write() = Some(PipelineState {
            path: path.to_string(),
            operations: Vec::new(),
            encode: EncodeOptions::default(),
        });
    }
    fn compress(&mut self, quality: Number, format: &str) -> Void {
        if let Some(state) = PIPELINE.write().as_mut() {
            // compress sets the encode intent, not an operation to run on pixels
            state.encode = EncodeOptions {
                quality: if quality < 0.0 { 85 } else { quality as u8 },
                format: format.to_string(),
            };
        }
    }fn crop(&mut self, x: Number, y: Number, width: Number, height: Number) -> Void {
        todo!()
    }fn flip(&mut self, horizontal: Boolean) -> Void {
        todo!()
    }fn resize(&mut self, width: Number, height: Number, fit: &str) -> Void {
        if let Some(state) = PIPELINE.write().as_mut() {
            state.operations.push(Operation::Resize {
                width,
                height,
                fit: fit.to_string(),
            });
        }
    }fn rotate(&mut self, degrees: Number) -> Void {
        todo!()
    }fn save(&mut self) -> Promise<ImageResult> {
        let result: Result<ImageResult, String> = (|| {
            let state = PIPELINE.write().take()
                .ok_or("No image loaded. Call setFilePath first.")?;

            if state.path.is_empty() {
                return Err("File path is empty".to_string());
            }
            let orig_size = image_utils::original_size(&state.path);
            let (mut img,_,_, orig_format) = image_utils::decode_image(&state.path)?;

            // Apply operations in order
            for op in &state.operations {
                match op {
                    Operation::Resize { width, height, fit } => {
                        let (tw, th) = image_utils::resolve_dimensions(
                            img.width(), img.height(), *width, *height
                        );
                        img = match fit.as_str() {
                            "cover" => img.resize_to_fill(tw, th, FilterType::Lanczos3),
                            "fill"  => img.resize_exact(tw, th, FilterType::Lanczos3),
                            _       => img.resize(tw, th, FilterType::Lanczos3),
                        };
                        eprintln!("✓ Resize → {}x{}", tw, th);
                    }
                    _ => {}
                }
            }
            let out_format = if state.encode.format.is_empty() {
                orig_format.clone()
            } else {
                state.encode.format.clone()
            };

            let quality = state.encode.quality;
            let bytes = match out_format.as_str() {
                "jpg" | "jpeg" => compress_fns::compress_jpeg(&img, quality)?,
                "png"          => compress_fns::compress_png(&img , quality)?,
                "webp"         => compress_fns::compress_webp(&img, quality)?,
                _              => compress_fns::compress_jpeg(&img, quality)?,
            };
            let file_name = image_utils::file_name_from_path(&state.path);

            let mut result = image_utils::save_bytes(&bytes, file_name, &out_format)?;
            result.compression_ratio = if result.size > 0.0 {
                orig_size / result.size
            } else {
                1.0
            };
            Ok(result)
        })();
        match result {
            Ok(r)  => promise::resolve(r),
            Err(e) => promise::reject(e),
        }
    }
}

