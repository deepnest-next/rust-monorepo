#![deny(clippy::all)]
use clipper2_sys::{
  ClipType as ClipTypeOrig, FillRule, Path64, PathD, Paths64, PathsD, Point64, PointD,
};

#[macro_use]
extern crate napi_derive;

#[napi]
#[derive(Debug)]
pub enum PolyType {
  Subject,
  Clip,
}

#[napi]
#[derive(Debug)]
pub enum ClipType {
  None,
  Intersection,
  Union,
  Difference,
  Xor,
}

#[napi]
#[derive(Debug)]
pub enum FillType {
  EvenOdd,
  NonZero,
  Positive,
  Negative,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[napi(object)]
pub struct PointFloat64 {
  pub x: f64,
  pub y: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[napi(object)]
pub struct Point {
  pub x: i64,
  pub y: i64,
}

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

// Minkowski SUM for Integer 64
#[napi]
pub fn minkowski_sum_i64(a: Vec<Point>, b: Vec<Point>, closed: bool) -> Vec<Vec<Point>> {
  let mut path_a: Vec<clipper2_sys::Point64> = vec![];
  let mut path_b: Vec<clipper2_sys::Point64> = vec![];
  for p in a {
    path_a.push(Point64::new(p.x, p.y));
  }
  for p in b {
    path_b.push(Point64::new(p.x, p.y));
  }
  let path_a = Path64::new(&path_a);
  let path_b = Path64::new(&path_b);
  let path_c = path_a.minkowski_sum(&path_b, closed);
  let mut result = vec![];

  for i in 0..path_c.len() {
    let p = path_c.get_path(i);
    let mut path = vec![];
    for j in 0..p.len() {
      let point = p.get_point(j);
      path.push(Point {
        x: point.x as i64,
        y: point.y as i64,
      });
    }
    result.push(path);
  }
  result
}

// Minkowski SUM for Float64
#[napi]
pub fn minkowski_sum_f64(
  a: Vec<PointFloat64>,
  b: Vec<PointFloat64>,
  closed: bool,
  precision: i32, // 10^precision
) -> Vec<Vec<PointFloat64>> {
  let mut path_a: Vec<clipper2_sys::PointD> = vec![];
  let mut path_b: Vec<clipper2_sys::PointD> = vec![];
  for p in a {
    path_a.push(PointD::new(p.x, p.y));
  }
  for p in b {
    path_b.push(PointD::new(p.x, p.y));
  }
  let path_a = PathD::new(&path_a);
  let path_b = PathD::new(&path_b);
  let path_c = path_a.minkowski_sum(&path_b, closed, precision);
  let mut result = vec![];

  for i in 0..path_c.len() {
    let p = path_c.get_path(i);
    let mut path = vec![];
    for j in 0..p.len() {
      let point = p.get_point(j);
      path.push(PointFloat64 {
        x: point.x as f64,
        y: point.y as f64,
      });
    }
    result.push(path);
  }
  result
}

#[napi]
pub struct Clipper {
  clipper: clipper2_sys::Clipper64,
}

#[napi]
impl Clipper {
  #[napi]
  pub fn new() -> Self {
    Clipper {
      clipper: clipper2_sys::Clipper64::new(),
    }
  }
  #[napi]
  pub fn add_paths(&mut self, path: Vec<Vec<Point>>, poly_type: PolyType) {
    let mut path_a = Paths64::new(&vec![]);
    for p in path {
      let mut path_b = Path64::new(&vec![]);
      for p in p {
        path_b.add_point(Point64::new(p.x, p.y));
      }
      path_a.add_path(path_b);
    }
    if let PolyType::Clip = poly_type {
      self.clipper.add_clip(path_a);
    } else if let PolyType::Subject = poly_type {
      self.clipper.add_subject(path_a);
    } else {
      panic!("Invalid PolyType");
    }
  }

  #[napi]
  pub fn execute(&mut self, clip_type: ClipType, fill_type: FillType) -> Vec<Vec<Point>> {
    let clip_type = match clip_type {
      ClipType::None => ClipTypeOrig::None,
      ClipType::Intersection => ClipTypeOrig::Intersection,
      ClipType::Union => ClipTypeOrig::Union,
      ClipType::Difference => ClipTypeOrig::Difference,
      ClipType::Xor => ClipTypeOrig::Xor,
    };
    let fill_type = match fill_type {
      FillType::EvenOdd => FillRule::EvenOdd,
      FillType::NonZero => FillRule::NonZero,
      FillType::Positive => FillRule::Positive,
      FillType::Negative => FillRule::Negative,
    };
    let path_c = self.clipper.boolean_operation(clip_type, fill_type);
    let mut result = vec![];

    for i in 0..path_c.len() {
      let p = path_c.get_path(i);
      let mut path = vec![];
      for j in 0..p.len() {
        let point = p.get_point(j);
        path.push(Point {
          x: point.x as i64,
          y: point.y as i64,
        });
      }
      result.push(path);
    }
    result
  }

  #[napi]
  pub fn clear(&mut self) {
    self.clipper.clear();
  }
}

#[napi]
pub struct ClipperFloat64 {
  clipper: clipper2_sys::ClipperD,
}

#[napi]
impl ClipperFloat64 {
  #[napi]
  pub fn new(precision: i32) -> Self {
    ClipperFloat64 {
      clipper: clipper2_sys::ClipperD::new(precision),
    }
  }
  #[napi]
  pub fn add_paths(&mut self, path: Vec<Vec<PointFloat64>>, poly_type: PolyType) {
    let mut path_a = PathsD::new(&vec![]);
    for p in path {
      let mut path_b = PathD::new(&vec![]);
      for p in p {
        path_b.add_point(PointD::new(p.x, p.y));
      }
      path_a.add_path(path_b);
    }
    if let PolyType::Clip = poly_type {
      self.clipper.add_clip(path_a);
    } else if let PolyType::Subject = poly_type {
      self.clipper.add_subject(path_a);
    } else {
      panic!("Invalid PolyType: {:?}", poly_type);
    }
  }

  #[napi]
  pub fn execute(&mut self, clip_type: ClipType, fill_type: FillType) -> Vec<Vec<PointFloat64>> {
    let clip_type = match clip_type {
      ClipType::None => ClipTypeOrig::None,
      ClipType::Intersection => ClipTypeOrig::Intersection,
      ClipType::Union => ClipTypeOrig::Union,
      ClipType::Difference => ClipTypeOrig::Difference,
      ClipType::Xor => ClipTypeOrig::Xor,
      _ => panic!("Invalid PolyType: {:?}", clip_type),
    };
    let fill_type = match fill_type {
      FillType::EvenOdd => FillRule::EvenOdd,
      FillType::NonZero => FillRule::NonZero,
      FillType::Positive => FillRule::Positive,
      FillType::Negative => FillRule::Negative,
      _ => panic!("Invalid PolyType: {:?}", fill_type),
    };
    let path_c = self.clipper.boolean_operation(clip_type, fill_type);
    let mut result = vec![];

    for i in 0..path_c.len() {
      let p = path_c.get_path(i);
      let mut path = vec![];
      for j in 0..p.len() {
        let point = p.get_point(j);
        path.push(PointFloat64 {
          x: point.x as f64,
          y: point.y as f64,
        });
      }
      result.push(path);
    }
    result
  }

  #[napi]
  pub fn clear(&mut self) {
    self.clipper.clear();
  }
}
