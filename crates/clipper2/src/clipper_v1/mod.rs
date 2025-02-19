#![deny(clippy::all)]
use std::cell::RefCell;
use std::rc::Rc;

use std::f64::consts::PI;
use std::primitive::f64;
use std::primitive::i128;

use std::cmp::Ordering;

/// Alias for 64-bit integers.
pub type CInt = i64;

const SCALE: f64 = 1e7;

// Missing constants from the C# version:
pub const HORIZONTAL: f64 = -3.4E38;
pub const SKIP: i32 = -2;
pub const UNASSIGNED: i32 = -1;
pub const TOLERANCE: f64 = 1.0E-20;

pub const HI_RANGE: CInt = 0x3FFFFFFFFFFFFFFF;
pub const LO_RANGE: CInt = 0x3FFFFFFF;

////////////////////////////////////////////////////////////////////////////////
// DoublePoint
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoublePoint {
    pub x: f64,
    pub y: f64,
}

impl DoublePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<IntPoint> for DoublePoint {
    fn from(ip: IntPoint) -> Self {
        Self {
            x: ip.x as f64 / SCALE,
            y: ip.y as f64 / SCALE,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// IntPoint
////////////////////////////////////////////////////////////////////////////////

/// A port of the C# IntPoint struct.
///
/// This struct provides:
/// - A constructor from two CInt values.
/// - A constructor from two f64 values (rounding to the nearest integer).
/// - A copy constructor.
/// - A conversion from DoublePoint (scaling the coordinates by 1e7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntPoint {
    pub x: CInt,
    pub y: CInt,
}

impl IntPoint {
    /// Constructs a new IntPoint from two CInt values.
    pub fn new(x: CInt, y: CInt) -> Self {
        Self { x, y }
    }

    /// Constructs a new IntPoint from two f64 values.
    /// This mirrors the C# constructor IntPoint(double x, double y)
    /// by rounding the doubles to the nearest integer.
    pub fn from_doubles(x: f64, y: f64) -> Self {
        Self {
            x: x.round() as CInt,
            y: y.round() as CInt,
        }
    }

    /// Constructs a new IntPoint by copying another one.
    pub fn from_int_point(pt: &IntPoint) -> Self {
        *pt
    }
}

impl From<DoublePoint> for IntPoint {
    /// Converts a DoublePoint into an IntPoint using the fixed SCALE factor.
    /// (Multiplies the double values by 1e7 and rounds to the nearest integer.)
    fn from(dp: DoublePoint) -> Self {
        Self {
            x: (dp.x * SCALE).round() as CInt,
            y: (dp.y * SCALE).round() as CInt,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// IntRect
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntRect {
    pub left: CInt,
    pub top: CInt,
    pub right: CInt,
    pub bottom: CInt,
}

impl IntRect {
    /// Constructs a new IntRect with the specified boundaries.
    pub fn new(left: CInt, top: CInt, right: CInt, bottom: CInt) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Constructs a new IntRect by copying another one.
    pub fn from_rect(other: &IntRect) -> Self {
        Self {
            left: other.left,
            top: other.top,
            right: other.right,
            bottom: other.bottom,
        }
    }
}
// A Path is simply a vector of IntPoint.
pub type Path = Vec<IntPoint>;
// Paths is a vector of Path.
pub type Paths = Vec<Path>;

////////////////////////////////////////////////////////////////////////////////
// PolyNode and PolyTree structure.
////////////////////////////////////////////////////////////////////////////////

/// A port of the C# PolyNode class.
///
/// In C#, PolyNode contains internal members such as m_Parent, m_polygon,
/// m_Childs, etc. Here we mirror those as public fields (for simplicity)
/// and provide methods for:
/// - Adding a child node (which sets the parent and index).
/// - Getting a next node (first child or recursively a next sibling up).
/// - Determining whether the node is a hole (by walking up the parent chain).
#[derive(Debug, Clone)]
pub struct PolyNode {
    /// Parent node. (In C# this was internal.)
    pub parent: Option<Box<PolyNode>>,
    /// The node’s contour (polygon).
    pub polygon: Path,
    /// The index of this node among its parent's children.
    pub index: usize,
    /// The join type, as in the original.
    pub jointype: JoinType,
    /// The end type, as in the original.
    pub endtype: EndType,
    /// Child nodes.
    pub childs: Vec<PolyNode>,
    /// Indicates if this node represents an open path.
    pub is_open: bool,
}

impl PolyNode {
    /// Constructs a new, empty PolyNode.
    pub fn new() -> Self {
        Self {
            parent: None,
            polygon: Vec::new(),
            index: 0,
            jointype: JoinType::Square, // default value; adjust as needed
            endtype: EndType::ClosedPolygon, // default value; adjust as needed
            childs: Vec::new(),
            is_open: false,
        }
    }

    /// Returns the number of child nodes.
    pub fn child_count(&self) -> usize {
        self.childs.len()
    }

    /// Returns a reference to the node's contour (its polygon).
    pub fn contour(&self) -> &Path {
        &self.polygon
    }

    /// Adds a child to the node.
    /// Sets the child's parent and index.
    pub fn add_child(&mut self, mut child: PolyNode) {
        // Set child's parent to a clone of the current node.
        child.parent = Some(Box::new(self.clone()));
        child.index = self.childs.len();
        self.childs.push(child);
    }

    /// Returns the next node in a traversal.
    /// In the C# version, GetNext() returns the first child if available;
    /// otherwise, it returns the next sibling up the tree.
    pub fn get_next(&self) -> Option<&PolyNode> {
        if !self.childs.is_empty() {
            Some(&self.childs[0])
        } else {
            self.get_next_sibling_up()
        }
    }

    /// Returns the next sibling up the parent chain.
    /// (If no such node exists, returns None.)
    pub fn get_next_sibling_up(&self) -> Option<&PolyNode> {
        if let Some(ref parent) = self.parent {
            if self.index + 1 < parent.childs.len() {
                Some(&parent.childs[self.index + 1])
            } else {
                parent.get_next_sibling_up()
            }
        } else {
            None
        }
    }

    /// Determines whether this node is a hole.
    /// The algorithm walks up the parent chain and toggles a boolean flag.
    pub fn is_hole(&self) -> bool {
        let mut result = false;
        let mut node = &self.parent;
        while let Some(ref p) = node {
            result = !result;
            node = &p.parent;
        }
        result
    }
}

/// A port of the C# PolyTree class.
///
/// In ClipperLib C#, PolyTree inherits from PolyNode and maintains a list of all nodes.
/// Here we represent a PolyTree as a structure containing a root PolyNode and a vector
/// of all its nodes (all_polys). The methods Clear, total, and get_first are provided
/// to mimic the original functionality.
#[derive(Debug, Clone)]
pub struct PolyTree {
    /// The root node. In the C# version PolyTree inherits from PolyNode, so the root
    /// represents the outer (hidden) polygon.
    pub root: PolyNode,
    /// A list of all nodes in the tree.
    pub all_polys: Vec<PolyNode>,
}

impl PolyTree {
    /// Constructs a new, empty PolyTree.
    pub fn new() -> Self {
        Self {
            root: PolyNode::new(),
            all_polys: Vec::new(),
        }
    }

    /// Clears the PolyTree.
    ///
    /// This method removes all child nodes from the root and clears the internal list.
    pub fn clear(&mut self) {
        self.root.childs.clear();
        self.all_polys.clear();
    }

    /// Returns the total number of nodes in the tree.
    ///
    /// In the C# version, if there are nodes and the first child of the root differs
    /// from the first element in the internal list, one is subtracted.
    /// This implementation simply returns the number of nodes currently stored.
    pub fn total(&self) -> usize {
        self.all_polys.len()
    }

    /// Returns the first node in a traversal.
    ///
    /// In the C# version, GetFirst() returns the first child of the root if available.
    pub fn get_first(&self) -> Option<&PolyNode> {
        if !self.root.childs.is_empty() {
            Some(&self.root.childs[0])
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Enums corresponding to Fill types, Poly types, etc.
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipType {
    Intersection,
    Union,
    Difference,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyType {
    Subject,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyFillType {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Square,
    Round,
    Miter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndType {
    ClosedPolygon,
    ClosedLine,
    OpenButt,
    OpenSquare,
    OpenRound,
}

// Internal enums

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    RightToLeft,
    LeftToRight,
}

////////////////////////////////////////////////////////////////////////////////
// Main Clipper struct with method stubs.
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Clipper {
    pub base: ClipperBase,
    pub clip_type: ClipType,
    pub maxima: Option<Box<Maxima>>,
    pub sorted_edges: Option<Rc<RefCell<TEdge>>>,
    pub intersect_list: Vec<IntersectNode>,
    pub joins: Vec<Join>,
    pub ghost_joins: Vec<Join>,
    pub execute_locked: bool,
    pub clip_fill_type: PolyFillType,
    pub subj_fill_type: PolyFillType,
    pub using_poly_tree: bool,
    // Options passed at construction
    pub reverse_solution: bool,
    pub strictly_simple: bool,
}

impl Clipper {
    /// Creates a new Clipper instance.
    pub fn new(init_options: i32) -> Self {
        // Options bits (from C#):
        // ioReverseSolution = 1, ioStrictlySimple = 2, ioPreserveCollinear = 4.
        let reverse_solution = (init_options & 1) != 0;
        let strictly_simple = (init_options & 2) != 0;
        // Note: PreserveCollinear is actually a property on ClipperBase
        let mut base = ClipperBase::new();
        base.preserve_collinear = (init_options & 4) != 0;
        Self {
            base,
            clip_type: ClipType::Intersection, // default – will be set in Execute
            maxima: None,
            sorted_edges: None,
            intersect_list: Vec::new(),
            joins: Vec::new(),
            ghost_joins: Vec::new(),
            execute_locked: false,
            clip_fill_type: PolyFillType::EvenOdd, // default fill type
            subj_fill_type: PolyFillType::EvenOdd,
            using_poly_tree: false,
            reverse_solution,
            strictly_simple,
        }
    }

    /// Returns true if the solution polygons should be reversed.
    pub fn reverse_solution(&self) -> bool {
        self.reverse_solution
    }

    /// Returns true if the output will be processed in strictly simple mode.
    pub fn strictly_simple(&self) -> bool {
        self.strictly_simple
    }

    /// Executes the clipping operation and outputs the solution as a set of paths.
    pub fn execute(
        &mut self,
        clip_type: ClipType,
        solution: &mut Paths,
        fill_type: PolyFillType,
    ) -> bool {
        self.execute_with_fill_types(clip_type, solution, fill_type, fill_type)
    }

    /// Executes the clipping operation and outputs the solution in a PolyTree.
    pub fn execute_poly_tree(
        &mut self,
        clip_type: ClipType,
        polytree: &mut PolyTree,
        fill_type: PolyFillType,
    ) -> bool {
        self.execute_poly_tree_with_fill_types(clip_type, polytree, fill_type, fill_type)
    }

    /// Executes the clipping operation with specified subject and clip fill types.
    pub fn execute_with_fill_types(
        &mut self,
        clip_type: ClipType,
        solution: &mut Paths,
        subj_fill_type: PolyFillType,
        clip_fill_type: PolyFillType,
    ) -> bool {
        if self.execute_locked {
            return false;
        }
        if self.base.has_open_paths {
            panic!("Error: PolyTree struct is needed for open path clipping.");
        }

        self.execute_locked = true;
        solution.clear();
        self.subj_fill_type = subj_fill_type;
        self.clip_fill_type = clip_fill_type;
        self.clip_type = clip_type;
        self.using_poly_tree = false;
        let succeeded;
        {
            succeeded = self.execute_internal();
            // Build the return polygons
            if succeeded {
                self.build_result(solution);
            }
        }
        self.dispose_all_poly_pts();
        self.execute_locked = false;
        succeeded
    }

    /// Executes the clipping operation with specified subject and clip fill types, outputting to a PolyTree.
    pub fn execute_poly_tree_with_fill_types(
        &mut self,
        clip_type: ClipType,
        polytree: &mut PolyTree,
        subj_fill_type: PolyFillType,
        clip_fill_type: PolyFillType,
    ) -> bool {
        if self.execute_locked {
            return false;
        }
        self.execute_locked = true;
        self.subj_fill_type = subj_fill_type;
        self.clip_fill_type = clip_fill_type;
        self.clip_type = clip_type;
        self.using_poly_tree = true;
        let succeeded;
        {
            succeeded = self.execute_internal();
            // Build the return polygons
            if succeeded {
                self.build_result_poly_tree(polytree);
            }
        }
        self.dispose_all_poly_pts();
        self.execute_locked = false;
        succeeded
    }

    /// Executes the core algorithm.
    ///
    /// This function processes scanbeams, handles intersections, and updates the active edge list.
    fn execute_internal(&mut self) -> bool {
        // Reset state.
        self.base.reset();
        self.sorted_edges = None;
        self.maxima = None;

        let mut bot_y: CInt;
        let mut top_y: CInt;

        if !self.base.pop_scanbeam().map_or(false, |y| {
            bot_y = y;
            true
        }) {
            return false;
        }

        self.insert_local_minima_into_ael(bot_y);

        while self.base.pop_scanbeam().map_or(false, |y| {
            top_y = y;
            true
        }) || self.base.local_minima_pending() {
            self.process_horizontals();
            self.ghost_joins.clear();
            if !self.process_intersections(top_y) {
                return false;
            }
            self.process_edges_at_top_of_scanbeam(top_y);
            bot_y = top_y;
            self.insert_local_minima_into_ael(bot_y);
        }

        // Fix orientations
        for out_rec in &mut self.base.poly_outs {
            if out_rec.pts.is_none() || out_rec.is_open {
                continue;
            }
            if (out_rec.is_hole ^ self.reverse_solution) == (self.area(out_rec) > 0.0) {
                self.reverse_poly_pt_links(out_rec.pts.as_ref().unwrap());
            }
        }

        self.join_common_edges();

        for out_rec in &mut self.base.poly_outs {
            if out_rec.pts.is_none() {
                continue;
            } else if out_rec.is_open {
                self.fixup_out_polyline(out_rec);
            } else {
                self.fixup_out_polygon(out_rec);
            }
        }

        if self.strictly_simple {
            self.do_simple_polygons();
        }

        true
    }

    /// Processes edges at the top of the scanbeam.
    fn process_edges_at_top_of_scanbeam(&mut self, top_y: CInt) {
        let mut e = self.base.active_edges.clone();
        while let Some(ref edge) = e {
            let is_maxima_edge = self.is_maxima(edge, top_y);

            if is_maxima_edge {
                let e_max_pair = self.get_maxima_pair_ex(edge);
                let is_maxima_edge = e_max_pair.is_none() || !ClipperBase::is_horizontal(&e_max_pair.unwrap().borrow());

                if is_maxima_edge {
                    if self.strictly_simple {
                        self.insert_maxima(edge.borrow().top.x);
                    }
                    let e_prev = edge.borrow().prev_in_ael.clone();
                    self.do_maxima(edge);
                    e = if let Some(ref e_prev) = e_prev {
                        e_prev.borrow().next_in_ael.clone()
                    } else {
                        self.base.active_edges.clone()
                    };
                    continue;
                }
            }

            if self.is_intermediate(edge, top_y) && ClipperBase::is_horizontal(&edge.borrow().next_in_lml.as_ref().unwrap().borrow()) {
                self.base.update_edge_into_ael(edge);
                if edge.borrow().out_idx >= 0 {
                    self.add_out_pt(edge, edge.borrow().bot);
                }
                self.add_edge_to_sel(edge);
            } else {
                edge.borrow_mut().curr.x = self.top_x(edge, top_y);
                edge.borrow_mut().curr.y = top_y;
            }

            if self.strictly_simple {
                let e_prev = edge.borrow().prev_in_ael.clone();
                if edge.borrow().out_idx >= 0 && edge.borrow().wind_delta != 0 && e_prev.is_some() && e_prev.as_ref().unwrap().borrow().out_idx >= 0 && e_prev.as_ref().unwrap().borrow().curr.x == edge.borrow().curr.x && e_prev.as_ref().unwrap().borrow().wind_delta != 0 {
                    let ip = IntPoint::new(edge.borrow().curr.x, edge.borrow().curr.y);
                    let op = self.add_out_pt(e_prev.as_ref().unwrap(), ip);
                    let op2 = self.add_out_pt(edge, ip);
                    self.add_join(op, op2, ip);
                }
            }

            e = edge.borrow().next_in_ael.clone();
        }

        self.process_horizontals();
        self.maxima = None;

        e = self.base.active_edges.clone();
        while let Some(ref edge) = e {
            if self.is_intermediate(edge, top_y) {
                let mut op = None;
                if edge.borrow().out_idx >= 0 {
                    op = Some(self.add_out_pt(edge, edge.borrow().top));
                }
                self.base.update_edge_into_ael(edge);

                let e_prev = edge.borrow().prev_in_ael.clone();
                let e_next = edge.borrow().next_in_ael.clone();
                if let Some(ref e_prev) = e_prev {
                    if e_prev.borrow().curr.x == edge.borrow().bot.x && e_prev.borrow().curr.y == edge.borrow().bot.y && op.is_some() && e_prev.borrow().out_idx >= 0 && e_prev.borrow().curr.y > e_prev.borrow().top.y && ClipperBase::slopes_equal(&edge.borrow(), &e_prev.borrow(), self.base.use_full_range) && edge.borrow().wind_delta != 0 && e_prev.borrow().wind_delta != 0 {
                        let op2 = self.add_out_pt(e_prev, edge.borrow().bot);
                        self.add_join(op.unwrap(), op2, edge.borrow().top);
                    }
                } else if let Some(ref e_next) = e_next {
                    if e_next.borrow().curr.x == edge.borrow().bot.x && e_next.borrow().curr.y == edge.borrow().bot.y && op.is_some() && e_next.borrow().out_idx >= 0 && e_next.borrow().curr.y > e_next.borrow().top.y && ClipperBase::slopes_equal(&edge.borrow(), &e_next.borrow(), self.base.use_full_range) && edge.borrow().wind_delta != 0 && e_next.borrow().wind_delta != 0 {
                        let op2 = self.add_out_pt(e_next, edge.borrow().bot);
                        self.add_join(op.unwrap(), op2, edge.borrow().top);
                    }
                }
            }
            e = edge.borrow().next_in_ael.clone();
        }
    }

    /// Builds the final solution paths from the internal OutRec structures.
    fn build_result(&mut self, solution: &mut Paths) {
        // Clear the solution and fill it based on the poly_outs stored in base.
        solution.clear();
        // Iterate through each OutRec and extract the polygon (Path)
        // This is a simplified placeholder implementation.
        for out_rec in &self.base.poly_outs {
            if let Some(ref out_pts) = out_rec.pts {
                // A helper function (to be implemented) counts points and builds a Path.
                let path = Self::extract_path_from_outrec(out_pts.borrow().clone());
                if !path.is_empty() {
                    solution.push(path);
                }
            }
        }
    }

    /// Builds the PolyTree output from the internal OutRec structures.
    fn build_result_poly_tree(&mut self, polytree: &mut PolyTree) {
        polytree.clear();

        // Add each output polygon/contour to polytree
        polytree.all_polys.reserve(self.base.poly_outs.len());
        for out_rec in &self.base.poly_outs {
            let cnt = Self::point_count(out_rec.pts.as_ref().unwrap());
            if (out_rec.is_open && cnt < 2) || (!out_rec.is_open && cnt < 3) {
                continue;
            }
            self.fix_hole_linkage(out_rec);
            let mut pn = PolyNode::new();
            polytree.all_polys.push(pn.clone());
            out_rec.poly_node = Some(pn.clone());
            pn.polygon.reserve(cnt);
            let mut op = out_rec.pts.as_ref().unwrap().borrow().prev.as_ref().unwrap().clone();
            for _ in 0..cnt {
                pn.polygon.push(op.borrow().pt);
                op = op.borrow().prev.as_ref().unwrap().clone();
            }
        }

        // Fixup PolyNode links etc.
        polytree.root.childs.reserve(self.base.poly_outs.len());
        for out_rec in &self.base.poly_outs {
            if let Some(ref poly_node) = out_rec.poly_node {
                if out_rec.is_open {
                    poly_node.is_open = true;
                    polytree.root.add_child(poly_node.clone());
                } else if let Some(ref first_left) = out_rec.first_left {
                    if let Some(ref first_left_poly_node) = first_left.borrow().poly_node {
                        first_left_poly_node.add_child(poly_node.clone());
                    } else {
                        polytree.root.add_child(poly_node.clone());
                    }
                } else {
                    polytree.root.add_child(poly_node.clone());
                }
            }
        }
    }

    /// Disposes internal OutRec point lists.
    fn dispose_all_poly_pts(&mut self) {
        for i in 0..self.base.poly_outs.len() {
            self.base.dispose_out_rec(i);
        }
        self.base.poly_outs.clear();
    }

    /// Adds a join between two output points with an offset.
    fn add_join(&mut self, op1: &OutPt, op2: &OutPt, off_pt: IntPoint) {
        let join = Join {
            out_pt1: Some(Rc::new(RefCell::new(op1.clone()))),
            out_pt2: Some(Rc::new(RefCell::new(op2.clone()))),
            off_pt,
        };
        self.joins.push(join);
    }

    /// Adds a ghost join for horizontal edges.
    fn add_ghost_join(&mut self, op: &OutPt, off_pt: IntPoint) {
        let join = Join {
            out_pt1: Some(Rc::new(RefCell::new(op.clone()))),
            out_pt2: None,
            off_pt,
        };
        self.ghost_joins.push(join);
    }

    /// Inserts a maxima value (new maximal x) into the maxima list.
    fn insert_maxima(&mut self, x: CInt) {
        // Double-linked list: sorted ascending, ignoring duplicates.
        let new_max = Box::new(Maxima {
            x,
            next: None,
            prev: None,
        });

        if self.maxima.is_none() {
            self.maxima = Some(new_max);
        } else if x < self.maxima.as_ref().unwrap().x {
            new_max.next = self.maxima.take();
            self.maxima.as_mut().unwrap().prev = Some(new_max);
            self.maxima = Some(new_max);
        } else {
            let mut m = self.maxima.as_mut().unwrap();
            while let Some(ref next) = m.next {
                if x < next.x {
                    break;
                }
                m = m.next.as_mut().unwrap();
            }
            if x == m.x {
                return; // Ignore duplicates
            }
            // Insert new_max between m and m.next
            new_max.next = m.next.take();
            new_max.prev = Some(Box::new(m.clone()));
            if let Some(ref mut next) = new_max.next {
                next.prev = Some(new_max.clone());
            }
            m.next = Some(new_max);
        }
    }

    /// Fixes the hole linkage for an OutRec.
    fn fix_hole_linkage(&mut self, out_rec: &mut OutRec) {
        // Skip if an outermost polygon or already points to the correct FirstLeft.
        if out_rec.first_left.is_none() || 
           (out_rec.is_hole != out_rec.first_left.as_ref().unwrap().borrow().is_hole &&
            out_rec.first_left.as_ref().unwrap().borrow().pts.is_some()) {
            return;
        }

        let mut orfl = out_rec.first_left.clone();
        while let Some(ref orfl_ref) = orfl {
            if orfl_ref.borrow().is_hole != out_rec.is_hole && orfl_ref.borrow().pts.is_some() {
                break;
            }
            orfl = orfl_ref.borrow().first_left.clone();
        }
        out_rec.first_left = orfl;
    }

    /// Inserts local minima into the active edge list.
    fn insert_local_minima_into_ael(&mut self, bot_y: CInt) {
        while let Some(lm) = self.base.pop_local_minima(bot_y) {
            let lb = lm.left_bound.clone();
            let rb = lm.right_bound.clone();

            let mut op1: Option<Rc<RefCell<OutPt>>> = None;
            if lb.is_none() {
                let rb = rb.unwrap();
                self.insert_edge_into_ael(&rb, None);
                self.set_winding_count(&rb);
                if self.is_contributing(&rb) {
                    op1 = Some(self.add_out_pt(&rb, rb.borrow().bot));
                }
            } else if rb.is_none() {
                let lb = lb.unwrap();
                self.insert_edge_into_ael(&lb, None);
                self.set_winding_count(&lb);
                if self.is_contributing(&lb) {
                    op1 = Some(self.add_out_pt(&lb, lb.borrow().bot));
                }
                self.base.insert_scanbeam(lb.borrow().top.y);
            } else {
                let lb = lb.unwrap();
                let rb = rb.unwrap();
                self.insert_edge_into_ael(&lb, None);
                self.insert_edge_into_ael(&rb, Some(&lb));
                self.set_winding_count(&lb);
                rb.borrow_mut().wind_cnt = lb.borrow().wind_cnt;
                rb.borrow_mut().wind_cnt2 = lb.borrow().wind_cnt2;
                if self.is_contributing(&lb) {
                    op1 = Some(self.add_local_min_poly(&lb, &rb, lb.borrow().bot));
                }
                self.base.insert_scanbeam(lb.borrow().top.y);
            }

            if let Some(rb) = rb {
                if self.base.is_horizontal(&rb.borrow()) {
                    if rb.borrow().next_in_lml.is_some() {
                        self.base.insert_scanbeam(rb.borrow().next_in_lml.as_ref().unwrap().borrow().top.y);
                    }
                    self.add_edge_to_sel(&rb);
                } else {
                    self.base.insert_scanbeam(rb.borrow().top.y);
                }
            }

            if lb.is_none() || rb.is_none() {
                continue;
            }

            let lb = lb.unwrap();
            let rb = rb.unwrap();

            if let Some(op1) = op1 {
                if self.base.is_horizontal(&rb.borrow()) && !self.ghost_joins.is_empty() && rb.borrow().wind_delta != 0 {
                    for j in &self.ghost_joins {
                        if self.horz_segments_overlap(j.out_pt1.as_ref().unwrap().borrow().pt.x, j.off_pt.x, rb.borrow().bot.x, rb.borrow().top.x) {
                            self.add_join(j.out_pt1.as_ref().unwrap(), &op1, j.off_pt);
                        }
                    }
                }

                if lb.borrow().out_idx >= 0 && lb.borrow().prev_in_ael.is_some() && lb.borrow().prev_in_ael.as_ref().unwrap().borrow().curr.x == lb.borrow().bot.x && lb.borrow().prev_in_ael.as_ref().unwrap().borrow().out_idx >= 0 && self.base.slopes_equal(&lb.borrow().prev_in_ael.as_ref().unwrap().borrow(), &lb.borrow(), self.base.use_full_range) && lb.borrow().wind_delta != 0 && lb.borrow().prev_in_ael.as_ref().unwrap().borrow().wind_delta != 0 {
                    let op2 = self.add_out_pt(&lb.borrow().prev_in_ael.as_ref().unwrap(), lb.borrow().bot);
                    self.add_join(&op1, &op2, lb.borrow().top);
                }

                if !Rc::ptr_eq(&lb.borrow().next_in_ael, &Some(Rc::new(RefCell::new(rb.borrow().clone())))) {
                    if rb.borrow().out_idx >= 0 && rb.borrow().prev_in_ael.as_ref().unwrap().borrow().out_idx >= 0 && self.base.slopes_equal(&rb.borrow().prev_in_ael.as_ref().unwrap().borrow(), &rb.borrow(), self.base.use_full_range) && rb.borrow().wind_delta != 0 && rb.borrow().prev_in_ael.as_ref().unwrap().borrow().wind_delta != 0 {
                        let op2 = self.add_out_pt(&rb.borrow().prev_in_ael.as_ref().unwrap(), rb.borrow().bot);
                        self.add_join(&op1, &op2, rb.borrow().top);
                    }

                    let mut e = lb.borrow().next_in_ael.clone();
                    while let Some(ref e_ref) = e {
                        if Rc::ptr_eq(e_ref, &rb) {
                            break;
                        }
                        self.intersect_edges(&rb, e_ref, lb.borrow().curr);
                        e = e_ref.borrow().next_in_ael.clone();
                    }
                }
            }
        }
    }

    /// Inserts an edge into the active edge list.
    fn insert_edge_into_ael(&mut self, edge: &Rc<RefCell<TEdge>>, start_edge: Option<&Rc<RefCell<TEdge>>>) {
        if self.base.active_edges.is_none() {
            edge.borrow_mut().prev_in_ael = None;
            edge.borrow_mut().next_in_ael = None;
            self.base.active_edges = Some(edge.clone());
        } else if start_edge.is_none() && self.e2_inserts_before_e1(self.base.active_edges.as_ref().unwrap(), edge) {
            edge.borrow_mut().prev_in_ael = None;
            edge.borrow_mut().next_in_ael = self.base.active_edges.clone();
            self.base.active_edges.as_ref().unwrap().borrow_mut().prev_in_ael = Some(edge.clone());
            self.base.active_edges = Some(edge.clone());
        } else {
            let mut start_edge = start_edge.unwrap_or(self.base.active_edges.as_ref().unwrap()).clone();
            while start_edge.borrow().next_in_ael.is_some() && !self.e2_inserts_before_e1(start_edge.borrow().next_in_ael.as_ref().unwrap(), edge) {
                start_edge = start_edge.borrow().next_in_ael.as_ref().unwrap().clone();
            }
            edge.borrow_mut().next_in_ael = start_edge.borrow().next_in_ael.clone();
            if start_edge.borrow().next_in_ael.is_some() {
                start_edge.borrow().next_in_ael.as_ref().unwrap().borrow_mut().prev_in_ael = Some(edge.clone());
            }
            edge.borrow_mut().prev_in_ael = Some(start_edge.clone());
            start_edge.borrow_mut().next_in_ael = Some(edge.clone());
        }
    }

    /// Determines if edge e2 should be inserted before edge e1 in the active edge list.
    fn e2_inserts_before_e1(&self, e1: &Rc<RefCell<TEdge>>, e2: &Rc<RefCell<TEdge>>) -> bool {
        if e2.borrow().curr.x == e1.borrow().curr.x {
            if e2.borrow().top.y > e1.borrow().top.y {
                e2.borrow().top.x < Clipper::top_x(e1, e2.borrow().top.y)
            } else {
                e1.borrow().top.x > Clipper::top_x(e2, e1.borrow().top.y)
            }
        } else {
            e2.borrow().curr.x < e1.borrow().curr.x
        }
    }

    /// Calculates the top X coordinate of an edge at a given Y coordinate.
    fn top_x(edge: &Rc<RefCell<TEdge>>, current_y: CInt) -> CInt {
        if current_y == edge.borrow().top.y {
            edge.borrow().top.x
        } else {
            edge.borrow().bot.x + edge.borrow().dx as CInt * (current_y - edge.borrow().bot.y)
        }
    }

    /// Determines if the fill type for the edge is EvenOdd.
    fn is_even_odd_fill_type(&self, edge: &TEdge) -> bool {
        if edge.poly_typ == PolyType::Subject {
            self.subj_fill_type == PolyFillType::EvenOdd
        } else {
            self.clip_fill_type == PolyFillType::EvenOdd
        }
    }

    /// Determines if the alternate fill type for the edge is EvenOdd.
    fn is_even_odd_alt_fill_type(&self, edge: &TEdge) -> bool {
        if edge.poly_typ == PolyType::Subject {
            self.clip_fill_type == PolyFillType::EvenOdd
        } else {
            self.subj_fill_type == PolyFillType::EvenOdd
        }
    }

    /// Determines if the edge is contributing to the final solution.
    fn is_contributing(&self, edge: &TEdge) -> bool {
        let (pft, pft2) = if edge.poly_typ == PolyType::Subject {
            (self.subj_fill_type, self.clip_fill_type)
        } else {
            (self.clip_fill_type, self.subj_fill_type)
        };

        match pft {
            PolyFillType::EvenOdd => {
                if edge.wind_delta == 0 && edge.wind_cnt != 1 {
                    return false;
                }
            }
            PolyFillType::NonZero => {
                if edge.wind_cnt.abs() != 1 {
                    return false;
                }
            }
            PolyFillType::Positive => {
                if edge.wind_cnt != 1 {
                    return false;
                }
            }
            PolyFillType::Negative => {
                if edge.wind_cnt != -1 {
                    return false;
                }
            }
        }

        match self.clip_type {
            ClipType::Intersection => match pft2 {
                PolyFillType::EvenOdd | PolyFillType::NonZero => edge.wind_cnt2 != 0,
                PolyFillType::Positive => edge.wind_cnt2 > 0,
                PolyFillType::Negative => edge.wind_cnt2 < 0,
            },
            ClipType::Union => match pft2 {
                PolyFillType::EvenOdd | PolyFillType::NonZero => edge.wind_cnt2 == 0,
                PolyFillType::Positive => edge.wind_cnt2 <= 0,
                PolyFillType::Negative => edge.wind_cnt2 >= 0,
            },
            ClipType::Difference => {
                if edge.poly_typ == PolyType::Subject {
                    match pft2 {
                        PolyFillType::EvenOdd | PolyFillType::NonZero => edge.wind_cnt2 == 0,
                        PolyFillType::Positive => edge.wind_cnt2 <= 0,
                        PolyFillType::Negative => edge.wind_cnt2 >= 0,
                    }
                } else {
                    match pft2 {
                        PolyFillType::EvenOdd | PolyFillType::NonZero => edge.wind_cnt2 != 0,
                        PolyFillType::Positive => edge.wind_cnt2 > 0,
                        PolyFillType::Negative => edge.wind_cnt2 < 0,
                    }
                }
            }
            ClipType::Xor => {
                if edge.wind_delta == 0 {
                    match pft2 {
                        PolyFillType::EvenOdd | PolyFillType::NonZero => edge.wind_cnt2 == 0,
                        PolyFillType::Positive => edge.wind_cnt2 <= 0,
                        PolyFillType::Negative => edge.wind_cnt2 >= 0,
                    }
                } else {
                    true
                }
            }
        }
    }

    /// Sets the winding count for the edge.
    fn set_winding_count(&mut self, edge: &Rc<RefCell<TEdge>>) {
        let mut e = edge.borrow().prev_in_ael.clone();
        // Find the edge of the same polytype that immediately precedes 'edge' in AEL
        while let Some(ref e_ref) = e {
            if e_ref.borrow().poly_typ == edge.borrow().poly_typ && e_ref.borrow().wind_delta != 0 {
                break;
            }
            e = e_ref.borrow().prev_in_ael.clone();
        }

        if e.is_none() {
            let pft = if edge.borrow().poly_typ == PolyType::Subject {
                self.subj_fill_type
            } else {
                self.clip_fill_type
            };
            if edge.borrow().wind_delta == 0 {
                edge.borrow_mut().wind_cnt = if pft == PolyFillType::Negative { -1 } else { 1 };
            } else {
                edge.borrow_mut().wind_cnt = edge.borrow().wind_delta;
            }
            edge.borrow_mut().wind_cnt2 = 0;
            e = self.base.active_edges.clone(); // Get ready to calculate WindCnt2
        } else if edge.borrow().wind_delta == 0 && self.clip_type != ClipType::Union {
            edge.borrow_mut().wind_cnt = 1;
            edge.borrow_mut().wind_cnt2 = e.as_ref().unwrap().borrow().wind_cnt2;
            e = e.unwrap().borrow().next_in_ael.clone(); // Get ready to calculate WindCnt2
        } else if self.is_even_odd_fill_type(&edge.borrow()) {
            // EvenOdd filling
            if edge.borrow().wind_delta == 0 {
                // Are we inside a subj polygon?
                let mut inside = true;
                let mut e2 = e.as_ref().unwrap().borrow().prev_in_ael.clone();
                while let Some(ref e2_ref) = e2 {
                    if e2_ref.borrow().poly_typ == e.as_ref().unwrap().borrow().poly_typ && e2_ref.borrow().wind_delta != 0 {
                        inside = !inside;
                    }
                    e2 = e2_ref.borrow().prev_in_ael.clone();
                }
                edge.borrow_mut().wind_cnt = if inside { 0 } else { 1 };
            } else {
                edge.borrow_mut().wind_cnt = edge.borrow().wind_delta;
            }
            edge.borrow_mut().wind_cnt2 = e.as_ref().unwrap().borrow().wind_cnt2;
            e = e.unwrap().borrow().next_in_ael.clone(); // Get ready to calculate WindCnt2
        } else {
            // NonZero, Positive, or Negative filling
            if e.as_ref().unwrap().borrow().wind_cnt * e.as_ref().unwrap().borrow().wind_delta < 0 {
                // Previous edge is 'decreasing' WindCount (WC) toward zero
                // so we're outside the previous polygon
                if e.as_ref().unwrap().borrow().wind_cnt.abs() > 1 {
                    // Outside previous poly but still inside another
                    // When reversing direction of previous poly use the same WC
                    if e.as_ref().unwrap().borrow().wind_delta * edge.borrow().wind_delta < 0 {
                        edge.borrow_mut().wind_cnt = e.as_ref().unwrap().borrow().wind_cnt;
                    } else {
                        // Otherwise continue to 'decrease' WC
                        edge.borrow_mut().wind_cnt = e.as_ref().unwrap().borrow().wind_cnt + edge.borrow().wind_delta;
                    }
                } else {
                    // Now outside all polys of same polytype so set own WC
                    edge.borrow_mut().wind_cnt = if edge.borrow().wind_delta == 0 { 1 } else { edge.borrow().wind_delta };
                }
            } else {
                // Previous edge is 'increasing' WindCount (WC) away from zero
                // so we're inside the previous polygon
                if edge.borrow().wind_delta == 0 {
                    edge.borrow_mut().wind_cnt = if e.as_ref().unwrap().borrow().wind_cnt < 0 {
                        e.as_ref().unwrap().borrow().wind_cnt - 1
                    } else {
                        e.as_ref().unwrap().borrow().wind_cnt + 1
                    };
                } else if e.as_ref().unwrap().borrow().wind_delta * edge.borrow().wind_delta < 0 {
                    // If wind direction is reversing previous then use same WC
                    edge.borrow_mut().wind_cnt = e.as_ref().unwrap().borrow().wind_cnt;
                } else {
                    // Otherwise add to WC
                    edge.borrow_mut().wind_cnt = e.as_ref().unwrap().borrow().wind_cnt + edge.borrow().wind_delta;
                }
            }
            edge.borrow_mut().wind_cnt2 = e.as_ref().unwrap().borrow().wind_cnt2;
            e = e.unwrap().borrow().next_in_ael.clone(); // Get ready to calculate WindCnt2
        }

        // Calculate WindCnt2
        if self.is_even_odd_alt_fill_type(&edge.borrow()) {
            while let Some(ref e_ref) = e {
                if e_ref.borrow().poly_typ != edge.borrow().poly_typ && e_ref.borrow().wind_delta != 0 {
                    edge.borrow_mut().wind_cnt2 = if edge.borrow_mut().wind_cnt2 == 0 { 1 } else { 0 };
                }
                e = e_ref.borrow().next_in_ael.clone();
            }
        } else {
            while let Some(ref e_ref) = e {
                if e_ref.borrow().poly_typ != edge.borrow().poly_typ && e_ref.borrow().wind_delta != 0 {
                    edge.borrow_mut().wind_cnt2 += e_ref.borrow().wind_delta;
                }
                e = e_ref.borrow().next_in_ael.clone();
            }
        }
    }

    /// Adds an edge to the sorted edge list (SEL).
    fn add_edge_to_sel(&mut self, edge: &Rc<RefCell<TEdge>>) {
        // SEL pointers in TEdge are used to build transient lists of horizontal edges.
        // However, since we don't need to worry about processing order, all additions
        // are made to the front of the list.
        if self.sorted_edges.is_none() {
            self.sorted_edges = Some(edge.clone());
            edge.borrow_mut().prev_in_sel = None;
            edge.borrow_mut().next_in_sel = None;
        } else {
            edge.borrow_mut().next_in_sel = self.sorted_edges.clone();
            edge.borrow_mut().prev_in_sel = None;
            self.sorted_edges.as_ref().unwrap().borrow_mut().prev_in_sel = Some(edge.clone());
            self.sorted_edges = Some(edge.clone());
        }
    }

    /// Pops an edge from the front of the sorted edge list (SEL).
    fn pop_edge_from_sel(&mut self) -> Option<Rc<RefCell<TEdge>>> {
        // Pop edge from front of SEL (i.e., SEL is a FILO list).
        if let Some(e) = self.sorted_edges.take() {
            let old_e = e.clone();
            self.sorted_edges = e.borrow().next_in_sel.clone();
            if let Some(ref sorted_edges) = self.sorted_edges {
                sorted_edges.borrow_mut().prev_in_sel = None;
            }
            old_e.borrow_mut().next_in_sel = None;
            old_e.borrow_mut().prev_in_sel = None;
            Some(old_e)
        } else {
            None
        }
    }

    /// Copies the active edge list (AEL) to the sorted edge list (SEL).
    fn copy_ael_to_sel(&mut self) {
        let mut e = self.base.active_edges.clone();
        self.sorted_edges = e.clone();
        while let Some(ref edge) = e {
            edge.borrow_mut().prev_in_sel = edge.borrow().prev_in_ael.clone();
            edge.borrow_mut().next_in_sel = edge.borrow().next_in_ael.clone();
            e = edge.borrow().next_in_ael.clone();
        }
    }

    /// Swaps positions of two edges in the sorted edge list (SEL).
    fn swap_positions_in_sel(&mut self, edge1: &Rc<RefCell<TEdge>>, edge2: &Rc<RefCell<TEdge>>) {
        if edge1.borrow().next_in_sel.is_none() && edge1.borrow().prev_in_sel.is_none() {
            return;
        }
        if edge2.borrow().next_in_sel.is_none() && edge2.borrow().prev_in_sel.is_none() {
            return;
        }

        if Rc::ptr_eq(&edge1.borrow().next_in_sel, &Some(edge2.clone())) {
            let next = edge2.borrow().next_in_sel.clone();
            if let Some(ref next) = next {
                next.borrow_mut().prev_in_sel = Some(edge1.clone());
            }
            let prev = edge1.borrow().prev_in_sel.clone();
            if let Some(ref prev) = prev {
                prev.borrow_mut().next_in_sel = Some(edge2.clone());
            }
            edge2.borrow_mut().prev_in_sel = prev;
            edge2.borrow_mut().next_in_sel = Some(edge1.clone());
            edge1.borrow_mut().prev_in_sel = Some(edge2.clone());
            edge1.borrow_mut().next_in_sel = next;
        } else if Rc::ptr_eq(&edge2.borrow().next_in_sel, &Some(edge1.clone())) {
            let next = edge1.borrow().next_in_sel.clone();
            if let Some(ref next) = next {
                next.borrow_mut().prev_in_sel = Some(edge2.clone());
            }
            let prev = edge2.borrow().prev_in_sel.clone();
            if let Some(ref prev) = prev {
                prev.borrow_mut().next_in_sel = Some(edge1.clone());
            }
            edge1.borrow_mut().prev_in_sel = prev;
            edge1.borrow_mut().next_in_sel = Some(edge2.clone());
            edge2.borrow_mut().prev_in_sel = Some(edge1.clone());
            edge2.borrow_mut().next_in_sel = next;
        } else {
            let next = edge1.borrow().next_in_sel.clone();
            let prev = edge1.borrow().prev_in_sel.clone();
            edge1.borrow_mut().next_in_sel = edge2.borrow().next_in_sel.clone();
            if let Some(ref next) = edge1.borrow().next_in_sel {
                next.borrow_mut().prev_in_sel = Some(edge1.clone());
            }
            edge1.borrow_mut().prev_in_sel = edge2.borrow().prev_in_sel.clone();
            if let Some(ref prev) = edge1.borrow().prev_in_sel {
                prev.borrow_mut().next_in_sel = Some(edge1.clone());
            }
            edge2.borrow_mut().next_in_sel = next;
            if let Some(ref next) = edge2.borrow().next_in_sel {
                next.borrow_mut().prev_in_sel = Some(edge2.clone());
            }
            edge2.borrow_mut().prev_in_sel = prev;
            if let Some(ref prev) = edge2.borrow().prev_in_sel {
                prev.borrow_mut().next_in_sel = Some(edge2.clone());
            }
        }

        if edge1.borrow().prev_in_sel.is_none() {
            self.sorted_edges = Some(edge1.clone());
        } else if edge2.borrow().prev_in_sel.is_none() {
            self.sorted_edges = Some(edge2.clone());
        }
    }

    /// Adds a local maximum polygon.
    fn add_local_max_poly(&mut self, e1: &Rc<RefCell<TEdge>>, e2: &Rc<RefCell<TEdge>>, pt: IntPoint) {
        self.add_out_pt(e1, pt);
        if e2.borrow().wind_delta == 0 {
            self.add_out_pt(e2, pt);
        }
        if e1.borrow().out_idx == e2.borrow().out_idx {
            e1.borrow_mut().out_idx = UNASSIGNED;
            e2.borrow_mut().out_idx = UNASSIGNED;
        } else if e1.borrow().out_idx < e2.borrow().out_idx {
            self.append_polygon(e1, e2);
        } else {
            self.append_polygon(e2, e1);
        }
    }

    /// Adds a local minimum polygon.
    fn add_local_min_poly(&mut self, e1: &Rc<RefCell<TEdge>>, e2: &Rc<RefCell<TEdge>>, pt: IntPoint) -> Rc<RefCell<OutPt>> {
        let result;
        let e;
        let prev_e;
        if self.base.is_horizontal(&e2.borrow()) || e1.borrow().dx > e2.borrow().dx {
            result = self.add_out_pt(e1, pt);
            e2.borrow_mut().out_idx = e1.borrow().out_idx;
            e1.borrow_mut().side = EdgeSide::Left;
            e2.borrow_mut().side = EdgeSide::Right;
            e = e1.clone();
            if Rc::ptr_eq(&e.borrow().prev_in_ael, &e2) {
                prev_e = e2.borrow().prev_in_ael.clone();
            } else {
                prev_e = e.borrow().prev_in_ael.clone();
            }
        } else {
            result = self.add_out_pt(e2, pt);
            e1.borrow_mut().out_idx = e2.borrow().out_idx;
            e1.borrow_mut().side = EdgeSide::Right;
            e2.borrow_mut().side = EdgeSide::Left;
            e = e2.clone();
            if Rc::ptr_eq(&e.borrow().prev_in_ael, &e1) {
                prev_e = e1.borrow().prev_in_ael.clone();
            } else {
                prev_e = e.borrow().prev_in_ael.clone();
            }
        }

        if let Some(ref prev_e) = prev_e {
            if prev_e.borrow().out_idx >= 0 && prev_e.borrow().top.y < pt.y && e.borrow().top.y < pt.y {
                let x_prev = self.top_x(prev_e, pt.y);
                let x_e = self.top_x(&e, pt.y);
                if x_prev == x_e && e.borrow().wind_delta != 0 && prev_e.borrow().wind_delta != 0 && self.base.slopes_equal_points(IntPoint::new(x_prev, pt.y), prev_e.borrow().top, IntPoint::new(x_e, pt.y), e.borrow().top, self.base.use_full_range) {
                    let out_pt = self.add_out_pt(prev_e, pt);
                    self.add_join(&result, &out_pt, e.borrow().top);
                }
            }
        }
        result
    }

    /// Adds an output point to the polygon.
    fn add_out_pt(&mut self, e: &Rc<RefCell<TEdge>>, pt: IntPoint) -> Rc<RefCell<OutPt>> {
        if e.borrow().out_idx < 0 {
            let mut out_rec = self.base.create_out_rec();
            out_rec.is_open = e.borrow().wind_delta == 0;
            let new_op = Rc::new(RefCell::new(OutPt {
                idx: out_rec.idx,
                pt,
                next: None,
                prev: None,
            }));
            out_rec.pts = Some(new_op.clone());
            new_op.borrow_mut().next = Some(new_op.clone());
            new_op.borrow_mut().prev = Some(new_op.clone());
            if !out_rec.is_open {
                self.set_hole_state(e, &mut out_rec);
            }
            e.borrow_mut().out_idx = out_rec.idx;
            return new_op;
        } else {
            let out_rec = &self.base.poly_outs[e.borrow().out_idx as usize];
            let op = out_rec.pts.as_ref().unwrap();
            let to_front = e.borrow().side == EdgeSide::Left;
            if to_front && pt == op.borrow().pt {
                return op.clone();
            } else if !to_front && pt == op.borrow().prev.as_ref().unwrap().borrow().pt {
                return op.borrow().prev.as_ref().unwrap().clone();
            }

            let new_op = Rc::new(RefCell::new(OutPt {
                idx: out_rec.idx,
                pt,
                next: Some(op.clone()),
                prev: op.borrow().prev.clone(),
            }));
            new_op.borrow().prev.as_ref().unwrap().borrow_mut().next = Some(new_op.clone());
            op.borrow_mut().prev = Some(new_op.clone());
            if to_front {
                out_rec.pts = Some(new_op.clone());
            }
            return new_op;
        }
    }

    /// Gets the last output point of the edge.
    fn get_last_out_pt(&self, e: &Rc<RefCell<TEdge>>) -> Rc<RefCell<OutPt>> {
        let out_rec = &self.base.poly_outs[e.borrow().out_idx as usize];
        if e.borrow().side == EdgeSide::Left {
            out_rec.pts.as_ref().unwrap().clone()
        } else {
            out_rec.pts.as_ref().unwrap().borrow().prev.as_ref().unwrap().clone()
        }
    }

    /// Swaps two points.
    fn swap_points(pt1: &mut IntPoint, pt2: &mut IntPoint) {
        std::mem::swap(pt1, pt2);
    }

    /// Determines if two horizontal segments overlap.
    fn horz_segments_overlap(&self, seg1a: CInt, seg1b: CInt, seg2a: CInt, seg2b: CInt) -> bool {
        let (mut seg1a, mut seg1b) = (seg1a, seg1b);
        let (mut seg2a, mut seg2b) = (seg2a, seg2b);
        if seg1a > seg1b {
            ClipperBase::swap(&mut seg1a, &mut seg1b);
        }
        if seg2a > seg2b {
            ClipperBase::swap(&mut seg2a, &mut seg2b);
        }
        seg1a < seg2b && seg2a < seg1b
    }

    /// Sets the hole state for an output record.
    fn set_hole_state(&mut self, e: &Rc<RefCell<TEdge>>, out_rec: &mut OutRec) {
        let mut e2 = e.borrow().prev_in_ael.clone();
        let mut e_tmp: Option<Rc<RefCell<TEdge>>> = None;
        while let Some(ref e2_ref) = e2 {
            if e2_ref.borrow().out_idx >= 0 && e2_ref.borrow().wind_delta != 0 {
                if e_tmp.is_none() {
                    e_tmp = Some(e2_ref.clone());
                } else if e_tmp.as_ref().unwrap().borrow().out_idx == e2_ref.borrow().out_idx {
                    e_tmp = None; // paired
                }
            }
            e2 = e_ref.borrow().prev_in_ael.clone();
        }

        if let Some(e_tmp) = e_tmp {
            out_rec.first_left = Some(self.base.poly_outs[e_tmp.borrow().out_idx as usize].clone());
            out_rec.is_hole = !out_rec.first_left.as_ref().unwrap().borrow().is_hole;
        } else {
            out_rec.first_left = None;
            out_rec.is_hole = false;
        }
    }

    /// Calculates the slope (dx) between two points.
    fn get_dx(pt1: IntPoint, pt2: IntPoint) -> f64 {
        if pt1.y == pt2.y {
            HORIZONTAL
        } else {
            (pt2.x - pt1.x) as f64 / (pt2.y - pt1.y) as f64
        }
    }

    /// Determines if the first bottom point is the bottom-most point.
    fn first_is_bottom_pt(&self, btm_pt1: &Rc<RefCell<OutPt>>, btm_pt2: &Rc<RefCell<OutPt>>) -> bool {
        let mut p = btm_pt1.borrow().prev.clone().unwrap();
        while p.borrow().pt == btm_pt1.borrow().pt && !Rc::ptr_eq(&p, btm_pt1) {
            p = p.borrow().prev.clone().unwrap();
        }
        let dx1p = (Self::get_dx(btm_pt1.borrow().pt, p.borrow().pt)).abs();
        p = btm_pt1.borrow().next.clone().unwrap();
        while p.borrow().pt == btm_pt1.borrow().pt && !Rc::ptr_eq(&p, btm_pt1) {
            p = p.borrow().next.clone().unwrap();
        }
        let dx1n = (Self::get_dx(btm_pt1.borrow().pt, p.borrow().pt)).abs();

        p = btm_pt2.borrow().prev.clone().unwrap();
        while p.borrow().pt == btm_pt2.borrow().pt && !Rc::ptr_eq(&p, btm_pt2) {
            p = p.borrow().prev.clone().unwrap();
        }
        let dx2p = (Self::get_dx(btm_pt2.borrow().pt, p.borrow().pt)).abs();
        p = btm_pt2.borrow().next.clone().unwrap();
        while p.borrow().pt == btm_pt2.borrow().pt && !Rc::ptr_eq(&p, btm_pt2) {
            p = p.borrow().next.clone().unwrap();
        }
        let dx2n = (Self::get_dx(btm_pt2.borrow().pt, p.borrow().pt)).abs();

        if dx1p.max(dx1n) == dx2p.max(dx2n) && dx1p.min(dx1n) == dx2p.min(dx2n) {
            self.area(btm_pt1) > 0.0 // if otherwise identical use orientation
        } else {
            (dx1p >= dx2p && dx1p >= dx2n) || (dx1n >= dx2p && dx1n >= dx2n)
        }
    }

    /// Gets the bottom-most point of the polygon.
    fn get_bottom_pt(&self, pp: &Rc<RefCell<OutPt>>) -> Rc<RefCell<OutPt>> {
        let mut pp = pp.clone();
        let mut dups: Option<Rc<RefCell<OutPt>>> = None;
        let mut p = pp.borrow().next.clone().unwrap();
        while !Rc::ptr_eq(&p, &pp) {
            if p.borrow().pt.y > pp.borrow().pt.y {
                pp = p.clone();
                dups = None;
            } else if p.borrow().pt.y == pp.borrow().pt.y && p.borrow().pt.x <= pp.borrow().pt.x {
                if p.borrow().pt.x < pp.borrow().pt.x {
                    dups = None;
                    pp = p.clone();
                } else {
                    if !Rc::ptr_eq(&p.borrow().next.clone().unwrap(), &pp) && !Rc::ptr_eq(&p.borrow().prev.clone().unwrap(), &pp) {
                        dups = Some(p.clone());
                    }
                }
            }
            p = p.borrow().next.clone().unwrap();
        }
        if let Some(mut dups) = dups {
            // there appears to be at least 2 vertices at bottomPt so ...
            while !Rc::ptr_eq(&dups, &p) {
                if !self.first_is_bottom_pt(&p, &dups) {
                    pp = dups.clone();
                }
                dups = dups.borrow().next.clone().unwrap();
                while dups.borrow().pt != pp.borrow().pt {
                    dups = dups.borrow().next.clone().unwrap();
                }
            }
        }
        pp
    }

    /// Determines which OutRec has the lower bottom point.
    fn get_lowermost_rec(&mut self, out_rec1: &Rc<RefCell<OutRec>>, out_rec2: &Rc<RefCell<OutRec>>) -> Rc<RefCell<OutRec>> {
        if out_rec1.borrow().bottom_pt.is_none() {
            out_rec1.borrow_mut().bottom_pt = Some(self.get_bottom_pt(out_rec1.borrow().pts.as_ref().unwrap()));
        }
        if out_rec2.borrow().bottom_pt.is_none() {
            out_rec2.borrow_mut().bottom_pt = Some(self.get_bottom_pt(out_rec2.borrow().pts.as_ref().unwrap()));
        }
        let b_pt1 = out_rec1.borrow().bottom_pt.as_ref().unwrap();
        let b_pt2 = out_rec2.borrow().bottom_pt.as_ref().unwrap();
        if b_pt1.borrow().pt.y > b_pt2.borrow().pt.y {
            out_rec1.clone()
        } else if b_pt1.borrow().pt.y < b_pt2.borrow().pt.y {
            out_rec2.clone()
        } else if b_pt1.borrow().pt.x < b_pt2.borrow().pt.x {
            out_rec1.clone()
        } else if b_pt1.borrow().pt.x > b_pt2.borrow().pt.x {
            out_rec2.clone()
        } else if Rc::ptr_eq(&b_pt1.borrow().next.as_ref().unwrap(), b_pt1) {
            out_rec2.clone()
        } else if Rc::ptr_eq(&b_pt2.borrow().next.as_ref().unwrap(), b_pt2) {
            out_rec1.clone()
        } else if self.first_is_bottom_pt(b_pt1, b_pt2) {
            out_rec1.clone()
        } else {
            out_rec2.clone()
        }
    }

    /// Determines if OutRec1 is to the right of OutRec2.
    fn out_rec1_right_of_out_rec2(&self, mut out_rec1: &Rc<RefCell<OutRec>>, out_rec2: &Rc<RefCell<OutRec>>) -> bool {
        while let Some(ref first_left) = out_rec1.borrow().first_left {
            out_rec1 = first_left;
            if Rc::ptr_eq(out_rec1, out_rec2) {
                return true;
            }
        }
        false
    }

    /// Gets the OutRec for the given index.
    fn get_out_rec(&self, idx: i32) -> Rc<RefCell<OutRec>> {
        let mut out_rec = self.base.poly_outs[idx as usize].clone();
        while !Rc::ptr_eq(&out_rec, &self.base.poly_outs[out_rec.borrow().idx as usize]) {
            out_rec = self.base.poly_outs[out_rec.borrow().idx as usize].clone();
        }
        out_rec
    }

    /// Appends one polygon to another.
    fn append_polygon(&mut self, e1: &Rc<RefCell<TEdge>>, e2: &Rc<RefCell<TEdge>>) {
        let out_rec1 = self.get_out_rec(e1.borrow().out_idx);
        let out_rec2 = self.get_out_rec(e2.borrow().out_idx);

        let hole_state_rec = if self.out_rec1_right_of_out_rec2(&out_rec1, &out_rec2) {
            out_rec2.clone()
        } else if self.out_rec1_right_of_out_rec2(&out_rec2, &out_rec1) {
            out_rec1.clone()
        } else {
            self.get_lowermost_rec(&out_rec1, &out_rec2)
        };

        let p1_lft = out_rec1.borrow().pts.as_ref().unwrap().clone();
        let p1_rt = p1_lft.borrow().prev.as_ref().unwrap().clone();
        let p2_lft = out_rec2.borrow().pts.as_ref().unwrap().clone();
        let p2_rt = p2_lft.borrow().prev.as_ref().unwrap().clone();

        if e1.borrow().side == EdgeSide::Left {
            if e2.borrow().side == EdgeSide::Left {
                self.reverse_poly_pt_links(&p2_lft);
                p2_lft.borrow_mut().next = Some(p1_lft.clone());
                p1_lft.borrow_mut().prev = Some(p2_lft.clone());
                p1_rt.borrow_mut().next = Some(p2_rt.clone());
                p2_rt.borrow_mut().prev = Some(p1_rt.clone());
                out_rec1.borrow_mut().pts = Some(p2_rt.clone());
            } else {
                p2_rt.borrow_mut().next = Some(p1_lft.clone());
                p1_lft.borrow_mut().prev = Some(p2_rt.clone());
                p2_lft.borrow_mut().prev = Some(p1_rt.clone());
                p1_rt.borrow_mut().next = Some(p2_lft.clone());
                out_rec1.borrow_mut().pts = Some(p2_lft.clone());
            }
        } else {
            if e2.borrow().side == EdgeSide::Right {
                self.reverse_poly_pt_links(&p2_lft);
                p1_rt.borrow_mut().next = Some(p2_rt.clone());
                p2_rt.borrow_mut().prev = Some(p1_rt.clone());
                p2_lft.borrow_mut().next = Some(p1_lft.clone());
                p1_lft.borrow_mut().prev = Some(p2_lft.clone());
            } else {
                p1_rt.borrow_mut().next = Some(p2_lft.clone());
                p2_lft.borrow_mut().prev = Some(p1_rt.clone());
                p1_lft.borrow_mut().prev = Some(p2_rt.clone());
                p2_rt.borrow_mut().next = Some(p1_lft.clone());
            }
        }

        out_rec1.borrow_mut().bottom_pt = None;
        if Rc::ptr_eq(&hole_state_rec, &out_rec2) {
            if !Rc::ptr_eq(&out_rec2.borrow().first_left.as_ref().unwrap(), &out_rec1) {
                out_rec1.borrow_mut().first_left = out_rec2.borrow().first_left.clone();
            }
            out_rec1.borrow_mut().is_hole = out_rec2.borrow().is_hole;
        }
        out_rec2.borrow_mut().pts = None;
        out_rec2.borrow_mut().bottom_pt = None;
        out_rec2.borrow_mut().first_left = Some(out_rec1.clone());

        let ok_idx = e1.borrow().out_idx;
        let obsolete_idx = e2.borrow().out_idx;

        e1.borrow_mut().out_idx = UNASSIGNED;
        e2.borrow_mut().out_idx = UNASSIGNED;

        let mut e = self.base.active_edges.clone();
        while let Some(ref e_ref) = e {
            if e_ref.borrow().out_idx == obsolete_idx {
                e_ref.borrow_mut().out_idx = ok_idx;
                e_ref.borrow_mut().side = e1.borrow().side;
                break;
            }
            e = e_ref.borrow().next_in_ael.clone();
        }
        out_rec2.borrow_mut().idx = out_rec1.borrow().idx;
    }

    /// Reverses the links of a polygon.
    fn reverse_poly_pt_links(&self, pp: &Rc<RefCell<OutPt>>) {
        if pp.is_none() {
            return;
        }
        let mut pp1 = pp.clone();
        loop {
            let pp2 = pp1.borrow().next.clone().unwrap();
            pp1.borrow_mut().next = pp1.borrow().prev.clone();
            pp1.borrow_mut().prev = Some(pp2.clone());
            pp1 = pp2;
            if Rc::ptr_eq(&pp1, pp) {
                break;
            }
        }
    }

    /// Swaps the sides of two edges.
    fn swap_sides(edge1: &Rc<RefCell<TEdge>>, edge2: &Rc<RefCell<TEdge>>) {
        let side = edge1.borrow().side;
        edge1.borrow_mut().side = edge2.borrow().side;
        edge2.borrow_mut().side = side;
    }

    /// Swaps the polygon indexes of two edges.
    fn swap_poly_indexes(edge1: &Rc<RefCell<TEdge>>, edge2: &Rc<RefCell<TEdge>>) {
        let out_idx = edge1.borrow().out_idx;
        edge1.borrow_mut().out_idx = edge2.borrow().out_idx;
        edge2.borrow_mut().out_idx = out_idx;
    }

    /// Intersects two edges at a given point.
    fn intersect_edges(&mut self, e1: &Rc<RefCell<TEdge>>, e2: &Rc<RefCell<TEdge>>, pt: IntPoint) {
        let e1_contributing = e1.borrow().out_idx >= 0;
        let e2_contributing = e2.borrow().out_idx >= 0;

        // Handle open paths
        if e1.borrow().wind_delta == 0 || e2.borrow().wind_delta == 0 {
            if e1.borrow().wind_delta == 0 && e2.borrow().wind_delta == 0 {
                return;
            } else if e1.borrow().poly_typ == e2.borrow().poly_typ && e1.borrow().wind_delta != e2.borrow().wind_delta && self.clip_type == ClipType::Union {
                if e1.borrow().wind_delta == 0 {
                    if e2_contributing {
                        self.add_out_pt(e1, pt);
                        if e1_contributing {
                            e1.borrow_mut().out_idx = UNASSIGNED;
                        }
                    }
                } else {
                    if e1_contributing {
                        self.add_out_pt(e2, pt);
                        if e2_contributing {
                            e2.borrow_mut().out_idx = UNASSIGNED;
                        }
                    }
                }
            } else if e1.borrow().poly_typ != e2.borrow().poly_typ {
                if e1.borrow().wind_delta == 0 && e2.borrow().wind_cnt.abs() == 1 && (self.clip_type != ClipType::Union || e2.borrow().wind_cnt2 == 0) {
                    self.add_out_pt(e1, pt);
                    if e1_contributing {
                        e1.borrow_mut().out_idx = UNASSIGNED;
                    }
                } else if e2.borrow().wind_delta == 0 && e1.borrow().wind_cnt.abs() == 1 && (self.clip_type != ClipType::Union || e1.borrow().wind_cnt2 == 0) {
                    self.add_out_pt(e2, pt);
                    if e2_contributing {
                        e2.borrow_mut().out_idx = UNASSIGNED;
                    }
                }
            }
            return;
        }

        // Update winding counts
        if e1.borrow().poly_typ == e2.borrow().poly_typ {
            if self.is_even_odd_fill_type(&e1.borrow()) {
                let old_e1_wind_cnt = e1.borrow().wind_cnt;
                e1.borrow_mut().wind_cnt = e2.borrow().wind_cnt;
                e2.borrow_mut().wind_cnt = old_e1_wind_cnt;
            } else {
                if e1.borrow().wind_cnt + e2.borrow().wind_delta == 0 {
                    e1.borrow_mut().wind_cnt = -e1.borrow().wind_cnt;
                } else {
                    e1.borrow_mut().wind_cnt += e2.borrow().wind_delta;
                }
                if e2.borrow().wind_cnt - e1.borrow().wind_delta == 0 {
                    e2.borrow_mut().wind_cnt = -e2.borrow().wind_cnt;
                } else {
                    e2.borrow_mut().wind_cnt -= e1.borrow().wind_delta;
                }
            }
        } else {
            if !self.is_even_odd_fill_type(&e2.borrow()) {
                e1.borrow_mut().wind_cnt2 += e2.borrow().wind_delta;
            } else {
                e1.borrow_mut().wind_cnt2 = if e1.borrow().wind_cnt2 == 0 { 1 } else { 0 };
            }
            if !self.is_even_odd_fill_type(&e1.borrow()) {
                e2.borrow_mut().wind_cnt2 -= e1.borrow().wind_delta;
            } else {
                e2.borrow_mut().wind_cnt2 = if e2.borrow().wind_cnt2 == 0 { 1 } else { 0 };
            }
        }

        let (e1_fill_type, e2_fill_type, e1_fill_type2, e2_fill_type2) = if e1.borrow().poly_typ == PolyType::Subject {
            (self.subj_fill_type, self.clip_fill_type, self.clip_fill_type, self.subj_fill_type)
        } else {
            (self.clip_fill_type, self.subj_fill_type, self.subj_fill_type, self.clip_fill_type)
        };

        let e1_wc = match e1_fill_type {
            PolyFillType::Positive => e1.borrow().wind_cnt,
            PolyFillType::Negative => -e1.borrow().wind_cnt,
            _ => e1.borrow().wind_cnt.abs(),
        };

        let e2_wc = match e2_fill_type {
            PolyFillType::Positive => e2.borrow().wind_cnt,
            PolyFillType::Negative => -e2.borrow().wind_cnt,
            _ => e2.borrow().wind_cnt.abs(),
        };

        if e1_contributing && e2_contributing {
            if (e1_wc != 0 && e1_wc != 1) || (e2_wc != 0 && e2_wc != 1) || (e1.borrow().poly_typ != e2.borrow().poly_typ && self.clip_type != ClipType::Xor) {
                self.add_local_max_poly(e1, e2, pt);
            } else {
                self.add_out_pt(e1, pt);
                self.add_out_pt(e2, pt);
                Clipper::swap_sides(e1, e2);
                Clipper::swap_poly_indexes(e1, e2);
            }
        } else if e1_contributing {
            if e2_wc == 0 || e2_wc == 1 {
                self.add_out_pt(e1, pt);
                Clipper::swap_sides(e1, e2);
                Clipper::swap_poly_indexes(e1, e2);
            }
        } else if e2_contributing {
            if e1_wc == 0 || e1_wc == 1 {
                self.add_out_pt(e2, pt);
                Clipper::swap_sides(e1, e2);
                Clipper::swap_poly_indexes(e1, e2);
            }
        } else if (e1_wc == 0 || e1_wc == 1) && (e2_wc == 0 || e2_wc == 1) {
            let e1_wc2 = match e1_fill_type2 {
                PolyFillType::Positive => e1.borrow().wind_cnt2,
                PolyFillType::Negative => -e1.borrow().wind_cnt2,
                _ => e1.borrow().wind_cnt2.abs(),
            };

            let e2_wc2 = match e2_fill_type2 {
                PolyFillType::Positive => e2.borrow().wind_cnt2,
                PolyFillType::Negative => -e2.borrow().wind_cnt2,
                _ => e2.borrow().wind_cnt2.abs(),
            };

            if e1.borrow().poly_typ != e2.borrow().poly_typ {
                self.add_local_min_poly(e1, e2, pt);
            } else if e1_wc == 1 && e2_wc == 1 {
                match self.clip_type {
                    ClipType::Intersection => {
                        if e1_wc2 > 0 && e2_wc2 > 0 {
                            self.add_local_min_poly(e1, e2, pt);
                        }
                    }
                    ClipType::Union => {
                        if e1_wc2 <= 0 && e2_wc2 <= 0 {
                            self.add_local_min_poly(e1, e2, pt);
                        }
                    }
                    ClipType::Difference => {
                        if (e1.borrow().poly_typ == PolyType::Clip && e1_wc2 > 0 && e2_wc2 > 0) || (e1.borrow().poly_typ == PolyType::Subject && e1_wc2 <= 0 && e2_wc2 <= 0) {
                            self.add_local_min_poly(e1, e2, pt);
                        }
                    }
                    ClipType::Xor => {
                        self.add_local_min_poly(e1, e2, pt);
                    }
                }
            } else {
                self.swap_sides(e1, e2);
            }
        }
    }

    /// Deletes an edge from the sorted edge list (SEL).
    fn delete_from_sel(&mut self, e: &Rc<RefCell<TEdge>>) {
        let sel_prev = e.borrow().prev_in_sel.clone();
        let sel_next = e.borrow().next_in_sel.clone();
        if sel_prev.is_none() && sel_next.is_none() && !Rc::ptr_eq(&Some(e.clone()), &self.sorted_edges) {
            return; // already deleted
        }
        if let Some(ref sel_prev) = sel_prev {
            sel_prev.borrow_mut().next_in_sel = sel_next.clone();
        } else {
            self.sorted_edges = sel_next.clone();
        }
        if let Some(ref sel_next) = sel_next {
            sel_next.borrow_mut().prev_in_sel = sel_prev.clone();
        }
        e.borrow_mut().next_in_sel = None;
        e.borrow_mut().prev_in_sel = None;
    }

    /// Processes all horizontal edges in the sorted edge list (SEL).
    fn process_horizontals(&mut self) {
        while let Some(horz_edge) = self.pop_edge_from_sel() {
            self.process_horizontal(&horz_edge);
        }
    }

    /// Determines the direction and left/right bounds of a horizontal edge.
    fn get_horz_direction(&self, horz_edge: &Rc<RefCell<TEdge>>, dir: &mut Direction, left: &mut CInt, right: &mut CInt) {
        if horz_edge.borrow().bot.x < horz_edge.borrow().top.x {
            *left = horz_edge.borrow().bot.x;
            *right = horz_edge.borrow().top.x;
            *dir = Direction::LeftToRight;
        } else {
            *left = horz_edge.borrow().top.x;
            *right = horz_edge.borrow().bot.x;
            *dir = Direction::RightToLeft;
        }
    }

    /// Processes a horizontal edge.
    fn process_horizontal(&mut self, horz_edge: &Rc<RefCell<TEdge>>) {
        let mut dir = Direction::LeftToRight;
        let mut horz_left = 0;
        let mut horz_right = 0;
        let is_open = horz_edge.borrow().wind_delta == 0;

        self.get_horz_direction(horz_edge, &mut dir, &mut horz_left, &mut horz_right);

        let mut e_last_horz = horz_edge.clone();
        let mut e_max_pair = None;
        while e_last_horz.borrow().next_in_lml.is_some() && self.base.is_horizontal(&e_last_horz.borrow().next_in_lml.as_ref().unwrap().borrow()) {
            e_last_horz = e_last_horz.borrow().next_in_lml.as_ref().unwrap().clone();
        }
        if e_last_horz.borrow().next_in_lml.is_none() {
            e_max_pair = self.get_maxima_pair(&e_last_horz);
        }

        let mut curr_max = self.maxima.clone();
        if let Some(ref curr_max) = curr_max {
            if dir == Direction::LeftToRight {
                while let Some(ref curr_max) = curr_max {
                    if curr_max.x > horz_edge.borrow().bot.x {
                        break;
                    }
                    curr_max = curr_max.next.clone();
                }
                if let Some(ref curr_max) = curr_max {
                    if curr_max.x >= e_last_horz.borrow().top.x {
                        curr_max = None;
                    }
                }
            } else {
                while let Some(ref curr_max) = curr_max.next {
                    if curr_max.x >= horz_edge.borrow().bot.x {
                        break;
                    }
                    curr_max = curr_max.next.clone();
                }
                if let Some(ref curr_max) = curr_max {
                    if curr_max.x <= e_last_horz.borrow().top.x {
                        curr_max = None;
                    }
                }
            }
        }

        let mut op1 = None;
        loop {
            let is_last_horz = Rc::ptr_eq(&horz_edge, &e_last_horz);
            let mut e = self.get_next_in_ael(horz_edge, dir);
            while let Some(ref e_ref) = e {
                if let Some(ref curr_max) = curr_max {
                    if dir == Direction::LeftToRight {
                        while let Some(ref curr_max) = curr_max {
                            if curr_max.x >= e_ref.borrow().curr.x {
                                break;
                            }
                            if horz_edge.borrow().out_idx >= 0 && !is_open {
                                self.add_out_pt(horz_edge, IntPoint::new(curr_max.x, horz_edge.borrow().bot.y));
                            }
                            curr_max = curr_max.next.clone();
                        }
                    } else {
                        while let Some(ref curr_max) = curr_max {
                            if curr_max.x <= e_ref.borrow().curr.x {
                                break;
                            }
                            if horz_edge.borrow().out_idx >= 0 && !is_open {
                                self.add_out_pt(horz_edge, IntPoint::new(curr_max.x, horz_edge.borrow().bot.y));
                            }
                            curr_max = curr_max.prev.clone();
                        }
                    }
                }

                if (dir == Direction::LeftToRight && e_ref.borrow().curr.x > horz_right) || (dir == Direction::RightToLeft && e_ref.borrow().curr.x < horz_left) {
                    break;
                }

                if e_ref.borrow().curr.x == horz_edge.borrow().top.x && horz_edge.borrow().next_in_lml.is_some() && e_ref.borrow().dx < horz_edge.borrow().next_in_lml.as_ref().unwrap().borrow().dx {
                    break;
                }

                if horz_edge.borrow().out_idx >= 0 && !is_open {
                    op1 = Some(self.add_out_pt(horz_edge, e_ref.borrow().curr));
                    let mut e_next_horz = self.sorted_edges.clone();
                    while let Some(ref e_next_horz) = e_next_horz {
                        if e_next_horz.borrow().out_idx >= 0 && self.horz_segments_overlap(horz_edge.borrow().bot.x, horz_edge.borrow().top.x, e_next_horz.borrow().bot.x, e_next_horz.borrow().top.x) {
                            let op2 = self.get_last_out_pt(e_next_horz);
                            self.add_join(&op2, &op1.as_ref().unwrap(), e_next_horz.borrow().top);
                        }
                        e_next_horz = e_next_horz.borrow().next_in_sel.clone();
                    }
                    self.add_ghost_join(&op1.as_ref().unwrap(), horz_edge.borrow().bot);
                }

                if Rc::ptr_eq(e_ref, &e_max_pair.as_ref().unwrap()) && is_last_horz {
                    if horz_edge.borrow().out_idx >= 0 {
                        self.add_local_max_poly(horz_edge, &e_max_pair.as_ref().unwrap(), horz_edge.borrow().top);
                    }
                    self.base.delete_from_ael(horz_edge);
                    self.base.delete_from_ael(&e_max_pair.as_ref().unwrap());
                    return;
                }

                let pt = IntPoint::new(e_ref.borrow().curr.x, horz_edge.borrow().curr.y);
                if dir == Direction::LeftToRight {
                    self.intersect_edges(horz_edge, e_ref, pt);
                } else {
                    self.intersect_edges(e_ref, horz_edge, pt);
                }
                let e_next = self.get_next_in_ael(e_ref, dir);
                self.base.swap_positions_in_ael(horz_edge, e_ref);
                e = e_next;
            }

            if horz_edge.borrow().next_in_lml.is_none() || !self.base.is_horizontal(&horz_edge.borrow().next_in_lml.as_ref().unwrap().borrow()) {
                break;
            }

            self.base.update_edge_into_ael(horz_edge);
            if horz_edge.borrow().out_idx >= 0 {
                self.add_out_pt(horz_edge, horz_edge.borrow().bot);
            }
            self.get_horz_direction(horz_edge, &mut dir, &mut horz_left, &mut horz_right);
        }

        if horz_edge.borrow().out_idx >= 0 && op1.is_none() {
            op1 = Some(self.get_last_out_pt(horz_edge));
            let mut e_next_horz = self.sorted_edges.clone();
            while let Some(ref e_next_horz) = e_next_horz {
                if e_next_horz.borrow().out_idx >= 0 && self.horz_segments_overlap(horz_edge.borrow().bot.x, horz_edge.borrow().top.x, e_next_horz.borrow().bot.x, e_next_horz.borrow().top.x) {
                    let op2 = self.get_last_out_pt(e_next_horz);
                    self.add_join(&op2, &op1.as_ref().unwrap(), e_next_horz.borrow().top);
                }
                e_next_horz = e_next_horz.borrow().next_in_sel.clone();
            }
            self.add_ghost_join(&op1.as_ref().unwrap(), horz_edge.borrow().top);
        }

        if horz_edge.borrow().next_in_lml.is_some() {
            if horz_edge.borrow().out_idx >= 0 {
                op1 = Some(self.add_out_pt(horz_edge, horz_edge.borrow().top));
                self.base.update_edge_into_ael(horz_edge);
                if horz_edge.borrow().wind_delta == 0 {
                    return;
                }
                let e_prev = horz_edge.borrow().prev_in_ael.clone();
                let e_next = horz_edge.borrow().next_in_ael.clone();
                if let Some(ref e_prev) = e_prev {
                    if e_prev.borrow().curr.x == horz_edge.borrow().bot.x && e_prev.borrow().curr.y == horz_edge.borrow().bot.y && e_prev.borrow().wind_delta != 0 && e_prev.borrow().out_idx >= 0 && e_prev.borrow().curr.y > e_prev.borrow().top.y && self.base.slopes_equal(&horz_edge.borrow(), &e_prev.borrow(), self.base.use_full_range) {
                        let op2 = self.add_out_pt(e_prev, horz_edge.borrow().bot);
                        self.add_join(&op1.as_ref().unwrap(), &op2, horz_edge.borrow().top);
                    }
                } else if let Some(ref e_next) = e_next {
                    if e_next.borrow().curr.x == horz_edge.borrow().bot.x && e_next.borrow().curr.y == horz_edge.borrow().bot.y && e_next.borrow().wind_delta != 0 && e_next.borrow().out_idx >= 0 && e_next.borrow().curr.y > e_next.borrow().top.y && self.base.slopes_equal(&horz_edge.borrow(), &e_next.borrow(), self.base.use_full_range) {
                        let op2 = self.add_out_pt(e_next, horz_edge.borrow().bot);
                        self.add_join(&op1.as_ref().unwrap(), &op2, horz_edge.borrow().top);
                    }
                }
            } else {
                self.base.update_edge_into_ael(horz_edge);
            }
        } else {
            if horz_edge.borrow().out_idx >= 0 {
                self.add_out_pt(horz_edge, horz_edge.borrow().top);
            }
            self.base.delete_from_ael(horz_edge);
        }
    }

    /// Gets the next edge in the active edge list (AEL) based on the direction.
    fn get_next_in_ael(&self, e: &Rc<RefCell<TEdge>>, direction: Direction) -> Option<Rc<RefCell<TEdge>>> {
        if direction == Direction::LeftToRight {
            e.borrow().next_in_ael.clone()
        } else {
            e.borrow().prev_in_ael.clone()
        }
    }

    /// Checks if the edge is a minima.
    fn is_minima(&self, e: &Rc<RefCell<TEdge>>) -> bool {
        e.borrow().prev.as_ref().unwrap().borrow().next_in_lml.as_ref().map_or(false, |next_in_lml| !Rc::ptr_eq(next_in_lml, e))
            && e.borrow().next.as_ref().unwrap().borrow().next_in_lml.as_ref().map_or(false, |next_in_lml| !Rc::ptr_eq(next_in_lml, e))
    }

    /// Checks if the edge is a maxima at the given Y coordinate.
    fn is_maxima(&self, e: &Rc<RefCell<TEdge>>, y: CInt) -> bool {
        e.borrow().top.y == y && e.borrow().next_in_lml.is_none()
    }

    /// Checks if the edge is intermediate at the given Y coordinate.
    fn is_intermediate(&self, e: &Rc<RefCell<TEdge>>, y: CInt) -> bool {
        e.borrow().top.y == y && e.borrow().next_in_lml.is_some()
    }

    /// Gets the maxima pair for the given edge.
    fn get_maxima_pair(&self, e: &Rc<RefCell<TEdge>>) -> Option<Rc<RefCell<TEdge>>> {
        if e.borrow().next.as_ref().unwrap().borrow().top == e.borrow().top && e.borrow().next.as_ref().unwrap().borrow().next_in_lml.is_none() {
            Some(e.borrow().next.as_ref().unwrap().clone())
        } else if e.borrow().prev.as_ref().unwrap().borrow().top == e.borrow().top && e.borrow().prev.as_ref().unwrap().borrow().next_in_lml.is_none() {
            Some(e.borrow().prev.as_ref().unwrap().clone())
        } else {
            None
        }
    }

    /// Gets the maxima pair for the given edge, ensuring it is in the active edge list.
    fn get_maxima_pair_ex(&self, e: &Rc<RefCell<TEdge>>) -> Option<Rc<RefCell<TEdge>>> {
        let result = self.get_maxima_pair(e);
        if let Some(ref result) = result {
            if result.borrow().out_idx == SKIP || (result.borrow().next_in_ael.is_none() && result.borrow().prev_in_ael.is_none() && !self.base.is_horizontal(&result.borrow())) {
                return None;
            }
        }
        result
    }

    /// Processes intersections at the given top Y coordinate.
    fn process_intersections(&mut self, top_y: CInt) -> bool {
        if self.base.active_edges.is_none() {
            return true;
        }
        // ...existing code...
        self.build_intersect_list(top_y);
        if self.intersect_list.is_empty() {
            return true;
        }
        if self.intersect_list.len() == 1 || self.fixup_intersection_order() {
            self.process_intersect_list();
        } else {
            return false;
        }
        self.sorted_edges = None;
        true
    }

    /// Builds the list of intersections at the given top Y coordinate.
    fn build_intersect_list(&mut self, top_y: CInt) {
        if self.base.active_edges.is_none() {
            return;
        }

        // Prepare for sorting
        let mut e = self.base.active_edges.clone();
        self.sorted_edges = e.clone();
        while let Some(ref edge) = e {
            edge.borrow_mut().prev_in_sel = edge.borrow().prev_in_ael.clone();
            edge.borrow_mut().next_in_sel = edge.borrow().next_in_ael.clone();
            edge.borrow_mut().curr.x = self.top_x(edge, top_y);
            e = edge.borrow().next_in_ael.clone();
        }

        // Bubble sort
        let mut is_modified = true;
        while is_modified && self.sorted_edges.is_some() {
            is_modified = false;
            e = self.sorted_edges.clone();
            while let Some(ref edge) = e {
                if let Some(ref next_edge) = edge.borrow().next_in_sel {
                    if edge.borrow().curr.x > next_edge.borrow().curr.x {
                        let mut pt = IntPoint::new(0, 0);
                        self.intersect_point(edge, next_edge, &mut pt);
                        if pt.y < top_y {
                            pt = IntPoint::new(self.top_x(edge, top_y), top_y);
                        }
                        let new_node = IntersectNode {
                            edge1: Some(edge.clone()),
                            edge2: Some(next_edge.clone()),
                            pt,
                        };
                        self.intersect_list.push(new_node);

                        self.swap_positions_in_sel(edge, next_edge);
                        is_modified = true;
                    } else {
                        e = edge.borrow().next_in_sel.clone();
                    }
                } else {
                    break;
                }
            }
            if let Some(ref edge) = e {
                if edge.borrow().prev_in_sel.is_some() {
                    edge.borrow_mut().prev_in_sel.as_ref().unwrap().borrow_mut().next_in_sel = None;
                } else {
                    break;
                }
            }
        }
        self.sorted_edges = None;
    }

    /// Checks if the edges in the intersect node are adjacent.
    fn edges_adjacent(&self, inode: &IntersectNode) -> bool {
        Rc::ptr_eq(&inode.edge1.as_ref().unwrap().borrow().next_in_sel, &inode.edge2)
            || Rc::ptr_eq(&inode.edge1.as_ref().unwrap().borrow().prev_in_sel, &inode.edge2)
    }

    /// Sorts intersect nodes by their Y coordinate.
    fn intersect_node_sort(node1: &IntersectNode, node2: &IntersectNode) -> Ordering {
        node2.pt.y.cmp(&node1.pt.y)
    }

    /// Fixes the order of intersections to ensure they are made between adjacent edges.
    fn fixup_intersection_order(&mut self) -> bool {
        // Pre-condition: intersections are sorted bottom-most first.
        // Now it's crucial that intersections are made only between adjacent edges,
        // so to ensure this the order of intersections may need adjusting.
        self.intersect_list.sort_by(Clipper::intersect_node_sort);

        self.copy_ael_to_sel();
        let cnt = self.intersect_list.len();
        for i in 0..cnt {
            if !self.edges_adjacent(&self.intersect_list[i]) {
                let mut j = i + 1;
                while j < cnt && !self.edges_adjacent(&self.intersect_list[j]) {
                    j += 1;
                }
                if j == cnt {
                    return false;
                }

                self.intersect_list.swap(i, j);
            }
            self.swap_positions_in_sel(
                &self.intersect_list[i].edge1.as_ref().unwrap(),
                &self.intersect_list[i].edge2.as_ref().unwrap(),
            );
        }
        true
    }

    /// Processes the list of intersections.
    fn process_intersect_list(&mut self) {
        for i in 0..self.intersect_list.len() {
            let i_node = self.intersect_list[i].clone();
            let edge1 = i_node.edge1.unwrap();
            let edge2 = i_node.edge2.unwrap();
            let pt = i_node.pt;
            self.intersect_edges(&edge1, &edge2, pt);
            self.base.swap_positions_in_ael(
                &mut i_node.edge1.as_ref().unwrap().borrow_mut(),
                &mut i_node.edge2.as_ref().unwrap().borrow_mut(),
            );
        }
        self.intersect_list.clear();
    }

    /// Rounds a floating-point value to the nearest integer.
    fn round(value: f64) -> CInt {
        if value < 0.0 {
            (value - 0.5).floor() as CInt
        } else {
            (value + 0.5).floor() as CInt
        }
    }

    /// Calculates the intersection point of two edges.
    fn intersect_point(edge1: &TEdge, edge2: &TEdge) -> IntPoint {
        let mut ip = IntPoint::new(0, 0);
        if edge1.dx == edge2.dx {
            ip.y = edge1.curr.y;
            ip.x = Clipper::top_x(&Rc::new(RefCell::new(edge1.clone())), ip.y);
            return ip;
        }

        if edge1.delta.x == 0 {
            ip.x = edge1.bot.x;
            if ClipperBase::is_horizontal(edge2) {
                ip.y = edge2.bot.y;
            } else {
                let b2 = edge2.bot.y as f64 - (edge2.bot.x as f64 / edge2.dx);
                ip.y = Clipper::round(ip.x as f64 / edge2.dx + b2);
            }
        } else if edge2.delta.x == 0 {
            ip.x = edge2.bot.x;
            if ClipperBase::is_horizontal(edge1) {
                ip.y = edge1.bot.y;
            } else {
                let b1 = edge1.bot.y as f64 - (edge1.bot.x as f64 / edge1.dx);
                ip.y = Clipper::round(ip.x as f64 / edge1.dx + b1);
            }
        } else {
            let b1 = edge1.bot.x as f64 - edge1.bot.y as f64 * edge1.dx;
            let b2 = edge2.bot.x as f64 - edge2.bot.y as f64 * edge2.dx;
            let q = (b2 - b1) / (edge1.dx - edge2.dx);
            ip.y = Clipper::round(q);
            if edge1.dx.abs() < edge2.dx.abs() {
                ip.x = Clipper::round(edge1.dx * q + b1);
            } else {
                ip.x = Clipper::round(edge2.dx * q + b2);
            }
        }

        if ip.y < edge1.top.y || ip.y < edge2.top.y {
            if edge1.top.y > edge2.top.y {
                ip.y = edge1.top.y;
            } else {
                ip.y = edge2.top.y;
            }
            if edge1.dx.abs() < edge2.dx.abs() {
                ip.x = Clipper::top_x(edge1, ip.y);
            } else {
                ip.x = Clipper::top_x(edge2, ip.y);
            }
        }

        if ip.y > edge1.curr.y {
            ip.y = edge1.curr.y;
            if edge1.dx.abs() > edge2.dx.abs() {
                ip.x = Clipper::top_x(edge2, ip.y);
            } else {
                ip.x = Clipper::top_x(edge1, ip.y);
            }
        }

        ip
    }

    /// Processes maxima for a given edge.
    fn do_maxima(&mut self, e: &Rc<RefCell<TEdge>>) {
        let e_max_pair = self.get_maxima_pair_ex(e);
        if e_max_pair.is_none() {
            if e.borrow().out_idx >= 0 {
                self.add_out_pt(e, e.borrow().top);
            }
            self.base.delete_from_ael(e);
            return;
        }

        let mut e_next = e.borrow().next_in_ael.clone();
        while let Some(ref e_next_ref) = e_next {
            if Rc::ptr_eq(e_next_ref, &e_max_pair.as_ref().unwrap()) {
                break;
            }
            self.intersect_edges(e, e_next_ref, e.borrow().top);
            self.base.swap_positions_in_ael(e, e_next_ref);
            e_next = e.borrow().next_in_ael.clone();
        }

        if e.borrow().out_idx == UNASSIGNED && e_max_pair.as_ref().unwrap().borrow().out_idx == UNASSIGNED {
            self.base.delete_from_ael(e);
            self.base.delete_from_ael(e_max_pair.as_ref().unwrap());
        } else if e.borrow().out_idx >= 0 && e_max_pair.as_ref().unwrap().borrow().out_idx >= 0 {
            self.add_local_max_poly(e, e_max_pair.as_ref().unwrap(), e.borrow().top);
            self.base.delete_from_ael(e);
            self.base.delete_from_ael(e_max_pair.as_ref().unwrap());
        } else if e.borrow().wind_delta == 0 {
            if e.borrow().out_idx >= 0 {
                self.add_out_pt(e, e.borrow().top);
                e.borrow_mut().out_idx = UNASSIGNED;
            }
            self.base.delete_from_ael(e);

            if e_max_pair.as_ref().unwrap().borrow().out_idx >= 0 {
                self.add_out_pt(e_max_pair.as_ref().unwrap(), e.borrow().top);
                e_max_pair.as_ref().unwrap().borrow_mut().out_idx = UNASSIGNED;
            }
            self.base.delete_from_ael(e_max_pair.as_ref().unwrap());
        } else {
            panic!("DoMaxima error");
        }
    }

    /// Reverses the paths in the given vector of paths.
    pub fn reverse_paths(polys: &mut Paths) {
        for poly in polys.iter_mut() {
            poly.reverse();
        }
    }

    /// Determines the orientation of a path.
    pub fn orientation(poly: &Path) -> bool {
        Self::area(poly) >= 0.0
    }

    /// Counts the number of points in an OutPt circular linked list.
    fn point_count(pts: &OutPt) -> usize {
        if pts.is_none() {
            return 0;
        }
        let mut result = 0;
        let mut p = pts.clone();
        loop {
            result += 1;
            p = p.borrow().next.as_ref().unwrap().clone();
            if Rc::ptr_eq(&p, &pts) {
                break;
            }
        }
        result
    }

    /// Builds the result paths from the internal OutRec structures.
    fn build_result(&mut self, polyg: &mut Paths) {
        polyg.clear();
        polyg.reserve(self.base.poly_outs.len());
        for out_rec in &self.base.poly_outs {
            if let Some(ref pts) = out_rec.pts {
                let p = pts.borrow().prev.as_ref().unwrap().clone();
                let cnt = Self::point_count(&p.borrow());
                if cnt < 2 {
                    continue;
                }
                let mut pg = Path::with_capacity(cnt);
                let mut p = p.clone();
                for _ in 0..cnt {
                    pg.push(p.borrow().pt);
                    p = p.borrow().prev.as_ref().unwrap().clone();
                }
                polyg.push(pg);
            }
        }
    }

    /// Fixes up the output polyline by removing duplicate points.
    fn fixup_out_polyline(&mut self, out_rec: &mut OutRec) {
        let mut pp = out_rec.pts.as_ref().unwrap().clone();
        let mut last_pp = pp.borrow().prev.as_ref().unwrap().clone();
        while !Rc::ptr_eq(&pp, &last_pp) {
            pp = pp.borrow().next.as_ref().unwrap().clone();
            if pp.borrow().pt == pp.borrow().prev.as_ref().unwrap().borrow().pt {
                if Rc::ptr_eq(&pp, &last_pp) {
                    last_pp = pp.borrow().prev.as_ref().unwrap().clone();
                }
                let tmp_pp = pp.borrow().prev.as_ref().unwrap().clone();
                tmp_pp.borrow_mut().next = pp.borrow().next.clone();
                pp.borrow().next.as_ref().unwrap().borrow_mut().prev = Some(tmp_pp.clone());
                pp = tmp_pp;
            }
        }
        if Rc::ptr_eq(&pp, &pp.borrow().prev.as_ref().unwrap()) {
            out_rec.pts = None;
        }
    }

    /// Fixes up the output polygon by removing duplicate points and simplifying consecutive parallel edges.
    fn fixup_out_polygon(&mut self, out_rec: &mut OutRec) {
        let mut last_ok: Option<Rc<RefCell<OutPt>>> = None;
        out_rec.bottom_pt = None;
        let mut pp = out_rec.pts.as_ref().unwrap().clone();
        let preserve_col = self.base.preserve_collinear || self.strictly_simple;

        loop {
            if Rc::ptr_eq(&pp, &pp.borrow().prev.as_ref().unwrap()) || Rc::ptr_eq(&pp.borrow().prev.as_ref().unwrap(), &pp.borrow().next.as_ref().unwrap()) {
                out_rec.pts = None;
                return;
            }

            if pp.borrow().pt == pp.borrow().next.as_ref().unwrap().borrow().pt
                || pp.borrow().pt == pp.borrow().prev.as_ref().unwrap().borrow().pt
                || (ClipperBase::slopes_equal_points(
                    pp.borrow().prev.as_ref().unwrap().borrow().pt,
                    pp.borrow().pt,
                    pp.borrow().next.as_ref().unwrap().borrow().pt,
                    self.base.use_full_range,
                ) && (!preserve_col
                    || !self.base.pt2_is_between_pt1_and_pt3(
                        pp.borrow().prev.as_ref().unwrap().borrow().pt,
                        pp.borrow().pt,
                        pp.borrow().next.as_ref().unwrap().borrow().pt,
                    )))
            {
                last_ok = None;
                pp.borrow().prev.as_ref().unwrap().borrow_mut().next = pp.borrow().next.clone();
                pp.borrow().next.as_ref().unwrap().borrow_mut().prev = pp.borrow().prev.clone();
                pp = pp.borrow().prev.as_ref().unwrap().clone();
            } else if Some(pp.clone()) == last_ok {
                break;
            } else {
                if last_ok.is_none() {
                    last_ok = Some(pp.clone());
                }
                pp = pp.borrow().next.as_ref().unwrap().clone();
            }
        }
        out_rec.pts = Some(pp);
    }

    /// Duplicates an output point.
    fn dup_out_pt(&self, out_pt: &Rc<RefCell<OutPt>>, insert_after: bool) -> Rc<RefCell<OutPt>> {
        let result = Rc::new(RefCell::new(OutPt {
            pt: out_pt.borrow().pt,
            idx: out_pt.borrow().idx,
            next: None,
            prev: None,
        }));

        if insert_after {
            result.borrow_mut().next = out_pt.borrow().next.clone();
            result.borrow_mut().prev = Some(out_pt.clone());
            out_pt.borrow().next.as_ref().unwrap().borrow_mut().prev = Some(result.clone());
            out_pt.borrow_mut().next = Some(result.clone());
        } else {
            result.borrow_mut().prev = out_pt.borrow().prev.clone();
            result.borrow_mut().next = Some(out_pt.clone());
            out_pt.borrow().prev.as_ref().unwrap().borrow_mut().next = Some(result.clone());
            out_pt.borrow_mut().prev = Some(result.clone());
        }

        result
    }

    /// Gets the overlap between two ranges.
    fn get_overlap(a1: CInt, a2: CInt, b1: CInt, b2: CInt, left: &mut CInt, right: &mut CInt) -> bool {
        if a1 < a2 {
            if b1 < b2 {
                *left = a1.max(b1);
                *right = a2.min(b2);
            } else {
                *left = a1.max(b2);
                *right = a2.min(b1);
            }
        } else {
            if b1 < b2 {
                *left = a2.max(b1);
                *right = a1.min(b2);
            } else {
                *left = a2.max(b2);
                *right = a1.min(b1);
            }
        }
        *left < *right
    }

    /// Joins two horizontal edges.
    fn join_horz(
        &self,
        op1: &Rc<RefCell<OutPt>>,
        mut op1b: Rc<RefCell<OutPt>>,
        op2: &Rc<RefCell<OutPt>>,
        mut op2b: Rc<RefCell<OutPt>>,
        pt: IntPoint,
        discard_left: bool,
    ) -> bool {
        let dir1 = if op1.borrow().pt.x > op1b.borrow().pt.x {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        };
        let dir2 = if op2.borrow().pt.x > op2b.borrow().pt.x {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        };

        if dir1 == dir2 {
            return false;
        }

        if dir1 == Direction::LeftToRight {
            while op1.borrow().next.as_ref().unwrap().borrow().pt.x <= pt.x
                && op1.borrow().next.as_ref().unwrap().borrow().pt.x >= op1.borrow().pt.x
                && op1.borrow().next.as_ref().unwrap().borrow().pt.y == pt.y
            {
                op1 = op1.borrow().next.as_ref().unwrap().clone();
            }
            if discard_left && op1.borrow().pt.x != pt.x {
                op1 = op1.borrow().next.as_ref().unwrap().clone();
            }
            op1b = self.dup_out_pt(&op1, !discard_left);
            if op1b.borrow().pt != pt {
                op1 = op1b.clone();
                op1.borrow_mut().pt = pt;
                op1b = self.dup_out_pt(&op1, !discard_left);
            }
        } else {
            while op1.borrow().next.as_ref().unwrap().borrow().pt.x >= pt.x
                && op1.borrow().next.as_ref().unwrap().borrow().pt.x <= op1.borrow().pt.x
                && op1.borrow().next.as_ref().unwrap().borrow().pt.y == pt.y
            {
                op1 = op1.borrow().next.as_ref().unwrap().clone();
            }
            if !discard_left && op1.borrow().pt.x != pt.x {
                op1 = op1.borrow().next.as_ref().unwrap().clone();
            }
            op1b = self.dup_out_pt(&op1, discard_left);
            if op1b.borrow().pt != pt {
                op1 = op1b.clone();
                op1.borrow_mut().pt = pt;
                op1b = self.dup_out_pt(&op1, discard_left);
            }
        }

        if dir2 == Direction::LeftToRight {
            while op2.borrow().next.as_ref().unwrap().borrow().pt.x <= pt.x
                && op2.borrow().next.as_ref().unwrap().borrow().pt.x >= op2.borrow().pt.x
                && op2.borrow().next.as_ref().unwrap().borrow().pt.y == pt.y
            {
                op2 = op2.borrow().next.as_ref().unwrap().clone();
            }
            if discard_left && op2.borrow().pt.x != pt.x {
                op2 = op2.borrow().next.as_ref().unwrap().clone();
            }
            op2b = self.dup_out_pt(&op2, !discard_left);
            if op2b.borrow().pt != pt {
                op2 = op2b.clone();
                op2.borrow_mut().pt = pt;
                op2b = self.dup_out_pt(&op2, !discard_left);
            }
        } else {
            while op2.borrow().next.as_ref().unwrap().borrow().pt.x >= pt.x
                && op2.borrow().next.as_ref().unwrap().borrow().pt.x <= op2.borrow().pt.x
                && op2.borrow().next.as_ref().unwrap().borrow().pt.y == pt.y
            {
                op2 = op2.borrow().next.as_ref().unwrap().clone();
            }
            if !discard_left && op2.borrow().pt.x != pt.x {
                op2 = op2.borrow().next.as_ref().unwrap().clone();
            }
            op2b = self.dup_out_pt(&op2, discard_left);
            if op2b.borrow().pt != pt {
                op2 = op2b.clone();
                op2.borrow_mut().pt = pt;
                op2b = self.dup_out_pt(&op2, discard_left);
            }
        }

        if (dir1 == Direction::LeftToRight) == discard_left {
            op1.borrow_mut().prev = Some(op2.clone());
            op2.borrow_mut().next = Some(op1.clone());
            op1b.borrow_mut().next = Some(op2b.clone());
            op2b.borrow_mut().prev = Some(op1b.clone());
        } else {
            op1.borrow_mut().next = Some(op2.clone());
            op2.borrow_mut().prev = Some(op1.clone());
            op1b.borrow_mut().prev = Some(op2b.clone());
            op2b.borrow_mut().next = Some(op1b.clone());
        }
        true
    }

    fn join_points(&self, j: &mut Join, out_rec1: &OutRec, out_rec2: &OutRec) -> bool {
        let mut op1 = j.out_pt1.clone().unwrap();
        let mut op2 = j.out_pt2.clone().unwrap();
        let mut op1b;
        let mut op2b;

        let is_horizontal = op1.borrow().pt.y == j.off_pt.y;

        if is_horizontal && j.off_pt == op1.borrow().pt && j.off_pt == op2.borrow().pt {
            if out_rec1 != out_rec2 {
                return false;
            }
            op1b = op1.borrow().next.clone().unwrap();
            while op1b != op1 && op1b.borrow().pt == j.off_pt {
                op1b = op1b.borrow().next.clone().unwrap();
            }
            let reverse1 = op1b.borrow().pt.y > j.off_pt.y;
            op2b = op2.borrow().next.clone().unwrap();
            while op2b != op2 && op2b.borrow().pt == j.off_pt {
                op2b = op2b.borrow().next.clone().unwrap();
            }
            let reverse2 = op2b.borrow().pt.y > j.off_pt.y;
            if reverse1 == reverse2 {
                return false;
            }
            if reverse1 {
                op1b = self.dup_out_pt(&op1, false);
                op2b = self.dup_out_pt(&op2, true);
                op1.borrow_mut().prev = Some(op2.clone());
                op2.borrow_mut().next = Some(op1.clone());
                op1b.borrow_mut().next = Some(op2b.clone());
                op2b.borrow_mut().prev = Some(op1b.clone());
                j.out_pt1 = Some(op1.clone());
                j.out_pt2 = Some(op1b.clone());
                return true;
            } else {
                op1b = self.dup_out_pt(&op1, true);
                op2b = self.dup_out_pt(&op2, false);
                op1.borrow_mut().next = Some(op2.clone());
                op2.borrow_mut().prev = Some(op1.clone());
                op1b.borrow_mut().prev = Some(op2b.clone());
                op2b.borrow_mut().next = Some(op1b.clone());
                j.out_pt1 = Some(op1.clone());
                j.out_pt2 = Some(op1b.clone());
                return true;
            }
        } else if is_horizontal {
            op1b = op1.clone();
            while op1.borrow().prev.as_ref().unwrap().borrow().pt.y == op1.borrow().pt.y
                && op1.borrow().prev.as_ref().unwrap() != op1b
                && op1.borrow().prev.as_ref().unwrap() != op2
            {
                op1 = op1.borrow().prev.clone().unwrap();
            }
            while op1b.borrow().next.as_ref().unwrap().borrow().pt.y == op1b.borrow().pt.y
                && op1b.borrow().next.as_ref().unwrap() != op1
                && op1b.borrow().next.as_ref().unwrap() != op2
            {
                op1b = op1b.borrow().next.clone().unwrap();
            }
            if op1b.borrow().next.as_ref().unwrap() == op1 || op1b.borrow().next.as_ref().unwrap() == op2 {
                return false;
            }

            op2b = op2.clone();
            while op2.borrow().prev.as_ref().unwrap().borrow().pt.y == op2.borrow().pt.y
                && op2.borrow().prev.as_ref().unwrap() != op2b
                && op2.borrow().prev.as_ref().unwrap() != op1b
            {
                op2 = op2.borrow().prev.clone().unwrap();
            }
            while op2b.borrow().next.as_ref().unwrap().borrow().pt.y == op2b.borrow().pt.y
                && op2b.borrow().next.as_ref().unwrap() != op2
                && op2b.borrow().next.as_ref().unwrap() != op1
            {
                op2b = op2b.borrow().next.clone().unwrap();
            }
            if op2b.borrow().next.as_ref().unwrap() == op2 || op2b.borrow().next.as_ref().unwrap() == op1 {
                return false;
            }

            let (left, right);
            if !self.get_overlap(op1.borrow().pt.x, op1b.borrow().pt.x, op2.borrow().pt.x, op2b.borrow().pt.x, &mut left, &mut right) {
                return false;
            }

            let (pt, discard_left_side);
            if op1.borrow().pt.x >= left && op1.borrow().pt.x <= right {
                pt = op1.borrow().pt;
                discard_left_side = op1.borrow().pt.x > op1b.borrow().pt.x;
            } else if op2.borrow().pt.x >= left && op2.borrow().pt.x <= right {
                pt = op2.borrow().pt;
                discard_left_side = op2.borrow().pt.x > op2b.borrow().pt.x;
            } else if op1b.borrow().pt.x >= left && op1b.borrow().pt.x <= right {
                pt = op1b.borrow().pt;
                discard_left_side = op1b.borrow().pt.x > op1.borrow().pt.x;
            } else {
                pt = op2b.borrow().pt;
                discard_left_side = op2b.borrow().pt.x > op2.borrow().pt.x;
            }
            j.out_pt1 = Some(op1.clone());
            j.out_pt2 = Some(op2.clone());
            return self.join_horz(&op1, op1b, &op2, op2b, pt, discard_left_side);
        } else {
            op1b = op1.borrow().next.clone().unwrap();
            while op1b.borrow().pt == op1.borrow().pt && op1b != op1 {
                op1b = op1b.borrow().next.clone().unwrap();
            }
            let reverse1 = op1b.borrow().pt.y > op1.borrow().pt.y
                || !self.slopes_equal(op1.borrow().pt, op1b.borrow().pt, j.off_pt, self.base.use_full_range);
            if reverse1 {
                op1b = op1.borrow().prev.clone().unwrap();
                while op1b.borrow().pt == op1.borrow().pt && op1b != op1 {
                    op1b = op1b.borrow().prev.clone().unwrap();
                }
                if op1b.borrow().pt.y > op1.borrow().pt.y
                    || !self.slopes_equal(op1.borrow().pt, op1b.borrow().pt, j.off_pt, self.base.use_full_range)
                {
                    return false;
                }
            }
            op2b = op2.borrow().next.clone().unwrap();
            while op2b.borrow().pt == op2.borrow().pt && op2b != op2 {
                op2b = op2b.borrow().next.clone().unwrap();
            }
            let reverse2 = op2b.borrow().pt.y > op2.borrow().pt.y
                || !self.slopes_equal(op2.borrow().pt, op2b.borrow().pt, j.off_pt, self.base.use_full_range);
            if reverse2 {
                op2b = op2.borrow().prev.clone().unwrap();
                while op2b.borrow().pt == op2.borrow().pt && op2b != op2 {
                    op2b = op2b.borrow().prev.clone().unwrap();
                }
                if op2b.borrow().pt.y > op2.borrow().pt.y
                    || !self.slopes_equal(op2.borrow().pt, op2b.borrow().pt, j.off_pt, self.base.use_full_range)
                {
                    return false;
                }
            }

            if op1b == op1 || op2b == op2 || op1b == op2b || (out_rec1 == out_rec2 && reverse1 == reverse2) {
                return false;
            }

            if reverse1 {
                op1b = self.dup_out_pt(&op1, false);
                op2b = self.dup_out_pt(&op2, true);
                op1.borrow_mut().prev = Some(op2.clone());
                op2.borrow_mut().next = Some(op1.clone());
                op1b.borrow_mut().next = Some(op2b.clone());
                op2b.borrow_mut().prev = Some(op1b.clone());
                j.out_pt1 = Some(op1.clone());
                j.out_pt2 = Some(op1b.clone());
                return true;
            } else {
                op1b = self.dup_out_pt(&op1, true);
                op2b = self.dup_out_pt(&op2, false);
                op1.borrow_mut().next = Some(op2.clone());
                op2.borrow_mut().prev = Some(op1.clone());
                op1b.borrow_mut().prev = Some(op2b.clone());
                op2b.borrow_mut().next = Some(op1b.clone());
                j.out_pt1 = Some(op1.clone());
                j.out_pt2 = Some(op1b.clone());
                return true;
            }
        }
    }

    pub fn point_in_polygon(pt: IntPoint, path: &Path) -> i32 {
        // Returns 0 if false, +1 if true, -1 if pt ON polygon boundary
        // See "The Point in Polygon Problem for Arbitrary Polygons" by Hormann & Agathos
        // http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.88.5498&rep=rep1&type=pdf
        let mut result = 0;
        let cnt = path.len();
        if cnt < 3 {
            return 0;
        }
        let mut ip = path[0];
        for i in 1..=cnt {
            let ip_next = if i == cnt { path[0] } else { path[i] };
            if ip_next.y == pt.y {
                if ip_next.x == pt.x || (ip.y == pt.y && (ip_next.x > pt.x) == (ip.x < pt.x)) {
                    return -1;
                }
            }
            if (ip.y < pt.y) != (ip_next.y < pt.y) {
                if ip.x >= pt.x {
                    if ip_next.x > pt.x {
                        result = 1 - result;
                    } else {
                        let d = (ip.x - pt.x) as f64 * (ip_next.y - pt.y) as f64
                            - (ip_next.x - pt.x) as f64 * (ip.y - pt.y) as f64;
                        if d == 0.0 {
                            return -1;
                        } else if (d > 0.0) == (ip_next.y > ip.y) {
                            result = 1 - result;
                        }
                    }
                } else if ip_next.x > pt.x {
                    let d = (ip.x - pt.x) as f64 * (ip_next.y - pt.y) as f64
                        - (ip_next.x - pt.x) as f64 * (ip.y - pt.y) as f64;
                    if d == 0.0 {
                        return -1;
                    } else if (d > 0.0) == (ip_next.y > ip.y) {
                        result = 1 - result;
                    }
                }
            }
            ip = ip_next;
        }
        result
    }

    fn point_in_polygon_out_pt(pt: IntPoint, op: &OutPt) -> i32 {
        // Returns 0 if false, +1 if true, -1 if pt ON polygon boundary
        let mut result = 0;
        let start_op = op.clone();
        let ptx = pt.x;
        let pty = pt.y;
        let mut poly0x = op.pt.x;
        let mut poly0y = op.pt.y;
        let mut op = op.next.as_ref().unwrap().clone();
        loop {
            let poly1x = op.pt.x;
            let poly1y = op.pt.y;
            if poly1y == pty {
                if poly1x == ptx || (poly0y == pty && (poly1x > ptx) == (poly0x < ptx)) {
                    return -1;
                }
            }
            if (poly0y < pty) != (poly1y < pty) {
                if poly0x >= ptx {
                    if poly1x > ptx {
                        result = 1 - result;
                    } else {
                        let d = (poly0x - ptx) as f64 * (poly1y - pty) as f64
                            - (poly1x - ptx) as f64 * (poly0y - pty) as f64;
                        if d == 0.0 {
                            return -1;
                        } else if (d > 0.0) == (poly1y > poly0y) {
                            result = 1 - result;
                        }
                    }
                } else if poly1x > ptx {
                    let d = (poly0x - ptx) as f64 * (poly1y - pty) as f64
                        - (poly1x - ptx) as f64 * (poly0y - pty) as f64;
                    if d == 0.0 {
                        return -1;
                    } else if (d > 0.0) == (poly1y > poly0y) {
                        result = 1 - result;
                    }
                }
            }
            poly0x = poly1x;
            poly0y = poly1y;
            op = op.next.as_ref().unwrap().clone();
            if Rc::ptr_eq(&op, &start_op) {
                break;
            }
        }
        result
    }

    fn poly2_contains_poly1(out_pt1: &OutPt, out_pt2: &OutPt) -> bool {
        let mut op = out_pt1.clone();
        loop {
            let res = Clipper::point_in_polygon_out_pt(op.pt, out_pt2);
            if res >= 0 {
                return res > 0;
            }
            op = op.next.as_ref().unwrap().clone();
            if Rc::ptr_eq(&op, &out_pt1) {
                break;
            }
        }
        true
    }

    fn fixup_first_lefts1(&mut self, old_out_rec: &Rc<RefCell<OutRec>>, new_out_rec: &Rc<RefCell<OutRec>>) {
        for out_rec in &self.base.poly_outs {
            let first_left = Clipper::parse_first_left(&out_rec.first_left);
            if out_rec.pts.is_some() && Rc::ptr_eq(&first_left, old_out_rec) {
                if Clipper::poly2_contains_poly1(out_rec.pts.as_ref().unwrap(), new_out_rec.borrow().pts.as_ref().unwrap()) {
                    out_rec.first_left = Some(new_out_rec.clone());
                }
            }
        }
    }

    fn parse_first_left(first_left: &Option<Rc<RefCell<OutRec>>>) -> Rc<RefCell<OutRec>> {
        let mut first_left = first_left.clone();
        while let Some(ref fl) = first_left {
            if fl.borrow().pts.is_some() {
                break;
            }
            first_left = fl.borrow().first_left.clone();
        }
        first_left.unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ClipperOffset struct with empty stubs.
////////////////////////////////////////////////////////////////////////////////

/// A port of the C# ClipperOffset class.
///
/// This struct holds fields to manage offsetting parameters and intermediate
/// polygon data. The overall process involves:
///  - Storing the source and destination paths;
///  - Computing normals for each segment;
///  - Determining join styles (miter, round, square);
///  - Generating the offset (expanded or inset) polygons.
///  
/// The detailed offsetting algorithm is quite involved so that logic is left as
/// unimplemented!() stubs.
#[derive(Debug)]
pub struct ClipperOffset {
    /// The destination (offset) polygons.
    pub dest_polys: Option<Paths>,
    /// The source polygon (a single path) being offset.
    pub src_poly: Option<Path>,
    /// An intermediate destination polygon, if needed.
    pub dest_poly: Option<Path>,
    /// Normals computed for each edge of the source polygon.
    pub normals: Vec<DoublePoint>,
    /// The offset distance (set later during execution).
    pub delta: f64,
    /// Sine of the angle between normals (used for miter join calculations).
    pub sin_a: f64,
    /// A working sine value.
    pub sin: f64,
    /// A working cosine value.
    pub cos: f64,
    /// The miter limit.
    pub miter_lim: f64,
    /// The number of steps per radian when approximating circular arcs.
    pub steps_per_rad: f64,
    /// The lowest point in the source polygon.
    pub lowest: Option<IntPoint>,
    /// A PolyNode structure for building the offset output.
    pub poly_nodes: PolyNode,
    /// The arc tolerance which controls how finely curves are approximated.
    pub arc_tolerance: f64,
    /// A duplicate of the miter limit (for compatibility with the C# API).
    pub miter_limit: f64,
}

// Constants used by ClipperOffset.
const TWO_PI: f64 = PI * 2.0;
const DEF_ARC_TOLERANCE: f64 = 0.25;

impl ClipperOffset {
    /// Constructs a new ClipperOffset instance.
    ///
    /// # Arguments
    ///
    /// * `miter_limit` - The miter limit for joins.
    /// * `arc_tolerance` - The tolerance used to approximate arcs.
    pub fn new(miter_limit: f64, arc_tolerance: f64) -> Self {
        Self {
            dest_polys: None,
            src_poly: None,
            dest_poly: None,
            normals: Vec::new(),
            delta: 0.0,
            sin_a: 0.0,
            sin: 0.0,
            cos: 0.0,
            miter_lim: miter_limit,
            steps_per_rad: 0.0,
            lowest: None,
            poly_nodes: PolyNode::new(),
            arc_tolerance,
            miter_limit,
        }
    }

    /// Clears the internal state of the ClipperOffset.
    pub fn clear(&mut self) {
        self.lowest = None;
        self.poly_nodes = PolyNode::new();
    }

    /// Sets the offset distance.
    pub fn set_offset(&mut self, offset: f64) {
        self.delta = offset;
    }

    /// Adds the specified paths to be offset.
    ///
    /// # Arguments
    ///
    /// * `_paths` - The input polygon paths.
    /// * `_join_type` - Defines how joints between segments are handled.
    /// * `_end_type` - Defines how the ends of open paths are handled.
    ///
    /// This method should translate the logic from the original C# AddPaths method,
    /// which processes input polygon paths and stores them for offsetting.
    pub fn add_paths(&mut self, _paths: &Paths, _join_type: JoinType, _end_type: EndType) {
        // TODO: Implement the logic to add multiple paths for offsetting.
        unimplemented!("Implement add_paths for ClipperOffset")
    }

    /// Executes the offsetting operation.
    ///
    /// # Arguments
    ///
    /// * `_delta` - The offset distance (positive for outward expansion,
    ///              negative for inward contraction).
    /// * `_solution` - The output where offset paths will be stored.
    ///
    /// The method should:
    ///  - Compute normals for each edge of the source polygon.
    ///  - Use the join type (miter, round, square) to determine how vertices
    ///    are joined.
    ///  - Approximate curves with stepped arcs if necessary.
    ///  - Build the destination polygons accordingly.
    pub fn execute_poly_tree(&mut self, solution: &mut PolyTree, delta: f64) {
        solution.clear();
        self.fix_orientations();
        self.do_offset(delta);

        // Now clean up 'corners' ...
        let mut clpr = Clipper::new(0);
        clpr.add_paths(&self.dest_polys.as_ref().unwrap(), PolyType::Subject, true);
        if delta > 0.0 {
            clpr.execute_poly_tree(
                ClipType::Union,
                solution,
                PolyFillType::Positive,
                PolyFillType::Positive,
            );
        } else {
            let r = ClipperBase::get_bounds(&self.dest_polys.as_ref().unwrap());
            let mut outer = Path::new();
            outer.push(IntPoint::new(r.left - 10, r.bottom + 10));
            outer.push(IntPoint::new(r.right + 10, r.bottom + 10));
            outer.push(IntPoint::new(r.right + 10, r.top - 10));
            outer.push(IntPoint::new(r.left - 10, r.top - 10));

            clpr.add_path(&outer, PolyType::Subject, true);
            clpr.reverse_solution = true;
            clpr.execute_poly_tree(
                ClipType::Union,
                solution,
                PolyFillType::Negative,
                PolyFillType::Negative,
            );

            // Remove the outer PolyNode rectangle ...
            if solution.child_count() == 1 && solution.root.childs[0].child_count() > 0 {
                let outer_node = solution.root.childs.remove(0);
                solution.root.childs = outer_node.childs;
                for child in &mut solution.root.childs {
                    child.parent = Some(Box::new(solution.root.clone()));
                }
            } else {
                solution.clear();
            }
        }
    }

    /// Internal function to perform a miter join.
    ///
    /// Given:
    /// - `pt`: the original vertex (an IntPoint),
    /// - `j`: the index of one normal edge,
    /// - `k`: the index of the adjacent normal edge,
    ///
    /// it computes the offset join point using the formula:
    ///
    ///   r = 1.0 + Dot(normals[k], normals[j])
    ///   q = delta / r
    ///   offset_pt = pt + normals[k] * q
    ///
    /// If the computed `r` is less than the miter limit then it's clamped (or you might choose a different join).
    fn do_miter(&self, j: usize, k: usize, r: f64) {
        let q = self.delta / r;
        self.dest_poly.Add(IntPoint::new(
            ClipperOffset::round(self.src_poly[j].x + (self.normals[k].x + self.normals[j].x) * q),
            ClipperOffset::round(self.src_poly[j].y + (self.normals[k].y + self.normals[j].y) * q),
        ));
    }

    /// Internal function to perform a square join.
    ///
    /// Given:
    /// - `j`: the index of one adjacent normal edge,
    /// - `k`: the index of the other adjacent normal edge,
    ///
    /// it computes two offset points:
    ///
    ///   pt1 = pt + normals[j] * delta
    ///   pt2 = pt + normals[k] * delta
    ///
    /// These two points, when added to the resulting polygon (in order),
    /// produce a square (or beveled) join.
    fn do_square(&self, j: usize, k: usize) {
        let dx = (self
            .sin_a
            .atan2(self.normals[k].x * self.normals[j].x + self.normals[k].y * self.normals[j].y)
            / 4)
        .tan();

        self.dest_poly.Add(IntPoint::new(
            ClipperOffset::round(
                self.src_poly[j].x + self.delta * (m_normals[k].x - self.normals[k].y * dx),
            ),
            ClipperOffset::round(
                self.src_poly[j].y + self.delta * (m_normals[k].y + self.normals[k].x * dx),
            ),
        ));
        self.dest_poly.Add(IntPoint::new(
            ClipperOffset::round(
                self.src_poly[j].x + self.delta * (m_normals[j].x + self.normals[j].y * dx),
            ),
            ClipperOffset::round(
                self.src_poly[j].y + self.delta * (m_normals[j].y - self.normals[j].x * dx),
            ),
        ));
    }

    fn do_round(&self, j: usize, k: usize) {
        let a = self
            .sin_a
            .atan2(self.normals[k].x * self.normals[j].x + self.normals[k].y * self.normals[j].y);
        let steps = ClipperOffset::round(self.steps_per_rad * a.abs()).max(1);

        let x = self.normals[k].x;
        let y = self.normals[k].y;
        let x2;
        for i in 0..steps {
            self.dest_poly.Add(IntPoint::new(
                ClipperOffset::round(self.src_poly[j].x + x * self.delta),
                ClipperOffset::round(self.src_poly[j].y + y * m_delta),
            ));
            x2 = x;
            x = x * self.cos - self.sin * y;
            y = x2 * self.sin + y * self.cos;
        }
        self.dest_poly.Add(IntPoint::new(
            ClipperOffset::round(m_srcPoly[j].X + m_normals[j].X * m_delta),
            ClipperOffset::round(m_srcPoly[j].Y + m_normals[j].Y * m_delta),
        ));
    }

    /// Rounds a floating-point value to the nearest integer.
    fn round(value: f64) -> CInt {
        let result = if value < 0 {
            (value - 0.5).round() as CInt
        } else {
            (value + 0.5).round() as CInt
        };
        result
    }

    fn offset_point(&mut self, j: usize, k: &mut usize, jointype: JoinType) {
        // Cross product
        self.sin_a = self.normals[*k].x * self.normals[j].y - self.normals[j].x * self.normals[*k].y;

        if (self.sin_a * self.delta).abs() < 1.0 {
            // Dot product
            let cos_a = self.normals[*k].x * self.normals[j].x + self.normals[j].y * self.normals[*k].y;
            if cos_a > 0.0 {
                // Angle ==> 0 degrees
                self.dest_poly.as_mut().unwrap().push(IntPoint::new(
                    ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].x as f64 + self.normals[*k].x * self.delta),
                    ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].y as f64 + self.normals[*k].y * self.delta),
                ));
                return;
            }
            // Else angle ==> 180 degrees
        } else if self.sin_a > 1.0 {
            self.sin_a = 1.0;
        } else if self.sin_a < -1.0 {
            self.sin_a = -1.0;
        }

        if self.sin_a * self.delta < 0.0 {
            self.dest_poly.as_mut().unwrap().push(IntPoint::new(
                ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].x as f64 + self.normals[*k].x * self.delta),
                ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].y as f64 + self.normals[*k].y * self.delta),
            ));
            self.dest_poly.as_mut().unwrap().push(self.src_poly.as_ref().unwrap()[j]);
            self.dest_poly.as_mut().unwrap().push(IntPoint::new(
                ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].x as f64 + self.normals[j].x * self.delta),
                ClipperOffset::round(self.src_poly.as_ref().unwrap()[j].y as f64 + self.normals[j].y * self.delta),
            ));
        } else {
            match jointype {
                JoinType::Miter => {
                    let r = 1.0 + (self.normals[j].x * self.normals[*k].x + self.normals[j].y * self.normals[*k].y);
                    if r >= self.miter_lim {
                        self.do_miter(j, *k, r);
                    } else {
                        self.do_square(j, *k);
                    }
                }
                JoinType::Square => self.do_square(j, *k),
                JoinType::Round => self.do_round(j, *k),
            }
        }
        *k = j;
    }
}

impl Default for ClipperOffset {
    fn default() -> Self {
        // Use the C# defaults: miter_limit = 2.0, arc_tolerance = DEF_ARC_TOLERANCE.
        Self::new(2.0, DEF_ARC_TOLERANCE)
    }
}
////////////////////////////////////////////////////////////////////////////////
// Other types and functions (e.g., Int128, helper functions, etc.) can be added
// as needed by the port.
// For brevity we leave these as stubs.
////////////////////////////////////////////////////////////////////////////////

pub type Int128 = i128;

/// A port of the C# TEdge class using Rc<RefCell<TEdge>> for pointer fields.
///
/// Fields such as next, prev, next_in_lml, etc., are now stored as
/// Option<Rc<RefCell<TEdge>>>, which enables shared ownership and interior mutability.
///
#[derive(Debug)]
pub struct TEdge {
    /// The bottom point of the edge.
    pub bot: IntPoint,
    /// The current point (updated per scanbeam).
    pub curr: IntPoint,
    /// The top point of the edge.
    pub top: IntPoint,
    /// The delta vector (top - bot).
    pub delta: IntPoint,
    /// The reciprocal of the slope (dx).
    pub dx: f64,
    /// The polygon type (subject or clip).
    pub poly_typ: PolyType,
    /// The side (left or right) for the current solution.
    pub side: EdgeSide,
    /// Winding value: 1 or -1 based on direction.
    pub wind_delta: i32,
    /// Winding count.
    pub wind_cnt: i32,
    /// Winding count for the opposite poly type.
    pub wind_cnt2: i32,
    /// An index into the output array; -1 if not yet set.
    pub out_idx: i32,
    /// Next edge in linked list.
    pub next: Option<Rc<RefCell<TEdge>>>,
    /// Previous edge in linked list.
    pub prev: Option<Rc<RefCell<TEdge>>>,
    /// Next edge in the local minima list.
    pub next_in_lml: Option<Rc<RefCell<TEdge>>>,
    /// Next edge in the active edge list.
    pub next_in_ael: Option<Rc<RefCell<TEdge>>>,
    /// Previous edge in the active edge list.
    pub prev_in_ael: Option<Rc<RefCell<TEdge>>>,
    /// Next edge in the sorted edge list.
    pub next_in_sel: Option<Rc<RefCell<TEdge>>>,
    /// Previous edge in the sorted edge list.
    pub prev_in_sel: Option<Rc<RefCell<TEdge>>>,
}

impl TEdge {
    /// Constructs a new TEdge.
    ///
    /// This constructor creates a TEdge based on bottom and top points, along with the polygon type,
    /// side, and wind delta, and computes delta and dx.
    pub fn new(
        bot: IntPoint,
        top: IntPoint,
        poly_typ: PolyType,
        side: EdgeSide,
        wind_delta: i32,
    ) -> Self {
        // Compute the delta vector.
        let delta = IntPoint::new(top.x - bot.x, top.y - bot.y);
        // Compute dx, guarding against division by zero.
        let dx = if delta.y == 0 {
            0.0
        } else {
            delta.x as f64 / delta.y as f64
        };
        Self {
            bot,
            curr: bot, // Initially, curr is set to bot.
            top,
            delta,
            dx,
            poly_typ,
            side,
            wind_delta,
            wind_cnt: 0,
            wind_cnt2: 0,
            out_idx: -1,
            next: None,
            prev: None,
            next_in_lml: None,
            next_in_ael: None,
            prev_in_ael: None,
            next_in_sel: None,
            prev_in_sel: None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IntersectNode
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct IntersectNode {
    // Using shared pointers for edges so that an edge can appear in multiple lists.
    pub edge1: Option<Rc<RefCell<TEdge>>>,
    pub edge2: Option<Rc<RefCell<TEdge>>>,
    pub pt: IntPoint,
}

///////////////////////////////////////////////////////////////////////////////
// MyIntersectNodeSort
///////////////////////////////////////////////////////////////////////////////

pub struct MyIntersectNodeSort;

impl MyIntersectNodeSort {
    /// Compares two IntersectNodes for sorting.
    /// Here we compare by the Y coordinate of the intersection point,
    /// then by the X coordinate.
    pub fn compare(node1: &IntersectNode, node2: &IntersectNode) -> Ordering {
        // Since IntPoint derives Ord, we can use its ordering.
        node1.pt.cmp(&node2.pt)
    }
}

///////////////////////////////////////////////////////////////////////////////
// LocalMinima
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LocalMinima {
    pub y: CInt,
    pub left_bound: Option<Rc<RefCell<TEdge>>>,
    pub right_bound: Option<Rc<RefCell<TEdge>>>,
    pub next: Option<Box<LocalMinima>>,
}

///////////////////////////////////////////////////////////////////////////////
// Scanbeam
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Scanbeam {
    pub y: CInt,
    pub next: Option<Box<Scanbeam>>,
}

///////////////////////////////////////////////////////////////////////////////
// Maxima
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Maxima {
    pub x: CInt,
    pub next: Option<Box<Maxima>>,
    pub prev: Option<Box<Maxima>>,
}

///////////////////////////////////////////////////////////////////////////////
// OutRec
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct OutRec {
    pub idx: i32,
    pub is_hole: bool,
    pub is_open: bool,
    // Using Rc<RefCell<OutRec>> so that multiple edges may share a reference.
    pub first_left: Option<Rc<RefCell<OutRec>>>,
    // The polygon result is stored as a circular linked list of OutPt.
    pub pts: Option<Rc<RefCell<OutPt>>>,
    pub bottom_pt: Option<Rc<RefCell<OutPt>>>,
    // For simplicity, we store the associated PolyNode by value.
    pub poly_node: Option<PolyNode>,
}

///////////////////////////////////////////////////////////////////////////////
// OutPt
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct OutPt {
    pub idx: i32,
    pub pt: IntPoint,
    // OutPt is stored in a circular doubly-linked list.
    pub next: Option<Rc<RefCell<OutPt>>>,
    pub prev: Option<Rc<RefCell<OutPt>>>,
}

///////////////////////////////////////////////////////////////////////////////
// Join
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Join {
    pub out_pt1: Option<Rc<RefCell<OutPt>>>,
    pub out_pt2: Option<Rc<RefCell<OutPt>>>,
    pub off_pt: IntPoint,
}

#[derive(Debug)]
pub struct ClipperBase {
    pub preserve_collinear: bool,
    pub minima_list: Option<Box<LocalMinima>>,
    pub current_lm: Option<Box<LocalMinima>>,
    pub edges: Vec<Vec<TEdge>>,
    pub scanbeam: Option<Box<Scanbeam>>,
    pub poly_outs: Vec<OutRec>,
    // Active edges are stored as shared pointers since they are modified during execution.
    pub active_edges: Option<Rc<RefCell<TEdge>>>,
    pub use_full_range: bool,
    pub has_open_paths: bool,
}

impl ClipperBase {
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

    /// Swaps two CInt values.
    pub fn swap(val1: &mut CInt, val2: &mut CInt) {
        std::mem::swap(val1, val2);
    }

    /// Determines if an edge is horizontal.
    pub fn is_horizontal(e: &TEdge) -> bool {
        // Implement checking if the delta y is zero.
        e.delta.y == 0
    }

    /// Checks if a point is a vertex of a polygon (OutPt).
    pub fn point_is_vertex(&self, pt: IntPoint, pp: &OutPt) -> bool {
        let mut pp2 = pp;
        loop {
            if pp2.pt == pt {
                return true;
            }
            pp2 = &pp2.next.as_ref().unwrap().borrow();
            if Rc::ptr_eq(&pp2, &pp) {
                break;
            }
        }
        false
    }

    /// Determines if a point lies on the segment [line_pt1, line_pt2].
    pub fn point_on_line_segment(
        &self,
        pt: IntPoint,
        line_pt1: IntPoint,
        line_pt2: IntPoint,
        use_full_range: bool,
    ) -> bool {
        if use_full_range {
            (pt.x == line_pt1.x && pt.y == line_pt1.y)
                || (pt.x == line_pt2.x && pt.y == line_pt2.y)
                || ((pt.x > line_pt1.x) == (pt.x < line_pt2.x)
                    && (pt.y > line_pt1.y) == (pt.y < line_pt2.y)
                    && (Int128::from(pt.x - line_pt1.x) * Int128::from(line_pt2.y - line_pt1.y)
                        == Int128::from(line_pt2.x - line_pt1.x) * Int128::from(pt.y - line_pt1.y)))
        } else {
            (pt.x == line_pt1.x && pt.y == line_pt1.y)
                || (pt.x == line_pt2.x && pt.y == line_pt2.y)
                || ((pt.x > line_pt1.x) == (pt.x < line_pt2.x)
                    && (pt.y > line_pt1.y) == (pt.y < line_pt2.y)
                    && (pt.x - line_pt1.x) * (line_pt2.y - line_pt1.y)
                        == (line_pt2.x - line_pt1.x) * (pt.y - line_pt1.y))
        }
    }

    /// Determines if a point lies on a polygon (list of OutPt) using the above helper.
    pub fn point_on_polygon(&self, pt: IntPoint, pp: &OutPt, use_full_range: bool) -> bool {
        let mut pp2 = pp;
        loop {
            if self.point_on_line_segment(pt, pp2.pt, pp2.next.as_ref().unwrap().borrow().pt, use_full_range) {
                return true;
            }
            pp2 = &pp2.next.as_ref().unwrap().borrow();
            if Rc::ptr_eq(&pp2, &pp) {
                break;
            }
        }
        false
    }

    /// Compares slopes of two edges for equality.
    pub fn slopes_equal(e1: &TEdge, e2: &TEdge, use_full_range: bool) -> bool {
        if use_full_range {
            Int128::from(e1.delta.y) * Int128::from(e2.delta.x)
                == Int128::from(e1.delta.x) * Int128::from(e2.delta.y)
        } else {
            (e1.delta.y as CInt) * (e2.delta.x as CInt)
                == (e1.delta.x as CInt) * (e2.delta.y as CInt)
        }
    }

    /// Compares slopes among three points.
    pub fn slopes_equal_points(
        pt1: IntPoint,
        pt2: IntPoint,
        pt3: IntPoint,
        use_full_range: bool,
    ) -> bool {
        if use_full_range {
            Int128::from(pt1.y - pt2.y) * Int128::from(pt2.x - pt3.x)
                == Int128::from(pt1.x - pt2.x) * Int128::from(pt2.y - pt3.y)
        } else {
            (pt1.y - pt2.y) * (pt2.x - pt3.x) - (pt1.x - pt2.x) * (pt2.y - pt3.y) == 0
        }
    }

    /// Compares slopes among four points.
    pub fn slopes_equal_points4(
        pt1: IntPoint,
        pt2: IntPoint,
        pt3: IntPoint,
        pt4: IntPoint,
        use_full_range: bool,
    ) -> bool {
        if use_full_range {
            Int128::from(pt1.y - pt2.y) * Int128::from(pt3.x - pt4.x)
                == Int128::from(pt1.x - pt2.x) * Int128::from(pt3.y - pt4.y)
        } else {
            (pt1.y - pt2.y) * (pt3.x - pt4.x) - (pt1.x - pt2.x) * (pt3.y - pt4.y) == 0
        }
    }

    /// Clears any internal state.
    pub fn clear(&mut self) {
        self.dispose_local_minima_list();
        for edge_list in &mut self.edges {
            for edge in edge_list {
                *edge = TEdge::new(IntPoint::new(0, 0), IntPoint::new(0, 0), PolyType::Subject, EdgeSide::Left, 0);
            }
            edge_list.clear();
        }
        self.edges.clear();
        self.use_full_range = false;
        self.has_open_paths = false;
    }

    /// Disposes the local minima list.
    pub fn dispose_local_minima_list(&mut self) {
        while let Some(mut lm) = self.minima_list.take() {
            self.minima_list = lm.next.take();
        }
        self.current_lm = None;
    }

    /// Tests if a point fits within a coordinate range.
    pub fn range_test(&self, pt: IntPoint, use_full_range: &mut bool) {
        if *use_full_range {
            if pt.x > HI_RANGE || pt.y > HI_RANGE || -pt.x > HI_RANGE || -pt.y > HI_RANGE {
                panic!("Coordinate outside allowed range");
            }
        } else if pt.x > LO_RANGE || pt.y > LO_RANGE || -pt.x > LO_RANGE || -pt.y > LO_RANGE {
            *use_full_range = true;
            self.range_test(pt, use_full_range);
        }
    }

    /// Initializes an edge with its next and previous references and a starting point.
    pub fn init_edge(&mut self, e: &mut TEdge, e_next: &TEdge, e_prev: &TEdge, pt: IntPoint) {
        e.next = Some(Rc::new(RefCell::new(e_next.clone())));
        e.prev = Some(Rc::new(RefCell::new(e_prev.clone())));
        e.curr = pt;
        e.out_idx = UNASSIGNED;
    }

    /// Sets additional parameters on an edge.
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

    /// Finds the next local minimum from an edge.
    pub fn find_next_loc_min(&self, mut e: &TEdge) -> TEdge {
        let mut e2: &TEdge;
        loop {
            while e.bot != e.prev.as_ref().unwrap().borrow().bot || e.curr == e.top {
                e = &e.next.as_ref().unwrap().borrow();
            }
            if e.dx != HORIZONTAL && e.prev.as_ref().unwrap().borrow().dx != HORIZONTAL {
                break;
            }
            while e.prev.as_ref().unwrap().borrow().dx == HORIZONTAL {
                e = &e.prev.as_ref().unwrap().borrow();
            }
            e2 = e;
            while e.dx == HORIZONTAL {
                e = &e.next.as_ref().unwrap().borrow();
            }
            if e.top.y == e.prev.as_ref().unwrap().borrow().bot.y {
                continue; // just an intermediate horizontal
            }
            if e2.prev.as_ref().unwrap().borrow().bot.x < e.bot.x {
                e = e2;
            }
            break;
        }
        e.clone()
    }

    /// Processes an edge bound.
    pub fn process_bound(&mut self, mut e: Rc<RefCell<TEdge>>, left_bound_is_forward: bool) -> Rc<RefCell<TEdge>> {
        let mut e_start: Rc<RefCell<TEdge>>;
        let mut result = e.clone();
        let mut horz: Rc<RefCell<TEdge>>;

        if result.borrow().out_idx == SKIP {
            // Check if there are edges beyond the skip edge in the bound and if so
            // create another LocMin and call ProcessBound once more.
            e = result.clone();
            if left_bound_is_forward {
                while e.borrow().top.y == e.borrow().next.as_ref().unwrap().borrow().bot.y {
                    e = e.borrow().next.as_ref().unwrap().clone();
                }
                while e != result && e.borrow().dx == HORIZONTAL {
                    e = e.borrow().prev.as_ref().unwrap().clone();
                }
            } else {
                while e.borrow().top.y == e.borrow().prev.as_ref().unwrap().borrow().bot.y {
                    e = e.borrow().prev.as_ref().unwrap().clone();
                }
                while e != result && e.borrow().dx == HORIZONTAL {
                    e = e.borrow().next.as_ref().unwrap().clone();
                }
            }
            if e == result {
                if left_bound_is_forward {
                    result = e.borrow().next.as_ref().unwrap().clone();
                } else {
                    result = e.borrow().prev.as_ref().unwrap().clone();
                }
            } else {
                // There are more edges in the bound beyond result starting with e.
                if left_bound_is_forward {
                    e = result.borrow().next.as_ref().unwrap().clone();
                } else {
                    e = result.borrow().prev.as_ref().unwrap().clone();
                }
                let mut loc_min = LocalMinima {
                    next: None,
                    y: e.borrow().bot.y,
                    left_bound: None,
                    right_bound: Some(e.clone()),
                };
                e.borrow_mut().wind_delta = 0;
                result = self.process_bound(e, left_bound_is_forward);
                self.insert_local_minima(loc_min);
            }
            return result;
        }

        if e.borrow().dx == HORIZONTAL {
            // We need to be careful with open paths because this may not be a
            // true local minima (i.e., e may be following a skip edge).
            // Also, consecutive horizontal edges may start heading left before going right.
            if left_bound_is_forward {
                e_start = e.borrow().prev.as_ref().unwrap().clone();
            } else {
                e_start = e.borrow().next.as_ref().unwrap().clone();
            }
            if e_start.borrow().dx == HORIZONTAL {
                // i.e., an adjoining horizontal skip edge.
                if e_start.borrow().bot.x != e.borrow().bot.x && e_start.borrow().top.x != e.borrow().bot.x {
                    self.reverse_horizontal(&e);
                }
            } else if e_start.borrow().bot.x != e.borrow().bot.x {
                self.reverse_horizontal(&e);
            }
        }

        e_start = e.clone();
        if left_bound_is_forward {
            while result.borrow().top.y == result.borrow().next.as_ref().unwrap().borrow().bot.y && result.borrow().next.as_ref().unwrap().borrow().out_idx != SKIP {
                result = result.borrow().next.as_ref().unwrap().clone();
            }
            if result.borrow().dx == HORIZONTAL && result.borrow().next.as_ref().unwrap().borrow().out_idx != SKIP {
                // At the top of a bound, horizontals are added to the bound
                // only when the preceding edge attaches to the horizontal's left vertex
                // unless a Skip edge is encountered when that becomes the top divide.
                horz = result.clone();
                while horz.borrow().prev.as_ref().unwrap().borrow().dx == HORIZONTAL {
                    horz = horz.borrow().prev.as_ref().unwrap().clone();
                }
                if horz.borrow().prev.as_ref().unwrap().borrow().top.x > result.borrow().next.as_ref().unwrap().borrow().top.x {
                    result = horz.borrow().prev.as_ref().unwrap().clone();
                }
            }
            while e != result {
                e.borrow_mut().next_in_lml = Some(e.borrow().next.as_ref().unwrap().clone());
                if e.borrow().dx == HORIZONTAL && e != e_start && e.borrow().bot.x != e.borrow().prev.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&e);
                }
                e = e.borrow().next.as_ref().unwrap().clone();
            }
            if e.borrow().dx == HORIZONTAL && e != e_start && e.borrow().bot.x != e.borrow().prev.as_ref().unwrap().borrow().top.x {
                self.reverse_horizontal(&e);
            }
            result = result.borrow().next.as_ref().unwrap().clone(); // Move to the edge just beyond current bound.
        } else {
            while result.borrow().top.y == result.borrow().prev.as_ref().unwrap().borrow().bot.y && result.borrow().prev.as_ref().unwrap().borrow().out_idx != SKIP {
                result = result.borrow().prev.as_ref().unwrap().clone();
            }
            if result.borrow().dx == HORIZONTAL && result.borrow().prev.as_ref().unwrap().borrow().out_idx != SKIP {
                horz = result.clone();
                while horz.borrow().next.as_ref().unwrap().borrow().dx == HORIZONTAL {
                    horz = horz.borrow().next.as_ref().unwrap().clone();
                }
                if horz.borrow().next.as_ref().unwrap().borrow().top.x == result.borrow().prev.as_ref().unwrap().borrow().top.x || horz.borrow().next.as_ref().unwrap().borrow().top.x > result.borrow().prev.as_ref().unwrap().borrow().top.x {
                    result = horz.borrow().next.as_ref().unwrap().clone();
                }
            }

            while e != result {
                e.borrow_mut().next_in_lml = Some(e.borrow().prev.as_ref().unwrap().clone());
                if e.borrow().dx == HORIZONTAL && e != e_start && e.borrow().bot.x != e.borrow().next.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&e);
                }
                e = e.borrow().prev.as_ref().unwrap().clone();
            }
            if e.borrow().dx == HORIZONTAL && e != e_start && e.borrow().bot.x != e.borrow().next.as_ref().unwrap().borrow().top.x {
                self.reverse_horizontal(&e);
            }
            result = result.borrow().prev.as_ref().unwrap().clone(); // Move to the edge just beyond current bound.
        }
        result
    }

    /// Resets internal state.
    pub fn reset(&mut self) {
        self.current_lm = self.minima_list.clone();
        if self.current_lm.is_none() {
            return; // nothing to process
        }

        // Reset all edges
        self.scanbeam = None;
        let mut lm = self.minima_list.clone();
        while let Some(local_minima) = lm {
            self.insert_scanbeam(local_minima.y);
            if let Some(ref left_bound) = local_minima.left_bound {
                let mut left_bound_mut = left_bound.borrow_mut();
                left_bound_mut.curr = left_bound_mut.bot;
                left_bound_mut.out_idx = UNASSIGNED;
            }
            if let Some(ref right_bound) = local_minima.right_bound {
                let mut right_bound_mut = right_bound.borrow_mut();
                right_bound_mut.curr = right_bound_mut.bot;
                right_bound_mut.out_idx = UNASSIGNED;
            }
            lm = local_minima.next;
        }
        self.active_edges = None;
    }

    /// Computes bounds for the given paths.
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
        for path in &paths[i..] {
            for pt in path {
                if pt.x < result.left {
                    result.left = pt.x;
                } else if pt.x > result.right {
                    result.right = pt.x;
                }
                if pt.y < result.top {
                    result.top = pt.y;
                } else if pt.y > result.bottom {
                    result.bottom = pt.y;
                }
            }
        }
        result
    }

