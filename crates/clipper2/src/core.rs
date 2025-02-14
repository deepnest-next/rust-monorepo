// lib.rs

// Enable the optional z coordinate with: `--features "usingz"`
// Run with: cargo run --features "usingz"

use std::cmp::{max, min};
use std::fmt;
use std::ops::{Add, Sub};

/// Rounds a f64 value "away from zero" (i.e. 2.5→3, -2.5→-3)
pub fn round_away(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        (x - 0.5).ceil()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Point64
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point64 {
    pub x: i64,
    pub y: i64,
    #[cfg(feature = "usingz")]
    pub z: i64,
}

impl Point64 {
    /// Creates a new Point64 from two i64 values.
    pub fn new(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
            #[cfg(feature = "usingz")]
            z: 0,
        }
    }

    /// Creates a new Point64 with an explicit Z (only available with the "usingz" feature).
    #[cfg(feature = "usingz")]
    pub fn new_with_z(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Copy constructor.
    pub fn from_point(pt: &Self) -> Self {
        *pt
    }

    /// Creates a new Point64 from another Point64 scaled by `scale`.
    pub fn from_scaled(pt: &Self, scale: f64) -> Self {
        Self {
            x: round_away(pt.x as f64 * scale) as i64,
            y: round_away(pt.y as f64 * scale) as i64,
            #[cfg(feature = "usingz")]
            z: round_away(pt.z as f64 * scale) as i64,
        }
    }

    /// Creates a new Point64 from two f64 values using away-from-zero rounding.
    pub fn from_f64(x: f64, y: f64) -> Self {
        Self {
            x: round_away(x) as i64,
            y: round_away(y) as i64,
            #[cfg(feature = "usingz")]
            z: 0,
        }
    }

    /// Creates a new Point64 from two f64 values with an explicit z.
    #[cfg(feature = "usingz")]
    pub fn from_f64_with_z(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: round_away(x) as i64,
            y: round_away(y) as i64,
            z: round_away(z) as i64,
        }
    }

    /// Creates a new Point64 from a PointD (by rounding its x and y).
    pub fn from_pointd(pt: &PointD) -> Self {
        Self {
            x: round_away(pt.x) as i64,
            y: round_away(pt.y) as i64,
            #[cfg(feature = "usingz")]
            z: pt.z,
        }
    }

    /// Creates a new Point64 from a PointD scaled by `scale`.
    pub fn from_pointd_scaled(pt: &PointD, scale: f64) -> Self {
        Self {
            x: round_away(pt.x * scale) as i64,
            y: round_away(pt.y * scale) as i64,
            #[cfg(feature = "usingz")]
            z: pt.z,
        }
    }
}

// Implement addition and subtraction for Point64
impl Add for Point64 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            #[cfg(feature = "usingz")]
            z: self.z + other.z,
        }
    }
}

impl Sub for Point64 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            #[cfg(feature = "usingz")]
            z: self.z - other.z,
        }
    }
}

// Display: produces a trailing space as in the original C# ToString.
impl fmt::Display for Point64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "usingz")]
        {
            write!(f, "{},{},{} ", self.x, self.y, self.z)
        }
        #[cfg(not(feature = "usingz"))]
        {
            write!(f, "{},{} ", self.x, self.y)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// PointD
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointD {
    pub x: f64,
    pub y: f64,
    #[cfg(feature = "usingz")]
    pub z: i64,
}

impl PointD {
    /// Creates a new PointD from two f64 values.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            #[cfg(feature = "usingz")]
            z: 0,
        }
    }

    /// Creates a new PointD with an explicit z (only available with the "usingz" feature).
    #[cfg(feature = "usingz")]
    pub fn new_with_z(x: f64, y: f64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Copy constructor.
    pub fn from_point(pt: &Self) -> Self {
        *pt
    }

    /// Creates a new PointD from a Point64.
    pub fn from_point64(pt: &Point64) -> Self {
        Self {
            x: pt.x as f64,
            y: pt.y as f64,
            #[cfg(feature = "usingz")]
            z: pt.z,
        }
    }

    /// Creates a new PointD from a Point64 scaled by `scale`.
    pub fn from_point64_scaled(pt: &Point64, scale: f64) -> Self {
        Self {
            x: pt.x as f64 * scale,
            y: pt.y as f64 * scale,
            #[cfg(feature = "usingz")]
            z: pt.z,
        }
    }

    /// Creates a new PointD by scaling an existing PointD.
    pub fn from_scaled(pt: &Self, scale: f64) -> Self {
        Self {
            x: pt.x * scale,
            y: pt.y * scale,
            #[cfg(feature = "usingz")]
            z: pt.z,
        }
    }

    /// Negates the point.
    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
    }

    /// Formats the point using a specified precision.
    pub fn to_string_with_precision(&self, precision: usize) -> String {
        #[cfg(feature = "usingz")]
        {
            // Z is formatted as an integer.
            format!("{:.prec$},{:.prec$},{}", self.x, self.y, self.z, prec = precision)
        }
        #[cfg(not(feature = "usingz"))]
        {
            format!("{:.prec$},{:.prec$}", self.x, self.y, prec = precision)
        }
    }
}

