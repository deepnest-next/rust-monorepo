#![deny(clippy::all)]
#[macro_use]
extern crate napi_derive;
mod path_data_parser;
mod points_on_curve;
mod points_on_path;
mod convex_hull;
use std::fs;
use std::path::Path;
use usvg::Tree;

// Export both internal functions for benchmarking and testing
pub use points_on_path::points_on_path;
pub use points_on_path::points_on_path_with_closed_info;
pub use points_on_curve::Point;
pub use convex_hull::compute_convex_hull;


// Thread-local cache for reusing the panic handler
thread_local! {
    static PANIC_MSG: std::cell::RefCell<String> = std::cell::RefCell::new(String::with_capacity(128));
}

#[napi]
pub fn points_on_svg_path(
  path: String,
  tolerance: Option<f64>,
  distance: Option<f64>,
) -> napi::Result<Vec<Vec<points_on_curve::Point>>> {
  // Wrap the call with panic handling to avoid crashing Node.js
  // We use thread-local storage for panic messages to avoid allocations in the hot path
  PANIC_MSG.with(|msg_cell| {
    let msg = &mut *msg_cell.borrow_mut();
    msg.clear();
    
    let result = std::panic::catch_unwind(|| {
      points_on_path::points_on_path(path, tolerance, distance)
    });
    
    match result {
      Ok(path_result) => path_result,
      Err(err) => {
        if let Some(s) = err.downcast_ref::<String>() {
          *msg = s.clone();
        } else if let Some(s) = err.downcast_ref::<&str>() {
          *msg = s.to_string();
        } else {
          *msg = "Unknown internal error".to_string();
        }
        Err(napi::Error::from_reason(format!("Internal error processing SVG path: {}", msg)))
      }
    }
  })
}

#[napi]
pub fn points_on_svg_path_with_closed_info(
  path: String,
  tolerance: Option<f64>,
  distance: Option<f64>,
) -> napi::Result<points_on_path::PathResult> {
  // Wrap the call with panic handling to avoid crashing Node.js
  PANIC_MSG.with(|msg_cell| {
    let msg = &mut *msg_cell.borrow_mut();
    msg.clear();
    
    let result = std::panic::catch_unwind(|| {
      points_on_path::points_on_path_with_closed_info(path, tolerance, distance)
    });
    
    match result {
      Ok(path_result) => path_result,
      Err(err) => {
        if let Some(s) = err.downcast_ref::<String>() {
          *msg = s.clone();
        } else if let Some(s) = err.downcast_ref::<&str>() {
          *msg = s.to_string();
        } else {
          *msg = "Unknown internal error".to_string();
        }
        Err(napi::Error::from_reason(format!("Internal error processing SVG path: {}", msg)))
      }
    }
  })
}

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