    /// Inserts a scanbeam value into the scanbeam list.
    pub fn insert_scanbeam(&mut self, y: CInt) {
        // Single-linked list: sorted descending, ignoring duplicates.
        if self.scanbeam.is_none() {
            self.scanbeam = Some(Box::new(Scanbeam { y, next: None }));
        } else if y > self.scanbeam.as_ref().unwrap().y {
            let new_sb = Box::new(Scanbeam {
                y,
                next: self.scanbeam.take(),
            });
            self.scanbeam = Some(new_sb);
        } else {
            let mut sb2 = self.scanbeam.as_mut().unwrap();
            while let Some(ref next) = sb2.next {
                if y <= next.y {
                    sb2 = sb2.next.as_mut().unwrap();
                } else {
                    break;
                }
            }
            if y == sb2.y {
                return; // Ignore duplicates
            }
            let new_sb = Box::new(Scanbeam {
                y,
                next: sb2.next.take(),
            });
            sb2.next = Some(new_sb);
        }
    }

    /// Pops the next scanbeam value.
    pub fn pop_scanbeam(&mut self) -> Option<CInt> {
        if let Some(scanbeam) = self.scanbeam.take() {
            let y = scanbeam.y;
            self.scanbeam = scanbeam.next;
            Some(y)
        } else {
            None
        }
    }

