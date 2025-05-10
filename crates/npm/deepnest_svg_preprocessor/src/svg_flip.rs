use std::fs;
use std::path::Path;
use usvg::Tree;

#[derive(Debug)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct FlipSVGResult {
    pub success: bool,
    pub result: String,
}

pub fn flip_svg_string(svg_data: String, direction: FlipDirection) -> FlipSVGResult {
    match Tree::from_str(&svg_data, &usvg::Options::default()) {
        Ok(tree) => {
            let size = tree.size();
            let width = size.width();
            let height = size.height();
            let transform_str = match direction {
                FlipDirection::Horizontal => format!("matrix(-1,0,0,1,{},0)", width),
                FlipDirection::Vertical => format!("matrix(1,0,0,-1,0,{})", height),
            };
            let original = tree.to_string(&usvg::WriteOptions::default());
            let output = wrap_svg_with_flip(&original, &transform_str);
            FlipSVGResult { success: true, result: output }
        },
        Err(err) => FlipSVGResult {
            success: false,
            result: format!("Error parsing SVG: {}", err),
        },
    }
}

pub fn flip_svg_file(svg_path: String, direction: FlipDirection) -> FlipSVGResult {
    let path = Path::new(&svg_path);
    if !path.exists() {
        return FlipSVGResult {
            success: false,
            result: format!("File not found: {}", svg_path),
        };
    }
    let svg_data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            return FlipSVGResult {
                success: false,
                result: format!("Error reading file: {}", err),
            };
        }
    };
    match Tree::from_str(&svg_data, &usvg::Options::default()) {
        Ok(tree) => {
            let size = tree.size();
            let width = size.width();
            let height = size.height();
            let transform_str = match direction {
                FlipDirection::Horizontal => format!("matrix(-1,0,0,1,{},0)", width),
                FlipDirection::Vertical => format!("matrix(1,0,0,-1,0,{})", height),
            };
            let original = tree.to_string(&usvg::WriteOptions::default());
            let output = wrap_svg_with_flip(&original, &transform_str);
            FlipSVGResult { success: true, result: output }
        },
        Err(err) => FlipSVGResult {
            success: false,
            result: format!("Error parsing SVG: {}", err),
        },
    }
}

fn wrap_svg_with_flip(svg: &str, transform: &str) -> String {
    if let Some(svg_tag_end) = svg.find('>') {
        let (start, rest) = svg.split_at(svg_tag_end + 1);
        if let Some(svg_close) = rest.rfind("</svg>") {
            let (inner, end) = rest.split_at(svg_close);
            return format!("{}<g transform=\"{}\">{}</g>{}", start, transform, inner, end);
        }
    }
    svg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // A basic SVG for testing.
    const SIMPLE_SVG: &str = r#"<svg width="100" height="200" xmlns="http://www.w3.org/2000/svg">
<rect width="100" height="200" fill="red"/>
</svg>"#;

    #[test]
    fn test_flip_svg_string_horizontal() {
        let result = flip_svg_string(SIMPLE_SVG.to_string(), FlipDirection::Horizontal);
        assert!(result.success, "Flip operation should succeed");
        // Check if the output contains the transformation wrapper.
        assert!(result.result.contains("<g transform=\"matrix(-1,0,0,1,"), "Output should include horizontal flip matrix");
        println!("{}", result.result);
    }

    #[test]
    fn test_flip_svg_string_vertical() {
        let result = flip_svg_string(SIMPLE_SVG.to_string(), FlipDirection::Vertical);
        assert!(result.success, "Flip operation should succeed");
        assert!(result.result.contains("<g transform=\"matrix(1,0,0,-1,0,"), "Output should include vertical flip matrix");
        println!("{}", result.result);
    }
}
