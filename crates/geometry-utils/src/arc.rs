use deepnest_types::Point;
use derive_more::{From, Into};
use napi::bindgen_prelude::*;
use std::collections::VecDeque;

use crate::geometryutils::{degrees_to_radians, radians_to_degrees, GeometryUtils};

#[derive(Debug, Clone, Copy, From, Into)]
#[napi]
pub struct Arc;

/// A center–arc representation.
/// This defines an elliptical arc by its center, radii, a starting angle (`theta`),
/// a sweep (`extent` in degrees), and the x‑axis rotation (`angle`, in degrees).
#[derive(Debug, Clone, Copy)]
pub struct CenterArc {
  pub center: Point,
  pub rx: f64,
  pub ry: f64,
  pub theta: f64,  // starting angle in degrees
  pub extent: f64, // sweep (extent) in degrees
  pub angle: f64,  // x-axis rotation in degrees
}

/// An SVG-style arc description.
/// (The SVG arc command is defined by its start point `p1`, end point `p2`, radii,
/// x‑axis rotation (in radians), and the large‑arc and sweep flags.)
#[derive(Debug, Clone, Copy)]
pub struct SvgArc {
  pub p1: Point,
  pub p2: Point,
  pub rx: f64,
  pub ry: f64,
  pub angle: f64, // in radians
  pub largearc: bool,
  pub sweep: bool,
}

/// Convert from a center–arc definition to an SVG-style arc.
///
/// This function is ported from techniques described in:
/// http://commons.oreilly.com/wiki/index.php/SVG_Essentials/Paths
///
/// # Parameters
/// - `center`: center of the ellipse.
/// - `rx`, `ry`: radii of the ellipse.
/// - `theta1`: start angle (in degrees).
/// - `extent`: sweep (extent) (in degrees).
/// - `angle_degrees`: x‑axis rotation (in degrees).
///
/// # Returns
/// An `SvgArc` with computed start (`p1`) and end (`p2`) points.
pub fn center_to_svg(
  center: Point,
  rx: f64,
  ry: f64,
  theta1: f64,
  extent: f64,
  angle_degrees: f64,
) -> SvgArc {
  let theta2 = theta1 + extent;

  let theta1_rad = degrees_to_radians(theta1);
  let theta2_rad = degrees_to_radians(theta2);
  let angle_rad = degrees_to_radians(angle_degrees);

  let cos_angle = angle_rad.cos();
  let sin_angle = angle_rad.sin();

  let t1cos = theta1_rad.cos();
  let t1sin = theta1_rad.sin();
  let t2cos = theta2_rad.cos();
  let t2sin = theta2_rad.sin();

  // Compute the start and end points.
  let x0 = center.x + cos_angle * rx * t1cos - sin_angle * ry * t1sin;
  let y0 = center.y + sin_angle * rx * t1cos + cos_angle * ry * t1sin;
  let x1 = center.x + cos_angle * rx * t2cos - sin_angle * ry * t2sin;
  let y1 = center.y + sin_angle * rx * t2cos + cos_angle * ry * t2sin;

  let largearc = extent.abs() > 180.0;
  let sweep = extent > 0.0;

  SvgArc {
    p1: Point { x: x0, y: y0 },
    p2: Point { x: x1, y: y1 },
    rx,
    ry,
    angle: angle_rad,
    largearc,
    sweep,
  }
}

