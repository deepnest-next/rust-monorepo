use std::f64::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use super::{types::*, ClipperBaseStatic as CBS};
use super::clipper::Clipper;
use super::error::*;

const TWO_PI: f64 = PI * 2.0;
const DEFAULT_ARC_TOLERANCE: f64 = 0.25;
const TOLERANCE: f64 = 1.0e-20;

/// Offset polygon paths by a specified amount
pub struct ClipperOffset {
    dest_polys: Paths,
    src_poly: Path,
    dest_poly: Path,
    normals: Vec<DoublePoint>,
    delta: f64,
    sin_a: f64,
    sin: f64,
    cos: f64,
    miter_lim: f64,
    steps_per_rad: f64,
    lowest: IntPoint,
    arc_tolerance: f64,
    miter_limit: f64,
}

impl ClipperOffset {
    /// Creates a new ClipperOffset instance
    pub fn new(miter_limit: Option<f64>, arc_tolerance: Option<f64>) -> Self {
        let miter_limit = miter_limit.unwrap_or(2.0);
        let arc_tolerance = arc_tolerance.unwrap_or(DEFAULT_ARC_TOLERANCE);
        Self {
            dest_polys: Paths::new(),
            src_poly: Path::new(),
            dest_poly: Path::new(),
            normals: Vec::new(),
            delta: 0.0,
            sin_a: 0.0,
            sin: 0.0,
            cos: 0.0,
            miter_lim: 2.0,
            steps_per_rad: DEFAULT_ARC_TOLERANCE,
            lowest: IntPoint::new(0, 0),
            arc_tolerance: arc_tolerance.max(DEFAULT_ARC_TOLERANCE),
            miter_limit: miter_limit.max(2.0),
        }
    }

    /// Adds a path to be offset
    pub fn add_path(&mut self, path: &Path, join_type: JoinType, end_type: EndType) {
        let mut high_i = path.len() - 1;
        if high_i < 0 {
            return;
        }

        let mut new_node = Path::new();
        
        // If path is closed polygon
        if end_type == EndType::ClosedPolygon || end_type == EndType::ClosedLine {
            // Skip duplicate points at end
            let mut j = high_i;
            while j > 0 && path[j] == path[0] {
                j -= 1;
            }
            if j < 2 {
                return;
            }
            high_i = j;
        }

        // Store vertices
        new_node.reserve(high_i + 1);
        for pt in path.iter().take(high_i + 1) {
            new_node.push(*pt);
        }

        // Handle different end types
        match end_type {
            EndType::ClosedPolygon => {
                if new_node.len() < 3 {
                    return;
                }
                new_node.push(new_node[0]);
            }
            EndType::ClosedLine => {
                if new_node.len() < 2 {
                    return;
                }
                new_node.push(new_node[0]);
            }
            _ => {} // Open paths handled as-is
        }

        // Track lowest point for orientation fixing
        if self.lowest.x < 0 {
            self.lowest = IntPoint::new(0, 0);
            for j in 0..new_node.len() {
                if new_node[j].y > self.lowest.y || 
                   (new_node[j].y == self.lowest.y && new_node[j].x < self.lowest.x) {
                    self.lowest = new_node[j];
                }
            }
        }

        self.dest_polys.push(new_node);
    }

    /// Adds multiple paths to be offset
    pub fn add_paths(&mut self, paths: &Paths, join_type: JoinType, end_type: EndType) {
        for path in paths {
            self.add_path(path, join_type, end_type);
        }
    }

    /// Executes the offset operation
    pub fn execute(&mut self, solution: &mut Paths, delta: f64) -> Result<()> {
        solution.clear();
        self.fix_orientations();
        self.do_offset(delta);

        // Create clipping solution
        let clipper = Clipper::new(0);
        clipper.execute(
            ClipType::Union,
            solution,
            PolyFillType::Positive,
            PolyFillType::Positive,
        )?;
        
        Ok(())
    }

    /// Main offset algorithm implementation
    fn do_offset(&mut self, delta: f64) {
        self.dest_polys.clear();
        self.delta = delta;

        // Special handling for zero offset
        if CBS::near_zero(delta) {
            self.dest_polys = self.src_poly.clone();
            return;
        }

        // Calculate offset parameters
        if self.miter_limit > 2.0 {
            self.miter_lim = 2.0 / (self.miter_limit * self.miter_limit);
        } else {
            self.miter_lim = 0.5;
        }

        // Calculate arc steps
        let steps = PI / (self.arc_tolerance.max(0.25).acos());
        self.steps_per_rad = 1.0 / steps;
        
        if delta < 0.0 {
            self.delta = -delta;
        }

        // Process each polygon
        for path in &self.src_poly {
            let cnt = path.len();
            if cnt == 0 || (cnt < 3 && delta <= 0.0) {
                continue;
            }

            // Generate normals
            self.normals.clear();
            self.normals.reserve(cnt);
            for j in 0..cnt - 1 {
                self.normals.push(self.get_unit_normal(
                    &path[j], 
                    &path[j + 1]
                ));
            }
            if path[cnt-1] == path[0] {
                self.normals.push(self.get_unit_normal(
                    &path[cnt-1],
                    &path[0]
                ));
            } else {
                self.normals.push(self.get_unit_normal(
                    &path[cnt-1],
                    &path[cnt-2]
                ));
            }

            self.dest_poly.clear();
            
            // Handle different join types
            for j in 0..cnt {
                self.offset_point(j);
            }

            // Create the offset polygon
            self.dest_polys.push(self.dest_poly.clone());
        }
    }

