use super::Point;

/// Klont einen Punkt. Da `Point` ein Copy-Typ ist, genügt hier die einfache Rückgabe.
fn clone_point(p: Point) -> Point {
    p
}

/// Wandelt einen Kurvenverlauf in eine Folge von kubischen Bézierkurven um.
/// - `points_in`: Eingangspunkte der Kurve (muss mindestens drei Punkte enthalten).
/// - `curve_tightness`: Bestimmt die Straffheit der Kurve (Standard: 0.0)
#[allow(unused)]
pub fn curve_to_bezier(points_in: &[Point], curve_tightness: f64) -> Vec<Point> {
    let len = points_in.len();
    if len < 3 {
        panic!("A curve must have at least three points.");
    }
    let mut out: Vec<Point> = Vec::new();

    if len == 3 {
        out.push(clone_point(points_in[0]));
        out.push(clone_point(points_in[1]));
        out.push(clone_point(points_in[2]));
        out.push(clone_point(points_in[2]));
    } else {
        // Erzeuge ein neues Punkte-Array, in dem das erste Element verdoppelt wird
        let mut points: Vec<Point> = Vec::new();
        points.push(points_in[0]);
        points.push(points_in[0]);
        for i in 1..len {
            points.push(points_in[i]);
            if i == len - 1 {
                points.push(points_in[i]);
            }
        }
        let s = 1.0 - curve_tightness;
        out.push(clone_point(points[0]));
        // Für i von 1 bis (points.len() - 3), da die Bedingung (i + 2) < points.len() erfüllt sein muss
        for i in 1..(points.len() - 2) {
            let cached = points[i];
            let b1: Point = [
                cached.x + (s * points[i + 1].x - s * points[i - 1].x) / 6.0,
                cached.y + (s * points[i + 1].y - s * points[i - 1].y) / 6.0,
            ].into();
            let b2: Point = [
                points[i + 1].x + (s * points[i].x - s * points[i + 2].x) / 6.0,
                points[i + 1].y + (s * points[i].y - s * points[i + 2].y) / 6.0,
            ].into();
            let b3 = points[i + 1];
            out.push(b1);
            out.push(b2);
            out.push(b3);
        }
    }
    out
}