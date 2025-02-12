use crate::constants::{DEFAULT_CURVE_TOLERANCE,DEFAULT_TOLERANCE};
use deepnest_types::{Point, Polygon, Rect, Vector};
use derive_more::{From, Into};
use napi::bindgen_prelude::*;
use std::collections::VecDeque;



#[derive(Debug, Clone, Copy, From, Into)]
#[napi]
pub struct QuadraticBezier;

/// A helper struct representing a quadratic Bézier segment defined by
/// endpoints `p1` and `p2` and a single control point `c1`.
#[derive(Debug, Clone, Copy, From, Into)]
#[napi]
pub struct BezierSegment {
  pub p1: Point,
  pub p2: Point,
  pub c1: Point,
}

#[napi]
impl QuadraticBezier {
  /// Returns `true` if the quadratic Bézier curve defined by endpoints `p1`, `p2`
  /// and control point `c1` is sufficiently flat according to Roger Willcocks's
  /// flatness criterion with the given tolerance.
  ///
  /// The tolerance is adjusted internally as `4 * tol²`.
  // TODO: do we need it public?
  pub fn is_flat(p1: Point, p2: Point, c1: Point, tolerance: Option<f64>) -> bool {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    let tol_adjusted = 4.0 * tol * tol;
    let ux = 2.0 * c1.x - p1.x - p2.x;
    let uy = 2.0 * c1.y - p1.y - p2.y;
    (ux * ux + uy * uy) <= tol_adjusted
  }

  /// Subdivides a quadratic Bézier segment at parameter `t` (typically 0.5)
  /// using the de Casteljau algorithm.
  ///
  /// Returns a tuple with two new `BezierSegment` values representing the
  /// subdivided curves.
  // TODO: do we need it public?
  pub fn subdivide(p1: Point, p2: Point, c1: Point, tolerance: Option<f64>) -> (BezierSegment, BezierSegment) {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    let mid1 = Point {
      x: p1.x + (c1.x - p1.x) * tol,
      y: p1.y + (c1.y - p1.y) * tol,
    };
    let mid2 = Point {
      x: c1.x + (p2.x - c1.x) * tol,
      y: c1.y + (p2.y - c1.y) * tol,
    };
    let mid3 = Point {
      x: mid1.x + (mid2.x - mid1.x) * tol,
      y: mid1.y + (mid2.y - mid1.y) * tol,
    };

    let seg1 = BezierSegment {
      p1,
      p2: mid3,
      c1: mid1,
    };
    let seg2 = BezierSegment {
      p1: mid3,
      p2,
      c1: mid2,
    };

    (seg1, seg2)
  }

  /// Converts (linearizes) a quadratic Bézier curve into a sequence of points
  /// by repeatedly subdividing the curve until each segment is flat enough
  /// according to the provided tolerance.
  ///
  /// The returned vector contains the starting point and the end point of each
  /// flat segment, in order along the curve.
  #[napi]
  pub fn linearize(p1: Point, p2: Point, c1: Point, tolerance: Option<f64>) -> Vec<Point> {
    let tol = tolerance.unwrap_or(DEFAULT_CURVE_TOLERANCE);
    let mut finished = Vec::new();
    finished.push(p1);

    // Use a deque to avoid recursion stack overflows.
    let mut todo: VecDeque<BezierSegment> = VecDeque::new();
    todo.push_back(BezierSegment { p1, p2, c1 });

    while let Some(segment) = todo.pop_front() {
      if QuadraticBezier::is_flat(segment.p1, segment.p2, segment.c1, Some(tol)) {
        // When flat, accept the segment by adding its endpoint.
        finished.push(segment.p2);
      } else {
        // Subdivide the segment at t=0.5 and process both halves.
        let (seg1, seg2) = QuadraticBezier::subdivide(segment.p1, segment.p2, segment.c1, Some(0.5));
        // Insert in order so that seg1 (the first half) is processed next.
        todo.push_front(seg2);
        todo.push_front(seg1);
      }
    }
    finished
  }
}
