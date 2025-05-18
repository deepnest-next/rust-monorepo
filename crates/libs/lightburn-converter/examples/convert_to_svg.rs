   //! Example demonstrating how to convert LightBurn to SVG

use std::fs;
use lightburn_converter::lightburn_to_svg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: {} <input.lbrn> <output.svg>", args[0]);
        return Ok(());
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    println!("Reading LightBurn file: {}", input_path);
    let content = fs::read_to_string(input_path)?;
    
    println!("Converting to SVG...");
    let svg = lightburn_to_svg(&content)?;
    
    println!("Writing SVG file: {}", output_path);
    fs::write(output_path, svg)?;
    
    println!("Successfully converted {} to {}", input_path, output_path);
    
    Ok(())
}