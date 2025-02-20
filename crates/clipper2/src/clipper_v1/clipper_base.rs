use std::cell::RefCell;
use std::ops::Deref as _;
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
    pub fn range_test(pt: IntPoint, use_full_range: bool) -> Result<((), bool)> {
        let mut use_full = use_full_range;
        if use_full {
            if pt.x > HI_RANGE || pt.y > HI_RANGE || -pt.x > HI_RANGE || -pt.y > HI_RANGE {
                return Err(ClipperError::CoordinateOutOfRange);
            }
        } else if pt.x > LO_RANGE || pt.y > LO_RANGE || -pt.x > LO_RANGE || -pt.y > LO_RANGE {
            use_full = true;
            Self::range_test(pt, use_full)?;
        }
        Ok(((), use_full))
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
        let use_full_range = self.use_full_range;
        let (_, use_full_range) = Self::range_test(path[0], use_full_range)?;
        let (_, use_full_range) = Self::range_test(path[high_i as usize], use_full_range)?;
        self.use_full_range = use_full_range;
        // Initialize first and last edges
        let last_idx = edges.len() - 1;
        {
            let (first, rest) = edges.split_at_mut(1);
            self.init_edge(&mut first[0], &rest[0], &rest[last_idx-1], path[0]);
        }
        {
            let (left, right) = edges.split_at_mut(high_i as usize);
            self.init_edge(&mut right[0], &left[0], &left[left.len()-1], path[high_i as usize]);
        }
        
        for i in (1..high_i).rev() {
            let (_, use_full_range) = Self::range_test(path[i as usize], self.use_full_range)?;
            self.use_full_range = use_full_range;
            let (_, use_full_range) = Self::range_test(path[i as usize], self.use_full_range)?;
            self.use_full_range = use_full_range;
            let (left, right) = edges.split_at_mut(i as usize);
            let (curr, right) = right.split_at_mut(1);
            self.init_edge(&mut curr[0], &right[0], &left[left.len()-1], path[i as usize]);
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
                 !CBS::pt2_is_between_pt1_and_pt3(
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
            let next_edge = e.next.as_ref().unwrap().borrow().clone();
            e = next_edge;
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
            {
                // Create temporary mutable reference within a new scope
                let mut temp_e = e.clone();
                self.init_edge2(&mut temp_e, poly_type);
                e = temp_e;
            }
            
            let next_edge = e.next.as_ref().unwrap().borrow().clone();
            if is_flat && next_edge.curr.y != e_start.curr.y {
                is_flat = false;
            }
            if next_edge.eq(&e_start) {
                break;
            }
            e = next_edge;
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
                let next_edge = e.next.as_ref().unwrap().borrow().clone();
                e = next_edge;
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
            {
                let next = e.next.as_ref().unwrap().borrow().clone();
                e = next;
            }
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
                let prev_edge = e.prev.as_ref().unwrap().borrow().clone();
                e = prev_edge;
            }
        }

        Ok(true)
    }

    /// Adds multiple paths to the clipper
    pub fn add_paths(&mut self, ppg: &Paths, poly_type: PolyType, closed: bool) -> Result<bool> {
        let mut result = false;
        for path in ppg {
            if self.add_path(path, poly_type, closed)? {
                result = true;
            }
        }
        Ok(result)
    }

    /// Clears internal state and disposes of allocated resources
    pub fn clear(&mut self) {
        // Dispose of the local minima list
        self.dispose_local_minima_list();

        // Clear edges vectors
        for edge_list in &mut self.edges {
            // Mark edges as removed by nulling their prev pointers
            for edge in edge_list.iter_mut() {
                edge.prev = None;
            }
            edge_list.clear();
        }
        self.edges.clear();

        // Reset flags
        self.use_full_range = false;
        self.has_open_paths = false;
    }

    /// Disposes of the local minima list by nulling references
    fn dispose_local_minima_list(&mut self) {
        while let Some(mut lm) = self.minima_list.take() {
            // Take next minima before dropping current one
            self.minima_list = lm.next.take();
        }
        self.current_lm = None;
    }

    /// Resets the clipper state for a new operation 
    pub fn reset(&mut self) {
        self.current_lm = self.minima_list.clone();
        if self.current_lm.is_none() {
            return;
        }

        self.scanbeam = None;
        let mut lm = self.minima_list.clone();
        while let Some(mut local_minima) = lm {
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

    /// Inserts a value into the scanbeam sorted by Y coordinate (descending)
    fn insert_scanbeam(&mut self, y: CInt) {
        // Create new scanbeam
        let new_sb = Box::new(Scanbeam {
            y,
            next: None,
        });

        // If no scanbeam exists, create first one
        if self.scanbeam.is_none() {
            self.scanbeam = Some(new_sb);
            return;
        }

        // If Y is greater than scanbeam.Y, insert at start
        if y > self.scanbeam.as_ref().unwrap().y {
            let mut new_sb = new_sb;
            new_sb.next = self.scanbeam.take();
            self.scanbeam = Some(new_sb);
            return;
        }

        // Find insertion point
        let mut sb = self.scanbeam.as_mut().unwrap();
        while sb.next.is_some() && y <= sb.next.as_ref().unwrap().y {
            sb = sb.next.as_mut().unwrap();
        }

        // Ignore if Y equals current scanbeam Y 
        if y == sb.y {
            return;
        }

        // Insert new scanbeam
        let mut new_sb = new_sb;
        new_sb.next = sb.next.take();
        sb.next = Some(new_sb);
    }

    /// Pops the topmost scanbeam from the list and returns its Y coordinate
    pub fn pop_scanbeam(&mut self) -> Result<Option<CInt>> {
        // Check if scanbeam list is empty
        if self.scanbeam.is_none() {
            return Ok(None);
        }

        // Get Y value from current scanbeam
        let y = self.scanbeam.as_ref().unwrap().y;
        
        // Move to next scanbeam
        self.scanbeam = self.scanbeam.take().unwrap().next;
        
        // Return the Y value
        Ok(Some(y))
    }

    /// Checks if a point is a vertex in the output polygon
    /// internal
    fn point_is_vertex(&self, pt: &IntPoint, pp: &OutPt) -> bool {
        let mut pp2 = pp.clone();
        loop {
            if pp2.pt == *pt {
                return true;
            }
            let next = pp2.next.as_ref().unwrap().borrow().clone();
            if std::ptr::eq(&next.clone(), pp) {
                break;
            }
            pp2 = next;
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

    /// Checks if a point lies on any edge of a polygon
    fn point_on_polygon(&self, pt: &IntPoint, pp: &OutPt, use_full_range: bool) -> bool {
        let pp2 = pp.clone();
        loop {
            if self.point_on_line_segment(pt, &pp2.pt, &pp2.next.as_ref().unwrap().borrow().pt, use_full_range) {
                return true;
            }
            let next_pp2 = pp2.next.as_ref().unwrap().borrow();
            if std::ptr::eq(&next_pp2.clone(), pp) {
                break;
            }
        }
        false
    }

    /// Finds the next local minimum edge in a sequence
    fn find_next_local_minimum(&self, edge: &TEdge) -> TEdge {
        let mut e = edge.clone();
        loop {
            // Skip edges where bottom equals previous bottom or current equals top
            while e.bot == e.prev.as_ref().unwrap().borrow().bot || e.curr == e.top {
                let next_edge = e.next.as_ref().unwrap().borrow().clone();
                e = next_edge;
            }

            // Skip horizontal edges
            if e.dx != HORIZONTAL && e.prev.as_ref().unwrap().borrow().dx != HORIZONTAL {
                // Skip edges where previous is horizontal
                while e.prev.as_ref().unwrap().borrow().dx == HORIZONTAL {
                    let prev_edge = e.prev.as_ref().unwrap().borrow().clone();
                    e = prev_edge;
                }
                // Skip edges where current is horizontal
                while e.dx == HORIZONTAL {
                    let next_edge = e.next.as_ref().unwrap().borrow().clone();
                    e = next_edge;
                }
            }

            // Check if we've found a local minimum
            if e.top.y == e.prev.as_ref().unwrap().borrow().bot.y {
                // If previous bottom X is less than current bottom X
                if e.prev.as_ref().unwrap().borrow().bot.x < e.bot.x {
                    break;
                }
            }
            {
                let next_edge = e.next.as_ref().unwrap().borrow().clone();
                e = next_edge;
            }
        }
        e
    }

    /// Processes a bound edge and returns the processed edge
    fn process_bound(&mut self, edge: &TEdge, left_bound_is_forward: bool) -> TEdge {
        let mut result = edge.clone();
        let mut e = edge.clone();

        // Handle edges marked for skipping
        if result.out_idx == SKIP {
            // Check for additional edges beyond the skip edge
            if left_bound_is_forward {
                while e.top.y == e.next.as_ref().unwrap().borrow().bot.y {
                    let next_edge = e.next.as_ref().unwrap().borrow().clone();
                    e = next_edge;
                }
                let mut e_clone = e.clone();
                while e_clone != result && e_clone.dx == HORIZONTAL {
                    let prev_edge = e_clone.prev.as_ref().unwrap().borrow().clone();
                    e_clone = prev_edge;
                }
                e = e_clone;
            } else {
                while e.top.y == e.prev.as_ref().unwrap().borrow().bot.y {
                    let prev_edge = e.prev.as_ref().unwrap().borrow().clone();
                    e = prev_edge;
                }
                while e != result && e.dx == HORIZONTAL {
                    {
                        let next_edge = e.next.as_ref().unwrap().borrow().clone();
                        e = next_edge;
                    }
                }
            }
            if e == result {
                result = if left_bound_is_forward {
                    e.next.as_ref().unwrap().borrow().clone()
                } else {
                    e.prev.as_ref().unwrap().borrow().clone()
                };
            } else {
                // Create new local minima for remaining edges
                e = if left_bound_is_forward {
                    result.next.as_ref().unwrap().borrow().clone()
                } else {
                    result.prev.as_ref().unwrap().borrow().clone()
                };
                
                let mut local_min = LocalMinima {
                    y: e.bot.y,
                    left_bound: None,
                    right_bound: Some(Rc::new(RefCell::new(e.clone()))),
                    next: None,
                };
                
                e.wind_delta = 0;
                result = self.process_bound(&e, left_bound_is_forward);
                self.insert_local_minima(&mut local_min);
            }
            return result;
        }

        // Handle horizontal edges
        if e.dx == HORIZONTAL {
            // Handle consecutive horizontal edges carefully
            if left_bound_is_forward {
                let e_start = e.prev.as_ref().unwrap().borrow().clone();
                if e_start.dx == HORIZONTAL {
                    // Check if bot.x values match
                    if e_start.bot.x != e.bot.x && e_start.top.x != e.bot.x {
                        self.reverse_horizontal(&mut e);
                    }
                } else if e_start.bot.x != e.bot.x {
                    self.reverse_horizontal(&mut e);
                }
            } else {
                let e_start = e.next.as_ref().unwrap().borrow().clone();
                if e_start.dx == HORIZONTAL {
                    if e_start.bot.x != e.bot.x && e_start.top.x != e.bot.x {
                        self.reverse_horizontal(&mut e);
                    }
                } else if e_start.bot.x != e.bot.x {
                    self.reverse_horizontal(&mut e);
                }
            }
        }

        let e_start = e.clone();
        if left_bound_is_forward {
            while result.top.y == result.next.as_ref().unwrap().borrow().bot.y
                && result.next.as_ref().unwrap().borrow().out_idx != SKIP {
                let next_result = result.next.as_ref().unwrap().borrow().clone();
                result = next_result;
            }
            
            if result.dx == HORIZONTAL && result.next.as_ref().unwrap().borrow().out_idx != SKIP {
                let mut horz = result.clone();
                while horz.prev.as_ref().unwrap().borrow().dx == HORIZONTAL {
                    let temp_horz = horz.prev.as_ref().unwrap().borrow().clone();
                    horz = temp_horz;
                }
                if horz.prev.as_ref().unwrap().borrow().top.x > result.next.as_ref().unwrap().borrow().top.x {
                    result = horz.prev.as_ref().unwrap().borrow().clone();
                }
            }
            
            while e != result {
                e.next_in_lml = Some(Rc::new(RefCell::new(e.next.as_ref().unwrap().borrow().clone())));
                if e.dx == HORIZONTAL && e != e_start && e.bot.x != e.prev.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&mut e);
                }
                let next_edge = e.next.as_ref().unwrap().borrow().clone();
                e = next_edge;
            }
            
            if e.dx == HORIZONTAL && e != e_start && e.bot.x != e.prev.as_ref().unwrap().borrow().top.x {
                self.reverse_horizontal(&mut e);
            }
            let next_result = result.next.as_ref().unwrap().borrow().clone();
            result = next_result;
            
        } else {
            while result.top.y == result.prev.as_ref().unwrap().borrow().bot.y
                && result.prev.as_ref().unwrap().borrow().out_idx != SKIP {
                let prev_edge = result.prev.as_ref().unwrap().borrow().clone();
                result = prev_edge;
            }
            
            if result.dx == HORIZONTAL && result.prev.as_ref().unwrap().borrow().out_idx != SKIP {
                let mut horz = result.clone();
                while horz.next.as_ref().unwrap().borrow().dx == HORIZONTAL {
                    let next_horz = horz.next.as_ref().unwrap().borrow().clone();
                    horz = next_horz;
                }
                if horz.next.as_ref().unwrap().borrow().top.x == result.prev.as_ref().unwrap().borrow().top.x ||
                   horz.next.as_ref().unwrap().borrow().top.x > result.prev.as_ref().unwrap().borrow().top.x {
                    result = horz.next.as_ref().unwrap().borrow().clone();
                }
            }

            while e != result {
                e.next_in_lml = Some(Rc::new(RefCell::new(e.prev.as_ref().unwrap().borrow().clone())));
                if e.dx == HORIZONTAL && e != e_start && e.bot.x != e.next.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&mut e);
                }
                let prev_edge = e.prev.as_ref().unwrap().borrow().clone();
                e = prev_edge;
            }
            
            if e.dx == HORIZONTAL && e != e_start && e.bot.x != e.next.as_ref().unwrap().borrow().top.x {
                self.reverse_horizontal(&mut e);
            }
            let prev_edge = result.prev.as_ref().unwrap().borrow().clone();
            result = prev_edge;
        }
        
        result
    }

    /// Removes an edge from a double-linked list and returns the next edge
    fn remove_edge(&self, e: &TEdge) -> TEdge {
        // Get references to prev and next edges
        let prev = e.prev.as_ref().unwrap();
        let next = e.next.as_ref().unwrap();

        // Update next's prev pointer
        next.borrow_mut().prev = Some(prev.clone());
        
        // Update prev's next pointer
        prev.borrow_mut().next = Some(next.clone());

        // Return the next edge while marking e as removed by clearing its prev pointer
        let mut result = next.borrow().clone();
        result.prev = None; // flag as removed
        result
    }

    /// Inserts a local minima into the sorted minima list
    fn insert_local_minima(&mut self, new_lm: &mut LocalMinima) {
        if self.minima_list.is_none() {
            self.minima_list = Some(Box::new(new_lm.clone()));
        } else if new_lm.y >= self.minima_list.as_ref().unwrap().y {
            new_lm.next = self.minima_list.take();
            self.minima_list = Some(Box::new(new_lm.clone()));
        } else {
            let mut curr_lm = self.minima_list.as_mut().unwrap();
            while curr_lm.next.is_some() && new_lm.y < curr_lm.next.as_ref().unwrap().y {
                curr_lm = curr_lm.next.as_mut().unwrap();
            }
            new_lm.next = curr_lm.next.take();
            curr_lm.next = Some(Box::new(new_lm.clone()));
        }
    }

    /// Pops a local minima from the minima list at the specified Y coordinate
    pub fn pop_local_minima(&mut self, y: CInt) -> Option<Box<LocalMinima>> {
        // Check if there is a current local minima at the specified Y coordinate
        if let Some(lm) = self.current_lm.take() {
            if lm.y == y {
                // Move to next local minima and return current one
                self.current_lm = lm.next.clone();
                return Some(lm);
            }
            // Put back the current local minima if Y doesn't match
            self.current_lm = Some(lm);
        }
        None
    }

    /// Reverses a horizontal edge by swapping its top and bottom X coordinates
    fn reverse_horizontal(&self, edge: &mut TEdge) {
        // Swap horizontal edges' top and bottom x's so they follow the natural
        // progression of the bounds - ie so their xbots will align with the
        // adjoining lower edge
        CBS::swap(&mut edge.top.x, &mut edge.bot.x);
    }

    /// Returns true if there are more local minima to process
    pub fn local_minima_pending(&self) -> bool {
        self.current_lm.is_some()
    }

    /// Creates a new output record and adds it to the output list
    pub fn create_out_rec(&mut self) -> OutRec {
        let mut result = OutRec::default();

        // Add to poly_outs list and set index
        self.poly_outs.push(result.clone());
        result.idx = (self.poly_outs.len() - 1) as i32;

        result
    }

    /// Disposes of an output record at the specified index
    pub fn dispose_out_rec(&mut self, index: usize) {
        // Clear points reference and the record itself
        let mut out_rec = &mut self.poly_outs[index];
        out_rec.pts = None;
        
        // Clear the output record slot
        self.poly_outs[index] = OutRec::default();
    }

    /// Updates an edge in the active edge list with its next edge in line
    pub fn update_edge_into_ael(&mut self, e: &mut Rc<RefCell<TEdge>>) -> Result<()> {
        if e.borrow().next_in_lml.is_none() {
            return Err(ClipperError::OperationFailure("UpdateEdgeIntoAEL: invalid call".to_string()));
        }

        let ael_prev = e.borrow().prev_in_ael.clone();
        let ael_next = e.borrow().next_in_ael.clone();
        
        // Set up next edge with same properties
        let next = e.borrow().next_in_lml.clone().unwrap();
        next.borrow_mut().out_idx = e.borrow().out_idx;
        
        // Update AEL pointers for previous edge
        if let Some(prev) = ael_prev.clone() {
            prev.borrow_mut().next_in_ael = Some(next.clone());
        } else {
            self.active_edges = Some(next.clone());
        }

        // Update AEL pointers for next edge
        if let Some(next_edge) = ael_next.clone() {
            next_edge.borrow_mut().prev_in_ael = Some(next.clone());
        }

        // Copy remaining edge properties
        next.borrow_mut().side = e.borrow().side;
        next.borrow_mut().wind_delta = e.borrow().wind_delta;
        next.borrow_mut().wind_cnt = e.borrow().wind_cnt;
        next.borrow_mut().wind_cnt2 = e.borrow().wind_cnt2;

        // Update e to point to the new edge
        *e = next.clone();

        // Update current point and AEL pointers
        e.borrow_mut().curr = e.borrow().bot;
        e.borrow_mut().prev_in_ael = ael_prev;
        e.borrow_mut().next_in_ael = ael_next;

        // Insert new scanbeam if edge isn't horizontal
        if !CBS::is_horizontal(&e.borrow()) {
            self.insert_scanbeam(e.borrow().top.y);
        }

        Ok(())
    }

    /// Swaps the positions of two edges in the Active Edge List (AEL)
    pub fn swap_positions_in_ael(&mut self, edge1: &Rc<RefCell<TEdge>>, edge2: &Rc<RefCell<TEdge>>) {
        // Check that one or other edge hasn't already been removed from AEL
        if edge1.borrow().next_in_ael.as_ref() == edge1.borrow().prev_in_ael.as_ref() ||
           edge2.borrow().next_in_ael.as_ref() == edge2.borrow().prev_in_ael.as_ref() {
            return;
        }

        // Handle case where edges are adjacent
        if edge1.borrow().next_in_ael.as_ref() == Some(edge2) {
            // Get next edge after edge2
            let next = edge2.borrow().next_in_ael.clone();
            // Update next's prev pointer if it exists
            if let Some(ref next_edge) = next {
                next_edge.borrow_mut().prev_in_ael = Some(edge1.clone());
            }
            
            // Get prev edge before edge1
            let prev = edge1.borrow().prev_in_ael.clone();
            // Update prev's next pointer if it exists
            if let Some(ref prev_edge) = prev {
                prev_edge.borrow_mut().next_in_ael = Some(edge2.clone());
            }

            // Update edge2's pointers
            edge2.borrow_mut().prev_in_ael = prev;
            edge2.borrow_mut().next_in_ael = Some(edge1.clone());
            
            // Update edge1's pointers
            edge1.borrow_mut().prev_in_ael = Some(edge2.clone());
            edge1.borrow_mut().next_in_ael = next;
        }
        else if edge2.borrow().next_in_ael.as_ref() == Some(edge1) {
            // Get next edge after edge1
            let next = edge1.borrow().next_in_ael.clone();
            // Update next's prev pointer if it exists
            if let Some(ref next_edge) = next {
                next_edge.borrow_mut().prev_in_ael = Some(edge2.clone());
            }
            
            // Get prev edge before edge2
            let prev = edge2.borrow().prev_in_ael.clone();
            // Update prev's next pointer if it exists
            if let Some(ref prev_edge) = prev {
                prev_edge.borrow_mut().next_in_ael = Some(edge1.clone());
            }

            // Update edge1's pointers
            edge1.borrow_mut().prev_in_ael = prev;
            edge1.borrow_mut().next_in_ael = Some(edge2.clone());
            
            // Update edge2's pointers
            edge2.borrow_mut().prev_in_ael = Some(edge1.clone());
            edge2.borrow_mut().next_in_ael = next;
        }
        else {
            // Handle non-adjacent edges
            let next1 = edge1.borrow().next_in_ael.clone();
            let prev1 = edge1.borrow().prev_in_ael.clone();
            
            edge1.borrow_mut().next_in_ael = edge2.borrow().next_in_ael.clone();
            if let Some(ref next) = edge1.borrow().next_in_ael {
                next.borrow_mut().prev_in_ael = Some(edge1.clone());
            }
            
            edge1.borrow_mut().prev_in_ael = edge2.borrow().prev_in_ael.clone();
            if let Some(ref prev) = edge1.borrow().prev_in_ael {
                prev.borrow_mut().next_in_ael = Some(edge1.clone());
            }

            edge2.borrow_mut().next_in_ael = next1;
            if let Some(ref next) = edge2.borrow().next_in_ael {
                next.borrow_mut().prev_in_ael = Some(edge2.clone());
            }
            
            edge2.borrow_mut().prev_in_ael = prev1;
            if let Some(ref prev) = edge2.borrow().prev_in_ael {
                prev.borrow_mut().next_in_ael = Some(edge2.clone());
            }
        }

        // Update active_edges if either edge was at the start
        if edge1.borrow().prev_in_ael.is_none() {
            self.active_edges = Some(edge1.clone());
        }
        else if edge2.borrow().prev_in_ael.is_none() {
            self.active_edges = Some(edge2.clone());
        }
    }

    /// Deletes an edge from the Active Edge List (AEL)
    pub fn delete_from_ael(&mut self, e: &Rc<RefCell<TEdge>>) {
        let ael_prev = e.borrow().prev_in_ael.clone();
        let ael_next = e.borrow().next_in_ael.clone();
        
        // Check if edge is already deleted
        if ael_prev.is_none() && ael_next.is_none() && !Rc::ptr_eq(e, self.active_edges.as_ref().unwrap()) {
            return; // already deleted
        }

        // Update prev's next pointer
        if let Some(prev) = ael_prev.clone() {
            prev.borrow_mut().next_in_ael = ael_next.clone();
        } else {
            self.active_edges = ael_next.clone();
        }

        // Update next's prev pointer
        if let Some(next) = ael_next.clone() {
            next.borrow_mut().prev_in_ael = ael_prev.clone();
        }

        // Clear edge's pointers
        e.borrow_mut().next_in_ael = None;
        e.borrow_mut().prev_in_ael = None;
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

    #[test]
    fn test_point_on_polygon() {
        let clipper = ClipperBase::new();
        
        // Create a simple square polygon using OutPt circular linked list
        let pt1 = OutPt {
            idx: 0,
            pt: IntPoint::new(0, 0),
            next: None,
            prev: None,
        };
        let pt2 = OutPt {
            idx: 1,
            pt: IntPoint::new(10, 0),
            next: None,
            prev: None,
        };
        let pt3 = OutPt {
            idx: 2,
            pt: IntPoint::new(10, 10),
            next: None,
            prev: None,
        };
        let pt4 = OutPt {
            idx: 3,
            pt: IntPoint::new(0, 10),
            next: None,
            prev: None,
        };

        // Link the points in a circular list
        let pt1_rc = Rc::new(RefCell::new(pt1));
        let pt2_rc = Rc::new(RefCell::new(pt2));
        let pt3_rc = Rc::new(RefCell::new(pt3));
        let pt4_rc = Rc::new(RefCell::new(pt4));

        pt1_rc.borrow_mut().next = Some(pt2_rc.clone());
        pt2_rc.borrow_mut().next = Some(pt3_rc.clone());
        pt3_rc.borrow_mut().next = Some(pt4_rc.clone());
        pt4_rc.borrow_mut().next = Some(pt1_rc.clone());

        // Test points
        let point_on_edge = IntPoint::new(5, 0);  // point on bottom edge
        let point_off_edge = IntPoint::new(5, 5); // point inside polygon

        assert!(clipper.point_on_polygon(&point_on_edge, &pt1_rc.borrow(), false));
        assert!(!clipper.point_on_polygon(&point_off_edge, &pt1_rc.borrow(), false));
    }
}
