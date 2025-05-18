# LightBurn Converter

A Rust library and command-line tool for converting between LightBurn (`.lbrn`/`.lbrn2`) and SVG files.

## Features

- Convert LightBurn files (`.lbrn` and `.lbrn2`) to SVG
- Convert SVG files to LightBurn format
- Support for basic shape types (rectangles, ellipses, paths)
- Library and CLI interfaces

## Usage

### Command Line

```bash
# Convert from LightBurn to SVG
lightburn-converter to-svg --input file.lbrn --output file.svg

# Convert from SVG to LightBurn (format version 1)
lightburn-converter to-lightburn --input file.svg --output file.lbrn --format 1

# Convert from SVG to LightBurn (format version 2)
lightburn-converter to-lightburn --input file.svg --output file.lbrn2 --format 2
```

### Library Usage

```rust
use lightburn_converter::{lightburn_to_svg, svg_to_lightburn};

// Convert LightBurn to SVG
let lightburn_content = std::fs::read_to_string("file.lbrn").unwrap();
let svg_content = lightburn_to_svg(&lightburn_content).unwrap();

// Convert SVG to LightBurn (format 1 or 2)
let svg_content = std::fs::read_to_string("file.svg").unwrap();
let lightburn_content = svg_to_lightburn(&svg_content, 1).unwrap(); // Format version 1
   serde = { version = "1.0", features = ["derive"] }
   serde_xml_rs = "0.5"
   quick-xml = "0.22"
   xml-rs = "0.8"
   ```

### Step 2: Define the Data Structures

You will need to define the data structures that represent the LightBurn files. Here’s an example of how you might define these structures using Serde for serialization and deserialization.

```rust
// src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LightBurnProject {
    pub app_version: String,
    pub device_name: String,
    pub format_version: String,
    // Add other fields as necessary
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightBurnProjectV2 {
    pub app_version: String,
    pub device_name: String,
    pub format_version: String,
    // Add other fields as necessary
}
```

### Step 3: Implement the Conversion Logic

You will need functions to read LightBurn files, convert them to SVG, and vice versa. Below is a simplified example of how you might implement these functions.

```rust
// src/lib.rs
use std::fs;
use std::path::Path;

pub fn convert_lbrn_to_svg(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input_path)?;
    let project: LightBurnProject = serde_xml_rs::from_str(&content)?;

    // Convert to SVG format (this is a placeholder, implement actual conversion logic)
    let svg_content = format!("<svg><!-- Converted from LightBurn --></svg>");
    fs::write(output_path, svg_content)?;

    Ok(())
}

pub fn convert_svg_to_lbrn(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input_path)?;

    // Parse SVG and convert to LightBurn format (this is a placeholder)
    let project = LightBurnProject {
        app_version: "1.0".to_string(),
        device_name: "Default".to_string(),
        format_version: "1".to_string(),
    };

    let lbrn_content = serde_xml_rs::to_string(&project)?;
    fs::write(output_path, lbrn_content)?;

    Ok(())
}
```

### Step 4: Create a Command-Line Interface

You can use the `clap` crate to create a command-line interface for your converter.

1. **Add `clap` to your dependencies**:
   ```toml
   [dependencies]
   clap = "3.0"
   ```

2. **Implement the CLI**:
```rust
// src/main.rs
use clap::{App, Arg};
use lightburn_converter::{convert_lbrn_to_svg, convert_svg_to_lbrn};

fn main() {
    let matches = App::new("LightBurn Converter")
        .version("1.0")
        .author("Your Name <you@example.com>")
        .about("Converts LightBurn files to SVG and vice versa")
        .arg(
            Arg::new("input")
                .about("Input file path")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .about("Output file path")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("to-svg")
                .about("Convert LightBurn to SVG")
                .short('s')
                .long("to-svg"),
        )
        .arg(
            Arg::new("to-lbrn")
                .about("Convert SVG to LightBurn")
                .short('l')
                .long("to-lbrn"),
        )
        .get_matches();

    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    if matches.is_present("to-svg") {
        if let Err(e) = convert_lbrn_to_svg(input, output) {
            eprintln!("Error converting to SVG: {}", e);
        }
    } else if matches.is_present("to-lbrn") {
        if let Err(e) = convert_svg_to_lbrn(input, output) {
            eprintln!("Error converting to LightBurn: {}", e);
        }
    } else {
        eprintln!("Please specify a conversion direction: --to-svg or --to-lbrn");
    }
}
```

### Step 5: Build and Run the Project

1. **Build the project**:
   ```bash
   cargo build
   ```

2. **Run the converter**:
   ```bash
   cargo run -- <input_file> <output_file> --to-svg
   ```

### Notes

- The conversion logic in the provided code is a placeholder. You will need to implement the actual conversion logic based on the structure of the LightBurn files and the desired SVG output.
- Make sure to handle errors appropriately and validate the input files.
- You may want to expand the data structures to include all relevant fields from the LightBurn files.
- Consider adding tests to ensure the conversion works as expected.

This should give you a solid starting point for your LightBurn file converter in Rust!