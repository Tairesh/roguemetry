pub use direction::{Direction, DIR2_EDGE_ONCE, DIR4, DIR4_OPPOSITE, DIR8, DIR9};
pub use fov::{field_of_view, Fov};
pub use grid_size::{GridSize, GridSizeError};
pub use one_direction::{OneDimensionConvertError, OneDimensionalDirection};
pub use point::Point;

pub mod circles;
pub mod cp437;
mod direction;
mod fov;
mod grid_size;
mod one_direction;
mod point;

pub type Vec2 = vek::Vec2<f32>;
pub type Rect = vek::Rect<f32, f32>;
