pub mod arc;
pub mod cubic_bezier;

// lib.rs

#[macro_use]
extern crate napi_derive;

pub mod constants;
pub mod geometryutils;
pub mod quadratic_bezier;
pub use crate::geometryutils::*;


#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_works() {
    let result = GeometryUtils::almost_equal(2., 2., Some(5.));
    assert_eq!(result, true);
  }
}
