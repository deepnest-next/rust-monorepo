use babushka::kernelf64::{Point2D, Polygon};
use babushka::multi_polygon::MultiPolygon;
use babushka::no_fit_polygon::ComputeNoFitPolygon as _;

pub fn calculate_nfp(a: MultiPolygon<Polygon>, b: MultiPolygon<Polygon>) -> Vec<Vec<Point2D>> {
    let mut nfp_list = vec![];
    nfp_list.extend(
        a
            .outer()
            .no_fit_polygon(b.outer(), false, false)
            .unwrap(),
    );
    for hole in a.holes() {
        nfp_list.extend(hole.no_fit_polygon(b.outer(), true, false).unwrap());
    }
    // for v in piece_0.holes().first().unwrap().iter_vertices() {
    //     println!("{{x: {}, y: {}}},", v.x, v.y);
    // }
    nfp_list
}
