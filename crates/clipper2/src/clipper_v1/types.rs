use super::poly::*;
use super::tedge::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

/// Alias for 64-bit integers.
pub type CInt = i64;

// Missing constants from the C# version:
pub const HORIZONTAL: f64 = -3.4E38;
pub const SKIP: i32 = -2;
pub const UNASSIGNED: i32 = -1;
pub const TOLERANCE: f64 = 1.0E-20;

pub const HI_RANGE: CInt = 0x3FFFFFFFFFFFFFFF;
pub const LO_RANGE: CInt = 0x3FFFFFFF;

const SCALE: f64 = 1e7;

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

/// Enum representing the type of node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Any,
    Open,
    Closed,
}

///////////////////////////////////////////////////////////////////////////////
// IntersectNode
// TODO: This is a placeholder for now.
// We need to implement this struct and its sorting logic.
// Missing TEdge 1 and 2 from the C# version.
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct IntersectNode {
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

#[derive(Debug, Clone)]
pub struct LocalMinima {
    pub y: CInt,
    pub left_bound: Option<Rc<RefCell<TEdge>>>,
    pub right_bound: Option<Rc<RefCell<TEdge>>>,
    pub next: Option<Box<LocalMinima>>,
}

///////////////////////////////////////////////////////////////////////////////
// Scanbeam
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Scanbeam {
    pub y: CInt,
    pub next: Option<Box<Scanbeam>>,
}

///////////////////////////////////////////////////////////////////////////////
// Maxima
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Maxima {
    pub x: CInt,
    pub next: Option<Box<Maxima>>,
    pub prev: Option<Box<Maxima>>,
}

///////////////////////////////////////////////////////////////////////////////
// OutPt (Output Point)
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct OutPt {
    pub idx: i32,
    pub pt: IntPoint,
    pub next: Option<Rc<RefCell<OutPt>>>,
    pub prev: Option<Rc<RefCell<OutPt>>>,
}

///////////////////////////////////////////////////////////////////////////////
// OutRec (Output Record)
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct OutRec {
    pub idx: i32,
    pub is_hole: bool,
    pub is_open: bool,
    pub first_left: Option<Rc<RefCell<OutRec>>>,
    pub pts: Option<Rc<RefCell<OutPt>>>,
    pub bottom_pt: Option<Rc<RefCell<OutPt>>>,
    pub poly_node: Option<PolyNode>,
}

impl Default for OutRec {
    fn default() -> Self {
        Self {
            idx: UNASSIGNED,
            is_hole: false,
            is_open: false,
            first_left: None,
            pts: None,
            bottom_pt: None,
            poly_node: None,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Join
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Join {
    pub out_pt1: Option<Rc<RefCell<OutPt>>>,
    pub out_pt2: Option<Rc<RefCell<OutPt>>>,
    pub off_pt: IntPoint,
}
