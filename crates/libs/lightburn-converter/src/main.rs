   //! Command-line tool for converting between LightBurn files and SVG files.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use lightburn_converter::{lightburn_to_svg, svg_to_lightburn};
use anyhow::{Result, Context};

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(
    version,
    about = "Convert between LightBurn (.lbrn/.lbrn2) and SVG files",
    long_about = "A command-line tool for converting between LightBurn laser files and SVG files."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert from LightBurn to SVG
    #[command(name = "to-svg")]
    ToSvg {
        /// Input LightBurn file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output SVG file path
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Convert from SVG to LightBurn
    #[command(name = "to-lightburn")]
    ToLightburn {
        /// Input SVG file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output LightBurn file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// LightBurn format version (1 or 2)
        #[arg(short, long, default_value = "1")]
        format: u8,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::ToSvg { input, output } => {
            println!("Converting LightBurn file {} to SVG {}", input.display(), output.display());
            
            let content = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;
            
            let svg = lightburn_to_svg(&content)
                .with_context(|| "Failed to convert LightBurn to SVG")?;
            
            fs::write(&output, svg)
                .with_context(|| format!("Failed to write output file: {}", output.display()))?;
            
            println!("Successfully converted {} to {}", input.display(), output.display());
        },
        
        Commands::ToLightburn { input, output, format } => {
            println!("Converting SVG file {} to LightBurn {} (format {})", 
                    input.display(), output.display(), format);
            
            let content = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;
            
            let lightburn = svg_to_lightburn(&content, format)
                .with_context(|| "Failed to convert SVG to LightBurn")?;
            
            fs::write(&output, lightburn)
                .with_context(|| format!("Failed to write output file: {}", output.display()))?;
            
            println!("Successfully converted {} to {} (format version {})", 
                    input.display(), output.display(), format);
        },
    }
    
    Ok(())
}