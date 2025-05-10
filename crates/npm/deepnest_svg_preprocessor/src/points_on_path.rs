use crate::path_data_parser::{absolutize, normalize, parse_path};
use crate::points_on_curve::{points_on_bezier_curves, simplify, Point};
use napi::bindgen_prelude::*;

/// Information about a processed path including whether it's closed
#[derive(Debug, Clone)]
#[napi]
pub struct PathResult {
  /// Sets of points that approximate the path
  pub points: Vec<Vec<Point>>,
  /// Whether the path is closed (ends with 'Z' command or first point equals last point)
  pub closed: Vec<bool>,
}

/// Liefert für den übergebenen Pfad (als String) eine Liste von Punkt‑Sätzen zurück,
/// die den Pfad approximieren. Dabei können eine optionale Toleranz (für die
/// Bézier‑Kurven Approximation) sowie ein Epsilon (für eine anschließende Vereinfachung)
/// angegeben werden.
pub fn points_on_path(
  path: String,
  tolerance: Option<f64>,
  distance: Option<f64>,
) -> Result<Vec<Vec<Point>>> {
  // Parse, absolutiere und normalisiere den Pfad.
  let segments = match parse_path(&path) {
    Ok(segs) => segs,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("Path parsing error: {}", e))),
  };
  
  // Apply absolutization and normalization in sequence
  let abs_segments = absolutize(segments);
  let normalized = match normalize(abs_segments) {
    Ok(norm) => norm,
    Err(e) => return Err(Error::new(Status::GenericFailure, format!("Path normalization error: {}", e))),
  };

  // Pre-allocate vectors with capacity to avoid frequent reallocations
  let mut sets: Vec<Vec<Point>> = Vec::with_capacity(normalized.len() / 4 + 1);
  let mut current_points: Vec<Point> = Vec::with_capacity(32);
  let mut start: Point = [0.0, 0.0].into();
  let mut pending_curve: Vec<Point> = Vec::with_capacity(16);

  // Closure, um einen eventuell vorhandenen pending Curve-Abschnitt auszuwerten.
  let append_pending_curve =
    |pending_curve: &mut Vec<Point>, current_points: &mut Vec<Point>| {
      if pending_curve.len() >= 4 {
        // Hängt die Punkte, die durch die Bézierkurvenapproximation geliefert werden, an.
        // Reserve space for new points to prevent reallocations
        let new_points = points_on_bezier_curves(pending_curve, tolerance, None);
        // Avoid unnecessary copying by extending with new_points directly
        current_points.reserve(current_points.len() + new_points.len());
        current_points.extend_from_slice(&new_points);
      }
      pending_curve.clear();
    };

  // Closure, um einen pending Curve-Abschnitt auszuwerten und den aktuellen Punkt-Satz abzuschließen.
  let append_pending_points = |pending_curve: &mut Vec<Point>,
                                   current_points: &mut Vec<Point>,
                                   sets: &mut Vec<Vec<Point>>| {
    append_pending_curve(pending_curve, current_points);
    if !current_points.is_empty() {
      sets.push(std::mem::take(current_points));
      // Ensure capacity for next set of points
      current_points.reserve(32);
    }
  };

  // Iteriere über alle normalisierten Segmente.
  for seg in &normalized {
    match seg.key {
      'M' => {
        append_pending_points(&mut pending_curve, &mut current_points, &mut sets);
        if seg.data.len() >= 2 {
          start = [seg.data[0], seg.data[1]].into();
          current_points.push(start);
        }
      }
      'L' => {
        append_pending_curve(&mut pending_curve, &mut current_points);
        if seg.data.len() >= 2 {
          current_points.push([seg.data[0], seg.data[1]].into());
        }
      }
      'C' => {
        if pending_curve.is_empty() {
          let last_point = current_points.last().copied().unwrap_or(start);
          pending_curve.push(last_point);
        }
        // Füge die 3 Kontrollpunkte hinzu.
        if seg.data.len() >= 6 {
          pending_curve.push([seg.data[0], seg.data[1]].into());
          pending_curve.push([seg.data[2], seg.data[3]].into());
          pending_curve.push([seg.data[4], seg.data[5]].into());
        }
      }
      'Z' => {
        append_pending_curve(&mut pending_curve, &mut current_points);
        current_points.push(start);
      }
      _ => {
        // Andere Befehle werden ignoriert.
      }
    }
  }
  
  append_pending_points(&mut pending_curve, &mut current_points, &mut sets);

  // Wurde kein Simplify-Epsilon (distance) übergeben, so geben wir die bisherigen Punkt-Sets zurück.
  if distance.is_none() {
    return Ok(sets);
  }

  let d = distance.unwrap();
  // Pre-allocate output vector with same capacity as input
  let mut out: Vec<Vec<Point>> = Vec::with_capacity(sets.len());
  for set in sets {
    let simplified_set = simplify(&set, d);
    if !simplified_set.is_empty() {
      out.push(simplified_set);
    }
  }
  Ok(out)
}

