use clipper2_sys::{Point64, PointD, Path64, PathD, Paths64, PathsD};
use crate::types::Point as DnPoint;

pub trait ToDnPoint {
  fn to_dn_point(&self) -> DnPoint;
}

impl ToDnPoint for Point64 {
  fn to_dn_point(&self) -> DnPoint {
    DnPoint {
      x: self.x as f64,
      y: self.y as f64,
    }
  }
}

impl ToDnPoint for PointD {
  fn to_dn_point(&self) -> DnPoint {
    DnPoint {
      x: self.x,
      y: self.y,
    }
  }
}

/// --- New conversion traits for converting clipper2 paths to deepnest-types point vectors ---

pub trait ToDnPath {
  /// Converts a clipper2 path into a `Vec` of deepnest Points.
  fn to_dn_path(&self) -> Vec<DnPoint>;
}

impl ToDnPath for Path64 {
  fn to_dn_path(&self) -> Vec<DnPoint> {
    let mut result = Vec::with_capacity(self.len());
    for i in 0..self.len() {
      let point = self.get_point(i);
      result.push(point.to_dn_point());
    }
    result
  }
}

impl ToDnPath for PathD {
  fn to_dn_path(&self) -> Vec<DnPoint> {
    let mut result = Vec::with_capacity(self.len());
    for i in 0..self.len() {
      let point = self.get_point(i);
      result.push(point.to_dn_point());
    }
    result
  }
}

impl ToDnPaths for Paths64 {
  fn to_dn_paths(&self) -> Vec<Vec<DnPoint>> {
    let mut result = Vec::with_capacity(self.len());
    for i in 0..self.len() {
      result.push(self.get_path(i).to_dn_path());
    }
    result
  }
}

impl ToDnPaths for PathsD {
  fn to_dn_paths(&self) -> Vec<Vec<DnPoint>> {
    let mut result = Vec::with_capacity(self.len());
    for i in 0..self.len() {
      result.push(self.get_path(i).to_dn_path());
    }
    result
  }
}
pub trait ToDnPaths {
  /// Converts clipper2 paths into a `Vec` of deepnest point vectors.
  fn to_dn_paths(&self) -> Vec<Vec<DnPoint>>;
}

/// --- New conversion traits for converting FROM deepnest point to clipper2 points ---

pub trait ToClipperPoint64 {
  fn to_clipper_point64(&self) -> Point64;
}

impl ToClipperPoint64 for DnPoint {
  fn to_clipper_point64(&self) -> Point64 {
    // For a Point64, we convert the f64 values by rounding.
    Point64 {
      x: self.x.round() as i64,
      y: self.y.round() as i64,
    }
  }
}

pub trait ToClipperPointD {
  fn to_clipper_point_d(&self) -> PointD;
}

impl ToClipperPointD for DnPoint {
  fn to_clipper_point_d(&self) -> PointD {
    PointD {
      x: self.x,
      y: self.y,
    }
  }
}
