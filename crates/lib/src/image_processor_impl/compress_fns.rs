use std::io::{Cursor};
use std::panic::catch_unwind;
use image::{DynamicImage, ImageFormat};
use mozjpeg::{ColorSpace, Compress};
use oxipng::{Options, StripChunks};

pub fn compress_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let pixels = rgb.into_raw();
    // mozjpeg panics on error instead of returning Result
    // catch_unwind converts that panic into a Result
    let result = catch_unwind(|| -> std::io::Result<Vec<u8>> {
        let mut comp = Compress::new(ColorSpace::JCS_RGB);
        comp.set_size(width as usize, height as usize);
        comp.set_quality(quality as f32);
        comp.set_optimize_scans(true);

        // start_compress takes any io::Write — we pass Vec<u8> as the buffer
        // it returns a CompressStarted — settings are locked after this point
        let mut comp = comp.start_compress(Vec::new())?;

        // write_scanlines takes the full flat pixel buffer
        comp.write_scanlines(&pixels)?;

        // finish() finalizes the JPEG and returns the Vec<u8> writer back
        let jpeg_bytes = comp.finish()?;

        Ok(jpeg_bytes)
    });

    match result {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(io_err)) => Err(format!("mozjpeg IO error: {}", io_err)),
        Err(_panic)   => Err("mozjpeg panicked during compression".to_string()),
    }
}

// ─── PNG compression via image-rs (oxipng would go here in v2) ──────────────
pub fn compress_png(img: &DynamicImage,quality : u8) -> Result<Vec<u8>, String> {
    let mut initial_bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut initial_bytes), ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {}", e))?;

    // Step 2 — pass those bytes to oxipng for optimization
    // Options::from_preset maps 0-6 where 2 is default (fast+good), 6 is max
    // We map the user's 0-100 quality scale to oxipng's 0-6 preset levels
    let preset = map_quality_to_oxipng_preset(quality);
    let mut opts = Options::from_preset(preset);

    // Strip non-critical metadata to save extra bytes
    // "safe" keeps only chunks needed for correct rendering
    opts.strip = StripChunks::Safe;

    // Optimize alpha channel — lets oxipng alter transparent pixel colors
    // to improve compression without visible change
    opts.optimize_alpha = true;

    // oxipng uses rayon internally when the rayon feature is enabled
    // no extra code needed — it parallelizes automatically
    let optimized = oxipng::optimize_from_memory(&initial_bytes, &opts)
        .map_err(|e| format!("oxipng optimization failed: {}", e))?;

    eprintln!(
        "📦 PNG: {}KB → {}KB (preset {})",
        initial_bytes.len() / 1024,
        optimized.len() / 1024,
        preset
    );

    Ok(optimized)
}
// ─── WebP via image-rs basic encoder (libwebp-sys would go here in v2) ──────
pub fn compress_webp(img: &DynamicImage, quality: u8) -> Result<Vec<u8>,String> {
    // image-rs WebP encoder uses quality 1–100
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|e| format!("WebP encoding failed: {}", e))?;
    Ok(buf)
}

fn map_quality_to_oxipng_preset(quality: u8) -> u8 {
    match quality {
        90..=100 => 1, // fast, good enough for most cases
        70..=89  => 2, // default — best speed/size balance
        50..=69  => 3,
        30..=49  => 4,
        10..=29  => 5,
        _        => 6, // max compression, slowest
    }
}