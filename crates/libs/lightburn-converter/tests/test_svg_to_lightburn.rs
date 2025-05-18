   //! Tests for SVG to LightBurn conversion

use lightburn_converter::svg_to_lightburn;
use std::path::PathBuf;
use std::fs;

// Helper function to get path to test fixture files
fn fixture_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(filename);
    path
}

// Create fixture directory if it doesn't exist
fn ensure_fixtures_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    Ok(path)
}

// Create a sample SVG file for testing
fn create_test_svg_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let fixtures_dir = ensure_fixtures_dir()?;
    let test_file = fixtures_dir.join("test_circle.svg");
    
    // Simple SVG file with a circle
    let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="200mm" height="200mm" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <circle cx="100" cy="100" r="56.693" stroke="black" stroke-width="1" fill="none"/>
</svg>"#;
    
    fs::write(&test_file, content)?;
    Ok(test_file)
}

// Create a more complex SVG with multiple elements
fn create_test_complex_svg_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let fixtures_dir = ensure_fixtures_dir()?;
    let test_file = fixtures_dir.join("test_complex.svg");
    
    // SVG with rectangle, ellipse and path
    let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="200mm" height="200mm" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <rect x="10" y="10" width="50" height="30" stroke="black" stroke-width="1" fill="none"/>
  <ellipse cx="100" cy="50" rx="40" ry="20" stroke="black" stroke-width="1" fill="none"/>
  <path d="M 10,100 C 20,80 40,80 50,100 S 80,120 90,100" stroke="black" stroke-width="1" fill="none"/>
  <g transform="translate(100, 100)">
    <rect x="0" y="0" width="20" height="20" stroke="black" stroke-width="1" fill="none"/>
    <circle cx="30" cy="10" r="10" stroke="black" stroke-width="1" fill="none"/>
  </g>
</svg>"#;
    
    fs::write(&test_file, content)?;
    Ok(test_file)
}

#[test]
fn test_convert_svg_to_lightburn_v1() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test file
    let test_file = create_test_svg_file()?;
    
    // Read the test file
    let content = fs::read_to_string(&test_file)?;
    
    // Convert to LightBurn format 1
    let lightburn = svg_to_lightburn(&content, 1)?;
    
    // Check that the LightBurn file contains expected elements
    assert!(lightburn.contains("<LightBurnProject"));
    assert!(lightburn.contains("FormatVersion=\"0\"")); // v1 uses FormatVersion="0"
    assert!(lightburn.contains("<Shape Type=\"Ellipse\""));
    
    // Write the LightBurn file to a file for manual inspection
    let fixtures_dir = ensure_fixtures_dir()?;
    let output_file = fixtures_dir.join("test_circle_from_svg.lbrn");
    fs::write(&output_file, lightburn)?;
    
    Ok(())
}

#[test]
fn test_convert_svg_to_lightburn_v2() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test file
    let test_file = create_test_svg_file()?;
    
    // Read the test file
    let content = fs::read_to_string(&test_file)?;
    
    // Convert to LightBurn format 2
    let lightburn = svg_to_lightburn(&content, 2)?;
    
    // Check that the LightBurn file contains expected elements
    assert!(lightburn.contains("<LightBurnProject"));
    assert!(lightburn.contains("FormatVersion=\"1\"")); // v2 uses FormatVersion="1"
    assert!(lightburn.contains("<Shape Type=\"Ellipse\""));
    
    // Write the LightBurn file to a file for manual inspection
    let fixtures_dir = ensure_fixtures_dir()?;
    let output_file = fixtures_dir.join("test_circle_from_svg.lbrn2");
    fs::write(&output_file, lightburn)?;
    
    Ok(())
}

#[test]
fn test_convert_complex_svg_to_lightburn() -> Result<(), Box<dyn std::error::Error>> {
    // Create a complex test file
    let test_file = create_test_complex_svg_file()?;
    
    // Read the test file
    let content = fs::read_to_string(&test_file)?;
    
    // Convert to LightBurn format 1
    let lightburn = svg_to_lightburn(&content, 1)?;
    
    // Check that the LightBurn file contains expected elements
    assert!(lightburn.contains("<Shape Type=\"Rect\""));
    assert!(lightburn.contains("<Shape Type=\"Ellipse\""));
    assert!(lightburn.contains("<Shape Type=\"Path\""));
    assert!(lightburn.contains("<Shape Type=\"Group\""));
    
    // Write the LightBurn file to a file for manual inspection
    let fixtures_dir = ensure_fixtures_dir()?;
    let output_file = fixtures_dir.join("test_complex_from_svg.lbrn");
    fs::write(&output_file, lightburn)?;
    
    Ok(())
}

#[test]
fn test_invalid_svg() {
    // Invalid XML should return an error
    let result = svg_to_lightburn("<invalid-xml>", 1);
    assert!(result.is_err());
    
    // Empty SVG (no shapes) should work but produce a warning
    let empty_svg = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg"></svg>"#;
    
    let result = svg_to_lightburn(empty_svg, 1);
    assert!(result.is_ok());
    
    // Unsupported format should return an error
    let valid_svg = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
  <circle cx="50" cy="50" r="40" stroke="black" stroke-width="1" fill="none"/>
</svg>"#;
    
    let result = svg_to_lightburn(valid_svg, 3); // Format 3 is unsupported
    assert!(result.is_err());
}