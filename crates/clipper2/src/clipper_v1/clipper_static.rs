use std::f64::consts::PI;
use super::types::*;
use super::error::*;
use super::clipper::Clipper;

/// Static methods ported from C# Clipper class
pub struct ClipperStatic;

impl ClipperStatic {
    /// Reverses the order of points in a Paths collection
    pub fn reverse_paths(polys: &mut Paths) {
        for poly in polys.iter_mut() {
            poly.reverse();
        }
    }

    /// Calculates the area of a polygon
    pub fn area(poly: &Path) -> f64 {
        let cnt = poly.len();
        if cnt < 3 {
            return 0.0;
        }

        let mut a = 0.0;
        for i in 0..cnt {
            let j = if i == 0 { cnt - 1 } else { i - 1 };
            a += (poly[j].x + poly[i].x) as f64 * (poly[j].y - poly[i].y) as f64;
        }
        -a * 0.5
    }

    /// Determines if a polygon has clockwise orientation
    pub fn orientation(poly: &Path) -> bool {
        Self::area(poly) >= 0.0
    }

    /// Simplifies a polygon by removing consecutive points that are too close
    pub fn clean_polygon(path: &Path, distance: f64) -> Path {
        let cnt = path.len();
        if cnt == 0 {
            return Path::new();
        }

        let mut out_pts = Vec::with_capacity(cnt);
        let dist_sqrd = distance * distance;

        // Remove duplicate points and points that are too close
        let mut i = 0;
        while i < cnt {
            let mut keep = true;
            let j = if i == 0 { cnt - 1 } else { i - 1 };
            let k = if i == cnt - 1 { 0 } else { i + 1 };

            // Check distance to previous and next points
            if Self::point_is_too_close(&path[i], &path[j], dist_sqrd) ||
               Self::point_is_too_close(&path[i], &path[k], dist_sqrd) {
                keep = false;
            }
            // Check if point is collinear with previous and next points
            else if Self::points_are_collinear(&path[j], &path[i], &path[k], dist_sqrd) {
                keep = false;
            }

            if keep {
                out_pts.push(path[i]);
            }
            i += 1;
        }

        // Return result, ensuring at least 3 points for closed paths
        if out_pts.len() < 3 {
            Path::new()
        } else {
            out_pts
        }
    }

    /// Simplifies multiple polygons
    pub fn clean_polygons(polys: &Paths, distance: f64) -> Paths {
        let mut result = Paths::with_capacity(polys.len());
        for poly in polys {
            let cleaned = Self::clean_polygon(poly, distance);
            if !cleaned.is_empty() {
                result.push(cleaned);
            }
        }
        result
    }

    /// Calculates the squared distance between two points
    fn point_is_too_close(pt1: &IntPoint, pt2: &IntPoint, dist_sqrd: f64) -> bool {
        let dx = (pt1.x - pt2.x) as f64;
        let dy = (pt1.y - pt2.y) as f64;
        dx * dx + dy * dy < dist_sqrd
    }

    /// Determines if three points are collinear within a tolerance
    fn points_are_collinear(pt1: &IntPoint, pt2: &IntPoint, pt3: &IntPoint, dist_sqrd: f64) -> bool {
        // Calculate the area of the triangle formed by the three points
        // If area is near zero (within tolerance), points are collinear
        let area = ((pt2.x - pt1.x) as f64 * (pt3.y - pt1.y) as f64 -
                   (pt2.y - pt1.y) as f64 * (pt3.x - pt1.x) as f64).abs();
        area * area <= dist_sqrd * 16.0
    }

    /// Performs a Minkowski sum operation on two polygons
    pub fn minkowski_sum(pattern: &Path, path: &Path, is_sum: bool, is_closed: bool) -> Paths {
        let pattern_len = pattern.len();
        let path_len = path.len();
        let delta = if is_closed { 1 } else { 0 };
        
        let mut result = Paths::with_capacity(path_len);
        
        for i in 0..path_len - delta {
            let mut p = Path::with_capacity(pattern_len);
            if is_sum {
                for ip in pattern {
                    p.push(IntPoint::new(
                        path[i].x + ip.x,
                        path[i].y + ip.y
                    ));
                }
            } else {
                for ip in pattern {
                    p.push(IntPoint::new(
                        path[i].x - ip.x,
                        path[i].y - ip.y
                    ));
                }
            }
            result.push(p);
        }

        let mut quads = Paths::with_capacity((path_len - delta) * (pattern_len - 1));
        for i in 0..path_len - delta {
            for j in 0..pattern_len {
                let k = if j == pattern_len - 1 { 0 } else { j + 1 };
                let mut quad = Path::with_capacity(4);
                quad.push(result[i][j]);
                quad.push(result[i][k]);
                quad.push(result[(i + 1) % path_len][k]);
                quad.push(result[(i + 1) % path_len][j]);
                quads.push(quad);
            }
        }

        // Union all quads to get final result
        let mut clipper = Clipper::new(0);
        clipper.add_paths(&quads, PolyType::Subject, true)?;
        clipper.execute(ClipType::Union, &mut result, PolyFillType::NonZero, PolyFillType::NonZero)?;
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area() {
        let poly = vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];
        assert_eq!(ClipperStatic::area(&poly), 100.0);
    }

    #[test]
    fn test_orientation() {
        let poly = vec![
            IntPoint::new(0, 0),
            IntPoint::new(0, 10),
            IntPoint::new(10, 10),
            IntPoint::new(10, 0),
        ];
        assert!(!ClipperStatic::orientation(&poly)); // Counter-clockwise
    }
}
