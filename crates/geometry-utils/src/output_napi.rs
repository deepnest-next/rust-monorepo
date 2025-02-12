#[allow(unused_imports)]
use crate::output_lib::GeometryUtils;

#[allow(unused_imports)]
#[cfg(feature = "node")]
use deepnest_types::{Point, Rect};

/// Wir exportieren hier eine Klasse namens "GeometryUtils"
#[cfg_attr(feature = "node", napi(object, js_name = "GeometryUtils"))] // Prevents the napi proc_macro from being placed if the feature is not enabled
#[cfg(feature = "node")] // Removes the following code if the feature is not enabled
pub struct NodeGeometryUtils(GeometryUtils);

#[cfg_attr(feature = "node", napi)] // Prevents the napi proc_macro from being placed if the feature is not enabled
#[cfg(feature = "node")] // Removes the following code if the feature is not enabled
impl NodeGeometryUtils {
    // You have to delegate each function; you can write a helper macro to reduce duplication if needed.
    #[napi]
    pub fn almost_equal(a: f64, b: f64, tolerance: Option<f64>) -> bool {
        GeometryUtils::almost_equal(a.into(), b.into(), tolerance)
    }

    #[napi]
    pub fn polygon_area(polygon: Vec<Point>) -> f64 {
        GeometryUtils::polygon_area(&polygon)
    }

    #[napi]
    pub fn get_polygon_bounds(polygon: Vec<Point>) -> Option<Rect> {
        GeometryUtils::get_polygon_bounds(&polygon)
    }

    #[napi]
    pub fn within_distance(p1: Point, p2: Point, distance: f64) -> bool {
        GeometryUtils::within_distance(p1.into(), p2.into(), distance)
    }
}
