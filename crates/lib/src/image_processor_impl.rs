use craby::{prelude::*, throw};

use crate::ffi::bridging::*;
use crate::generated::*;
use std::fs;
use anyhow::Error;
use base64::{Engine as _, engine::general_purpose};
use image::GenericImageView;
pub struct ImageProcessor {
    ctx: Context,
}

#[craby_module]
impl ImageProcessorSpec for ImageProcessor {
    fn load_image(&mut self, path: &str) -> Promise<ImageResult> {
        // Do all the fallible work using a Result-returning closure
        let result: Result<ImageResult, String> = (|| {
            eprint!("Path {}", path);
            let clean_path = path.replace('\0', "");
            let clean_path = clean_path.trim();
            eprintln!("DEBUG path bytes: {:?}", clean_path.as_bytes());
            eprintln!("DEBUG path string: {}", clean_path);
            let bytes = fs::read(clean_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;

            let img = image::load_from_memory(&bytes)
                .map_err(|e| format!("Failed to decode image: {}", e))?;

            let (width, height) = img.dimensions();

            let format = path
                .rsplit('.')
                .next()
                .unwrap_or("unknown")
                .to_lowercase();

            let base64_str = general_purpose::STANDARD.encode(&bytes);

            Ok(ImageResult {
                base_64: base64_str,
                width: f64::from(width),
                height: f64::from(height),
                format,
            })
        })();

        match result {
            Ok(image) => promise::resolve(image),
            Err(e) => promise::reject(e),
        }
    }
}