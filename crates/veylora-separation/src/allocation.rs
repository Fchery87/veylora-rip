//! Allocation of logical planes to physical slots through a device channel map.
use crate::Plane;
use std::collections::BTreeMap;
use veylora_device::channels::{
    LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalPlanes {
    pub c: Plane,
    pub m: Plane,
    pub y: Plane,
    pub k: Plane,
    pub w: Plane,
}
impl LogicalPlanes {
    pub fn get(&self, channel: LogicalChannel) -> &Plane {
        match channel {
            LogicalChannel::Cyan => &self.c,
            LogicalChannel::Magenta => &self.m,
            LogicalChannel::Yellow => &self.y,
            LogicalChannel::Black => &self.k,
            LogicalChannel::White => &self.w,
        }
    }
    fn all(&self) -> [&Plane; 5] {
        [&self.c, &self.m, &self.y, &self.k, &self.w]
    }
}
/// A plane bound to a physical slot, addressed by its protocol channel.
#[derive(Clone, Debug, PartialEq)]
pub struct AllocatedPlane {
    pub slot: PhysicalSlotId,
    pub plane: Plane,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalPlanes {
    pub slots: BTreeMap<ProtocolChannel, AllocatedPlane>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationLimits {
    /// Cap on the total samples across all allocated physical planes.
    pub max_total_pixels: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocationError {
    DimensionMismatch,
    InvalidPlane,
    LimitExceeded,
}

/// Each binding receives its logical plane scaled by the binding's fraction, so a white channel
/// split across two slots never duplicates full white coverage.
pub fn allocate_logical_to_physical(
    logical: &LogicalPlanes,
    map: &LogicalChannelMap,
    limits: &AllocationLimits,
) -> Result<PhysicalPlanes, AllocationError> {
    for plane in logical.all() {
        plane
            .validate()
            .map_err(|_| AllocationError::InvalidPlane)?;
        if !plane.same_size(&logical.c) {
            return Err(AllocationError::DimensionMismatch);
        }
    }
    let per_plane = logical.c.pixels.len();
    if map
        .bindings()
        .len()
        .checked_mul(per_plane)
        .is_none_or(|total| total > limits.max_total_pixels)
    {
        return Err(AllocationError::LimitExceeded);
    }
    let mut slots = BTreeMap::new();
    for binding in map.bindings() {
        let source = logical.get(binding.logical);
        let fraction = binding.fraction as f32;
        let plane = Plane {
            width: source.width,
            height: source.height,
            pixels: source.pixels.iter().map(|v| v * fraction).collect(),
        };
        slots.insert(
            binding.protocol,
            AllocatedPlane {
                slot: binding.slot,
                plane,
            },
        );
    }
    Ok(PhysicalPlanes { slots })
}
