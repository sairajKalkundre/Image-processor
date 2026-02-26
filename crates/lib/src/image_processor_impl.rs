use std::sync::Once;
use craby::prelude::*;
use crate::ffi::bridging::*;
use crate::generated::*;
use image::imageops::FilterType;
use parking_lot::RwLock;
use log::info;

#[cfg(target_os = "android")]
extern crate android_log;

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
fn init_logging() {
    #[cfg(target_os = "android")]
    {
        static LOG_INIT: std::sync::Once = std::sync::Once::new();
        LOG_INIT.call_once(|| {
            android_log::init("ImageProcessor").ok();
        });
    }
}

fn log_memory(tag: &str) {
    #[cfg(target_os = "android")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") || line.starts_with("VmPeak:") {
                    info!("[{}] {}", tag, line.trim());
                }
            }
        }
    }
    // Silence unused variable warning on iOS
    #[cfg(not(target_os = "android"))]
    let _ = tag;
}

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
        if let Some(state) = PIPELINE.write().as_mut() {
            state.operations.push(Operation::Crop { x, y, width, height });
        }
    }fn flip(&mut self, horizontal: Boolean) -> Void {
        if let Some(state) = PIPELINE.write().as_mut() {
            state.operations.push(Operation::Flip { horizontal });
        }
    }fn resize(&mut self, width: Number, height: Number, fit: &str) -> Void {
        if let Some(state) = PIPELINE.write().as_mut() {
            state.operations.push(Operation::Resize {
                width,
                height,
                fit: fit.to_string(),
            });
        }
    }fn rotate(&mut self, degrees: Number) -> Void {
        if let Some(state) = PIPELINE.write().as_mut() {
            state.operations.push(Operation::Rotate { degrees });
        }
    }fn save(&mut self) -> Promise<ImageResult> {
        init_logging();
        let result: Result<ImageResult, String> = (|| {
            let state = PIPELINE.write().take()
                .ok_or("No image loaded. Call setFilePath first.")?;

            if state.path.is_empty() {
                return Err("File path is empty".to_string());
            }
            let orig_size = image_utils::original_size(&state.path);
            let (mut img,_,_, orig_format) = image_utils::decode_image(&state.path)?;
            log_memory("after-decode");
            info!("Starting resize {}x{}", img.width(), img.height());
            for op in &state.operations {
                match op {
                    Operation::Resize { width, height, fit } => {
                        let (tw, th) = image_utils::resolve_dimensions(
                            img.width(), img.height(), *width, *height
                        );
                        let fit_str = if fit.is_empty() { "contain" } else { fit.as_str() };
                        let img_to_resize = if img.width() > tw * 4 || img.height() > th * 4 {
                            // Quick nearest-neighbor pre-downsample to 2x target
                            // Then Lanczos3 on the smaller image — same quality, less RAM
                            img.resize(tw * 2, th * 2, FilterType::Nearest)
                        } else {
                            img
                        };
                        log_memory("before-resize");
                        let resized = match fit_str {
                            "cover" => img_to_resize.resize_to_fill(tw, th, FilterType::Lanczos3),
                            "fill"  => img_to_resize.resize_exact(tw, th, FilterType::Lanczos3),
                            _       => img_to_resize.resize(tw, th, FilterType::Lanczos3),
                        };
                        log_memory("after-resize");
                        img = resized;
                        eprintln!("✓ Resize → {}x{}", tw, th);
                    }
                    Operation::Crop { x, y, width, height } => {
                        img = img.crop_imm(
                            *x as u32, *y as u32,
                            *width as u32, *height as u32
                        );
                        info!("✓ Crop → {}x{}", width, height);
                    }
                    Operation::Flip { horizontal } => {
                        img = if *horizontal { img.fliph() } else { img.flipv() };
                        info!("✓ Flip {}", if *horizontal { "horizontal" } else { "vertical" });
                    }
                    Operation::Rotate { degrees } => {
                        img = match *degrees as i32 {
                            90  => img.rotate90(),
                            180 => img.rotate180(),
                            270 => img.rotate270(),
                            _   => img,
                        };
                        eprintln!("✓ Rotate {}°", degrees);
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
            drop(img);
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

