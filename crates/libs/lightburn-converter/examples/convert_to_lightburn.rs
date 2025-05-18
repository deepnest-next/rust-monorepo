   //! Example demonstrating how to convert SVG to LightBurn

use std::fs;
use lightburn_converter::svg_to_lightburn;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: {} <input.svg> <output.lbrn> [format_version]", args[0]);
        println!("format_version: 1 for .lbrn or 2 for .lbrn2 (default: 1)");
        return Ok(());
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    let format = args.get(3).map_or(1, |s| s.parse().unwrap_or(1));
    
    println!("Reading SVG file: {}", input_path);
    let content = fs::read_to_string(input_path)?;
    
    println!("Converting to LightBurn format {} ...", format);
    let lightburn = svg_to_lightburn(&content, format)?;
    
    println!("Writing LightBurn file: {}", output_path);
    fs::write(output_path, lightburn)?;
    
    println!("Successfully converted {} to {} (format version: {})", 
             input_path, output_path, format);
    
    Ok(())
}