    /// Gets the unit normal vector between two points
    fn get_unit_normal(&self, pt1: &IntPoint, pt2: &IntPoint) -> DoublePoint {
        let dx = (pt2.x - pt1.x) as f64;
        let dy = (pt2.y - pt1.y) as f64;
        if dx == 0.0 && dy == 0.0 {
            return DoublePoint::new(0.0, 0.0);
        }

        let f = 1.0 / (dx * dx + dy * dy).sqrt();
        DoublePoint::new(dy * f, -dx * f)
    }

    /// Offsets a point using the specified join type
    fn offset_point(&mut self, j: usize) {
        self.sin_a = (self.normals[j].x * self.normals[j + 1].y - 
                      self.normals[j].y * self.normals[j + 1].x);
        
        if self.sin_a.abs() < 1.0 {
            let cos_a = (self.normals[j].x * self.normals[j + 1].x +
                        self.normals[j].y * self.normals[j + 1].y);
            if cos_a > 0.0 {
                // Angle less than 90 degrees
                self.add_point(
                    self.src_poly[j].x + (self.normals[j].x + self.normals[j + 1].x) * self.delta,
                    self.src_poly[j].y + (self.normals[j].y + self.normals[j + 1].y) * self.delta
                );
                return;
            }
        } else if self.sin_a > 1.0 {
            self.sin_a = 1.0;
        } else if self.sin_a < -1.0 {
            self.sin_a = -1.0;
        }

        if self.sin_a * self.delta < 0.0 {
            self.add_point(
                self.src_poly[j].x + self.normals[j].x * self.delta,
                self.src_poly[j].y + self.normals[j].y * self.delta
            );
            self.add_point(self.src_poly[j]);
            self.add_point(
                self.src_poly[j].x + self.normals[j + 1].x * self.delta,
                self.src_poly[j].y + self.normals[j + 1].y * self.delta
            );
        } else {
            // Add miter or round join
            match self.join_type {
                JoinType::Miter => {
                    let r = 1.0 + (self.normals[j].x * self.normals[j + 1].x +
                                 self.normals[j].y * self.normals[j + 1].y);
                    if r >= self.miter_lim {
                        self.do_miter(j, r);
                    } else {
                        self.do_square(j);
                    }
                },
                JoinType::Round => self.do_round(j),
                JoinType::Square => self.do_square(j),
            }
        }
    }

    /// Adds a point to the destination polygon
    fn add_point(&mut self, x: CInt, y: CInt) {
        self.dest_poly.push(IntPoint::new(x, y));
    }

    /// Helper method to add a point from IntPoint
    fn add_point_from_int(&mut self, pt: IntPoint) {
        self.dest_poly.push(pt);
    }

    // Additional helper methods for different join types...
    fn do_square(&mut self, j: usize) {
        // Implementation for square joins
    }

    fn do_miter(&mut self, j: usize, r: f64) {
        // Implementation for miter joins
    }

    fn do_round(&mut self, j: usize) {
        // Implementation for round joins
    }

    fn fix_orientations(&mut self) {
        // Fix polygon orientations for proper offsetting
        if self.lowest.x >= 0 && !Clipper::orientation(&self.dest_polys[self.lowest.x as usize]) {
            for path in &mut self.dest_polys {
                path.reverse();
            }
        } else {
            // Handle open paths
            for path in &mut self.dest_polys {
                if !Clipper::orientation(path) {
                    path.reverse();
                }
            }
        }
    }
}

impl Default for ClipperOffset {
    fn default() -> Self {
        Self::new(2.0, DEFAULT_ARC_TOLERANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_offset() {
        let mut clipper = ClipperOffset::new(2.0, 0.25);
        
        // Create a simple square
        let square = vec![
            IntPoint::new(0, 0),
            IntPoint::new(100, 0),
            IntPoint::new(100, 100),
            IntPoint::new(0, 100),
            IntPoint::new(0, 0),
        ];

        clipper.add_path(&square, JoinType::Square, EndType::ClosedPolygon);
        
        let mut solution = Paths::new();
        clipper.execute(&mut solution, 10.0).unwrap();
        
        assert!(!solution.is_empty());
        assert!(solution[0].len() > 4); // Should have more points due to offset
    }
}