/// Enhanced version of points_on_path that also returns whether each set of points represents a closed path

pub fn points_on_path_with_closed_info(
  path: String,
  tolerance: Option<f64>,
  distance: Option<f64>,
) -> Result<PathResult> {
  // Parse, absolutiere und normalisiere den Pfad.
  let segments = match parse_path(&path) {
    Ok(segs) => segs,
    Err(e) => return Err(Error::new(Status::InvalidArg, format!("Path parsing error: {}", e))),
  };
  
  // Apply absolutization and normalization in sequence
  let abs_segments = absolutize(segments);
  let normalized = match normalize(abs_segments) {
    Ok(norm) => norm,
    Err(e) => return Err(Error::new(Status::GenericFailure, format!("Path normalization error: {}", e))),
  };

  // Pre-allocate vectors with capacity to avoid frequent reallocations
  let mut sets: Vec<Vec<Point>> = Vec::with_capacity(normalized.len() / 4 + 1);
  let mut closed_info: Vec<bool> = Vec::with_capacity(normalized.len() / 4 + 1);
  let mut current_points: Vec<Point> = Vec::with_capacity(32);
  let mut start: Point = [0.0, 0.0].into();
  let mut is_current_path_closed = false;
  let mut pending_curve: Vec<Point> = Vec::with_capacity(16);

  // Closure, um einen eventuell vorhandenen pending Curve-Abschnitt auszuwerten.
  let append_pending_curve =
    |pending_curve: &mut Vec<Point>, current_points: &mut Vec<Point>| {
      if pending_curve.len() >= 4 {
        // Hängt die Punkte, die durch die Bézierkurvenapproximation geliefert werden, an.
        // Reserve space for new points to prevent reallocations
        let new_points = points_on_bezier_curves(pending_curve, tolerance, None);
        // Avoid unnecessary copying by extending with new_points directly
        current_points.reserve(current_points.len() + new_points.len());
        current_points.extend_from_slice(&new_points);
      }
      pending_curve.clear();
    };

  // Closure, um einen pending Curve-Abschnitt auszuwerten und den aktuellen Punkt-Satz abzuschließen.
  let append_pending_points = |pending_curve: &mut Vec<Point>,
                                   current_points: &mut Vec<Point>,
                                   sets: &mut Vec<Vec<Point>>,
                                   closed_info: &mut Vec<bool>,
                                   is_current_path_closed: bool| {
    append_pending_curve(pending_curve, current_points);
    if !current_points.is_empty() {
      // Check if first and last points are the same (another indicator of closed path)
      let is_geometrically_closed = current_points.len() >= 2 && 
        is_point_equal(current_points[0], *current_points.last().unwrap());
        
      closed_info.push(is_current_path_closed || is_geometrically_closed);
      sets.push(std::mem::take(current_points));
      // Ensure capacity for next set of points
      current_points.reserve(32);
    }
  };

  // Iteriere über alle normalisierten Segmente.
  for seg in &normalized {
    match seg.key {
      'M' => {
        append_pending_points(&mut pending_curve, &mut current_points, &mut sets, &mut closed_info, is_current_path_closed);
        is_current_path_closed = false; // Reset for new path
        if seg.data.len() >= 2 {
          start = [seg.data[0], seg.data[1]].into();
          current_points.push(start);
        }
      }
      'L' => {
        append_pending_curve(&mut pending_curve, &mut current_points);
        if seg.data.len() >= 2 {
          current_points.push([seg.data[0], seg.data[1]].into());
        }
      }
      'C' => {
        if pending_curve.is_empty() {
          let last_point = current_points.last().copied().unwrap_or(start);
          pending_curve.push(last_point);
        }
        // Füge die 3 Kontrollpunkte hinzu.
        if seg.data.len() >= 6 {
          pending_curve.push([seg.data[0], seg.data[1]].into());
          pending_curve.push([seg.data[2], seg.data[3]].into());
          pending_curve.push([seg.data[4], seg.data[5]].into());
        }
      }
      'Z' => {
        append_pending_curve(&mut pending_curve, &mut current_points);
        current_points.push(start);
        is_current_path_closed = true; // Mark current path as closed
      }
      _ => {
        // Andere Befehle werden ignoriert.
      }
    }
  }
  
  append_pending_points(&mut pending_curve, &mut current_points, &mut sets, &mut closed_info, is_current_path_closed);

  // Wurde kein Simplify-Epsilon (distance) übergeben, so geben wir die bisherigen Punkt-Sets zurück.
  if distance.is_none() {
    return Ok(PathResult { 
      points: sets, 
      closed: closed_info 
    });
  }

  let d = distance.unwrap();
  // Pre-allocate output vectors with same capacity as input
  let mut out_points: Vec<Vec<Point>> = Vec::with_capacity(sets.len());
  let mut out_closed: Vec<bool> = Vec::with_capacity(sets.len());
  
  for (i, set) in sets.iter().enumerate() {
    let simplified_set = simplify(set, d);
    if !simplified_set.is_empty() {
      out_points.push(simplified_set);
      out_closed.push(closed_info[i]);
    }
  }
  
  Ok(PathResult { 
    points: out_points, 
    closed: out_closed 
  })
}

