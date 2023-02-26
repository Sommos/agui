pub(crate) const POS_MARGIN_OF_ERROR: f32 = 0.01;
pub(crate) const COLOR_MARGIN_OF_ERROR: f32 = 0.001;

mod axis;
mod blend_mode;
mod bounds;
mod clip_behavior;
mod color;
mod constraints;
mod data;
mod edge_insets;
mod font;
mod intrinsic_dimension;
mod key;
mod point;
mod rect;
mod shape;
mod size;
mod text_direction;

pub use self::axis::*;
pub use self::blend_mode::*;
pub use self::bounds::*;
pub use self::clip_behavior::*;
pub use self::color::*;
pub use self::constraints::*;
pub use self::data::*;
pub use self::edge_insets::*;
pub use self::font::*;
pub use self::intrinsic_dimension::*;
pub use self::key::*;
pub use self::point::*;
pub use self::rect::*;
pub use self::shape::*;
pub use self::size::*;
pub use self::text_direction::*;
