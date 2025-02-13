#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use std::fs;
use std::path::Path;
use usvg::Tree;

#[napi(object)]
pub struct LoadSVGResult {
  pub success: bool,
  pub result: String,
}

#[napi]
pub fn load_svg_string(svg_data: String) -> LoadSVGResult {
  match Tree::from_str(&svg_data, &usvg::Options::default()) {
    Ok(tree) => {
      let output = tree.to_string(&usvg::WriteOptions::default());
      LoadSVGResult {
        success: true,
        result: output,
      }
    }
    Err(err) => LoadSVGResult {
      success: false,
      result: format!("Fehler beim Parsen der SVG: {}", err),
    },
  }
}

#[napi]
pub fn load_svg_file(svg_path: String) -> LoadSVGResult {
  // Konvertieren Sie den String in ein Path
  let path = Path::new(&svg_path);

  // Prüfen, ob die Datei existiert
  if !path.exists() {
    return LoadSVGResult {
      success: false,
      result: format!("Datei nicht gefunden: {}", svg_path),
    };
  }

  // SVG-Datei einlesen
  let svg_data = match fs::read(path) {
    Ok(data) => data,
    Err(err) => {
      return LoadSVGResult {
        success: false,
        result: format!("Fehler beim Einlesen der Datei: {}", err),
      };
    }
  };

  // SVG in usvg::Tree parsen
  match Tree::from_data(&svg_data, &usvg::Options::default()) {
    Ok(tree) => {
      let output = tree.to_string(&usvg::WriteOptions::default());
      LoadSVGResult {
        success: true,
        result: output,
      }
    }
    Err(err) => LoadSVGResult {
      success: false,
      result: format!("Fehler beim Parsen der SVG: {}", err),
    },
  }
}
