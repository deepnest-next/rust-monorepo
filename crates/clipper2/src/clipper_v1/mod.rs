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
        if self.has_open_paths {
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
    /// This function should process scanbeams, handle intersections, and update the active edge list.
    fn execute_internal(&mut self) -> bool {
        // Reset state.
        self.base.reset();
        self.sorted_edges = None;
        self.maxima = None;

        // The main loop processes scanbeams until none remain.
        // (A proper translation would iterate over local minima and call routines to process horizontals etc.)
        unimplemented!("Implement ExecuteInternal: core algorithm loop");
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
        // Clear the poly tree and add each valid PolyNode.
        polytree.clear();
        // For each OutRec, create a corresponding PolyNode and add it to the PolyTree.
        for out_rec in &self.base.poly_outs {
            if let Some(ref _pts) = out_rec.pts {
                // Create a new PolyNode and assign its polygon (to be refined).
                let mut node = PolyNode::new();
                // A helper that extracts the polygon from the OutRec.
                // (Implement proper conversion from OutRec to PolyNode.)
                // Here we simply assign the polygon for demonstration.
                // node.m_polygon = extract_polygon(out_rec);
                polytree.root.add_child(node);
            }
        }
    }

    /// Disposes internal OutRec point lists.
    fn dispose_all_poly_pts(&mut self) {
        self.base.poly_outs.clear();
    }

    /// Adds a join between two output points with an offset.
    fn add_join(&mut self, op1: &OutPt, op2: &OutPt, off_pt: IntPoint) {
        self.joins.push(Join {
            out_pt1: Some(Rc::new(RefCell::new(op1.clone()))),
            out_pt2: Some(Rc::new(RefCell::new(op2.clone()))),
            off_pt,
        });
    }

    /// Adds a ghost join for horizontal edges.
    fn add_ghost_join(&mut self, op: &OutPt, off_pt: IntPoint) {
        self.ghost_joins.push(Join {
            out_pt1: Some(Rc::new(RefCell::new(op.clone()))),
            out_pt2: None,
            off_pt,
        });
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

#[derive(Debug)]
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
            if let Some(ref next) = edge1.next_in_ael {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            edge1.prev_in_ael = edge2.prev_in_ael.clone();
            if let Some(ref prev) = edge1.prev_in_ael {
                prev.borrow_mut().next_in_ael = Some(Rc::new(RefCell::new(edge1.clone())));
            }
            edge2.next_in_ael = next;
            if let Some(ref next) = edge2.next_in_ael {
                next.borrow_mut().prev_in_ael = Some(Rc::new(RefCell::new(edge2.clone())));
            }
            edge2.prev_in_ael = prev;
            if let Some(ref prev) = edge2.prev_in_ael {
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
