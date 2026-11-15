use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_separation::{
    allocate_logical_to_physical, AllocationError, AllocationLimits, LogicalPlanes, Plane,
};

const LIMITS: AllocationLimits = AllocationLimits {
    max_total_pixels: 1_000,
};

fn planes(w: usize, h: usize) -> LogicalPlanes {
    let p = |v| Plane::new(w, h, v);
    LogicalPlanes {
        c: p(0.1),
        m: p(0.2),
        y: p(0.3),
        k: p(0.4),
        w: p(0.8),
    }
}
fn binding(l: LogicalChannel, slot: u8, protocol: u8, fraction: f64) -> ChannelBinding {
    ChannelBinding {
        logical: l,
        slot: PhysicalSlotId(slot),
        protocol: ProtocolChannel(protocol),
        fraction,
    }
}
fn process() -> Vec<ChannelBinding> {
    use LogicalChannel::*;
    vec![
        binding(Cyan, 0, 10, 1.0),
        binding(Magenta, 1, 11, 1.0),
        binding(Yellow, 2, 12, 1.0),
        binding(Black, 3, 13, 1.0),
    ]
}
fn map(white: Vec<ChannelBinding>) -> LogicalChannelMap {
    let mut b = process();
    b.extend(white);
    LogicalChannelMap::new(1, b).unwrap()
}

#[test]
fn one_white_slot_receives_the_full_white_plane() {
    let m = map(vec![binding(LogicalChannel::White, 4, 14, 1.0)]);
    let out = allocate_logical_to_physical(&planes(2, 1), &m, &LIMITS).unwrap();
    assert_eq!(out.slots.len(), 5);
    let w = &out.slots[&ProtocolChannel(14)];
    assert_eq!(w.slot, PhysicalSlotId(4));
    assert_eq!(w.plane.pixels, vec![0.8, 0.8]);
    assert_eq!(out.slots[&ProtocolChannel(10)].plane.pixels, vec![0.1, 0.1]);
}
#[test]
fn two_white_slots_split_coverage_without_duplicating_it() {
    let m = map(vec![
        binding(LogicalChannel::White, 4, 14, 0.5),
        binding(LogicalChannel::White, 5, 15, 0.5),
    ]);
    let out = allocate_logical_to_physical(&planes(1, 1), &m, &LIMITS).unwrap();
    let a = out.slots[&ProtocolChannel(14)].plane.pixels[0];
    let b = out.slots[&ProtocolChannel(15)].plane.pixels[0];
    assert!((a + b - 0.8).abs() < 1e-6, "{a} + {b}");
}
#[test]
fn unavailable_white_slot_is_a_map_error_before_allocation() {
    // A map without any white binding cannot be constructed, so allocation never sees one.
    assert!(LogicalChannelMap::new(1, process()).is_err());
}
#[test]
fn mismatched_invalid_and_oversize_planes_are_rejected() {
    let m = map(vec![binding(LogicalChannel::White, 4, 14, 1.0)]);
    let mut p = planes(2, 1);
    p.w = Plane::new(3, 1, 0.5);
    assert_eq!(
        allocate_logical_to_physical(&p, &m, &LIMITS),
        Err(AllocationError::DimensionMismatch)
    );
    let mut p = planes(2, 1);
    p.k.pixels[0] = f32::NAN;
    assert_eq!(
        allocate_logical_to_physical(&p, &m, &LIMITS),
        Err(AllocationError::InvalidPlane)
    );
    let tight = AllocationLimits {
        max_total_pixels: 9,
    };
    assert_eq!(
        allocate_logical_to_physical(&planes(2, 1), &m, &tight),
        Err(AllocationError::LimitExceeded)
    );
    let exact = AllocationLimits {
        max_total_pixels: 10,
    };
    assert!(allocate_logical_to_physical(&planes(2, 1), &m, &exact).is_ok());
}
