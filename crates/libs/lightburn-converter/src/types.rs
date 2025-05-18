//! Common data types used for conversion

use std::collections::HashMap;
use thiserror::Error;

/// Error type for conversion operations
#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Failed to parse XML: {0}")]
    XmlParseError(String),
    
    #[error("Failed to generate XML: {0}")]
    XmlGenerateError(String),
    
    #[error("Unsupported LightBurn format version: {0}")]
    UnsupportedFormat(u8),
    
    #[error("Unsupported shape type: {0}")]
    UnsupportedShapeType(String),
    
    #[error("Invalid shape data: {0}")]
    InvalidShapeData(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Represents a LightBurn file structure
#[derive(Debug, Clone)]
pub struct LightburnFile {
    pub version: String,
    pub app_version: String,
    pub device_name: String,
    pub material_height: f64,
    pub mirror_x: bool,
    pub mirror_y: bool,
    pub thumbnail: Option<String>,
    pub cut_settings: Vec<CutSetting>,
    pub shapes: Vec<Shape>,
}

/// Represents an SVG file structure
#[derive(Debug, Clone)]
pub struct SvgFile {
    pub width: Option<String>,
    pub height: Option<String>,
    pub view_box: Option<String>,
    pub shapes: Vec<Shape>,
}

/// Represents a CutSetting in LightBurn
#[derive(Debug, Clone)]
pub struct CutSetting {
    pub index: usize,
    pub name: String,
    pub min_power: f64,
    pub max_power: f64,
    pub speed: f64,
    pub properties: HashMap<String, String>,
}

/// Types of shapes supported
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    Rect,
    Ellipse,
    Path,
    Group,
    Other(String),
}

/// Transformation matrix
#[derive(Debug, Clone)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

/// Represents a point in a path
#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub c0x: Option<f64>,
    pub c0y: Option<f64>,
    pub c1x: Option<f64>,
    pub c1y: Option<f64>,
}

/// Path data for path shapes
#[derive(Debug, Clone)]
pub struct Path {
    pub points: Vec<Point>,
    pub commands: Vec<PathCommand>,
}

/// Path command types for SVG path data
#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo(usize, usize),
    LineTo(usize, usize),
    BezierTo(usize, usize),
    Close,
}

/// Represents a shape in both LightBurn and SVG
#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub cut_index: Option<usize>,
    pub transform: Option<Transform>,
    
    // For rectangles
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub corner_radius: Option<f64>,
    
    // For ellipses
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    
    // For paths
    pub path: Option<Path>,
    
    // For groups
    pub children: Option<Vec<Shape>>,
    
    // Common SVG attributes
    pub style: Option<String>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<String>,
}
