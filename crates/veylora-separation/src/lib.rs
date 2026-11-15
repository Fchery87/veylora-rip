//! Separation of source planes into logical white and process planes, and allocation to physical
//! slots. Policy values come from the domain recipe and channel maps from the device setup; this
//! crate owns only the numerical processing.
#![forbid(unsafe_code)]

mod allocation;
mod morphology;
mod plane;
mod white;

pub use allocation::{
    allocate_logical_to_physical, AllocatedPlane, AllocationError, AllocationLimits, LogicalPlanes,
    PhysicalPlanes,
};
pub use morphology::MAX_MORPHOLOGY_RADIUS_PX;
pub use plane::{Plane, SeparationError};
pub use white::{generate_white, PixelPitch, SeparationInput, SuppliedMask};