impl fmt::Display for PointD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_precision(2))
    }
}

///////////////////////////////////////////////////////////////////////////////
// Rect64
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect64 {
    pub left: i64,
    pub top: i64,
    pub right: i64,
    pub bottom: i64,
}

impl Rect64 {
    /// Creates a new rectangle.
    pub fn new(l: i64, t: i64, r: i64, b: i64) -> Self {
        Self {
            left: l,
            top: t,
            right: r,
            bottom: b,
        }
    }

    /// Constructs a “valid” rectangle (or an invalid one) based on `is_valid`.
    pub fn new_valid(is_valid: bool) -> Self {
        if is_valid {
            Self {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            }
        } else {
            Self {
                left: i64::MAX,
                top: i64::MAX,
                right: i64::MIN,
                bottom: i64::MIN,
            }
        }
    }

    /// Returns the width.
    pub fn width(&self) -> i64 {
        self.right - self.left
    }

    /// Sets the width.
    pub fn set_width(&mut self, width: i64) {
        self.right = self.left + width;
    }

    /// Returns the height.
    pub fn height(&self) -> i64 {
        self.bottom - self.top
    }

    /// Sets the height.
    pub fn set_height(&mut self, height: i64) {
        self.bottom = self.top + height;
    }

    /// Checks if the rectangle is empty.
    pub fn is_empty(&self) -> bool {
        self.bottom <= self.top || self.right <= self.left
    }

    /// Checks if the rectangle is valid.
    pub fn is_valid(&self) -> bool {
        self.left < i64::MAX
    }

