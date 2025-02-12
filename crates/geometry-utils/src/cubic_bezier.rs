use crate::constants::DEFAULT_CURVE_TOLERANCE;
use deepnest_types::Point;
use derive_more::{From, Into};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, From, Into)]
#[napi]
pub struct CubicBezier;

/// A cubic Bézier segment defined by endpoints `p1` and `p2` and control points `c1` and `c2`.
#[derive(Debug, Clone, Copy)]
pub struct CubicBezierSegment {
  pub p1: Point,
  pub p2: Point,
  pub c1: Point,
  pub c2: Point,
}
#[napi]
impl CubicBezier {
  /// Returns `true` if the cubic Bézier curve defined by endpoints `p1` and `p2` and control points
  /// `c1` and `c2` is flat enough according to the modified Roger Willcocks criterion.
  ///
  /// Internally, the tolerance is adjusted as: `tol_adjusted = 16 * tol²`.
  // TODO: do we need it public?
  pub fn is_flat(
    p1: Point,
    p2: Point,
    c1: Point,
    c2: Point,
    tolerance: Option<f64>,
  ) -> bool {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    let tol_adjusted = 16.0 * tol * tol;

    let mut ux = 3.0 * c1.x - 2.0 * p1.x - p2.x;
    ux = ux * ux;

    let mut uy = 3.0 * c1.y - 2.0 * p1.y - p2.y;
    uy = uy * uy;

    let mut vx = 3.0 * c2.x - 2.0 * p2.x - p1.x;
    vx = vx * vx;

    let mut vy = 3.0 * c2.y - 2.0 * p2.y - p1.y;
    vy = vy * vy;

    if ux < vx {
      ux = vx;
    }
    if uy < vy {
      uy = vy;
    }

    ux + uy <= tol_adjusted
  }

  /// Subdivides a cubic Bézier segment at the parameter `t` (commonly 0.5) using de Casteljau’s algorithm.
  ///
  /// Returns two new `CubicBezierSegment` values representing the subdivided curves.
  // TODO: do we need it public?
  pub fn subdivide(
    p1: Point,
    p2: Point,
    c1: Point,
    c2: Point,
    tolerance: Option<f64>,
  ) -> (CubicBezierSegment, CubicBezierSegment) {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    // Compute the linear interpolations.
    let mid1 = Point {
      x: p1.x + (c1.x - p1.x) * tol,
      y: p1.y + (c1.y - p1.y) * tol,
    };

    let mid2 = Point {
      x: c2.x + (p2.x - c2.x) * tol,
      y: c2.y + (p2.y - c2.y) * tol,
    };

    let mid3 = Point {
      x: c1.x + (c2.x - c1.x) * tol,
      y: c1.y + (c2.y - c1.y) * tol,
    };

    // Further interpolate to get the subdivision points.
    let mida = Point {
      x: mid1.x + (mid3.x - mid1.x) * tol,
      y: mid1.y + (mid3.y - mid1.y) * tol,
    };

    let midb = Point {
      x: mid3.x + (mid2.x - mid3.x) * tol,
      y: mid3.y + (mid2.y - mid3.y) * tol,
    };

    let midx = Point {
      x: mida.x + (midb.x - mida.x) * tol,
      y: mida.y + (midb.y - mida.y) * tol,
    };

    let seg1 = CubicBezierSegment {
      p1,
      p2: midx,
      c1: mid1,
      c2: mida,
    };

    let seg2 = CubicBezierSegment {
      p1: midx,
      p2,
      c1: midb,
      c2: mid2,
    };

    (seg1, seg2)
  }

  /// Approximates (linearizes) a cubic Bézier curve by subdividing until each segment is flat enough.
  ///
  /// The function returns a vector of points along the curve. The initial point `p1` is included,
  /// and each flat segment’s end point is appended.
  #[napi]
  pub fn linearize(
    p1: Point,
    p2: Point,
    c1: Point,
    c2: Point,
    tolerance: Option<f64>,
  ) -> Vec<Point> {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    let mut finished = Vec::new();
    finished.push(p1);

    let mut todo: VecDeque<CubicBezierSegment> = VecDeque::new();
    todo.push_back(CubicBezierSegment { p1, p2, c1, c2 });

    // Iteratively process segments until all are sufficiently flat.
    while let Some(segment) = todo.pop_front() {
      if CubicBezier::is_flat(segment.p1, segment.p2, segment.c1, segment.c2, Some(tol)) {
        // When the segment is flat, add its endpoint.
        finished.push(segment.p2);
      } else {
        // Otherwise, subdivide the segment and process both halves.
        let (seg1, seg2) =
          CubicBezier::subdivide(segment.p1, segment.p2, segment.c1, segment.c2, Some(0.5));
        // Push seg1 and seg2 to the front so that they are processed next.
        todo.push_front(seg2);
        todo.push_front(seg1);
      }
    }
    finished
  }
}
