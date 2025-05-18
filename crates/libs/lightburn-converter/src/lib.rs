//! Library for converting between LightBurn files and SVG files.

pub mod types;
pub mod lightburn;
pub mod svg;

pub use types::{
    ConversionError, LightburnFile, SvgFile, 
    Shape, ShapeType, Point, Path, Transform,
};

/// Convert a LightBurn file to an SVG file
pub fn lightburn_to_svg(content: &str) -> Result<String, ConversionError> {
    let lightburn_file = lightburn::parse(content)?;
    svg::generate(&lightburn_file)
}

/// Convert an SVG file to a LightBurn file
/// 
/// # Arguments
/// 
/// * `content` - The SVG file content
/// * `format` - The LightBurn format version (1 or 2)
pub fn svg_to_lightburn(content: &str, format: u8) -> Result<String, ConversionError> {
    let svg_file = svg::parse(content)?;
    match format {
        1 => lightburn::generate_v1(&svg_file),
        2 => lightburn::generate_v2(&svg_file),
        _ => Err(ConversionError::UnsupportedFormat(format)),
    }
}
