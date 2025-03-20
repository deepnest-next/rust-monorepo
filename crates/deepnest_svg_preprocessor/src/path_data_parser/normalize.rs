use super::Segment;
use std::f64::consts::PI;

/// Normalisiert den Pfad, sodass nur die Befehle M, L, C und Z enthalten sind.
pub fn normalize(segments: Vec<Segment>) -> Result<Vec<Segment>, String> {
    let mut out: Vec<Segment> = Vec::new();

    // Wir nutzen hier einen Platzhalter für "letzte Befehlsart".
    // In JS war dies ein leerer String; in Rust verwenden wir '\0' als Initialwert.
    let mut last_type: char = '\0';

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut subx = 0.0;
    let mut suby = 0.0;
    let mut lcx = 0.0;
    let mut lcy = 0.0;

    for seg in segments {
        let key = seg.key;
        let data = seg.data;
        match key {
            'M' => {
                if data.len() >= 2 {
                    out.push(Segment { key: 'M', data: data.clone() });
                    cx = data[0];
                    cy = data[1];
                    subx = cx;
                    suby = cy;
                } else {
                    return Err("Invalid M command: not enough parameters".to_string());
                }
            }
            'C' => {
                if data.len() >= 6 {
                    out.push(Segment { key: 'C', data: data.clone() });
                    cx = data[4];
                    cy = data[5];
                    lcx = data[2];
                    lcy = data[3];
                } else {
                    return Err("Invalid C command: not enough parameters".to_string());
                }
            }
            'L' => {
                if data.len() >= 2 {
                    out.push(Segment { key: 'L', data: data.clone() });
                    cx = data[0];
                    cy = data[1];
                } else {
                    return Err("Invalid L command: not enough parameters".to_string());
                }
            }
            'H' => {
                if !data.is_empty() {
                    cx = data[0];
                    out.push(Segment { key: 'L', data: vec![cx, cy] });
                } else {
                    return Err("Invalid H command: not enough parameters".to_string());
                }
            }
            'V' => {
                if !data.is_empty() {
                    cy = data[0];
                    out.push(Segment { key: 'L', data: vec![cx, cy] });
                } else {
                    return Err("Invalid V command: not enough parameters".to_string());
                }
            }
            'S' => {
                if data.len() >= 4 {
                    let (cx1, cy1) = if last_type == 'C' || last_type == 'S' {
                        (cx + (cx - lcx), cy + (cy - lcy))
                    } else {
                        (cx, cy)
                    };
                    let mut new_data = vec![cx1, cy1];
                    new_data.extend(data.clone());
                    out.push(Segment { key: 'C', data: new_data });
                    lcx = data[0];
                    lcy = data[1];
                    cx = data[2];
                    cy = data[3];
                } else {
                    return Err("Invalid S command: not enough parameters".to_string());
                }
            }
            'T' => {
                if data.len() >= 2 {
                    let x = data[0];
                    let y = data[1];
                    let (x1, y1) = if last_type == 'Q' || last_type == 'T' {
                        (cx + (cx - lcx), cy + (cy - lcy))
                    } else {
                        (cx, cy)
                    };
                    let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                    let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                    let cx2 = x + 2.0 * (x1 - x) / 3.0;
                    let cy2 = y + 2.0 * (y1 - y) / 3.0;
                    out.push(Segment { key: 'C', data: vec![cx1, cy1, cx2, cy2, x, y] });
                    lcx = x1;
                    lcy = y1;
                    cx = x;
                    cy = y;
                } else {
                    return Err("Invalid T command: not enough parameters".to_string());
                }
            }
            'Q' => {
                if data.len() >= 4 {
                    let x1 = data[0];
                    let y1 = data[1];
                    let x = data[2];
                    let y = data[3];
                    let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                    let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                    let cx2 = x + 2.0 * (x1 - x) / 3.0;
                    let cy2 = y + 2.0 * (y1 - y) / 3.0;
                    out.push(Segment { key: 'C', data: vec![cx1, cy1, cx2, cy2, x, y] });
                    lcx = x1;
                    lcy = y1;
                    cx = x;
                    cy = y;
                } else {
                    return Err("Invalid Q command: not enough parameters".to_string());
                }
            }
            'A' => {
                if data.len() >= 7 {
                    let r1 = data[0].abs();
                    let r2 = data[1].abs();
                    let angle = data[2];
                    let large_arc_flag = data[3];
                    let sweep_flag = data[4];
                    let x = data[5];
                    let y = data[6];
                    if r1 == 0.0 || r2 == 0.0 {
                        out.push(Segment { key: 'C', data: vec![cx, cy, x, y, x, y] });
                        cx = x;
                        cy = y;
                    } else {
                        if (cx != x) || (cy != y) {
                            match arc_to_cubic_curves(cx, cy, x, y, r1, r2, angle, large_arc_flag, sweep_flag, None) {
                                Ok(curves) => {
                                    for curve in curves {
                                        out.push(Segment { key: 'C', data: curve });
                                    }
                                    cx = x;
                                    cy = y;
                                },
                                Err(e) => return Err(format!("Arc conversion error: {}", e))
                            }
                        }
                    }
                } else {
                    return Err("Invalid A command: not enough parameters".to_string());
                }
            }
            'Z' | 'z' => {
                out.push(Segment { key: 'Z', data: Vec::new() });
                cx = subx;
                cy = suby;
            }
            _ => {
                return Err(format!("Unknown command: {}", key));
            }
        }
        last_type = key;
    }
    Ok(out)
}

