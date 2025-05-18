   //! Tests for LightBurn to SVG conversion

use lightburn_converter::lightburn_to_svg;
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

// Create a sample LightBurn file for testing
fn create_test_lightburn_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let fixtures_dir = ensure_fixtures_dir()?;
    let test_file = fixtures_dir.join("test_circle.lbrn");
    
    // Simple LightBurn file with a circle
    let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<LightBurnProject AppVersion="1.7.08" DeviceName="GRBL-PicoCNC" FormatVersion="0" MaterialHeight="0" MirrorX="False" MirrorY="False">
    <Thumbnail Source=""/>
    <VariableText>
        <Start Value="0"/>
        <End Value="999"/>
        <Current Value="0"/>
        <Increment Value="1"/>
        <AutoAdvance Value="0"/>
    </VariableText>
    <CutSetting type="Cut">
        <index Value="0"/>
        <name Value="Schneiden"/>
        <minPower Value="17.5"/>
        <maxPower Value="90"/>
        <maxPower2 Value="20"/>
        <speed Value="4.16667"/>
    </CutSetting>
    <Shape Type="Ellipse" CutIndex="0" Rx="56.693001" Ry="56.693001">
        <XForm>1 0 0 1 100 100</XForm>
    </Shape>
    <Notes ShowOnLoad="0" Notes=""/>
</LightBurnProject>"#;
    
    fs::write(&test_file, content)?;
    Ok(test_file)
}

#[test]
fn test_convert_lightburn_to_svg() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test file
    let test_file = create_test_lightburn_file()?;
    
    // Read the test file
    let content = fs::read_to_string(&test_file)?;
    
    // Convert to SVG
    let svg = lightburn_to_svg(&content)?;
    
    // Check that the SVG contains expected elements
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<ellipse"));
    
    // Write the SVG to a file for manual inspection if needed
    let fixtures_dir = ensure_fixtures_dir()?;
    let output_file = fixtures_dir.join("test_circle.svg");
    fs::write(&output_file, svg)?;
    
    Ok(())
}

#[test]
fn test_convert_lightburn2_to_svg() -> Result<(), Box<dyn std::error::Error>> {
    // Create fixtures directory
    let fixtures_dir = ensure_fixtures_dir()?;
    let test_file = fixtures_dir.join("test_circle.lbrn2");
    
    // Simple LightBurn v2 file with a circle
    let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<LightBurnProject AppVersion="1.7.08" DeviceName="GRBL-PicoCNC" FormatVersion="1" MaterialHeight="0" MirrorX="False" MirrorY="False">
    <Thumbnail Source=""/>
    <VariableText>
        <Start Value="0"/>
        <End Value="999"/>
        <Current Value="0"/>
        <Increment Value="1"/>
        <AutoAdvance Value="0"/>
    </VariableText>
    <CutSetting type="Cut">
        <index Value="0"/>
        <name Value="Schneiden"/>
        <minPower Value="17.5"/>
        <maxPower Value="90"/>
        <maxPower2 Value="20"/>
        <speed Value="4.16667"/>
    </CutSetting>
    <Shape Type="Ellipse" CutIndex="0" Rx="56.693001" Ry="56.693001">
        <XForm>1 0 0 1 100 100</XForm>
    </Shape>
    <Notes ShowOnLoad="0" Notes=""/>
</LightBurnProject>"#;
    
    fs::write(&test_file, content)?;
    
    // Read the test file
    let content = fs::read_to_string(&test_file)?;
    
    // Convert to SVG
    let svg = lightburn_to_svg(&content)?;
    
    // Check that the SVG contains expected elements
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<ellipse"));
    
    // Write the SVG to a file for manual inspection if needed
    let output_file = fixtures_dir.join("test_circle_v2.svg");
    fs::write(&output_file, svg)?;
    
    Ok(())
}

#[test]
fn test_parse_invalid_xml() {
    // Invalid XML should return an error
    let result = lightburn_to_svg("<invalid-xml>");
    assert!(result.is_err());
}