/// Checks if two points are equal within a small epsilon to account for floating point precision
#[inline]
fn is_point_equal(p1: Point, p2: Point) -> bool {
  const EPSILON: f64 = 1e-9;
  (p1.x - p2.x).abs() < EPSILON && (p1.y - p2.y).abs() < EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points_on_path() {
        let path = "M532.094,806.71c6.595,91.184 45.177,175.149 106.414,244.044c-18.85,3.17 -38.222,4.822 -57.982,4.822c-189.194,-0 -342.795,-151.386 -342.795,-337.85c0,-186.465 153.601,-337.851 342.795,-337.851c57.886,0 112.441,14.172 160.283,39.185c-75.737,49.42 -135.471,115.393 -171.506,191.444c-11.119,-4.197 -23.167,-6.494 -35.747,-6.494c-55.939,0 -101.355,45.416 -101.355,101.355c0,55.452 44.627,100.562 99.893,101.345Z";
        let result = points_on_path(path.to_string(), Some(0.5), None).unwrap();
        // Ensure we receive at least one set of points.
        assert!(!result.is_empty());
        // Ensure that the first set contains more than one point.
        assert!(result[0].len() > 1);
        println!("Points: {:?}", result);
    }
    
    #[test]
    fn test_closed_path_detection() {
        // Test with explicit Z command
        let closed_path = "M10,10 L20,10 L20,20 L10,20 Z";
        let result = points_on_path_with_closed_info(closed_path.to_string(), None, None).unwrap();
        assert!(!result.points.is_empty());
        assert!(result.closed[0], "Path with Z command should be detected as closed");
        println!("Points: {:?}", result);
        
        // Test geometrically closed path (first point equals last point)
        let implicitly_closed = "M10,10 L20,10 L20,20 L10,20 L10,10";
        let result = points_on_path_with_closed_info(implicitly_closed.to_string(), None, None).unwrap();
        assert!(!result.points.is_empty());
        assert!(result.closed[0], "Path with first=last point should be detected as closed");
        println!("Points: {:?}", result);
        
        // Test open path
        let open_path = "M10,10 L20,10 L20,20 L10,20";
        let result = points_on_path_with_closed_info(open_path.to_string(), None, None).unwrap();
        assert!(!result.points.is_empty());
        assert!(!result.closed[0], "Open path should be detected as not closed");
        println!("Points: {:?}", result);
    }
}