#[cfg(feature = "node")]
use napi::bindgen_prelude::*;
#[allow(unused_imports)]
use delegate::delegate;
use derive_more::{From, Into};

#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Copy, From, Into)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}