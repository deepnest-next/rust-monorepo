#[allow(unused_imports)]
#[cfg(feature = "node")]
use napi::bindgen_prelude::*;
#[cfg(feature = "node")]
#[macro_use]
extern crate napi_derive;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod vector;

pub use point::*;
pub use polygon::*;
pub use rect::*;
pub use vector::*;