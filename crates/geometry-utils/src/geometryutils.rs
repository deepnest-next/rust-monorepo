use crate::constants::DEFAULT_TOLERANCE;
use deepnest_types::{Point, Polygon, Rect};
use derive_more::{From, Into};
// this is preventive for {isize,f64}.rem_euclid(rhs)
#[allow(unused_imports)]
use std::primitive::{f64, isize};

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
    a: Point,
    b: Point,
    e: Point,
    f: Point,
    infinite: Option<bool>,
  ) -> Option<Point> {
    let infinite = infinite.unwrap_or(false);
    // Compute coefficients for the line equations:
    // For AB: a1 * x + b1 * y + c1 = 0
    let a1 = b.y - a.y;
    let b1 = a.x - b.x;
    let c1 = b.x * a.y - a.x * b.y;

    // For EF: a2 * x + b2 * y + c2 = 0
    let a2 = f.y - e.y;
    let b2 = e.x - f.x;
    let c2 = f.x * e.y - e.x * f.y;

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
      if !GeometryUtils::in_range(x, a.x, b.x, None) || !GeometryUtils::in_range(y, a.y, b.y, None)
      {
        return None;
      }
      if !GeometryUtils::in_range(x, e.x, f.x, None) || !GeometryUtils::in_range(y, e.y, f.y, None)
      {
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

  /// Determines if `point` lies inside the given `polygon`.
  ///
  /// Returns:
  /// - `Some(true)` if `point` is strictly inside the polygon,
  /// - `Some(false)` if `point` is strictly outside,
  /// - `None` if `point` lies exactly on a vertex or on an edge (within `tolerance`).
  #[napi]
  pub fn point_in_polygon(
    point: Point,
    polygon: Polygon,
    tolerance: Option<f64>,
  ) -> Option<bool> {
    if polygon.points.len() < 3 {
      return None;
    }
    let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);

    // Use the provided offsets (defaulting to 0.0 if None).
    let offsetx = polygon.offsetx.unwrap_or(0.0);
    let offsety = polygon.offsety.unwrap_or(0.0);
    let mut inside = false;
    let n = polygon.points.len();

    // Iterate over each edge of the polygon. The polygon is assumed closed,
    // so the "previous" vertex for index 0 is the last vertex.
    for i in 0..n {
      let j = if i == 0 { n - 1 } else { i - 1 };

      // Adjust the vertices with the offsets.
      let xi = polygon.points[i].x + offsetx;
      let yi = polygon.points[i].y + offsety;
      let xj = polygon.points[j].x + offsetx;
      let yj = polygon.points[j].y + offsety;

      // If the point is approximately equal to a vertex, return None.
      if GeometryUtils::almost_equal(xi, point.x, Some(tol))
        && GeometryUtils::almost_equal(yi, point.y, Some(tol))
      {
        return None;
      }

      // If the point lies on the current segment, return None.
      if GeometryUtils::on_segment(
        Point { x: xi, y: yi },
        Point { x: xj, y: yj },
        point,
        Some(tol),
      ) {
        return None;
      }

      // Skip degenerate segments.
      if GeometryUtils::almost_equal(xi, xj, Some(tol))
        && GeometryUtils::almost_equal(yi, yj, Some(tol))
      {
        continue;
      }

      // Ray-casting: check if a horizontal ray from `point` crosses this edge.
      let cond1 = (yi > point.y) != (yj > point.y);
      if cond1 {
        let intersect_x = ((xj - xi) * (point.y - yi) / (yj - yi)) + xi;
        if point.x < intersect_x {
          inside = !inside;
        }
      }
    }

    Some(inside)
  }

  /// Returns true if the edges of polygon A and polygon B intersect.
  /// The function tests every segment of A (with its optional offset) against every segment of B.
  #[napi]
  pub fn intersect(
    a: Polygon,
    b: Polygon,
  ) -> bool {
    let a_offsetx = a.offsetx.unwrap_or(0.0);
    let a_offsety = a.offsety.unwrap_or(0.0);
    let b_offsetx = b.offsetx.unwrap_or(0.0);
    let b_offsety = b.offsety.unwrap_or(0.0);

    let a_points = &a.points;
    let b_points = &b.points;
    let a_len = a_points.len();
    let b_len = b_points.len();

    // We need at least two points (one segment) in each polygon.
    if a_len < 2 || b_len < 2 {
      return false;
    }

    // Iterate over each segment of polygon A (using consecutive vertices).
    for i in 0..(a_len - 1) {
      // Similarly, iterate over each segment of polygon B.
      for j in 0..(b_len - 1) {
        // Compute segment endpoints with offsets.
        let a1 = Point {
          x: a_points[i].x + a_offsetx,
          y: a_points[i].y + a_offsety,
        };
        let a2 = Point {
          x: a_points[i + 1].x + a_offsetx,
          y: a_points[i + 1].y + a_offsety,
        };
        let b1 = Point {
          x: b_points[j].x + b_offsetx,
          y: b_points[j].y + b_offsety,
        };
        let b2 = Point {
          x: b_points[j + 1].x + b_offsetx,
          y: b_points[j + 1].y + b_offsety,
        };

        // Determine neighboring indices (with wrap-around).
        let mut prevbindex = if j == 0 { b_len - 1 } else { j - 1 };
        let mut prevaindex = if i == 0 { a_len - 1 } else { i - 1 };
        let mut nextbindex = if j + 1 == b_len - 1 { 0 } else { j + 2 };
        let mut nextaindex = if i + 1 == a_len - 1 { 0 } else { i + 2 };

        // Adjust backward indices if the previous vertex equals (or nearly equals) the current one.
        if (b_points[prevbindex].x == b_points[j].x && b_points[prevbindex].y == b_points[j].y)
          || (GeometryUtils::almost_equal(b_points[prevbindex].x, b_points[j].x, None)
            && GeometryUtils::almost_equal(b_points[prevbindex].y, b_points[j].y, None))
        {
          prevbindex = if prevbindex == 0 {
            b_len - 1
          } else {
            prevbindex - 1
          };
        }
        if (a_points[prevaindex].x == a_points[i].x && a_points[prevaindex].y == a_points[i].y)
          || (GeometryUtils::almost_equal(a_points[prevaindex].x, a_points[i].x, None)
            && GeometryUtils::almost_equal(a_points[prevaindex].y, a_points[i].y, None))
        {
          prevaindex = if prevaindex == 0 {
            a_len - 1
          } else {
            prevaindex - 1
          };
        }

        // Adjust forward indices if the next vertex equals (or nearly equals) the following one.
        if (b_points[nextbindex].x == b_points[j + 1].x
          && b_points[nextbindex].y == b_points[j + 1].y)
          || (GeometryUtils::almost_equal(b_points[nextbindex].x, b_points[j + 1].x, None)
            && GeometryUtils::almost_equal(b_points[nextbindex].y, b_points[j + 1].y, None))
        {
          nextbindex = if nextbindex == b_len - 1 {
            0
          } else {
            nextbindex + 1
          };
        }
        if (a_points[nextaindex].x == a_points[i + 1].x
          && a_points[nextaindex].y == a_points[i + 1].y)
          || (GeometryUtils::almost_equal(a_points[nextaindex].x, a_points[i + 1].x, None)
            && GeometryUtils::almost_equal(a_points[nextaindex].y, a_points[i + 1].y, None))
        {
          nextaindex = if nextaindex == a_len - 1 {
            0
          } else {
            nextaindex + 1
          };
        }

        // Compute neighboring points with offsets.
        let a0 = Point {
          x: a_points[prevaindex].x + a_offsetx,
          y: a_points[prevaindex].y + a_offsety,
        };
        let b0 = Point {
          x: b_points[prevbindex].x + b_offsetx,
          y: b_points[prevbindex].y + b_offsety,
        };
        let a3 = Point {
          x: a_points[nextaindex].x + a_offsetx,
          y: a_points[nextaindex].y + a_offsety,
        };
        let b3 = Point {
          x: b_points[nextbindex].x + b_offsetx,
          y: b_points[nextbindex].y + b_offsety,
        };

        // For each candidate edge pair, perform several tests.

        // Test 1: if b1 lies on segment (a1,a2) (or nearly equals a1), then check neighbors.
        if GeometryUtils::on_segment(a1, a2, b1, None)
          || (GeometryUtils::almost_equal(a1.x, b1.x, None)
            && GeometryUtils::almost_equal(a1.y, b1.y, None))
        {
          let b0in = GeometryUtils::point_in_polygon(b0, a.clone(), None);
          let b2in = GeometryUtils::point_in_polygon(b2, a.clone(), None);
          if (b0in == Some(true) && b2in == Some(false))
            || (b0in == Some(false) && b2in == Some(true))
          {
            return true;
          } else {
            continue;
          }
        }

        // Test 2: if b2 lies on segment (a1,a2) (or nearly equals a2), then check neighbors.
        if GeometryUtils::on_segment(a1, a2, b2, None)
          || (GeometryUtils::almost_equal(a2.x, b2.x, None)
            && GeometryUtils::almost_equal(a2.y, b2.y, None))
        {
          let b1in = GeometryUtils::point_in_polygon(b1, a.clone(), None);
          let b3in = GeometryUtils::point_in_polygon(b3, a.clone(), None);
          if (b1in == Some(true) && b3in == Some(false))
            || (b1in == Some(false) && b3in == Some(true))
          {
            return true;
          } else {
            continue;
          }
        }

        // Test 3: if a1 lies on segment (b1,b2) (or nearly equals b2), then check neighbors.
        if GeometryUtils::on_segment(b1, b2, a1, None)
          || (GeometryUtils::almost_equal(a1.x, b2.x, None)
            && GeometryUtils::almost_equal(a1.y, b2.y, None))
        {
          let a0in = GeometryUtils::point_in_polygon(a0, b.clone(), None);
          let a2in = GeometryUtils::point_in_polygon(a2, b.clone(), None);
          if (a0in == Some(true) && a2in == Some(false))
            || (a0in == Some(false) && a2in == Some(true))
          {
            return true;
          } else {
            continue;
          }
        }

        // Test 4: if a2 lies on segment (b1,b2) (or nearly equals b1), then check neighbors.
        if GeometryUtils::on_segment(b1, b2, a2, None)
          || (GeometryUtils::almost_equal(a2.x, b1.x, None)
            && GeometryUtils::almost_equal(a2.y, b1.y, None))
        {
          let a1in = GeometryUtils::point_in_polygon(a1, b.clone(), None);
          let a3in = GeometryUtils::point_in_polygon(a3, b.clone(), None);
          if (a1in == Some(true) && a3in == Some(false))
            || (a1in == Some(false) && a3in == Some(true))
          {
            return true;
          } else {
            continue;
          }
        }

        // Finally, try a simple line–line intersection test.
        if let Some(_p) = GeometryUtils::line_intersect(b1, b2, a1, a2, None) {
          return true;
        }
      }
    }

    false
  }

  // placement algos as outlined in [1] http://www.cs.stir.ac.uk/~goc/papers/EffectiveHueristic2DAOR2013.pdf

  // returns a continuous polyline representing the normal-most edge of the given polygon
  // eg. a normal vector of [-1, 0] will return the left-most edge of the polygon
  // this is essentially algo 8 in [1], generalized for any vector direction
  // TODO: add polygonEdge
  // TODO: add pointLineDistance
  // TODO: add pointDistance
  // TODO: add segmentDistance
  // TODO: add polygonSlideDistance
  // TODO: add polygonProjectionDistance
  // TODO: add searchStartPoint
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon
  // TODO: add noFitPolygon

  /// Returns true if every vertex of the polygon lies (within tolerance) on either
  /// the left/right or the top/bottom bounds of its bounding box.
  /// If the polygon has fewer than 3 points, it returns false.
  #[napi]
  pub fn is_rectangle(
    poly: Polygon,
    tolerance: Option<f64>,
  ) -> bool {
    if poly.points.len() < 3 {
      return false;
    }
    let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);

    let bb = match GeometryUtils::get_polygon_bounds(poly.points.clone()) {
      Some(rect) => rect,
      None => return false,
    };

    // For each vertex, check that its x is nearly equal to bb.x or bb.x + bb.width,
    // and its y is nearly equal to bb.y or bb.y + bb.height.
    for pt in poly.points.iter() {
      if !GeometryUtils::almost_equal(pt.x, bb.x, Some(tol))
        && !GeometryUtils::almost_equal(pt.x, bb.x + bb.width, Some(tol))
      {
        return false;
      }
      if !GeometryUtils::almost_equal(pt.y, bb.y, Some(tol))
        && !GeometryUtils::almost_equal(pt.y, bb.y + bb.height, Some(tol))
      {
        return false;
      }
    }
    true
  }

  /// Returns an interior No-Fit Polygon (NFP) for the special case where A is a rectangle.
  ///
  /// Given two polygons A and B (each as a slice of `Point`), the function computes their
  /// bounding boxes. If B’s bounding box is larger than A’s in either dimension, then B cannot
  /// fit, and the function returns `None`. Otherwise, it returns a single polygon (wrapped in a
  /// vector) whose vertices are computed by aligning the bounding boxes appropriately.
  ///
  /// # Arguments
  ///
  /// * `a` - A slice of `Point` representing polygon A (assumed to be a rectangle).
  /// * `b` - A slice of `Point` representing polygon B.
  ///
  /// # Returns
  ///
  /// * `Some(Vec<Vec<Point>>)` containing one polygon (a vector of 4 points) if an interior NFP exists.
  /// * `None` if B does not fit within A.
  ///
  /// # Example
  ///
  /// ```
  /// # #[derive(Debug, Clone, Copy, PartialEq)]
  /// # pub struct Point { pub x: f64, pub y: f64 }
  /// # fn no_fit_polygon_rectangle(a: &[Point], b: &[Point]) -> Option<Vec<Vec<Point>>> { /* ... */ }
  ///
  /// let rect_a = [
  ///     Point { x: 0.0, y: 0.0 },
  ///     Point { x: 10.0, y: 0.0 },
  ///     Point { x: 10.0, y: 5.0 },
  ///     Point { x: 0.0, y: 5.0 },
  /// ];
  ///
  /// let poly_b = [
  ///     Point { x: 2.0, y: 1.0 },
  ///     Point { x: 4.0, y: 1.0 },
  ///     Point { x: 4.0, y: 3.0 },
  ///     Point { x: 2.0, y: 3.0 },
  /// ];
  ///
  /// if let Some(nfp) = no_fit_polygon_rectangle(&rect_a, &poly_b) {
  ///     println!("Interior NFP: {:?}", nfp);
  /// } else {
  ///     println!("No valid NFP; polygon B is too large.");
  /// }
  /// ```
  #[napi]
  pub fn no_fit_polygon_rectangle(
    a: Vec<Point>,
    b: Vec<Point>,
  ) -> Option<Vec<Vec<Point>>> {
    // Ensure both polygons have at least one point.
    if a.is_empty() || b.is_empty() {
      return None;
    }

    // Compute bounding box for polygon A.
    let mut min_ax = a[0].x;
    let mut min_ay = a[0].y;
    let mut max_ax = a[0].x;
    let mut max_ay = a[0].y;

    for pt in a.iter().skip(1) {
      if pt.x < min_ax {
        min_ax = pt.x;
      }
      if pt.y < min_ay {
        min_ay = pt.y;
      }
      if pt.x > max_ax {
        max_ax = pt.x;
      }
      if pt.y > max_ay {
        max_ay = pt.y;
      }
    }

    // Compute bounding box for polygon B.
    let mut min_bx = b[0].x;
    let mut min_by = b[0].y;
    let mut max_bx = b[0].x;
    let mut max_by = b[0].y;

    for pt in b.iter().skip(1) {
      if pt.x < min_bx {
        min_bx = pt.x;
      }
      if pt.y < min_by {
        min_by = pt.y;
      }
      if pt.x > max_bx {
        max_bx = pt.x;
      }
      if pt.y > max_by {
        max_by = pt.y;
      }
    }

    // If B's bounding box is larger than A's in either dimension, return None.
    if (max_bx - min_bx) > (max_ax - min_ax) {
      return None;
    }
    if (max_by - min_by) > (max_ay - min_ay) {
      return None;
    }

    // Compute the vertices of the interior NFP.
    // The result is shifted by B[0] as an anchor.
    let p1 = Point {
      x: min_ax - min_bx + b[0].x,
      y: min_ay - min_by + b[0].y,
    };
    let p2 = Point {
      x: max_ax - max_bx + b[0].x,
      y: min_ay - min_by + b[0].y,
    };
    let p3 = Point {
      x: max_ax - max_bx + b[0].x,
      y: max_ay - max_by + b[0].y,
    };
    let p4 = Point {
      x: min_ax - min_bx + b[0].x,
      y: max_ay - max_by + b[0].y,
    };

    // Return the resulting NFP as a single polygon inside a vector.
    Some(vec![vec![p1, p2, p3, p4]])
  }

  /// Given two polygons A and B (which touch at at least one point but do not intersect)
  /// with the same winding direction, this function returns the outer perimeter (hull)
  /// of both polygons as a single continuous polygon.
  /// If the polygons are not touching (or inputs are invalid) the function returns None.
  ///
  /// The algorithm:
  /// 1. Find an extreme point (lowest y after applying offsets) among both A and B.
  /// 2. Ensure the starting polygon is assigned to A (swap if necessary).
  /// 3. “Scan forward” from the start point along A until a touching vertex or edge is found on B.
  /// 4. “Scan backward” from the start point along A to find a second touching point on B.
  /// 5. Append the points from B that lie between these two intercepts.
  /// 6. Deduplicate consecutive vertices and return the result.
  #[napi]
  pub fn polygon_hull(
    a: Polygon,
    b: Polygon,
  ) -> Option<Vec<Point>> {
    // Ensure both polygons have at least 3 vertices.
    if a.points.len() < 3 || b.points.len() < 3 {
      return None;
    }

    // Get offsets (defaulting to 0.0 if not provided)
    let mut a_offsetx = a.offsetx.unwrap_or(0.0);
    let mut a_offsety = a.offsety.unwrap_or(0.0);
    let mut b_offsetx = b.offsetx.unwrap_or(0.0);
    let mut b_offsety = b.offsety.unwrap_or(0.0);

    // Work on local clones of the point vectors.
    let mut a_points = a.points.clone();
    let mut b_points = b.points.clone();

    // --- Determine starting extreme point ---
    // We choose the vertex (from either polygon) with the smallest (y + offset).
    let mut min_y = a_points[0].y + a_offsety;
    let mut start_polygon = "A"; // "A" or "B"
    let mut start_index = 0;
    for (i, pt) in a_points.iter().enumerate() {
      if pt.y + a_offsety < min_y {
        min_y = pt.y + a_offsety;
        start_polygon = "A";
        start_index = i;
      }
    }
    for (i, pt) in b_points.iter().enumerate() {
      if pt.y + b_offsety < min_y {
        min_y = pt.y + b_offsety;
        start_polygon = "B";
        start_index = i;
      }
    }
    // For simplicity, we want the starting polygon to be A.
    if start_polygon == "B" {
      // Swap A and B.
      std::mem::swap(&mut a_points, &mut b_points);
      std::mem::swap(&mut a_offsetx, &mut b_offsetx);
      std::mem::swap(&mut a_offsety, &mut b_offsety);
      // Also, set start_index to the one from the B side (now A).
      // (Note: here start_index remains the same because we swapped the arrays.)
    }

    // --- Build the hull (result polygon) in vector C ---
    let mut c: Vec<Point> = Vec::new();
    let mut intercept1: Option<usize> = None;
    let mut intercept2: Option<usize> = None;

    // --- Scan forward from the starting point along A ---
    let a_len = a_points.len();
    let b_len = b_points.len();
    let mut current = start_index;
    for _ in 0..(a_len + 1) {
      // Wrap-around if current equals a_len.
      if current == a_len {
        current = 0;
      }
      let next = if current == a_len - 1 { 0 } else { current + 1 };
      let mut touching = false;
      for j in 0..b_len {
        let nextj = if j == b_len - 1 { 0 } else { j + 1 };
        let a_cur = Point {
          x: a_points[current].x + a_offsetx,
          y: a_points[current].y + a_offsety,
        };
        // Check if A[current] equals B[j].
        let b_j = Point {
          x: b_points[j].x + b_offsetx,
          y: b_points[j].y + b_offsety,
        };
        if GeometryUtils::almost_equal(a_cur.x, b_j.x, None)
          && GeometryUtils::almost_equal(a_cur.y, b_j.y, None)
        {
          c.push(a_cur);
          intercept1 = Some(j);
          touching = true;
          break;
        }
        // Check if B[j] lies on segment A[current] -> A[next].
        let a_next = Point {
          x: a_points[next].x + a_offsetx,
          y: a_points[next].y + a_offsety,
        };
        if GeometryUtils::on_segment(a_cur, a_next, b_j, None) {
          c.push(a_cur);
          c.push(b_j);
          intercept1 = Some(j);
          touching = true;
          break;
        }
        // Check if A[current] lies on segment B[j] -> B[nextj].
        let b_next = Point {
          x: b_points[nextj].x + b_offsetx,
          y: b_points[nextj].y + b_offsety,
        };
        if GeometryUtils::on_segment(b_j, b_next, a_cur, None) {
          c.push(a_cur);
          c.push(b_next);
          intercept1 = Some(nextj);
          touching = true;
          break;
        }
      }
      if touching {
        break;
      }
      c.push(Point {
        x: a_points[current].x + a_offsetx,
        y: a_points[current].y + a_offsety,
      });
      current += 1;
    }

    // --- Scan backward from the starting point along A ---
    // We work with current as isize so we can decrement.
    let mut current = start_index as isize - 1;
    for _ in 0..(a_len + 1) {
      if current < 0 {
        current = (a_len as isize) - 1;
      }
      let next = if current == 0 {
        (a_len as isize) - 1
      } else {
        current - 1
      };
      let mut touching = false;
      for j in 0..b_len {
        let nextj = if j == b_len - 1 { 0 } else { j + 1 };
        let a_cur = Point {
          x: a_points[current as usize].x + a_offsetx,
          y: a_points[current as usize].y + a_offsety,
        };
        let b_j = Point {
          x: b_points[j].x + b_offsetx,
          y: b_points[j].y + b_offsety,
        };
        if GeometryUtils::almost_equal(a_cur.x, b_j.x, None)
          && GeometryUtils::almost_equal(a_cur.y, b_j.y, None)
        {
          c.insert(0, a_cur);
          intercept2 = Some(j);
          touching = true;
          break;
        }
        let a_next = Point {
          x: a_points[next as usize].x + a_offsetx,
          y: a_points[next as usize].y + a_offsety,
        };
        if GeometryUtils::on_segment(a_cur, a_next, b_j, None) {
          c.insert(0, a_cur);
          c.insert(0, b_j);
          intercept2 = Some(j);
          touching = true;
          break;
        }
        let b_next = Point {
          x: b_points[nextj].x + b_offsetx,
          y: b_points[nextj].y + b_offsety,
        };
        if GeometryUtils::on_segment(b_j, b_next, a_cur, None) {
          c.insert(0, a_cur);
          intercept2 = Some(j);
          touching = true;
          break;
        }
      }
      if touching {
        break;
      }
      c.insert(
        0,
        Point {
          x: a_points[current as usize].x + a_offsetx,
          y: a_points[current as usize].y + a_offsety,
        },
      );
      current -= 1;
    }

    // If we never found an intersection on one side, return None.
    if intercept1.is_none() || intercept2.is_none() {
      return None;
    }

    // --- Append points from B lying between intercept1 and intercept2 ---
    let mut current = intercept1.unwrap() + 1;
    for _ in 0..b_len {
      if current == b_len {
        current = 0;
      }
      c.push(Point {
        x: b_points[current].x + b_offsetx,
        y: b_points[current].y + b_offsety,
      });
      if current == intercept2.unwrap() {
        break;
      }
      current += 1;
    }

    // --- Deduplicate consecutive vertices (wrap-around considered) ---
    let mut i = 0;
    while i < c.len() {
      let next = if i == c.len() - 1 { 0 } else { i + 1 };
      if GeometryUtils::almost_equal(c[i].x, c[next].x, None)
        && GeometryUtils::almost_equal(c[i].y, c[next].y, None)
      {
        c.remove(i);
        // After removal, continue checking at the same index.
        if c.is_empty() {
          break;
        }
      } else {
        i += 1;
      }
    }

    Some(c)
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
    a: Point,
    b: Point,
    p: Point,
    tolerance: Option<f64>,
  ) -> bool {
    let tol = tolerance.unwrap_or(DEFAULT_TOLERANCE);
    // Exclude endpoints.
    if (GeometryUtils::almost_equal(p.x, a.x, Some(tol))
      && GeometryUtils::almost_equal(p.y, a.y, Some(tol)))
      || (GeometryUtils::almost_equal(p.x, b.x, Some(tol))
        && GeometryUtils::almost_equal(p.y, b.y, Some(tol)))
    {
      return false;
    }

    // Check that p lies within the bounding box of A and B.
    let (min_x, max_x) = if a.x < b.x { (a.x, b.x) } else { (b.x, a.x) };
    let (min_y, max_y) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
    if p.x < min_x - tol || p.x > max_x + tol || p.y < min_y - tol || p.y > max_y + tol {
      return false;
    }

    // Check collinearity using the cross product.
    let cross = (p.y - a.y) * (b.x - a.x) - (p.x - a.x) * (b.y - a.y);
    if cross.abs() > tol {
      return false;
    }

    // Check that p is strictly between A and B via the dot product.
    let dot = (p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y);
    if dot <= tol {
      return false;
    }
    let len2 = (b.x - a.x).powi(2) + (b.y - a.y).powi(2);
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
}
