use std::cell::RefCell;
use std::rc::Rc;
use crate::clipper_v1::types::*;

/// A port of the C# TEdge class using Rc<RefCell<TEdge>> for pointer fields.
#[derive(Debug, Clone, PartialEq)]
pub struct TEdge {
    /// The bottom point of the edge
    pub bot: IntPoint,
    /// The current point (updated per scanbeam)
    pub curr: IntPoint, 
    /// The top point of the edge
    pub top: IntPoint,
    /// The delta vector (top - bot)
    pub delta: IntPoint,
    /// The reciprocal of the slope (dx)
    pub dx: f64,
    /// The polygon type (subject or clip)
    pub poly_typ: PolyType,
    /// The side (left or right) for the current solution
    pub side: EdgeSide,
    /// Winding value: 1 or -1 based on direction
    pub wind_delta: i32,
    /// Winding count
    pub wind_cnt: i32,
    /// Winding count for the opposite poly type
    pub wind_cnt2: i32,
    /// Index into output array (-1 if not yet set)
    pub out_idx: i32,
    /// Links to adjacent edges
    pub next: Option<Rc<RefCell<TEdge>>>,
    pub prev: Option<Rc<RefCell<TEdge>>>,
    pub next_in_lml: Option<Rc<RefCell<TEdge>>>,
    pub next_in_ael: Option<Rc<RefCell<TEdge>>>,
    pub prev_in_ael: Option<Rc<RefCell<TEdge>>>,
    pub next_in_sel: Option<Rc<RefCell<TEdge>>>,
    pub prev_in_sel: Option<Rc<RefCell<TEdge>>>,
}

impl TEdge {
    /// Creates a new TEdge instance
    pub fn new() -> Self {
        Self {
            bot: IntPoint::new(0, 0),
            curr: IntPoint::new(0, 0),
            top: IntPoint::new(0, 0),
            delta: IntPoint::new(0, 0),
            dx: 0.0,
            poly_typ: PolyType::Subject,
            side: EdgeSide::Left,
            wind_delta: 0,
            wind_cnt: 0,
            wind_cnt2: 0,
            out_idx: UNASSIGNED,
            next: None,
            prev: None,
            next_in_lml: None,
            next_in_ael: None,
            prev_in_ael: None,
            next_in_sel: None,
            prev_in_sel: None,
        }
    }

    /// Initialize edge points and direction
    pub fn init(&mut self, pt_bottom: IntPoint, pt_top: IntPoint, poly_type: PolyType) {
        self.poly_typ = poly_type;
        
        if pt_top.y >= pt_bottom.y {
            self.bot = pt_bottom;
            self.top = pt_top;
        } else {
            self.bot = pt_top;
            self.top = pt_bottom;
        }
        
        self.curr = self.bot;
        self.update_delta();
    }

    /// Updates the delta and dx values based on top and bottom points
    pub fn update_delta(&mut self) {
        self.delta.x = self.top.x - self.bot.x;
        self.delta.y = self.top.y - self.bot.y;
        
        if self.delta.y == 0 {
            self.dx = HORIZONTAL;
        } else {
            self.dx = self.delta.x as f64 / self.delta.y as f64;
        }
    }

    /// Checks if the edge is horizontal
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        self.delta.y == 0
    }

    /// Gets X position at a given Y position
    #[inline]
    pub fn get_x_at_y(&self, y: CInt) -> CInt {
        if self.top.y == self.bot.y {
            return self.bot.x;
        }
        if y == self.top.y {
            return self.top.x;
        }
        if y == self.bot.y {
            return self.bot.x;
        }
        // Calculate X using the line equation
        return self.bot.x + ((y - self.bot.y) as f64 * self.dx) as CInt;
    }

    /// Copies values from another edge
    pub fn copy_from(&mut self, other: &TEdge) {
        self.bot = other.bot;
        self.curr = other.curr;
        self.top = other.top;
        self.delta = other.delta;
        self.dx = other.dx;
        self.poly_typ = other.poly_typ;
        self.side = other.side;
        self.wind_delta = other.wind_delta;
        self.wind_cnt = other.wind_cnt;
        self.wind_cnt2 = other.wind_cnt2;
        self.out_idx = other.out_idx;
    }
}

impl Default for TEdge {
    fn default() -> Self {
        Self::new()
    }
}
