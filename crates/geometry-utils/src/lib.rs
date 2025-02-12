// lib.rs
#[cfg(feature = "node")]
#[macro_use]
extern crate napi_derive;

pub mod constants;
pub mod output_lib;
pub use crate::output_lib::*;

// pub mod output_napi;
// #[cfg(feature = "node")]
// pub use crate::output_napi::NodeGeometryUtils;

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn it_works() {
    let result = almost_equal(2., 2., Some(5.));
    assert_eq!(result, true);
  }
}
