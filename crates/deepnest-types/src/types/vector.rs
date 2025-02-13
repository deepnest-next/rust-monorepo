use derive_more::{From, Into};

/// Vector
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Copy, From, Into)]
pub struct Vector {
  pub x: f64,
  pub y: f64,
}
