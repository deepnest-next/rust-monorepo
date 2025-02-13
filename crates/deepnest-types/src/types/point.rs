use derive_more::{From, Into};
use std::cmp::Ordering;

/// Point
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Copy, PartialEq, From, Into)]
pub struct Point {
  pub x: f64,
  pub y: f64,
}

#[cfg_attr(feature = "node", napi)]
pub fn rotate_point(point: Point, angle: f64) -> Point {
  let cos = angle.cos();
  let sin = angle.sin();
  Point {
    x: point.x * cos - point.y * sin,
    y: point.x * sin + point.y * cos,
  }
}
