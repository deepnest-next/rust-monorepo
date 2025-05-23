use napi::bindgen_prelude::*;
use napi_derive::napi;

// Babushka imports
use babushka::kernelf64::{Point2D as BabushkaKernelPoint, Polygon as BabushkaKernelPolygon};
use babushka::multi_polygon::MultiPolygon as BabushkaMultiPolygon;
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::point::Point2D as BabushkaPoint2DTrait; // For trait methods like p.x()
use babushka::polygon::Polygon as BabushkaPolygonTrait; // Renamed to avoid conflict

// NAPI Point structure (remains the same)
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<&Point> for BabushkaKernelPoint {
    fn from(p: &Point) -> Self {
        BabushkaKernelPoint { x: p.x, y: p.y }
    }
}

impl From<BabushkaKernelPoint> for Point {
    fn from(p: BabushkaKernelPoint) -> Self {
        Point { x: p.x(), y: p.y() }
    }
}

// NAPI Polygon structure - now with optional inner holes
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Polygon {
    pub outer: Vec<Point>,
    pub inner: Option<Vec<Vec<Point>>>, // List of hole paths
}

// Helper to convert NAPI points to BabushkaKernelPolygon
fn napi_path_to_babushka_polygon(path: &[Point]) -> BabushkaKernelPolygon {
    let babushka_vertices: Vec<BabushkaKernelPoint> =
        path.iter().map(|p| p.into()).collect();
    BabushkaKernelPolygon {
        vertices: babushka_vertices,
        offset: BabushkaKernelPoint { x: 0.0, y: 0.0 },
        rotation: 0.0,
    }
}

// Conversion from NAPI Polygon (outer boundary only) to babushka's concrete Polygon type
// This is used when a single path (outer or a hole) needs to be converted to a simple BabushkaKernelPolygon
impl From<&Vec<Point>> for BabushkaKernelPolygon {
    fn from(path_napi: &Vec<Point>) -> Self {
        napi_path_to_babushka_polygon(path_napi)
    }
}


// Conversion from NAPI Polygon (with potential holes) to BabushkaMultiPolygon
impl TryFrom<&Polygon> for BabushkaMultiPolygon<BabushkaKernelPolygon> {
    type Error = napi::Error;

    fn try_from(poly_napi: &Polygon) -> Result<Self, Self::Error> {
        if poly_napi.outer.is_empty() {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "Outer polygon path cannot be empty".to_string(),
            ));
        }
        let outer_babushka = napi_path_to_babushka_polygon(&poly_napi.outer);

        let mut holes_babushka: Vec<BabushkaKernelPolygon> = Vec::new();
        if let Some(inner_paths_napi) = &poly_napi.inner {
            for hole_path_napi in inner_paths_napi {
                if hole_path_napi.is_empty() {
                    return Err(napi::Error::new(
                        napi::Status::InvalidArg,
                        "Inner hole path cannot be empty".to_string(),
                    ));
                }
                holes_babushka.push(napi_path_to_babushka_polygon(hole_path_napi));
            }
        }
        Ok(BabushkaMultiPolygon::new(outer_babushka, holes_babushka))
    }
}

#[napi]
pub fn no_fit_polygon(
    polygon_a_napi: &Polygon, // Stationary polygon, can have holes
    polygon_b_napi: &Polygon, // Orbiting polygon, effectively only its outer boundary is used by babushka's MultiPolygon NFP
    include_outer_nfp: bool,  // Corresponds to MultiPolygon's include_outer: NFP of A's outer boundary vs B's outer boundary
    include_holes_nfp: bool,  // Corresponds to MultiPolygon's include_holes: NFPs of A's holes vs B's outer boundary
) -> Result<Vec<Vec<Point>>> { // Returns Vec<Vec<Point>> directly, can be empty if no NFP found. Option wrapper removed for simplicity, JS can check array length.

    // Convert NAPI Polygon A (stationary, with potential holes) to BabushkaMultiPolygon
    let multi_poly_a: BabushkaMultiPolygon<BabushkaKernelPolygon> = polygon_a_napi.try_into().map_err(|e| {
        napi::Error::new(
            e.status(),
            format!("Failed to convert polygon_a_napi: {}", e.reason()),
        )
    })?;

    // Convert NAPI Polygon B (orbiting) to BabushkaMultiPolygon.
    // Its holes are ignored by babushka's MultiPolygon::no_fit_polygon, so we only pass its outer boundary.
    let outer_b_babushka = napi_path_to_babushka_polygon(&polygon_b_napi.outer);
    let multi_poly_b = BabushkaMultiPolygon::new(outer_b_babushka, vec![]); // No holes for orbiting piece in this context

    // Call babushka's MultiPolygon no_fit_polygon method
    let nfp_result_paths: Vec<Vec<BabushkaKernelPoint>> = multi_poly_a.no_fit_polygon(
        &multi_poly_b,
        include_outer_nfp,
        include_holes_nfp,
    );

    // Convert result back to NAPI type (Vec<Vec<Point>>)
    let result_napi: Vec<Vec<Point>> = nfp_result_paths
        .into_iter()
        .map(|path| path.into_iter().map(|p| p.into()).collect())
        .filter(|path: &Vec<Point>| !path.is_empty()) // Ensure no empty paths are returned
        .collect();

    Ok(result_napi)
}
