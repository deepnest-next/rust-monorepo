use derive_more::{From, Into};

use crate::types::point::Point;

/// Polygon
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, From, Into)]
pub struct Polygon {
  pub points: Vec<Point>,
  pub children: Option<Vec<Vec<Point>>>,
  pub offsetx:Option<f64>,
  pub offsety:Option<f64>,
}

#[cfg_attr(feature = "node", napi)]
pub fn polygon_area(polygon: Polygon) -> f64 {
  let n = polygon.points.len();
  if n < 3 {
    // Not enough points for a polygon
    return 0.0;
  }
  let mut area = 0.0;
  // The JavaScript version uses j as the index of the previous point,
  // starting with the last point for the first iteration.
  for i in 0..n {
    let j = if i == 0 { n - 1 } else { i - 1 };
    area +=
      (polygon.points[j].x + polygon.points[i].x) * (polygon.points[j].y - polygon.points[i].y);
  }
  0.5 * area
}
