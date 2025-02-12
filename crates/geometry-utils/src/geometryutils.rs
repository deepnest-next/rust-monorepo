use crate::constants::DEFAULT_TOLERANCE;
use deepnest_types::{Point, Rect};
use derive_more::{From, Into};
use napi::bindgen_prelude::*;

#[derive(Debug, Clone, Copy, From, Into)]
#[napi]
pub struct GeometryUtils;

#[napi]
impl GeometryUtils {
  /// Returns `true` if `a` and `b` are approximately equal within the given tolerance.
  /// If `tolerance` is `None`, a default tolerance of `1e-9` is used.
  #[napi]
  pub fn almost_equal(
    a: f64,
    b: f64,
    tolerance: Option<f64>,
  ) -> bool {
    let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
    // TODO: is origin "<" or "<=" correct?
    (a - b).abs() <= tol
  }

  /// Calculate the area
  #[napi]
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

  /// get the polygon bounds
  #[napi]
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

  /// is p1 and p2 within distance
  #[napi]
  pub fn within_distance(
    p1: Point,
    p2: Point,
    distance: f64,
  ) -> bool {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;

    // TODO: is origin "<" or "<=" correct?
    (dx * dx) + (dy * dy) <= (distance * distance)
  }

  /// Computes the intersection of line AB with line EF.
  ///
  /// The lines are given in point–slope form as follows:
  /// - Line 1 passes through points A and B.
  /// - Line 2 passes through points E and F.
  ///
  /// If `infinite` is `true`, the lines are treated as infinite lines. If `false`,
  /// the intersection must lie within both finite segments (within tolerance).
  ///
  /// Returns `Some(Point)` if a valid intersection exists, or `None` if:
  /// - The lines are parallel or nearly parallel,
  /// - The computed intersection is not finite, or
  /// - For finite segments, the intersection lies outside at least one segment.
  #[napi]
  pub fn line_intersect(
    A: Point,
    B: Point,
    E: Point,
    F: Point,
    infinite: bool,
  ) -> Option<Point> {
    // Compute coefficients for the line equations:
    // For AB: a1 * x + b1 * y + c1 = 0
    let a1 = B.y - A.y;
    let b1 = A.x - B.x;
    let c1 = B.x * A.y - A.x * B.y;

    // For EF: a2 * x + b2 * y + c2 = 0
    let a2 = F.y - E.y;
    let b2 = E.x - F.x;
    let c2 = F.x * E.y - E.x * F.y;

    // Denominator for the intersection formulas.
    let denom = a1 * b2 - a2 * b1;
    if denom.abs() < DEFAULT_TOLERANCE {
      // Lines are parallel or coincident—no unique intersection.
      return None;
    }

    // Compute the intersection point.
    let x = (b1 * c2 - b2 * c1) / denom;
    let y = (a2 * c1 - a1 * c2) / denom;
    if !x.is_finite() || !y.is_finite() {
      return None;
    }

    let intersection = Point { x, y };

    if !infinite {
      // For finite segments, the intersection must lie within the bounding box of each segment.
      if !in_range(x, A.x, B.x, None) || !in_range(y, A.y, B.y, None) {
        return None;
      }
      if !in_range(x, E.x, F.x, None) || !in_range(y, E.y, F.y, None) {
        return None;
      }
    }

    Some(intersection)
  }

  /// Returns true if the Euclidean distance between points `a` and `b` is less than the given tolerance.
  ///
  /// If `tolerance` is `None`, the default tolerance `DEFAULT_TOLERANCE` is used.
  #[napi]
  pub fn almost_equal_points(
    a: Point,
    b: Point,
    tolerance: Option<f64>,
  ) -> bool {
    let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy) < (tol * tol)
  }
}

// START::Helper Functions

/// Convert Degrees to Radians
pub fn degrees_to_radians(angle: f64) -> f64 {
  angle * (std::f64::consts::PI / 180.0)
}

/// Convert Radians to Degrees
pub fn radians_to_degrees(angle: f64) -> f64 {
  angle * (180.0 / std::f64::consts::PI)
}

/// Normalizes the given vector into a unit vector.
/// If the vector is already of unit length (within tolerance), it is returned as-is.
// TODO: was Vector = Point?
pub fn normalize_vector(v: Point) -> Point {
  let squared_length = v.x * v.x + v.y * v.y;
  if GeometryUtils::almost_equal(squared_length, 1.0, None) {
    return v; // The vector is already normalized.
  }
  let len = squared_length.sqrt();
  let inverse = 1.0 / len;
  Point {
    x: v.x * inverse,
    y: v.y * inverse,
  }
}

/// Returns `true` if point `p` lies strictly on the line segment defined by `A` and `B`,
/// excluding the endpoints.
pub fn on_segment(
  A: Point,
  B: Point,
  p: Point,
  tolerance: Option<f64>,
) -> bool {
  let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
  // Exclude endpoints.
  if (GeometryUtils::almost_equal(p.x, A.x, Some(tol))
    && GeometryUtils::almost_equal(p.y, A.y, Some(tol)))
    || (GeometryUtils::almost_equal(p.x, B.x, Some(tol))
      && GeometryUtils::almost_equal(p.y, B.y, Some(tol)))
  {
    return false;
  }

  // Check that p lies within the bounding box of A and B.
  let (min_x, max_x) = if A.x < B.x { (A.x, B.x) } else { (B.x, A.x) };
  let (min_y, max_y) = if A.y < B.y { (A.y, B.y) } else { (B.y, A.y) };
  if p.x < min_x - tol || p.x > max_x + tol || p.y < min_y - tol || p.y > max_y + tol {
    return false;
  }

  // Check collinearity using the cross product.
  let cross = (p.y - A.y) * (B.x - A.x) - (p.x - A.x) * (B.y - A.y);
  if cross.abs() > tol {
    return false;
  }

  // Check that p is strictly between A and B via the dot product.
  let dot = (p.x - A.x) * (B.x - A.x) + (p.y - A.y) * (B.y - A.y);
  if dot <= tol {
    return false;
  }
  let len2 = (B.x - A.x).powi(2) + (B.y - A.y).powi(2);
  if dot >= len2 - tol {
    return false;
  }

  true
}

