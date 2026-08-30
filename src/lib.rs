pub use direction::{Direction, DIR2_EDGE_ONCE, DIR4, DIR4_OPPOSITE, DIR8, DIR9};
pub use one_direction::{OneDimensionConvertError, OneDimensionalDirection};
pub use point::Point;

pub mod circles;
pub mod cp437;
mod direction;
mod one_direction;
mod point;

pub type Vec2 = vek::Vec2<f32>;
pub type Rect = vek::Rect<f32, f32>;
