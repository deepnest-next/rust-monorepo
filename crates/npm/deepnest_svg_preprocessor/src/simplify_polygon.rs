use simplifyRS::{simplify, Point as SimplifyPoint};
use napi_derive::napi;
use crate::points_on_curve::Point;

type SimplifyPoint2D = SimplifyPoint<2, f64>;

impl From<Point> for SimplifyPoint2D {
    fn from(p: Point) -> Self {
        SimplifyPoint { vec: [p.x, p.y] }
    }
}

impl From<SimplifyPoint2D> for Point {
    fn from(p: SimplifyPoint2D) -> Self {
        Point { x: p.vec[0], y: p.vec[1] }
    }
}

#[napi]
pub fn simplify_polygon(points: Vec<Point>, tolerance: f64, high_quality: bool) -> Vec<Point> {
    // Convert from our Point type to the library's SimplifyPoint type
    let simplify_points: Vec<SimplifyPoint2D> = points.into_iter().map(Into::into).collect();
    
    // Run the simplification algorithm
    let simplified = simplify(&simplify_points, tolerance, high_quality);
    
    // Convert back to our Point type
    simplified.into_iter().map(Into::into).collect()
}
