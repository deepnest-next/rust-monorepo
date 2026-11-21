#![deny(clippy::all)]
#[macro_use]
extern crate napi_derive;
mod convex_hull;
mod path_data_parser;
mod points_on_curve;
mod points_on_path;
mod simplify_polygon;
pub mod svg_f64;
mod svg_flip; // added new module for SVG flip
use std::fs;
use std::path::Path;

// Export both internal functions for benchmarking and testing
pub use convex_hull::compute_convex_hull;
pub use points_on_curve::Point;
pub use points_on_path::points_on_path;
pub use points_on_path::points_on_path_with_closed_info;
pub use simplify_polygon::simplify_polygon;

// Re-export SVG flip API
pub use svg_flip::{flip_svg_file, flip_svg_string, FlipDirection, FlipSVGResult};

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

    let result =
      std::panic::catch_unwind(|| points_on_path::points_on_path(path, tolerance, distance));

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
        Err(napi::Error::from_reason(format!(
          "Internal error processing SVG path: {}",
          msg
        )))
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
        Err(napi::Error::from_reason(format!(
          "Internal error processing SVG path: {}",
          msg
        )))
      }
    }
  })
}

#[napi(object)]
pub struct LoadSVGResult {
  pub success: bool,
  pub result: String,
}

// usvg stores geometry as f32: a 1000 mm box came back 2834.6457 units wide instead of
// 2834.645669291339 (deepnest#137). `svg_f64` keeps the file's numbers in f64 and writes
// one absolute `<path>` per shape, all transforms applied. `//`, not `///`: napi copies doc
// comments into the tracked index.d.ts.
#[napi]
pub fn load_svg_string(svg_data: String, dpi_scale: Option<f64>) -> LoadSVGResult {
  let dpi = dpi_scale.unwrap_or(72.0);
  match svg_f64::parse(&svg_data, svg_f64::Units { px: 1.0, inch: dpi }) {
    Ok(svg) => LoadSVGResult {
      success: true,
      result: to_svg_string(&svg),
    },
    Err(err) => LoadSVGResult {
      success: false,
      result: format!("Fehler beim Parsen der SVG: {}", err),
    },
  }
}

#[napi]
pub fn load_svg_file(svg_path: String, dpi_scale: Option<f64>) -> LoadSVGResult {
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

  // SVGZ entpacken
  let svg_data = if svg_data.starts_with(&[0x1f, 0x8b]) {
    use std::io::Read;
    let mut out = Vec::new();
    if let Err(err) = flate2::read::GzDecoder::new(&svg_data[..]).read_to_end(&mut out) {
      return LoadSVGResult {
        success: false,
        result: format!("Fehler beim Entpacken der SVGZ: {}", err),
      };
    }
    out
  } else {
    svg_data
  };

  match String::from_utf8(svg_data) {
    Ok(text) => load_svg_string(text, dpi_scale),
    Err(_) => LoadSVGResult {
      success: false,
      result: "Fehler beim Parsen der SVG: keine gültige UTF-8-Datei".to_string(),
    },
  }
}

/// One `<path>` per visible shape, absolute coordinates, every number as the shortest
/// string that reads back to the same f64.
fn to_svg_string(svg: &svg_f64::Svg) -> String {
  use std::fmt::Write;
  use svg_f64::{Paint, Seg};
  let paint = |p: Paint| match p {
    Paint::None => "none".to_string(),
    Paint::Color([r, g, b]) => format!("#{r:02x}{g:02x}{b:02x}"),
    Paint::Server => "#000000".to_string(),
  };
  let mut out = format!(
    "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
    svg.width, svg.height
  );
  for shape in svg.shapes.iter().filter(|s| s.visible) {
    let mut d = String::new();
    for seg in &shape.segs {
      let _ = match *seg {
        Seg::Move([x, y]) => write!(d, "M {x} {y} "),
        Seg::Line([x, y]) => write!(d, "L {x} {y} "),
        Seg::Quad([x1, y1], [x, y]) => write!(d, "Q {x1} {y1} {x} {y} "),
        Seg::Cubic([x1, y1], [x2, y2], [x, y]) => write!(d, "C {x1} {y1} {x2} {y2} {x} {y} "),
        Seg::Close => write!(d, "Z "),
      };
    }
    let _ = writeln!(
      out,
      "    <path fill=\"{}\" stroke=\"{}\" d=\"{}\"/>",
      paint(shape.fill),
      paint(shape.stroke),
      d.trim_end()
    );
  }
  out.push_str("</svg>\n");
  out
}
