use std::cell::RefCell;
use std::rc::Rc;
use super::types::*;
use super::tedge::*;
use super::error::*;
use super::clipper_base_static::ClipperBaseStatic as CBS;

/// A port of the C# ClipperBase class.
#[derive(Debug, Clone)]
pub struct ClipperBase {
    /// PreserveCollinear property (defaults to false)
    pub preserve_collinear: bool,
    /// Minima list for local minima storage
    pub minima_list: Option<Box<LocalMinima>>,
    /// Current local minima for processing
    pub current_lm: Option<Box<LocalMinima>>,
    /// Storage for all edges
    pub edges: Vec<Vec<TEdge>>,
    /// Scanbeam for horizontal edge processing
    pub scanbeam: Option<Box<Scanbeam>>,
    /// Storage for output polygons
    pub poly_outs: Vec<OutRec>,
    /// Currently active edges
    pub active_edges: Option<Rc<RefCell<TEdge>>>,
    /// Whether to use full range values
    pub use_full_range: bool,
    /// Whether paths contain open paths
    pub has_open_paths: bool,
}

impl ClipperBase {
    /// Creates a new ClipperBase instance
    pub fn new() -> Self {
        Self {
            preserve_collinear: false,
            minima_list: None,
            current_lm: None,
            edges: Vec::new(),
            scanbeam: None,
            poly_outs: Vec::new(),
            active_edges: None,
            use_full_range: false,
            has_open_paths: false,
        }
    }

    /// Tests if a point fits within coordinate range
    pub fn range_test(&self, pt: IntPoint, use_full_range: &mut bool) -> Result<()> {
        if *use_full_range {
            if pt.x > HI_RANGE || pt.y > HI_RANGE || -pt.x > HI_RANGE || -pt.y > HI_RANGE {
                return Err(ClipperError::CoordinateOutOfRange);
            }
        } else if pt.x > LO_RANGE || pt.y > LO_RANGE || -pt.x > LO_RANGE || -pt.y > LO_RANGE {
            *use_full_range = true;
            self.range_test(pt, use_full_range)?;
        }
        Ok(())
    }

    /// Initializes edge fields
    pub fn init_edge(&mut self, e: &mut TEdge, e_next: &TEdge, e_prev: &TEdge, pt: IntPoint) {
        e.next = Some(Rc::new(RefCell::new((*e_next).clone())));
        e.prev = Some(Rc::new(RefCell::new((*e_prev).clone())));
        e.curr = pt;
        e.out_idx = UNASSIGNED;
    }

    /// Initializes edge fields for the second stage
    pub fn init_edge2(&mut self, e: &mut TEdge, poly_type: PolyType) {
        if e.curr.y >= e.next.as_ref().unwrap().borrow().curr.y {
            e.bot = e.curr;
            e.top = e.next.as_ref().unwrap().borrow().curr;
        } else {
            e.top = e.curr;
            e.bot = e.next.as_ref().unwrap().borrow().curr;
        }
        self.set_dx(e);
        e.poly_typ = poly_type;
    }

    /// Sets the delta and dx values for an edge
    pub fn set_dx(&self, e: &mut TEdge) {
        e.delta.x = e.top.x - e.bot.x;
        e.delta.y = e.top.y - e.bot.y;
        if e.delta.y == 0 {
            e.dx = HORIZONTAL;
        } else {
            e.dx = (e.delta.x as f64) / (e.delta.y as f64);
        }
    }

