//! Physical-unit layout geometry. Everything here is integer micrometres so that
//! preview and production resolve one identical transform.
//!
//! Dependencies point one way: this crate reads domain and device contracts and
//! owns no rendering, persistence, or transport logic.

pub mod editing;
pub mod geometry;
pub mod transforms;
pub mod validate;

pub use editing::{
    align, distribute, resize_width_locked, rotate_quarter_turns, set_size, Alignment, Axis,
    EditError,
};
pub use geometry::Rect;
pub use transforms::{resolve_transform, LayoutError, ResolvedLayout, ResolvedPlacement};
pub use validate::{validate_layout, LayoutPolicy};