    /// Tests if there are local minima pending.
    pub fn local_minima_pending(&self) -> bool {
        self.current_lm.is_some()
    }

    /// Creates a new OutRec for storing polygon output.
    pub fn create_out_rec(&mut self) -> OutRec {
        let mut result = OutRec {
            idx: UNASSIGNED,
            is_hole: false,
            is_open: false,
            first_left: None,
            pts: None,
            bottom_pt: None,
            poly_node: None,
        };
        self.poly_outs.push(result);
        result.idx = (self.poly_outs.len() - 1) as i32;
        result
    }

    /// Disposes an OutRec at the specified index.
    pub fn dispose_out_rec(&mut self, index: usize) {
        if let Some(out_rec) = self.poly_outs.get_mut(index) {
            out_rec.pts = None;
            *out_rec = OutRec {
                idx: UNASSIGNED,
                is_hole: false,
                is_open: false,
                first_left: None,
                pts: None,
                bottom_pt: None,
                poly_node: None,
            };
            self.poly_outs.remove(index);
        }
    }

    /// Updates an edge into the active edge list.
    pub fn update_edge_into_ael(&mut self, e: &mut TEdge) {
        if e.next_in_lml.is_none() {
            panic!("UpdateEdgeIntoAEL: invalid call");
        }
        let ael_prev = e.prev_in_ael.clone();
        let ael_next = e.next_in_ael.clone();
        e.next_in_lml.as_mut().unwrap().borrow_mut().out_idx = e.out_idx;
        if let Some(ref ael_prev) = ael_prev {
            ael_prev.borrow_mut().next_in_ael = e.next_in_lml.clone();
        } else {
            self.active_edges = e.next_in_lml.clone();
        }
        if let Some(ref ael_next) = ael_next {
            ael_next.borrow_mut().prev_in_ael = e.next_in_lml.clone();
        }
        let next_in_lml = e.next_in_lml.clone().unwrap();
        next_in_lml.borrow_mut().side = e.side;
        next_in_lml.borrow_mut().wind_delta = e.wind_delta;
        next_in_lml.borrow_mut().wind_cnt = e.wind_cnt;
        next_in_lml.borrow_mut().wind_cnt2 = e.wind_cnt2;
        *e = next_in_lml.borrow().clone();
        e.curr = e.bot;
        e.prev_in_ael = ael_prev;
        e.next_in_ael = ael_next;
        if !Self::is_horizontal(e) {
            self.insert_scanbeam(e.top.y);
        }
    }

