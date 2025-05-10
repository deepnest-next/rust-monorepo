#[macro_use]
extern crate napi_derive;

pub mod arc;
pub mod constants;
pub mod cubic_bezier;
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