    /// Returns the midpoint of the rectangle.
    pub fn mid_point(&self) -> Point64 {
        Point64::new((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }

    /// Checks if a point is contained within the rectangle.
    pub fn contains_point(&self, pt: &Point64) -> bool {
        pt.x > self.left && pt.x < self.right && pt.y > self.top && pt.y < self.bottom
    }

    /// Checks if another rectangle is completely contained.
    pub fn contains_rect(&self, other: &Rect64) -> bool {
        other.left >= self.left
            && other.right <= self.right
            && other.top >= self.top
            && other.bottom <= self.bottom
    }

    /// Checks if two rectangles intersect.
    pub fn intersects(&self, other: &Rect64) -> bool {
        max(self.left, other.left) <= min(self.right, other.right)
            && max(self.top, other.top) <= min(self.bottom, other.bottom)
    }

    /// Returns the rectangle as a closed path (a Vec of 4 points).
    pub fn as_path(&self) -> Path64 {
        vec![
            Point64::new(self.left, self.top),
            Point64::new(self.right, self.top),
            Point64::new(self.right, self.bottom),
            Point64::new(self.left, self.bottom),
        ]
    }
}

///////////////////////////////////////////////////////////////////////////////
// RectD
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectD {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl RectD {
    /// Creates a new rectangle.
    pub fn new(l: f64, t: f64, r: f64, b: f64) -> Self {
        Self {
            left: l,
            top: t,
            right: r,
            bottom: b,
        }
    }

    /// Constructs a “valid” rectangle (or an invalid one) based on `is_valid`.
    pub fn new_valid(is_valid: bool) -> Self {
        if is_valid {
            Self {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            }
        } else {
            Self {
                left: f64::MAX,
                top: f64::MAX,
                right: -f64::MAX,
                bottom: -f64::MAX,
            }
        }
    }

    /// Returns the width.
    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    /// Sets the width.
    pub fn set_width(&mut self, width: f64) {
        self.right = self.left + width;
    }

    /// Returns the height.
    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    /// Sets the height.
    pub fn set_height(&mut self, height: f64) {
        self.bottom = self.top + height;
    }

    /// Checks if the rectangle is empty.
    pub fn is_empty(&self) -> bool {
        self.bottom <= self.top || self.right <= self.left
    }

    /// Returns the midpoint.
    pub fn mid_point(&self) -> PointD {
        PointD::new((self.left + self.right) / 2.0, (self.top + self.bottom) / 2.0)
    }

    /// Checks if a point is contained.
    pub fn contains_point(&self, pt: &PointD) -> bool {
        pt.x > self.left && pt.x < self.right && pt.y > self.top && pt.y < self.bottom
    }

    /// Checks if another rectangle is completely contained.
    pub fn contains_rect(&self, other: &RectD) -> bool {
        other.left >= self.left
            && other.right <= self.right
            && other.top >= self.top
            && other.bottom <= self.bottom
    }

    /// Checks if two rectangles intersect.
    pub fn intersects(&self, other: &RectD) -> bool {
        self.left.max(other.left) < self.right.min(other.right)
            && self.top.max(other.top) < self.bottom.min(other.bottom)
    }

    /// Returns the rectangle as a closed path.
    pub fn as_path(&self) -> PathD {
        vec![
            PointD::new(self.left, self.top),
            PointD::new(self.right, self.top),
            PointD::new(self.right, self.bottom),
            PointD::new(self.left, self.bottom),
        ]
    }
}

///////////////////////////////////////////////////////////////////////////////
// Path types (using Vec)
///////////////////////////////////////////////////////////////////////////////
pub type Path64 = Vec<Point64>;
pub type Paths64 = Vec<Path64>;
pub type PathD = Vec<PointD>;
pub type PathsD = Vec<PathD>;

// Display implementations for the path types.
pub trait DisplayPath64 {
    fn to_string(&self) -> String;
}

impl DisplayPath64 for Path64 {
    fn to_string(&self) -> String {
        self.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
    }
}

pub trait DisplayPaths64 {
    fn to_string(&self) -> String;
}

impl DisplayPaths64 for Paths64 {
    fn to_string(&self) -> String {
        self.iter().map(|p| {
            let s: String = p.iter().map(|pt| format!("{}, ", pt)).collect();
            let s = s.trim_end_matches(", ");
            format!("{}\n", s)
        }).collect()
    }
}

pub trait DisplayPathD {
    fn to_string(&self) -> String;
}

impl DisplayPathD for PathD {
    fn to_string(&self) -> String {
        // Using a default precision of 2.
        let s: String = self
            .iter()
            .map(|p| format!("{}, ", p.to_string_with_precision(2)))
            .collect();
        let s = s.trim_end_matches(", ");
        s.to_string()
    }
}

// For PathsD we define a helper method.
pub trait ToStringWithPrecision {
    fn to_string_with_precision(&self, precision: usize) -> String;
}

impl ToStringWithPrecision for PathsD {
    fn to_string_with_precision(&self, precision: usize) -> String {
        self.iter()
            .map(|p| {
                let s: String = p
                    .iter()
                    .map(|p| format!("{}, ", p.to_string_with_precision(precision)))
                    .collect();
                let s = s.trim_end_matches(", ");
                format!("{}\n", s)
            })
            .collect()
    }
}

pub struct DisplayablePathsD(pub PathsD);

impl fmt::Display for DisplayablePathsD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string_with_precision(2))
    }
}

///////////////////////////////////////////////////////////////////////////////
// Enums
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipType {
    NoClip,
    Intersection,
    Union,
    Difference,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    Subject,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
}

/// Point in polygon result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipResult {
    Inside,
    Outside,
    OnEdge,
}

// For clarity, we alias PipResult to PointInPolygonResult.
pub type PointInPolygonResult = PipResult;

///////////////////////////////////////////////////////////////////////////////
// InternalClipper module
///////////////////////////////////////////////////////////////////////////////
pub mod internal_clipper {
    use super::*;
    use std::f64;

    pub const MAX_INT64: i64 = 9223372036854775807;
    pub const MAX_COORD: i64 = MAX_INT64 / 4;
    pub const MAX_COORD_F64: f64 = MAX_COORD as f64;
    pub const MIN_COORD_F64: f64 = -(MAX_COORD as f64);
    pub const INVALID64: i64 = MAX_INT64;

    pub const FLOATING_POINT_TOLERANCE: f64 = 1E-12;
    pub const DEFAULT_MINIMUM_EDGE_LENGTH: f64 = 0.1;

    const PRECISION_RANGE_ERROR: &str = "Error: Precision is out of range.";

    #[inline(always)]
    pub fn cross_product(p1: &Point64, p2: &Point64, p3: &Point64) -> f64 {
        ((p2.x - p1.x) as f64 * (p3.y - p2.y) as f64)
            - ((p2.y - p1.y) as f64 * (p3.x - p2.x) as f64)
    }

    #[cfg(feature = "usingz")]
    pub fn set_z(path: &Path64, z: i64) -> Path64 {
        path.iter()
            .map(|pt| {
                let mut new_pt = *pt;
                new_pt.z = z;
                new_pt
            })
            .collect()
    }

