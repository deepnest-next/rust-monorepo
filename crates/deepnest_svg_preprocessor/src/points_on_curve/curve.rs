#[derive(Copy, Clone, Debug)]
#[napi(object)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Point {
    fn from(arr: [f64; 2]) -> Self {
        Point { x: arr[0], y: arr[1] }
    }
}

impl From<(f64,f64)> for Point {
    fn from(arr: (f64,f64)) -> Self {
        Point { x: arr.0, y: arr.1 }
    }
}

/// Berechnet die quadratische Distanz zwischen zwei Punkten.
fn distance_sq(p1: Point, p2: Point) -> f64 {
    (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)
}

/// Berechnet die Distanz zwischen zwei Punkten.
fn distance(p1: Point, p2: Point) -> f64 {
    distance_sq(p1, p2).sqrt()
}

/// Berechnet die quadratische Distanz von Punkt `p` zu dem Liniensegment von `v` nach `w`.
fn distance_to_segment_sq(p: Point, v: Point, w: Point) -> f64 {
    let l2 = distance_sq(v, w);
    if l2 == 0.0 {
        return distance_sq(p, v);
    }
    let mut t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
    // t in [0, 1] einschränken
    t = t.max(0.0).min(1.0);
    distance_sq(p, lerp(v, w, t))
}

/// Lineare Interpolation zwischen den Punkten `a` und `b` mit Parameter `t`.
fn lerp(a: Point, b: Point, t: f64) -> Point {
    Point { 
        x: a.x + (b.x - a.x) * t, 
        y: a.y + (b.y - a.y) * t 
    }
}

/// Berechnet die "Flatness" einer Bézierkurve, die in `points` ab dem Index `offset` liegt.
/// Erwartet werden dabei genau 4 Punkte (kubische Bézierkurve).
#[inline]
fn flatness(points: &[Point], offset: usize) -> f64 {
    let p1 = points[offset];
    let p2 = points[offset + 1];
    let p3 = points[offset + 2];
    let p4 = points[offset + 3];

    let ux = 3.0 * p2.x - 2.0 * p1.x - p4.x;
    let uy = 3.0 * p2.y - 2.0 * p1.y - p4.y;
    let vx = 3.0 * p3.x - 2.0 * p4.x - p1.x;
    let vy = 3.0 * p3.y - 2.0 * p4.y - p1.y;

    let ux_sq = ux * ux;
    let uy_sq = uy * uy;
    let vx_sq = vx * vx;
    let vy_sq = vy * vy;

    let max_x = if ux_sq > vx_sq { ux_sq } else { vx_sq };
    let max_y = if uy_sq > vy_sq { uy_sq } else { vy_sq };

    max_x + max_y
}

/// Rekursive Funktion, die Punkte auf einer kubischen Bézierkurve (definiert durch 4 Punkte)
/// mittels Subdivision bestimmt. Der Parameter `offset` gibt an, ab welchem Index im Array
/// die 4 Kontrollpunkte liegen. Die gefundenen Punkte werden in `out_points` gesammelt.
fn get_points_on_bezier_curve_with_splitting(
    points: &[Point],
    offset: usize,
    tolerance: f64,
    out_points: &mut Vec<Point>,
) {
    // By checking the flatness before creating new points arrays, we avoid allocations
    if flatness(points, offset) < tolerance {
        let p0 = points[offset];
        if let Some(&last) = out_points.last() {
            if distance(last, p0) > 1.0 {
                out_points.push(p0);
            }
        } else {
            out_points.push(p0);
        }
        out_points.push(points[offset + 3]);
    } else {
        let t = 0.5;
        let p1 = points[offset];
        let p2 = points[offset + 1];
        let p3 = points[offset + 2];
        let p4 = points[offset + 3];

        // Calculate de Casteljau algorithm directly to avoid multiple lerp calls
        let q1x = p1.x + t * (p2.x - p1.x);
        let q1y = p1.y + t * (p2.y - p1.y);
        let q2x = p2.x + t * (p3.x - p2.x);
        let q2y = p2.y + t * (p3.y - p2.y);
        let q3x = p3.x + t * (p4.x - p3.x);
        let q3y = p3.y + t * (p4.y - p3.y);
        
        let r1x = q1x + t * (q2x - q1x);
        let r1y = q1y + t * (q2y - q1y);
        let r2x = q2x + t * (q3x - q2x);
        let r2y = q2y + t * (q3y - q2y);
        
        let redx = r1x + t * (r2x - r1x);
        let redy = r1y + t * (r2y - r1y);

        // Create Point structs only when needed
        let q1 = Point { x: q1x, y: q1y };
        let r1 = Point { x: r1x, y: r1y };
        let red = Point { x: redx, y: redy };
        let r2 = Point { x: r2x, y: r2y };
        let q3 = Point { x: q3x, y: q3y };

        // Stack-allocated arrays with 4 elements for the subdivided curves
        let left_curve = [p1, q1, r1, red];
        let right_curve = [red, r2, q3, p4];

        // Rekursiver Aufruf für die linke Teilkurve
        get_points_on_bezier_curve_with_splitting(&left_curve, 0, tolerance, out_points);
        // Rekursiver Aufruf für die rechte Teilkurve
        get_points_on_bezier_curve_with_splitting(&right_curve, 0, tolerance, out_points);
    }
}