    /// Adds a path to the clipper
    pub fn add_path(&mut self, path: &Path, poly_type: PolyType, closed: bool) -> Result<bool> {
        // Check for open paths in clip type
        if !closed && poly_type == PolyType::Clip {
            return Err(ClipperError::OpenPathsNotSupported);
        }

        // Find last valid vertex
        let mut high_i = path.len() as i32 - 1;
        if closed {
            while high_i > 0 && path[high_i as usize] == path[(high_i - 1) as usize] {
                high_i -= 1;
            }
        }
        while high_i > 0 && path[high_i as usize] == path[(high_i - 1) as usize] {
            high_i -= 1;
        }
        if (closed && high_i < 2) || (!closed && high_i < 1) {
            return Ok(false);
        }

        // Create edge array
        let mut edges = Vec::with_capacity((high_i + 1) as usize);
        for _ in 0..=high_i {
            edges.push(TEdge::new());
        }

        let mut is_flat = true;

        // 1. Basic (first) edge initialization
        edges[1].curr = path[1];
        self.range_test(path[0], &mut self.use_full_range)?;
        self.range_test(path[high_i as usize], &mut self.use_full_range)?;
        self.init_edge(&mut edges[0], &edges[1], &edges[high_i as usize], path[0]);
        self.init_edge(&mut edges[high_i as usize], &edges[0], &edges[(high_i - 1) as usize], path[high_i as usize]);
        
        for i in (1..high_i).rev() {
            self.range_test(path[i as usize], &mut self.use_full_range)?;
            self.init_edge(&mut edges[i as usize], &edges[i + 1], &edges[i - 1], path[i as usize]);
        }

        let mut e_start = edges[0].clone();

        // 2. Remove duplicate vertices and collinear edges
        let mut e = e_start.clone();
        let mut e_loop_stop = e_start.clone();
        loop {
            if e.curr == e.next.as_ref().unwrap().borrow().curr && (closed || !e.next.as_ref().unwrap().borrow().eq(&e_start)) {
                if e == e_start {
                    e_start = e.next.as_ref().unwrap().borrow().clone();
                }
                e = self.remove_edge(&e);
                e_loop_stop = e.clone();
                continue;
            }
            if e.prev.as_ref().unwrap().borrow().eq(&e.next.as_ref().unwrap().borrow()) {
                break;
            }
            if closed && 
            CBS::slopes_equal(
                    &e.prev.as_ref().unwrap().borrow().curr,
                    &e.curr,
                    &e.next.as_ref().unwrap().borrow().curr,
                    self.use_full_range
                ) &&
                (!self.preserve_collinear || 
                 !CBS::pt2_between_pt1_and_pt3(
                     &e.prev.as_ref().unwrap().borrow().curr,
                     &e.curr,
                     &e.next.as_ref().unwrap().borrow().curr
                 ))
            {
                if e == e_start {
                    e_start = e.next.as_ref().unwrap().borrow().clone();
                }
                e = self.remove_edge(&e);
                e_loop_stop = e.clone();
                continue;
            }
            e = e.next.as_ref().unwrap().borrow().clone();
            if e.eq(&e_loop_stop) || (!closed && e.next.as_ref().unwrap().borrow().eq(&e_start)) {
                break;
            }
        }

        if (!closed && e.prev.as_ref().unwrap().borrow().eq(&e.next.as_ref().unwrap().borrow())) ||
           (closed && e.prev.as_ref().unwrap().borrow().eq(&e.next.as_ref().unwrap().borrow())) {
            return Ok(false);
        }

        if !closed {
            self.has_open_paths = true;
            e_start.prev.as_ref().unwrap().borrow_mut().out_idx = SKIP;
        }

        // 3. Do second stage of edge initialization
        let mut e = e_start.clone();
        loop {
            self.init_edge2(&mut e, poly_type);
            e = e.next.as_ref().unwrap().borrow().clone();
            if is_flat && e.curr.y != e_start.curr.y {
                is_flat = false;
            }
            if e.eq(&e_start) {
                break;
            }
        }

        // 4. Finally, add local minima to LocalMinima list
        
        // Handle totally flat paths specially
        if is_flat {
            if closed {
                return Ok(false);
            }
            e.prev.as_ref().unwrap().borrow_mut().out_idx = SKIP;
            let mut local_min = LocalMinima {
                y: e.bot.y,
                left_bound: None,
                right_bound: Some(Rc::new(RefCell::new(e.clone()))),
                next: None,
            };
            local_min.right_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Right;
            local_min.right_bound.as_ref().unwrap().borrow_mut().wind_delta = 0;

            loop {
                if e.bot.x != e.prev.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&mut e);
                }
                if e.next.as_ref().unwrap().borrow().out_idx == SKIP {
                    break;
                }
                e = e.next.as_ref().unwrap().borrow().clone();
            }
            self.insert_local_minima(&mut local_min);
            self.edges.push(edges);
            return Ok(true);
        }

        // Regular path processing
        self.edges.push(edges);
        let mut left_bound_is_forward = true;
        let mut e_min = None;

        // Handle open paths with matching endpoints
        if e.prev.as_ref().unwrap().borrow().bot == e.prev.as_ref().unwrap().borrow().top {
            e = e.next.as_ref().unwrap().borrow().clone();
        }

        loop {
            e = self.find_next_local_minimum(&e);
            if e_min.is_some() && e.eq(e_min.as_ref().unwrap()) {
                break;
            }
            if e_min.is_none() {
                e_min = Some(e.clone());
            }

            // Process the local minima
            let mut local_min = LocalMinima {
                y: e.bot.y,
                left_bound: None,
                right_bound: None,
                next: None,
            };

            if e.dx < e.prev.as_ref().unwrap().borrow().dx {
                local_min.left_bound = Some(Rc::new(RefCell::new(e.prev.as_ref().unwrap().borrow().clone())));
                local_min.right_bound = Some(Rc::new(RefCell::new(e.clone())));
                left_bound_is_forward = false;
            } else {
                local_min.left_bound = Some(Rc::new(RefCell::new(e.clone())));
                local_min.right_bound = Some(Rc::new(RefCell::new(e.prev.as_ref().unwrap().borrow().clone())));
                left_bound_is_forward = true;
            }

            local_min.left_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Left;
            local_min.right_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Right;

            if !closed {
                local_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = 0;
            } else if local_min.left_bound.as_ref().unwrap().borrow().next.as_ref().unwrap().borrow().eq(
                local_min.right_bound.as_ref().unwrap().borrow().deref()
            ) {
                local_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = -1;
                local_min.right_bound.as_ref().unwrap().borrow_mut().wind_delta = 1;
            } else {
                local_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = 
                    if left_bound_is_forward { 1 } else { -1 };
                local_min.right_bound.as_ref().unwrap().borrow_mut().wind_delta = 
                    -local_min.left_bound.as_ref().unwrap().borrow().wind_delta;
            }

            self.insert_local_minima(&mut local_min);

            if !left_bound_is_forward {
                e = e.prev.as_ref().unwrap().borrow().clone();
            }
        }

        Ok(true)
    }

    /// Checks if a value is near zero
    pub fn near_zero(val: f64) -> bool {
        val > -TOLERANCE && val < TOLERANCE
    }

    /// Clears internal state
    pub fn clear(&mut self) {
        self.dispose_local_minima_list();
        for edge_list in &mut self.edges {
            edge_list.clear();
        }
        self.edges.clear();
        self.use_full_range = false;
        self.has_open_paths = false;
    }

    /// Resets the clipper state for a new operation
    pub fn reset(&mut self) {
        self.current_lm = self.minima_list.clone();
        if self.current_lm.is_none() {
            return;
        }

        self.scanbeam = None;
        let mut lm = self.minima_list.clone();
        while let Some(local_minima) = lm {
            self.insert_scanbeam(local_minima.y);
            // Reset edge properties
            if let Some(ref mut e) = local_minima.left_bound {
                e.borrow_mut().curr = e.borrow().bot;
                e.borrow_mut().out_idx = UNASSIGNED;
            }
            if let Some(ref mut e) = local_minima.right_bound {
                e.borrow_mut().curr = e.borrow().bot;
                e.borrow_mut().out_idx = UNASSIGNED;
            }
            lm = local_minima.next;
        }
        self.active_edges = None;
    }

    /// Checks if a point is a vertex in the output polygon
    /// internal
    fn point_is_vertex(&self, pt: &IntPoint, pp: &OutPt) -> bool {
        let mut pp2 = pp;
        loop {
            if pp2.pt == *pt {
                return true;
            }
            pp2 = &pp2.next.as_ref().unwrap().borrow();
            if pp2 == pp {
                break;
            }
        }
        false
    }

    /// Checks if a point lies on a line segment
    fn point_on_line_segment(
        &self, 
        pt: &IntPoint,
        line_pt1: &IntPoint,
        line_pt2: &IntPoint,
        use_full_range: bool
    ) -> bool {
        if (*pt == *line_pt1) || (*pt == *line_pt2) {
            return true;
        }
        
        // Check if point lies between endpoints using coordinate comparison
        let x_between = (pt.x > line_pt1.x) == (pt.x < line_pt2.x);
        let y_between = (pt.y > line_pt1.y) == (pt.y < line_pt2.y);

        if use_full_range {
            // High precision slope comparison using Int128
            // Equivalent to: (pt.X - linePt1.X) * (linePt2.Y - linePt1.Y) ==
            //               (linePt2.X - linePt1.X) * (pt.Y - linePt1.Y)
            let x_diff1 = (pt.x - line_pt1.x) as i128;
            let y_diff2 = (line_pt2.y - line_pt1.y) as i128;
            let x_diff2 = (line_pt2.x - line_pt1.x) as i128;
            let y_diff1 = (pt.y - line_pt1.y) as i128;
            x_diff1 * y_diff2 == x_diff2 * y_diff1
        } else {
            // Standard precision using i64
            ((pt.x - line_pt1.x) as i64) * ((line_pt2.y - line_pt1.y) as i64) ==
            ((line_pt2.x - line_pt1.x) as i64) * ((pt.y - line_pt1.y) as i64)
        }
    }
}

impl Default for ClipperBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_on_line_segment() {
        let clipper = ClipperBase::new();
        let pt = IntPoint::new(5, 5);
        let line_pt1 = IntPoint::new(0, 0);
        let line_pt2 = IntPoint::new(10, 10);
        
        // Test point on diagonal line
        assert!(clipper.point_on_line_segment(&pt, &line_pt1, &line_pt2, false));
        
        // Test endpoint
        assert!(clipper.point_on_line_segment(&line_pt1, &line_pt1, &line_pt2, false));
        
        // Test point not on line
        let pt_off = IntPoint::new(5, 6);
        assert!(!clipper.point_on_line_segment(&pt_off, &line_pt1, &line_pt2, false));
    }
}