    /// Swaps positions of two edges in the active edge list.
    pub fn swap_positions_in_ael(&mut self, edge1: &mut TEdge, edge2: &mut TEdge) {
        // Check that one or other edge hasn't already been removed from AEL ...
        if Rc::ptr_eq(&edge1.next_in_ael, &edge1.prev_in_ael) || Rc::ptr_eq(&edge2.next_in_ael, &edge2.prev_in_ael) {
            return;
        }

        if Rc::ptr_eq(&edge1.next_in_ael, &Some(Rc::new(RefCell::new(edge2.clone())))) {
            let next = edge2.next_in_ael.clone();
            if let Some(ref next) = next {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            let prev = edge1.prev_in_ael.clone();
            if let Some(ref prev) = prev {
                prev.borrow_mut().next_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            }
            edge2.prev_in_ael = prev;
            edge2.next_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            edge1.prev_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            edge1.next_in_ael = next;
        } else if Rc::ptr_eq(&edge2.next_in_ael, &Some(Rc::new(RefCell::new(edge1.clone())))) {
            let next = edge1.next_in_ael.clone();
            if let Some(ref next) = next {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            }
            let prev = edge2.prev_in_ael.clone();
            if let Some(ref prev) = prev {
                prev.borrow_mut().next_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            edge1.prev_in_ael = prev;
            edge1.next_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            edge2.prev_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            edge2.next_in_ael = next;
        } else {
            let next = edge1.next_in_ael.clone();
            let prev = edge1.prev_in_ael.clone();
            edge1.next_in_ael = edge2.next_in_ael.clone();
            if let Some(ref next) = edge1.borrow().next_in_ael {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            edge1.prev_in_ael = edge2.prev_in_ael.clone();
            if let Some(ref prev) = edge1.borrow().prev_in_ael {
                prev.borrow_mut().next_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            edge2.next_in_ael = next;
            if let Some(ref next) = edge2.borrow().next_in_ael {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            }
            edge2.prev_in_ael = prev;
            if let Some(ref prev) = edge2.borrow().prev_in_ael {
                prev.borrow_mut().next_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            }
        }

        if edge1.prev_in_ael.is_none() {
            self.active_edges = Some(Rc::new(RefCell::new(edge1.clone())));
        } else if edge2.prev_in_ael.is_none() {
            self.active_edges = Some(Rc::new(RefCell::new(edge2.clone())));
        }
    }

    /// Deletes an edge from the active edge list.
    pub fn delete_from_ael(&mut self, e: &TEdge) {
        let ael_prev = e.prev_in_ael.clone();
        let ael_next = e.next_in_ael.clone();
        if ael_prev.is_none() && ael_next.is_none() && !Rc::ptr_eq(&Some(Rc::new(RefCell::new(e.clone()))), &self.active_edges) {
            return; // already deleted
        }
        if let Some(ref ael_prev) = ael_prev {
            ael_prev.borrow_mut().next_in_ael = ael_next.clone();
        } else {
            self.active_edges = ael_next.clone();
        }
        if let Some(ref ael_next) = ael_next {
            ael_next.borrow_mut().prev_in_ael = ael_prev.clone();
        }
        e.next_in_ael = None;
        e.prev_in_ael = None;
    }

    /// Checks if a value is near zero within a tolerance.
    pub fn near_zero(val: f64) -> bool {
        (val > -TOLERANCE) && (val < TOLERANCE)
    }

    // Additional helper functions from the C# ClipperBase would be added here.
    pub fn add_path(&mut self, pg: &Path, poly_type: PolyType, closed: bool) -> bool {
        // Check for open paths in clip polygons.
        if !closed && poly_type == PolyType::Clip {
            panic!("AddPath: Open paths must be subject.");
        }

        let mut high_i = pg.len() as i32 - 1;
        if closed {
            while high_i > 0 && pg[high_i as usize] == pg[0] {
                high_i -= 1;
            }
        }
        while high_i > 0 && pg[high_i as usize] == pg[(high_i - 1) as usize] {
            high_i -= 1;
        }
        if (closed && high_i < 2) || (!closed && high_i < 1) {
            return false;
        }

        // Create a new edge array.
        let mut edges: Vec<TEdge> = Vec::with_capacity((high_i + 1) as usize);
        for _ in 0..=high_i {
            edges.push(TEdge::new(IntPoint::new(0, 0), IntPoint::new(0, 0), poly_type, EdgeSide::Left, 0));
        }

        let mut is_flat = true;

        // 1. Basic (first) edge initialization.
        edges[1].curr = pg[1];
        self.range_test(pg[0], &mut self.use_full_range);
        self.range_test(pg[high_i as usize], &mut self.use_full_range);
        self.init_edge(&mut edges[0], &edges[1], &edges[high_i as usize], pg[0]);
        self.init_edge(&mut edges[high_i as usize], &edges[0], &edges[(high_i - 1) as usize], pg[high_i as usize]);
        for i in (1..high_i).rev() {
            self.range_test(pg[i as usize], &mut self.use_full_range);
            self.init_edge(&mut edges[i as usize], &edges[(i + 1) as usize], &edges[(i - 1) as usize], pg[i as usize]);
        }
        let mut e_start = Rc::new(RefCell::new(edges[0].clone()));

        // 2. Remove duplicate vertices, and (when closed) collinear edges.
        let mut e = e_start.clone();
        let mut e_loop_stop = e_start.clone();
        loop {
            if e.borrow().curr == e.borrow().next.as_ref().unwrap().borrow().curr && (closed || e.borrow().next.as_ref().unwrap() != e_start) {
                if Rc::ptr_eq(&e, &e.borrow().next.as_ref().unwrap()) {
                    break;
                }
                if Rc::ptr_eq(&e, &e_start) {
                    e_start = e.borrow().next.as_ref().unwrap().clone();
                }
                e = self.remove_edge(e);
                e_loop_stop = e.clone();
                continue;
            }
            if Rc::ptr_eq(&e.borrow().prev.as_ref().unwrap(), &e.borrow().next.as_ref().unwrap()) {
                break; // Only two vertices.
            } else if closed && self.slopes_equal_points(e.borrow().prev.as_ref().unwrap().borrow().curr, e.borrow().curr, e.borrow().next.as_ref().unwrap().borrow().curr, self.use_full_range) && (!self.preserve_collinear || !self.pt2_is_between_pt1_and_pt3(e.borrow().prev.as_ref().unwrap().borrow().curr, e.borrow().curr, e.borrow().next.as_ref().unwrap().borrow().curr)) {
                // Collinear edges are allowed for open paths but in closed paths
                // the default is to merge adjacent collinear edges into a single edge.
                // However, if the PreserveCollinear property is enabled, only overlapping
                // collinear edges (i.e., spikes) will be removed from closed paths.
                if Rc::ptr_eq(&e, &e_start) {
                    e_start = e.borrow().next.as_ref().unwrap().clone();
                }
                e = self.remove_edge(e);
                e = e.borrow().prev.as_ref().unwrap().clone();
                e_loop_stop = e.clone();
                continue;
            }
            e = e.borrow().next.as_ref().unwrap().clone();
            if Rc::ptr_eq(&e, &e_loop_stop) || (!closed && Rc::ptr_eq(&e.borrow().next.as_ref().unwrap(), &e_start)) {
                break;
            }
        }

        if (!closed && Rc::ptr_eq(&e, &e.borrow().next.as_ref().unwrap())) || (closed && Rc::ptr_eq(&e.borrow().prev.as_ref().unwrap(), &e.borrow().next.as_ref().unwrap())) {
            return false;
        }

        if !closed {
            self.has_open_paths = true;
            e_start.borrow_mut().prev.as_ref().unwrap().borrow_mut().out_idx = SKIP;
        }

        // 3. Do second stage of edge initialization.
        e = e_start.clone();
        loop {
            self.init_edge2(&mut e.borrow_mut(), poly_type);
            e = e.borrow().next.as_ref().unwrap().clone();
            if is_flat && e.borrow().curr.y != e_start.borrow().curr.y {
                is_flat = false;
            }
            if Rc::ptr_eq(&e, &e_start) {
                break;
            }
        }

        // 4. Finally, add edge bounds to LocalMinima list.
        if is_flat {
            if closed {
                return false;
            }
            e.borrow_mut().prev.as_ref().unwrap().borrow_mut().out_idx = SKIP;
            let mut loc_min = LocalMinima {
                next: None,
                y: e.borrow().bot.y,
                left_bound: None,
                right_bound: Some(e.clone()),
            };
            loc_min.right_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Right;
            loc_min.right_bound.as_ref().unwrap().borrow_mut().wind_delta = 0;
            loop {
                if e.borrow().bot.x != e.borrow().prev.as_ref().unwrap().borrow().top.x {
                    self.reverse_horizontal(&e);
                }
                if e.borrow().next.as_ref().unwrap().borrow().out_idx == SKIP {
                    break;
                }
                e.borrow_mut().next_in_lml = Some(e.borrow().next.as_ref().unwrap().clone());
                e = e.borrow().next.as_ref().unwrap().clone();
            }
            self.insert_local_minima(loc_min);
            self.edges.push(edges);
            return true;
        }

        self.edges.push(edges);
        let mut left_bound_is_forward;
        let mut e_min: Option<Rc<RefCell<TEdge>>> = None;

        if e.borrow().prev.as_ref().unwrap().borrow().bot == e.borrow().prev.as_ref().unwrap().borrow().top {
            e = e.borrow().next.as_ref().unwrap().clone();
        }

        loop {
            e = self.find_next_loc_min(&e.borrow());
            if Some(e.clone()) == e_min {
                break;
            } else if e_min.is_none() {
                e_min = Some(e.clone());
            }

            let mut loc_min = LocalMinima {
                next: None,
                y: e.borrow().bot.y,
                left_bound: None,
                right_bound: None,
            };
            if e.borrow().dx < e.borrow().prev.as_ref().unwrap().borrow().dx {
                loc_min.left_bound = Some(e.borrow().prev.as_ref().unwrap().clone());
                loc_min.right_bound = Some(e.clone());
                left_bound_is_forward = false;
            } else {
                loc_min.left_bound = Some(e.clone());
                loc_min.right_bound = Some(e.borrow().prev.as_ref().unwrap().clone());
                left_bound_is_forward = true;
            }
            loc_min.left_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Left;
            loc_min.right_bound.as_ref().unwrap().borrow_mut().side = EdgeSide::Right;

            if !closed {
                loc_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = 0;
            } else if Rc::ptr_eq(&loc_min.left_bound.as_ref().unwrap().borrow().next.as_ref().unwrap(), &loc_min.right_bound.as_ref().unwrap()) {
                loc_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = -1;
            } else {
                loc_min.left_bound.as_ref().unwrap().borrow_mut().wind_delta = 1;
            }
            loc_min.right_bound.as_ref().unwrap().borrow_mut().wind_delta = -loc_min.left_bound.as_ref().unwrap().borrow().wind_delta;

            e = self.process_bound(loc_min.left_bound.as_ref().unwrap().clone(), left_bound_is_forward);
            if e.borrow().out_idx == SKIP {
                e = self.process_bound(e.clone(), left_bound_is_forward);
            }

            let mut e2 = self.process_bound(loc_min.right_bound.as_ref().unwrap().clone(), !left_bound_is_forward);
            if e2.borrow().out_idx == SKIP {
                e2 = self.process_bound(e2.clone(), !left_bound_is_forward);
            }

            if loc_min.left_bound.as_ref().unwrap().borrow().out_idx == SKIP {
                loc_min.left_bound = None;
            } else if loc_min.right_bound.as_ref().unwrap().borrow().out_idx == SKIP {
                loc_min.right_bound = None;
            }
            self.insert_local_minima(loc_min);
            if !left_bound_is_forward {
                e = e2;
            }
        }
        true
    }

    /// Adds multiple paths to the ClipperBase.
    pub fn add_paths(&mut self, ppg: &Paths, poly_type: PolyType, closed: bool) -> bool {
        let mut result = false;
        for path in ppg {
            if self.add_path(path, poly_type, closed) {
                result = true;
            }
        }
        result
    }

    /// Determines if pt2 is between pt1 and pt3.
    pub fn pt2_is_between_pt1_and_pt3(&self, pt1: IntPoint, pt2: IntPoint, pt3: IntPoint) -> bool {
        if (pt1 == pt3) || (pt1 == pt2) || (pt3 == pt2) {
            false
        } else if pt1.x != pt3.x {
            (pt2.x > pt1.x) == (pt2.x < pt3.x)
        } else {
            (pt2.y > pt1.y) == (pt2.y < pt3.y)
        }
    }

    /// Removes an edge from the double-linked list without removing it from memory.
    pub fn remove_edge(&self, e: Rc<RefCell<TEdge>>) -> Rc<RefCell<TEdge>> {
        let prev = e.borrow().prev.as_ref().unwrap().clone();
        let next = e.borrow().next.as_ref().unwrap().clone();
        prev.borrow_mut().next = Some(next.clone());
        next.borrow_mut().prev = Some(prev.clone());
        e.borrow_mut().prev = None; // Flag as removed
        next
    }

    /// Sets the delta and dx values for an edge.
    pub fn set_dx(&self, e: &mut TEdge) {
        e.delta.x = e.top.x - e.bot.x;
        e.delta.y = e.top.y - e.bot.y;
        if e.delta.y == 0 {
            e.dx = HORIZONTAL;
        } else {
            e.dx = e.delta.x as f64 / e.delta.y as f64;
        }
    }

    /// Inserts a local minima into the minima list.
    pub fn insert_local_minima(&mut self, new_lm: LocalMinima) {
        if self.minima_list.is_none() {
            self.minima_list = Some(Box::new(new_lm));
        } else if new_lm.y >= self.minima_list.as_ref().unwrap().y {
            let mut new_lm_box = Box::new(new_lm);
            new_lm_box.next = self.minima_list.take();
            self.minima_list = Some(new_lm_box);
        } else {
            let mut tmp_lm = self.minima_list.as_mut().unwrap();
            while let Some(ref next) = tmp_lm.next {
                if new_lm.y >= next.y {
                    break;
                }
                tmp_lm = tmp_lm.next.as_mut().unwrap();
            }
            let mut new_lm_box = Box::new(new_lm);
            new_lm_box.next = tmp_lm.next.take();
            tmp_lm.next = Some(new_lm_box);
        }
    }

    // TODO: maybe build it the same way as the C# version?
    /// Pops the next local minima if it matches the given Y coordinate.
    pub fn pop_local_minima(&mut self, y: CInt) -> Option<LocalMinima> {
        if let Some(ref current_lm) = self.current_lm {
            if current_lm.y == y {
                let mut current = self.current_lm.take();
                self.current_lm = current.as_mut().unwrap().next.take();
                return Some(*current.unwrap());
            }
        }
        None
    }

    /// Reverses the horizontal edge by swapping its top and bottom x coordinates.
    pub fn reverse_horizontal(&self, e: &Rc<RefCell<TEdge>>) {
        let mut e_mut = e.borrow_mut();
        Self::swap(&mut e_mut.top.x, &mut e_mut.bot.x);
    }
}
