use super::error::*;
use super::tedge::*; // Import TEdge from the parent module
use super::types::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Static methods ported from C# ClipperBase class
pub struct ClipperBaseStatic;

impl ClipperBaseStatic {

    /// Checks if a value is near zero
    #[inline]
    pub fn near_zero(val: f64) -> bool {
        val > -TOLERANCE && val < TOLERANCE
    }

    /// Swaps two CInt values
    #[inline]
    pub fn swap(val1: &mut CInt, val2: &mut CInt) {
        std::mem::swap(val1, val2);
    }

    /// Checks if points are collinear by comparing slopes
    pub fn slopes_equal(
        pt1: &IntPoint,
        pt2: &IntPoint,
        pt3: &IntPoint,
        use_full_range: bool,
    ) -> bool {
        if use_full_range {
            // Use Int128 for high precision comparison
            (pt1.y - pt2.y) * (pt2.x - pt3.x) == (pt1.x - pt2.x) * (pt2.y - pt3.y)
        } else {
            // Use standard integer math for lower precision
            (pt1.y - pt2.y) as i64 * (pt2.x - pt3.x) as i64
                == (pt1.x - pt2.x) as i64 * (pt2.y - pt3.y) as i64
        }
    }

    /// Checks if slopes of edges are equal
    pub fn slopes_equal_edge(e1: &TEdge, e2: &TEdge, use_full_range: bool) -> bool {
        if use_full_range {
            // Use Int128 for high precision comparison
            e1.delta.y * e2.delta.x == e1.delta.x * e2.delta.y
        } else {
            // Use standard integer math for lower precision
            (e1.delta.y as i64) * (e2.delta.x as i64) == (e1.delta.x as i64) * (e2.delta.y as i64)
        }
    }

    /// Checks if pt2 is between pt1 and pt3
    pub fn pt2_is_between_pt1_and_pt3(pt1: &IntPoint, pt2: &IntPoint, pt3: &IntPoint) -> bool {
        if *pt1 == *pt3 || *pt1 == *pt2 || *pt3 == *pt2 {
            return false;
        }

        if pt1.x != pt3.x {
            (pt2.x > pt1.x) == (pt2.x < pt3.x)
        } else {
            (pt2.y > pt1.y) == (pt2.y < pt3.y)
        }
    }

    /// Gets bounds of a paths collection
    pub fn get_bounds(paths: &Paths) -> IntRect {
        let mut i = 0;
        let cnt = paths.len();
        while i < cnt && paths[i].is_empty() {
            i += 1;
        }
        if i == cnt {
            return IntRect::new(0, 0, 0, 0);
        }

        let mut result = IntRect::new(paths[i][0].x, paths[i][0].y, paths[i][0].x, paths[i][0].y);

        for path in paths.iter().skip(i) {
            for pt in path {
                if pt.x < result.left {
                    result.left = pt.x;
                }
                if pt.x > result.right {
                    result.right = pt.x;
                }
                if pt.y < result.top {
                    result.top = pt.y;
                }
                if pt.y > result.bottom {
                    result.bottom = pt.y;
                }
            }
        }
        result
    }

    /// Checks if a polygon/edge is horizontal
    #[inline]
    pub fn is_horizontal(e: &TEdge) -> bool {
        e.delta.y == 0
    }

    /// Gets X coordinate at a given Y coordinate for an edge
    pub fn get_x_at_y(edge: &TEdge, current_y: CInt) -> CInt {
        if edge.top.y == edge.bot.y {
            return edge.bot.x;
        }
        if current_y == edge.top.y {
            return edge.top.x;
        }
        if current_y == edge.bot.y {
            return edge.bot.x;
        }

        // Calculate X using the line equation
        edge.bot.x + ((current_y - edge.bot.y) as f64 * edge.dx) as CInt
    }

    /// Calculates the intersection point of two edges
    pub fn get_intersection(
        edge1: &TEdge,
        edge2: &TEdge,
        use_full_range: bool,
    ) -> Option<IntPoint> {
        if edge1.delta.y == 0 || edge2.delta.y == 0 {
            return None; // One or both edges are horizontal
        }

        let dx = (edge2.delta.x as f64) / (edge2.delta.y as f64);
        let dy = (edge1.delta.x as f64) / (edge1.delta.y as f64);

        if dx == dy {
            return None; // Parallel edges
        }

        // Calculate intersection point
        let y = ((edge1.bot.x - edge2.bot.x) as f64 + dx * edge2.bot.y as f64
            - dy * edge1.bot.y as f64)
            / (dx - dy);
        let x = edge1.bot.x as f64 + dy * (y - edge1.bot.y as f64);

        // Check bounds
        if use_full_range {
            if x < -HI_RANGE as f64
                || x > HI_RANGE as f64
                || y < -HI_RANGE as f64
                || y > HI_RANGE as f64
            {
                return None;
            }
        }

        Some(IntPoint::new(x.round() as CInt, y.round() as CInt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slopes_equal() {
        let pt1 = IntPoint::new(0, 0);
        let pt2 = IntPoint::new(10, 10);
        let pt3 = IntPoint::new(20, 20);
        assert!(ClipperBaseStatic::slopes_equal(&pt1, &pt2, &pt3, false));
    }

    #[test]
    fn test_pt2_is_between_pt1_and_pt3() {
        let pt1 = IntPoint::new(0, 0);
        let pt2 = IntPoint::new(5, 5);
        let pt3 = IntPoint::new(10, 10);
        assert!(ClipperBaseStatic::pt2_is_between_pt1_and_pt3(&pt1, &pt2, &pt3));
    }
}