/// Vereinfacht (simplifiziert) die Kurve, indem die Punkte von `points` reduziert werden.
/// Dies ist eine Hilfsfunktion, die intern `simplify_points` aufruft.
pub fn simplify(points: &[Point], epsilon: f64) -> Vec<Point> {
    simplify_points(points, 0, points.len(), epsilon)
}

/// Implementierung des Ramer–Douglas–Peucker-Algorithmus.
/// `points` ist das Ausgangsarray, der Bereich wird durch `start` (inklusive) und `end` (exklusive)
/// bestimmt. Die Punkte, die erhalten bleiben, werden in `out_points` gesammelt.
pub fn simplify_points(
    points: &[Point],
    start: usize,
    end: usize,
    epsilon: f64,
) -> Vec<Point> {
    let mut out_points = Vec::new();
    _simplify_points(points, start, end, epsilon, &mut out_points);
    out_points
}

/// Rekursive Hilfsfunktion für den RDP-Algorithmus.
fn _simplify_points(
    points: &[Point],
    start: usize,
    end: usize,
    epsilon: f64,
    out_points: &mut Vec<Point>,
) {
    let s = points[start];
    let e = points[end - 1];
    let mut max_dist_sq = 0.0;
    let mut max_ndx = start + 1;

    for i in (start + 1)..(end - 1) {
        let d_sq = distance_to_segment_sq(points[i], s, e);
        if d_sq > max_dist_sq {
            max_dist_sq = d_sq;
            max_ndx = i;
        }
    }

    if max_dist_sq.sqrt() > epsilon {
        _simplify_points(points, start, max_ndx + 1, epsilon, out_points);
        _simplify_points(points, max_ndx, end, epsilon, out_points);
    } else {
        if out_points.is_empty() {
            out_points.push(s);
        }
        out_points.push(e);
    }
}

/// Bestimmt Punkte auf einer Folge von Bézierkurven.
/// Das Eingabearray `points` muss die Kontrollpunkte für eine oder mehrere kubische Bézierkurven enthalten,
/// wobei jede Kurve durch 4 Punkte definiert ist. Mit `tolerance` wird die Auflösung der Subdivision gesteuert.
/// Optional kann mit `distance` eine weitere Vereinfachung (RDP) durchgeführt werden.
pub fn points_on_bezier_curves(points: &[Point], tolerance: Option<f64>, distance: Option<f64>) -> Vec<Point> {
    let tolerance = tolerance.unwrap_or(0.15);
    
    // Calculate conservative capacity estimate
    let num_segments = (points.len() - 1) / 3;
    let estimated_points = num_segments * 10; // Heuristic for average points per segment
    
    let mut new_points = Vec::with_capacity(estimated_points);

    for i in 0..num_segments {
        let offset = i * 3;
        get_points_on_bezier_curve_with_splitting(points, offset, tolerance, &mut new_points);
    }
    
    if let Some(d) = distance {
        if d > 0.0 {
            return simplify_points(&new_points, 0, new_points.len(), d);
        }
    }
    new_points
}