/// Convert from an SVG arc definition to a center–arc representation.
///
/// # Parameters
/// - `p1`, `p2`: the start and end points of the arc.
/// - `rx`, `ry`: radii (which will be adjusted if necessary).
/// - `angle_degrees`: x‑axis rotation (in degrees).
/// - `largearc`: large‑arc flag.
/// - `sweep`: sweep flag.
///
/// # Returns
/// A `CenterArc` with the center, radii, start angle (`theta`), and sweep (`extent`)
/// in degrees.
pub fn svg_to_center(
  p1: Point,
  p2: Point,
  mut rx: f64,
  mut ry: f64,
  angle_degrees: f64,
  largearc: bool,
  sweep: bool,
) -> CenterArc {
  // Compute the midpoint and half-difference of the endpoints.
  let mid = Point {
    x: 0.5 * (p1.x + p2.x),
    y: 0.5 * (p1.y + p2.y),
  };
  let diff = Point {
    x: 0.5 * (p2.x - p1.x),
    y: 0.5 * (p2.y - p1.y),
  };

  let angle = degrees_to_radians(angle_degrees % 360.0);
  let cos_angle = angle.cos();
  let sin_angle = angle.sin();

  // Transform the difference vector.
  let x1 = cos_angle * diff.x + sin_angle * diff.y;
  let y1 = -sin_angle * diff.x + cos_angle * diff.y;

  rx = rx.abs();
  ry = ry.abs();
  let Prx = rx * rx;
  let Pry = ry * ry;
  let Px1 = x1 * x1;
  let Py1 = y1 * y1;

  // Ensure the radii are large enough.
  let radii_check = Px1 / Prx + Py1 / Pry;
  let radii_sqrt = radii_check.sqrt();
  if radii_check > 1.0 {
    rx *= radii_sqrt;
    ry *= radii_sqrt;
  }
  let Prx = rx * rx;
  let Pry = ry * ry;

  let sign = if largearc != sweep { -1.0 } else { 1.0 };
  let mut sq = (Prx * Pry - Prx * Py1 - Pry * Px1) / (Prx * Py1 + Pry * Px1);
  if sq < 0.0 {
    sq = 0.0;
  }
  let coef = sign * sq.sqrt();
  let cx1 = coef * ((rx * y1) / ry);
  let cy1 = coef * (-(ry * x1) / rx);

  // Compute the center in the original coordinate system.
  let cx = mid.x + (cos_angle * cx1 - sin_angle * cy1);
  let cy = mid.y + (sin_angle * cx1 + cos_angle * cy1);

  // Compute the start angle.
  let ux = (x1 - cx1) / rx;
  let uy = (y1 - cy1) / ry;
  let n = (ux * ux + uy * uy).sqrt();
  let p = ux;
  let sign_theta = if uy < 0.0 { -1.0 } else { 1.0 };
  let mut theta = sign_theta * (p / n).acos();
  theta = radians_to_degrees(theta);

  // Compute the sweep (extent) angle.
  let vx = (-x1 - cx1) / rx;
  let vy = (-y1 - cy1) / ry;
  let n2 = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
  let p2_val = ux * vx + uy * vy;
  let sign_delta = if (ux * vy - uy * vx) < 0.0 { -1.0 } else { 1.0 };
  let mut delta = sign_delta * (p2_val / n2).acos();
  delta = radians_to_degrees(delta);

  if sweep && delta > 0.0 {
    delta -= 360.0;
  } else if !sweep && delta < 0.0 {
    delta += 360.0;
  }
  theta = theta % 360.0;
  delta = delta % 360.0;

  CenterArc {
    center: Point { x: cx, y: cy },
    rx,
    ry,
    theta,
    extent: delta,
    angle: angle_degrees,
  }
}

#[napi]
impl Arc {
  /// Approximates (linearizes) an elliptical arc into a polyline (a vector of points).
  ///
  /// # Parameters
  /// - `p1`: start point (from the SVG arc definition).
  /// - `p2`: end point.
  /// - `rx`, `ry`: radii.
  /// - `angle`: x‑axis rotation (in degrees).
  /// - `largearc`: large‑arc flag.
  /// - `sweep`: sweep flag.
  /// - `tol`: tolerance for flatness.
  ///
  /// # Returns
  /// A vector of points approximating the arc.
  #[napi]
  pub fn linearize(
    p1: Point,
    p2: Point,
    rx: f64,
    ry: f64,
    angle: f64,
    largearc: bool,
    sweep: bool,
    tol: f64,
  ) -> Vec<Point> {
    // Start with the endpoint.
    let mut finished: Vec<Point> = vec![p2];
    let initial_arc = svg_to_center(p1, p2, rx, ry, angle, largearc, sweep);
    let mut todo: VecDeque<CenterArc> = VecDeque::new();
    todo.push_back(initial_arc);

    // Iteratively subdivide the arc until flat enough.
    while let Some(arc) = todo.pop_front() {
      // Obtain the full arc's SVG representation and one with half the extent.
      let fullarc = center_to_svg(arc.center, arc.rx, arc.ry, arc.theta, arc.extent, arc.angle);
      let subarc = center_to_svg(
        arc.center,
        arc.rx,
        arc.ry,
        arc.theta,
        0.5 * arc.extent,
        arc.angle,
      );
      let arcmid = subarc.p2;

      // Compute the midpoint of the line joining fullarc.p1 and fullarc.p2.
      let mid = Point {
        x: 0.5 * (fullarc.p1.x + fullarc.p2.x),
        y: 0.5 * (fullarc.p1.y + fullarc.p2.y),
      };

      // If the line's midpoint is within `tol` of the arc's midpoint, consider it flat.
      if GeometryUtils::within_distance(mid, arcmid, tol) {
        // Insert the endpoint at the beginning (to preserve order).
        finished.insert(0, fullarc.p2);
      } else {
        // Otherwise, subdivide the arc into two halves.
        let arc1 = CenterArc {
          center: arc.center,
          rx: arc.rx,
          ry: arc.ry,
          theta: arc.theta,
          extent: 0.5 * arc.extent,
          angle: arc.angle,
        };
        let arc2 = CenterArc {
          center: arc.center,
          rx: arc.rx,
          ry: arc.ry,
          theta: arc.theta + 0.5 * arc.extent,
          extent: 0.5 * arc.extent,
          angle: arc.angle,
        };
        // Process the subdivisions.
        todo.push_front(arc2);
        todo.push_front(arc1);
      }
    }
    finished
  }
}
