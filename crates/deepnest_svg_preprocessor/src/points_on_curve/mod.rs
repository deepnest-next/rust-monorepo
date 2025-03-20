#![allow(dead_code)]
#![allow(unused_imports)]
mod curve;
mod curve_to_bezier;

pub use curve::{points_on_bezier_curves, simplify, simplify_points,Point};
pub use curve_to_bezier::curve_to_bezier;