    #[inline(always)]
    pub fn check_precision(precision: i32) {
        if precision < -8 || precision > 8 {
            panic!("{}", PRECISION_RANGE_ERROR);
        }
    }

    #[inline(always)]
    pub fn is_almost_zero(value: f64) -> bool {
        value.abs() <= FLOATING_POINT_TOLERANCE
    }

    #[inline(always)]
    pub fn tri_sign(x: i64) -> i64 {
        if x < 0 {
            -1
        } else if x > 1 {
            1
        } else {
            0
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct MultiplyUInt64Result {
        pub lo64: u64,
        pub hi64: u64,
    }

    #[inline(always)]
    pub fn multiply_uint64(a: u64, b: u64) -> MultiplyUInt64Result {
        let x1 = (a & 0xFFFFFFFF) * (b & 0xFFFFFFFF);
        let x2 = (a >> 32) * (b & 0xFFFFFFFF) + (x1 >> 32);
        let x3 = (a & 0xFFFFFFFF) * (b >> 32) + (x2 & 0xFFFFFFFF);
        let lo64 = ((x3 & 0xFFFFFFFF) << 32) | (x1 & 0xFFFFFFFF);
        let hi64 = (a >> 32) * (b >> 32) + (x2 >> 32) + (x3 >> 32);
        MultiplyUInt64Result { lo64, hi64 }
    }

    #[inline(always)]
    pub fn products_are_equal(a: i64, b: i64, c: i64, d: i64) -> bool {
        let abs_a = a.abs() as u64;
        let abs_b = b.abs() as u64;
        let abs_c = c.abs() as u64;
        let abs_d = d.abs() as u64;

        let mul_ab = multiply_uint64(abs_a, abs_b);
        let mul_cd = multiply_uint64(abs_c, abs_d);

        let sign_ab = tri_sign(a) * tri_sign(b);
        let sign_cd = tri_sign(c) * tri_sign(d);

        (mul_ab.lo64 == mul_cd.lo64)
            && (mul_ab.hi64 == mul_cd.hi64)
            && (sign_ab == sign_cd)
    }

    #[inline(always)]
    pub fn is_collinear(pt1: &Point64, shared_pt: &Point64, pt2: &Point64) -> bool {
        let a = shared_pt.x - pt1.x;
        let b = pt2.y - shared_pt.y;
        let c = shared_pt.y - pt1.y;
        let d = pt2.x - shared_pt.x;
        products_are_equal(a, b, c, d)
    }

    #[inline(always)]
    pub fn dot_product(p1: &Point64, p2: &Point64, p3: &Point64) -> f64 {
        ((p2.x - p1.x) as f64 * (p3.x - p2.x) as f64)
            + ((p2.y - p1.y) as f64 * (p3.y - p2.y) as f64)
    }

    #[inline(always)]
    pub fn cross_product_d(vec1: &PointD, vec2: &PointD) -> f64 {
        vec1.y * vec2.x - vec2.y * vec1.x
    }

    #[inline(always)]
    pub fn dot_product_d(vec1: &PointD, vec2: &PointD) -> f64 {
        vec1.x * vec2.x + vec1.y * vec2.y
    }

    #[inline(always)]
    pub fn check_cast_int64(val: f64) -> i64 {
        if val >= MAX_COORD_F64 || val <= MIN_COORD_F64 {
            INVALID64
        } else {
            // Use round_away to mimic MidpointRounding.AwayFromZero.
            super::round_away(val) as i64
        }
    }

    #[inline(always)]
    pub fn get_segment_intersect_pt(
        ln1a: &Point64,
        ln1b: &Point64,
        ln2a: &Point64,
        ln2b: &Point64,
        ip: &mut Point64,
    ) -> bool {
        let dy1 = (ln1b.y - ln1a.y) as f64;
        let dx1 = (ln1b.x - ln1a.x) as f64;
        let dy2 = (ln2b.y - ln2a.y) as f64;
        let dx2 = (ln2b.x - ln2a.x) as f64;
        let det = dy1 * dx2 - dy2 * dx1;
        if det == 0.0 {
            return false;
        }
        let t = (((ln1a.x - ln2a.x) as f64 * dy2)
            - ((ln1a.y - ln2a.y) as f64 * dx2))
            / det;
        if t <= 0.0 {
            *ip = *ln1a;
        } else if t >= 1.0 {
            *ip = *ln1b;
        } else {
            ip.x = ln1a.x + (t * dx1) as i64;
            ip.y = ln1a.y + (t * dy1) as i64;
            #[cfg(feature = "usingz")]
            {
                ip.z = 0;
            }
        }
        true
    }

    /// Returns true if the segments intersect.
    pub fn segs_intersect(
        seg1a: &Point64,
        seg1b: &Point64,
        seg2a: &Point64,
        seg2b: &Point64,
        inclusive: bool,
    ) -> bool {
        if !inclusive {
            return cross_product(seg1a, seg2a, seg2b) * cross_product(seg1b, seg2a, seg2b) < 0.0
                && cross_product(seg2a, seg1a, seg1b) * cross_product(seg2b, seg1a, seg1b) < 0.0;
        }
        let res1 = cross_product(seg1a, seg2a, seg2b);
        let res2 = cross_product(seg1b, seg2a, seg2b);
        if res1 * res2 > 0.0 {
            return false;
        }
        let res3 = cross_product(seg2a, seg1a, seg1b);
        let res4 = cross_product(seg2b, seg1a, seg1b);
        if res3 * res4 > 0.0 {
            return false;
        }
        // ensure NOT collinear
        (res1 != 0.0 || res2 != 0.0 || res3 != 0.0 || res4 != 0.0)
    }

    /// Returns the closest point on a segment to an off‑point.
    pub fn get_closest_pt_on_segment(
        off_pt: &Point64,
        seg1: &Point64,
        seg2: &Point64,
    ) -> Point64 {
        if seg1.x == seg2.x && seg1.y == seg2.y {
            return *seg1;
        }
        let dx = (seg2.x - seg1.x) as f64;
        let dy = (seg2.y - seg1.y) as f64;
        let mut q = (((off_pt.x - seg1.x) as f64 * dx)
            + ((off_pt.y - seg1.y) as f64 * dy))
            / (dx * dx + dy * dy);
        if q < 0.0 {
            q = 0.0;
        } else if q > 1.0 {
            q = 1.0;
        }
        // Use round (ties to even) as in C#’s MidpointRounding.ToEven.
        Point64 {
            x: seg1.x + (q * dx).round() as i64,
            y: seg1.y + (q * dy).round() as i64,
            #[cfg(feature = "usingz")]
            z: 0,
        }
    }

    /// Implements the PointInPolygon test.
    pub fn point_in_polygon(pt: &Point64, polygon: &Path64) -> PipResult {
        let len = polygon.len();
        if len < 3 {
            return PipResult::Outside;
        }

        let mut start = 0;
        while start < len && polygon[start].y == pt.y {
            start += 1;
        }
        if start == len {
            return PipResult::Outside;
        }

        let mut is_above = polygon[start].y < pt.y;
        let starting_above = is_above;
        let mut val = 0;
        let mut i = start + 1;
        let mut end = len;

        loop {
            if i == end {
                if end == 0 || start == 0 {
                    break;
                }
                end = start;
                i = 0;
            }

            if is_above {
                while i < end && polygon[i].y < pt.y {
                    i += 1;
                }
            } else {
                while i < end && polygon[i].y > pt.y {
                    i += 1;
                }
            }

            if i == end {
                continue;
            }

            let curr = polygon[i];
            let prev = if i > 0 {
                polygon[i - 1]
            } else {
                polygon[len - 1]
            };

            if curr.y == pt.y {
                if curr.x == pt.x || (curr.y == prev.y && ((pt.x < prev.x) != (pt.x < curr.x))) {
                    return PipResult::OnEdge;
                }
                i += 1;
                if i == start {
                    break;
                }
                continue;
            }

            if pt.x < curr.x && pt.x < prev.x {
                // do nothing
            } else if pt.x > prev.x && pt.x > curr.x {
                val = 1 - val;
            } else {
                let d = cross_product(&prev, &curr, pt);
                if d == 0.0 {
                    return PipResult::OnEdge;
                }
                if (d < 0.0) == is_above {
                    val = 1 - val;
                }
            }
            is_above = !is_above;
            i += 1;
            if i == start {
                break;
            }
        }

        if is_above == starting_above {
            return if val == 0 {
                PipResult::Outside
            } else {
                PipResult::Inside
            };
        }
        if i == len {
            i = 0;
        }
        let d = if i == 0 {
            cross_product(&polygon[len - 1], &polygon[0], pt)
        } else {
            cross_product(&polygon[i - 1], &polygon[i], pt)
        };
        if d == 0.0 {
            return PipResult::OnEdge;
        }
        if (d < 0.0) == is_above {
            val = 1 - val;
        }
        if val == 0 {
            PipResult::Outside
        } else {
            PipResult::Inside
        }
    }
}