/// Wandelt Grad in Radiant um.
fn deg_to_rad(degrees: f64) -> f64 {
    PI * degrees / 180.0
}

/// Dreht den Punkt (x, y) um den Winkel `angle_rad` (im Uhrzeigersinn) und gibt das neue Koordinatenpaar zurück.
fn rotate(x: f64, y: f64, angle_rad: f64) -> (f64, f64) {
    let new_x = x * angle_rad.cos() - y * angle_rad.sin();
    let new_y = x * angle_rad.sin() + y * angle_rad.cos();
    (new_x, new_y)
}

/// Wandelt einen elliptischen Bogen (Arc) in eine Reihe von kubischen Bézierkurven um.
/// Der optionale Parameter `recursive` enthält, falls gesetzt, Werte [f1, f2, cx, cy].
fn arc_to_cubic_curves(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    mut r1: f64,
    mut r2: f64,
    angle: f64,
    large_arc_flag: f64,
    sweep_flag: f64,
    recursive: Option<[f64; 4]>,
) -> Result<Vec<Vec<f64>>, String> {
    // Optimization: If start and end points are too close, return a simple line
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx*dx + dy*dy < 1e-6 {
        return Ok(vec![vec![x1, y1, x2, y2, x2, y2]]);
    }

    let angle_rad = deg_to_rad(angle);
    
    // Pre-compute trig functions to avoid redundant calculations
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();
    
    // Pre-allocate with estimated capacity
    let mut params: Vec<Vec<f64>> = Vec::with_capacity(4);

    // Kopien der Eingangskoordinaten, die im Folgenden modifiziert werden.
    let (mut ax1, mut ay1) = (x1, y1);
    let (mut ax2, mut ay2) = (x2, y2);

    let (mut f1, mut f2, cx, cy);

    if let Some(rec) = recursive {
        f1 = rec[0];
        f2 = rec[1];
        cx = rec[2];
        cy = rec[3];
    } else {
        // Drehe die Punkte um -angle_rad - use precomputed trig values
        let (rx1, ry1) = (
            ax1 * cos_angle + ay1 * sin_angle,
            -ax1 * sin_angle + ay1 * cos_angle
        );
        let (rx2, ry2) = (
            ax2 * cos_angle + ay2 * sin_angle,
            -ax2 * sin_angle + ay2 * cos_angle
        );
        ax1 = rx1;
        ay1 = ry1;
        ax2 = rx2;
        ay2 = ry2;

        let x = (ax1 - ax2) / 2.0;
        let y = (ay1 - ay2) / 2.0;
        let mut h = (x * x) / (r1 * r1) + (y * y) / (r2 * r2);
        if h > 1.0 {
            h = h.sqrt();
            r1 = h * r1;
            r2 = h * r2;
        }

        let sign = if (large_arc_flag - sweep_flag).abs() < f64::EPSILON {
            -1.0
        } else {
            1.0
        };

        let r1_pow = r1 * r1;
        let r2_pow = r2 * r2;
        let left = r1_pow * r2_pow - r1_pow * y * y - r2_pow * x * x;
        
        // Check for degenerate cases that could cause square root of negative number
        if left < 0.0 {
            // Handle degenerate case
            return Ok(vec![vec![x1, y1, x2, y2, x2, y2]]);
        }
        
        let right = r1_pow * y * y + r2_pow * x * x;
        if right <= 0.0 {
            // Handle another edge case
            return Ok(vec![vec![x1, y1, x2, y2, x2, y2]]);
        }
        
        let k = sign * (left / right).abs().sqrt();

        cx = k * r1 * y / r2 + (ax1 + ax2) / 2.0;
        cy = k * -r2 * x / r1 + (ay1 + ay2) / 2.0;

        // Die toFixed(9)-Rundung aus JS entspricht hier dem Clampen in [-1,1] vor dem asin.
        let ay1_cy_r2 = (ay1 - cy) / r2;
        let ay2_cy_r2 = (ay2 - cy) / r2;
        
        // Fast path for common case
        f1 = if ay1_cy_r2 >= 1.0 { std::f64::consts::FRAC_PI_2 }
             else if ay1_cy_r2 <= -1.0 { -std::f64::consts::FRAC_PI_2 }
             else { ay1_cy_r2.asin() };
             
        f2 = if ay2_cy_r2 >= 1.0 { std::f64::consts::FRAC_PI_2 }
             else if ay2_cy_r2 <= -1.0 { -std::f64::consts::FRAC_PI_2 }
             else { ay2_cy_r2.asin() };

        if ax1 < cx {
            f1 = std::f64::consts::PI - f1;
        }
        if ax2 < cx {
            f2 = std::f64::consts::PI - f2;
        }
        if f1 < 0.0 {
            f1 = std::f64::consts::PI * 2.0 + f1;
        }
        if f2 < 0.0 {
            f2 = std::f64::consts::PI * 2.0 + f2;
        }
        if sweep_flag != 0.0 && f1 > f2 {
            f1 -= std::f64::consts::PI * 2.0;
        }
        if sweep_flag == 0.0 && f2 > f1 {
            f2 -= std::f64::consts::PI * 2.0;
        }
    }

    let mut df = f2 - f1;
    if df.abs() > (PI * 120.0 / 180.0) {
        let f2old = f2;
        let ax2old = ax2;
        let ay2old = ay2;
        if sweep_flag != 0.0 && f2 > f1 {
            f2 = f1 + (PI * 120.0 / 180.0);
        } else {
            f2 = f1 - (PI * 120.0 / 180.0);
        }
        ax2 = cx + r1 * f2.cos();
        ay2 = cy + r2 * f2.sin();
        match arc_to_cubic_curves(ax2, ay2, ax2old, ay2old, r1, r2, angle, 0.0, sweep_flag, Some([f2, f2old, cx, cy])) {
            Ok(result) => params = result,
            Err(e) => return Err(e)
        }
    }

    df = f2 - f1;
    let c1 = f1.cos();
    let s1 = f1.sin();
    let c2 = f2.cos();
    let s2 = f2.sin();
    let t = (df / 4.0).tan();
    let hx = 4.0 / 3.0 * r1 * t;
    let hy = 4.0 / 3.0 * r2 * t;

    let m1 = vec![ax1, ay1];
    let mut m2 = vec![ax1 + hx * s1, ay1 - hy * c1];
    let m3 = vec![ax2 + hx * s2, ay2 - hy * c2];
    let m4 = vec![ax2, ay2];

    // Anpassung von m2
    m2[0] = 2.0 * m1[0] - m2[0];
    m2[1] = 2.0 * m1[1] - m2[1];

    if recursive.is_some() {
        let mut result = vec![m2, m3, m4];
        result.extend(params);
        return Ok(result);
    } else {
        let mut combined = vec![m2, m3, m4];
        combined.extend(params);
        let mut curves = Vec::new();
        let mut i = 0;
        while i < combined.len() {
            if i + 2 < combined.len() {
                let (r1x, r1y) = rotate(combined[i][0], combined[i][1], angle_rad);
                let (r2x, r2y) = rotate(combined[i + 1][0], combined[i + 1][1], angle_rad);
                let (r3x, r3y) = rotate(combined[i + 2][0], combined[i + 2][1], angle_rad);
                curves.push(vec![r1x, r1y, r2x, r2y, r3x, r3y]);
            }
            i += 3;
        }
        return Ok(curves);
    }
}
