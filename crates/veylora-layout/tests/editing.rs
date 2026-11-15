mod common;
use common::*;
use veylora_domain::layout::Rotation;
use veylora_layout::geometry::placement_rect;
use veylora_layout::{
    align, distribute, resize_width_locked, rotate_quarter_turns, set_size, Alignment, Axis,
    EditError, Rect,
};

#[test]
fn aspect_lock_scales_height_with_round_half_up() {
    let p = placement(0, 0, 30_000, 20_000, Rotation::Deg0);
    let q = resize_width_locked(&p, 45_000).unwrap();
    assert_eq!((q.width_um, q.height_um), (45_000, 30_000));
    // 20_000 * 10_001 / 30_000 = 6_667.33 -> 6_667 ; 20_000 * 10_002 / 30_000 = 6_668
    assert_eq!(resize_width_locked(&p, 10_001).unwrap().height_um, 6_667);
    assert_eq!(resize_width_locked(&p, 10_002).unwrap().height_um, 6_668);
    // exact half rounds up: 3 * 1 / 2 -> use 3 x 1 aspect => width 1 gives 0.333; use 1:3 ratio
    let r = placement(0, 0, 2, 3, Rotation::Deg0);
    assert_eq!(resize_width_locked(&r, 1).unwrap().height_um, 2); // 1.5 -> 2
}

#[test]
fn aspect_lock_keeps_identity_crop_and_position() {
    let p = placement(7, 9, 30_000, 20_000, Rotation::Deg90);
    let q = resize_width_locked(&p, 15_000).unwrap();
    assert_eq!(
        (q.id, q.asset_id, q.x_um, q.y_um, q.rotation),
        (p.id, p.asset_id, 7, 9, Rotation::Deg90)
    );
    assert_eq!(q.crop_width_px, p.crop_width_px);
}

#[test]
fn degenerate_sizes_are_rejected() {
    let p = placement(0, 0, 30_000, 20_000, Rotation::Deg0);
    assert_eq!(resize_width_locked(&p, 0), Err(EditError::ZeroSize));
    // A height that would round to zero is rejected rather than clamped.
    let thin = placement(0, 0, 30_000, 20, Rotation::Deg0);
    assert_eq!(resize_width_locked(&thin, 1), Err(EditError::ZeroSize));
}

#[test]
fn numeric_size_sets_both_dimensions() {
    let p = placement(0, 0, 30_000, 20_000, Rotation::Deg0);
    let q = set_size(&p, 10, 20).unwrap();
    assert_eq!((q.width_um, q.height_um), (10, 20));
    assert_eq!(set_size(&p, 0, 5), Err(EditError::ZeroSize));
}

#[test]
fn quarter_turns_compose_and_wrap() {
    let p = placement(0, 0, 30_000, 20_000, Rotation::Deg270);
    assert_eq!(rotate_quarter_turns(&p, 1).rotation, Rotation::Deg0);
    assert_eq!(rotate_quarter_turns(&p, 4).rotation, Rotation::Deg270);
    assert_eq!(rotate_quarter_turns(&p, -1).rotation, Rotation::Deg180);
    assert_eq!(rotate_quarter_turns(&p, 1).id, p.id);
}

#[test]
fn align_left_to_the_selection_moves_only_x() {
    let mut a = placement(10_000, 1_000, 5_000, 5_000, Rotation::Deg0);
    let mut b = placement(30_000, 2_000, 8_000, 5_000, Rotation::Deg0);
    let ids = [a.id, b.id];
    align(&mut [&mut a, &mut b], &ids, Alignment::Left);
    assert_eq!((a.x_um, b.x_um), (10_000, 10_000));
    assert_eq!((a.y_um, b.y_um), (1_000, 2_000));
}

#[test]
fn align_uses_rotated_extents() {
    let mut a = placement(0, 0, 4_000, 10_000, Rotation::Deg0);
    let mut b = placement(50_000, 0, 10_000, 4_000, Rotation::Deg90); // bbox 4_000 x 10_000
    let ids = [a.id, b.id];
    align(&mut [&mut a, &mut b], &ids, Alignment::Right);
    assert_eq!(placement_rect(&a).right(), placement_rect(&b).right());
    assert_eq!(placement_rect(&b).right(), 54_000);
}

#[test]
fn align_center_floors_for_odd_differences() {
    let mut a = placement(0, 0, 10_000, 1_000, Rotation::Deg0);
    let mut b = placement(0, 5_000, 3_001, 1_000, Rotation::Deg0);
    let ids = [a.id, b.id];
    align(&mut [&mut a, &mut b], &ids, Alignment::HorizontalCenter);
    // selection spans 0..10_000, centre 5_000; b.x = 5_000 - 3_001/2 (floor 1_500) = 3_500
    assert_eq!(b.x_um, 3_500);
}

#[test]
fn distribute_makes_equal_gaps_and_keeps_the_outer_two_fixed() {
    let mut a = placement(0, 0, 10_000, 5_000, Rotation::Deg0);
    let mut b = placement(12_000, 0, 20_000, 5_000, Rotation::Deg0);
    let mut c = placement(100_000, 0, 10_000, 5_000, Rotation::Deg0);
    let ids = [a.id, b.id, c.id];
    distribute(&mut [&mut a, &mut b, &mut c], &ids, Axis::Horizontal);
    // span 0..110_000, content 40_000, 70_000 of gap -> 35_000 each
    assert_eq!((a.x_um, b.x_um, c.x_um), (0, 45_000, 100_000));
}

#[test]
fn distribute_assigns_the_remainder_to_the_earliest_gaps() {
    let mut a = placement(0, 0, 10, 5, Rotation::Deg0);
    let mut b = placement(1, 0, 10, 5, Rotation::Deg0);
    let mut c = placement(2, 0, 10, 5, Rotation::Deg0);
    let mut d = placement(33, 0, 10, 5, Rotation::Deg0);
    let ids = [a.id, b.id, c.id, d.id];
    distribute(
        &mut [&mut a, &mut b, &mut c, &mut d],
        &ids,
        Axis::Horizontal,
    );
    // span 0..43, content 40, gap total 3 over 3 gaps -> 1 each
    assert_eq!((a.x_um, b.x_um, c.x_um, d.x_um), (0, 11, 22, 33));
    let mut a = placement(0, 0, 10, 5, Rotation::Deg0);
    let mut b = placement(1, 0, 10, 5, Rotation::Deg0);
    let mut c = placement(2, 0, 10, 5, Rotation::Deg0);
    let mut d = placement(34, 0, 10, 5, Rotation::Deg0);
    let ids = [a.id, b.id, c.id, d.id];
    distribute(
        &mut [&mut a, &mut b, &mut c, &mut d],
        &ids,
        Axis::Horizontal,
    );
    // gap total 4 over 3 gaps -> 2,1,1 ; earliest gap takes the extra micrometre
    assert_eq!((a.x_um, b.x_um, c.x_um, d.x_um), (0, 12, 23, 34));
}

#[test]
fn selection_ignores_unlisted_placements_and_unknown_ids() {
    let mut a = placement(10_000, 0, 5_000, 5_000, Rotation::Deg0);
    let mut b = placement(30_000, 0, 5_000, 5_000, Rotation::Deg0);
    let mut c = placement(50_000, 0, 5_000, 5_000, Rotation::Deg0);
    let ids = [a.id, b.id, veylora_domain::ids::PlacementId::new()];
    align(&mut [&mut a, &mut b, &mut c], &ids, Alignment::Left);
    assert_eq!((a.x_um, b.x_um, c.x_um), (10_000, 10_000, 50_000));
    let _ = Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };
}
