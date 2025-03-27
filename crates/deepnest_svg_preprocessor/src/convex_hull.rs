use crate::points_on_curve::Point;
use napi::bindgen_prelude::*;
use parry2d_f64::math::Point as ParryPoint;
use parry2d_f64::transformation::convex_hull;

#[napi(object)]
pub struct ConvexHullResult {
  pub points: Vec<Point>,
}

#[napi]
pub fn compute_convex_hull(points: Vec<Point>) -> napi::Result<ConvexHullResult> {
  // Convert our Point type to parry2d's Point type
  let parry_points: Vec<ParryPoint<f64>> =
    points.iter().map(|p| ParryPoint::new(p.x, p.y)).collect();

  // Compute convex hull
  let hull_indices = convex_hull(&parry_points);

  // Extract the points from the original list using the indices
  let hull_points: Vec<Point> = hull_indices
    .iter()
    .map(|&i| Point { x: i.x, y: i.y })
    .collect();

  Ok(ConvexHullResult {
    points: hull_points,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_points_equal(a: &[Point], b: &[Point]) {
    assert_eq!(a.len(), b.len(), "Point arrays have different lengths");

    for (i, (p1, p2)) in a.iter().zip(b.iter()).enumerate() {
      assert!(
        (p1.x - p2.x).abs() < 1e-9 && (p1.y - p2.y).abs() < 1e-9,
        "Points at index {} differ: ({}, {}) vs ({}, {})",
        i,
        p1.x,
        p1.y,
        p2.x,
        p2.y
      );
    }
  }

  #[test]
  fn test_square() {
    let points = vec![
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: 1.0 },
      Point { x: 1.0, y: 1.0 },
      Point { x: 1.0, y: 0.0 },
    ];

    let result = compute_convex_hull(points.clone()).unwrap();

    // The convex hull of a square should include all four corner points
    assert_eq!(result.points.len(), 4);

    // Check that all original points are present in the hull
    for p in &points {
      assert!(result
        .points
        .iter()
        .any(|hp| (hp.x - p.x).abs() < 1e-9 && (hp.y - p.y).abs() < 1e-9));
    }
  }

  #[test]
  fn test_concave_shape() {
    // Create a concave shape (like a 'C')
    let points = vec![
      Point { x: 0.0, y: 0.0 }, // Bottom-left
      Point { x: 0.0, y: 2.0 }, // Top-left
      Point { x: 1.0, y: 2.0 }, // Top-middle
      Point { x: 1.0, y: 1.5 }, // Inner top
      Point { x: 0.5, y: 1.0 }, // Inner middle
      Point { x: 1.0, y: 0.5 }, // Inner bottom
      Point { x: 1.0, y: 0.0 }, // Bottom-right
    ];

    let result = compute_convex_hull(points).unwrap();

    // The convex hull should have 4 points for this shape
    assert_eq!(result.points.len(), 4);

    // Expected hull (only the corners)
    let expected_hull = vec![
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: 2.0 },
      Point { x: 1.0, y: 2.0 },
      Point { x: 1.0, y: 0.0 },
    ];

    // Check each point in the expected hull exists in the result
    for p in &expected_hull {
      assert!(result
        .points
        .iter()
        .any(|hp| (hp.x - p.x).abs() < 1e-9 && (hp.y - p.y).abs() < 1e-9));
    }
  }

  #[test]
  fn test_collinear_points() {
    // Points on a straight line
    let points = vec![
      Point { x: 0.0, y: 0.0 },
      Point { x: 1.0, y: 0.0 },
      Point { x: 2.0, y: 0.0 },
      Point { x: 3.0, y: 0.0 },
      Point { x: 4.0, y: 0.0 },
    ];

    let result = compute_convex_hull(points).unwrap();

    // For collinear points, the hull should only contain the endpoints
    assert_eq!(result.points.len(), 2);

    let expected_hull = vec![Point { x: 4.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }];
    println!("{:?}", result.points);
    assert_points_equal(&result.points, &expected_hull);
  }

  #[test]
  fn test_duplicate_points() {
    // Points with duplicates
    let points = vec![
      Point { x: 0.0, y: 0.0 },
      Point { x: 0.0, y: 0.0 }, // Duplicate
      Point { x: 1.0, y: 0.0 },
      Point { x: 0.0, y: 1.0 },
      Point { x: 1.0, y: 1.0 },
    ];

    let result = compute_convex_hull(points).unwrap();

    // The hull should have 4 points (the duplicates should be handled)
    assert_eq!(result.points.len(), 4);
  }
}