/*
/// Returns `true` if point `p` lies strictly on the line segment defined by `A` and `B`,
/// excluding the endpoints.
fn on_segment(A: Point, B: Point, p: Point, tolerance: Option<f64>) -> bool {
  let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
  // Vertical line check.
  if GeometryUtils::almost_equal(A.x, B.x, Some(tol)) && GeometryUtils::almost_equal(p.x, A.x, tol) {
      if !GeometryUtils::almost_equal(p.y, A.y, tolerance)
          && !GeometryUtils::almost_equal(p.y, B.y, tolerance)
          && (p.y < A.y.max(B.y))
          && (p.y > A.y.min(B.y))
      {
          return true;
      } else {
          return false;
      }
  }

  // Horizontal line check.
  if GeometryUtils::almost_equal(A.y, B.y, tolerance) && GeometryUtils::almost_equal(p.y, A.y, tolerance) {
      if !GeometryUtils::almost_equal(p.x, A.x, tolerance)
          && !GeometryUtils::almost_equal(p.x, B.x, tolerance)
          && (p.x < A.x.max(B.x))
          && (p.x > A.x.min(B.x))
      {
          return true;
      } else {
          return false;
      }
  }

  // Range check: if p is outside the bounding box of A and B, it cannot lie on the segment.
  if (p.x < A.x && p.x < B.x)
      || (p.x > A.x && p.x > B.x)
      || (p.y < A.y && p.y < B.y)
      || (p.y > A.y && p.y > B.y)
  {
      return false;
  }

  // Exclude endpoints.
  if (GeometryUtils::almost_equal(p.x, A.x, tolerance) && GeometryUtils::almost_equal(p.y, A.y, tolerance))
      || (GeometryUtils::almost_equal(p.x, B.x, tolerance) && GeometryUtils::almost_equal(p.y, B.y, tolerance))
  {
      return false;
  }

  // Check colinearity via the cross product.
  let cross = (p.y - A.y) * (B.x - A.x) - (p.x - A.x) * (B.y - A.y);
  if cross.abs() > tolerance {
      return false;
  }

  // Check that p lies strictly between A and B using dot products.
  let dot = (p.x - A.x) * (B.x - A.x) + (p.y - A.y) * (B.y - A.y);
  if dot < 0.0 || GeometryUtils::almost_equal(dot, 0.0, tolerance) {
      return false;
  }

  let len2 = (B.x - A.x).powi(2) + (B.y - A.y).powi(2);
  if dot > len2 || GeometryUtils::almost_equal(dot, len2, tolerance) {
      return false;
  }

  true
}
 */

/*
/// Computes the intersection of line AB with line EF.
///
/// If `infinite` is `true`, AB and EF are treated as infinite lines. Otherwise,
/// the intersection must lie strictly within the finite segments (excluding endpoints).
///
/// Returns `Some(Point)` if there is a valid intersection, or `None` otherwise.
fn line_intersect(A: Point, B: Point, E: Point, F: Point, infinite: bool) -> Option<Point> {
  // Compute coefficients for the line equations:
  // a1 * x + b1 * y + c1 = 0 for line AB, and
  // a2 * x + b2 * y + c2 = 0 for line EF.
  let a1 = B.y - A.y;
  let b1 = A.x - B.x;
  let c1 = B.x * A.y - A.x * B.y;

  let a2 = F.y - E.y;
  let b2 = E.x - F.x;
  let c2 = F.x * E.y - E.x * F.y;

  // Compute the denominator of the intersection formulas.
  let denom = a1 * b2 - a2 * b1;

  // Compute the intersection point.
  let x = (b1 * c2 - b2 * c1) / denom;
  let y = (a2 * c1 - a1 * c2) / denom;

  // Check for numerical issues (e.g. division by zero resulting in infinite values).
  if !x.is_finite() || !y.is_finite() {
      return None;
  }

  // When the segments are finite, ensure the intersection lies within both segments.
  if !infinite {
      // For segment AB, if the x-coordinates differ significantly, check x-range.
      if (A.x - B.x).abs() > DEFAULT_TOLERANCE && (x < A.x.min(B.x) || x > A.x.max(B.x)) {
          return None;
      }
      // For segment AB, if the y-coordinates differ significantly, check y-range.
      if (A.y - B.y).abs() > DEFAULT_TOLERANCE && (y < A.y.min(B.y) || y > A.y.max(B.y)) {
          return None;
      }

      // For segment EF, perform similar checks.
      if (E.x - F.x).abs() > DEFAULT_TOLERANCE && (x < E.x.min(F.x) || x > E.x.max(F.x)) {
          return None;
      }
      if (E.y - F.y).abs() > DEFAULT_TOLERANCE && (y < E.y.min(F.y) || y > E.y.max(F.y)) {
          return None;
      }
  }

  Some(Point { x, y })
} */

/// Returns `true` if `val` is between `a` and `b` (inclusive, within tolerance).
pub fn in_range(
  val: f64,
  a: f64,
  b: f64,
  tolerance: Option<f64>,
) -> bool {
  let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
  let (min_val, max_val) = if a < b { (a, b) } else { (b, a) };
  val >= min_val - tol && val <= max_val + tol
}

// END::Helper Functions
