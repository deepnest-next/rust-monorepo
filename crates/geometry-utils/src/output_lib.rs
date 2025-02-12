use crate::constants::TOL;
use deepnest_types::{Point, Polygon, Rect};
#[cfg(feature = "node")]
use napi::bindgen_prelude::*;

/// Returns `true` if `a` and `b` are approximately equal within the given tolerance.
/// If `tolerance` is `None`, a default tolerance of `1e-9` is used.
#[cfg_attr(feature = "node", napi)]
pub fn almost_equal(a: f64, b: f64, tolerance: Option<f64>) -> bool {
  let tol = tolerance.unwrap_or(TOL);
  (a - b).abs() < tol
}

/// Calculate the area
#[cfg_attr(feature = "node", napi)]
pub fn polygon_area(polygon: Vec<Point>) -> f64 {
  let n = polygon.len();
  if n < 3 {
    // Not enough points for a polygon
    return 0.0;
  }
  let mut area = 0.0;
  // The JavaScript version uses j as the index of the previous point,
  // starting with the last point for the first iteration.
  for i in 0..n {
    let j = if i == 0 { n - 1 } else { i - 1 };
    area += (polygon[j].x + polygon[i].x) * (polygon[j].y - polygon[i].y);
  }
  0.5 * area
}

#[cfg_attr(feature = "node", napi)]
pub fn get_polygon_bounds(polygon: Vec<Point>) -> Option<Rect> {
  // Ensure the polygon has at least 3 points
  if polygon.len() < 3 {
    return None;
  }

  // Start with the first point as the initial bounds
  let first = polygon[0];
  let (xmin, xmax, ymin, ymax) = polygon.iter().skip(1).fold(
    (first.x, first.x, first.y, first.y),
    |(xmin, xmax, ymin, ymax), p| (xmin.min(p.x), xmax.max(p.x), ymin.min(p.y), ymax.max(p.y)),
  );

  Some(Rect {
    x: xmin,
    y: ymin,
    width: xmax - xmin,
    height: ymax - ymin,
  })
}

#[cfg_attr(feature = "node", napi)]
pub fn within_distance(p1: Point, p2: Point, distance: f64) -> bool {
  let dx = p1.x - p2.x;
  let dy = p1.y - p2.y;
  dx * dx + dy * dy < distance * distance
}
