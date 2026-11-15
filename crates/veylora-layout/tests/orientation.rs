mod common;
use common::*;
use veylora_device::capability::MirrorOwner;
use veylora_domain::layout::Rotation;
use veylora_layout::{resolve_transform, LayoutError, Rect};

// A 100 mm x 20 mm ruler placed at (10 mm, 5 mm) on a 200 x 100 mm page.
// Ruler zero is at its left end; source point (sx, sy) is in pre-rotation um.
fn ruler(rotation: Rotation, mirror: bool, owner: MirrorOwner) -> veylora_layout::ResolvedLayout {
    let l = layout(
        200_000,
        100_000,
        mirror,
        vec![placement(10_000, 5_000, 100_000, 20_000, rotation)],
    );
    resolve_transform(&l, owner).unwrap()
}

#[test]
fn unmirrored_ruler_zero_stays_at_the_left_end() {
    let r = ruler(Rotation::Deg0, false, MirrorOwner::Operator);
    let p = &r.placements[0];
    assert_eq!(
        p.rect,
        Rect {
            x: 10_000,
            y: 5_000,
            width: 100_000,
            height: 20_000
        }
    );
    assert_eq!(p.map_source_um(0, 0), (10_000, 5_000));
    assert_eq!(p.map_source_um(100_000, 20_000), (110_000, 25_000));
    assert!(!r.mirror_applied);
}

#[test]
fn rip_owned_mirror_moves_ruler_zero_to_the_right_end_once() {
    let r = ruler(Rotation::Deg0, true, MirrorOwner::Rip);
    let p = &r.placements[0];
    // x' = page_width - x; the placement box reflects about the page centre line.
    assert_eq!(
        p.rect,
        Rect {
            x: 90_000,
            y: 5_000,
            width: 100_000,
            height: 20_000
        }
    );
    assert_eq!(p.map_source_um(0, 0), (190_000, 5_000));
    assert_eq!(p.map_source_um(100_000, 0), (90_000, 5_000));
    assert!(r.mirror_applied && p.mirrored);
}

#[test]
fn off_centre_marker_follows_the_same_mirror_as_the_artwork() {
    // 100 x 100 mm artwork with a registration mark 10 mm from its left, 20 mm from its top.
    let l = layout(
        200_000,
        100_000,
        true,
        vec![placement(0, 0, 100_000, 100_000, Rotation::Deg0)],
    );
    let r = resolve_transform(&l, MirrorOwner::Rip).unwrap();
    assert_eq!(
        r.placements[0].map_source_um(10_000, 20_000),
        (190_000, 20_000)
    );
}

#[test]
fn asymmetric_text_corners_rotate_clockwise_about_the_anchor() {
    // Source is 40 wide x 10 high. Its top-left corner is at (0,0).
    let at = |rot| {
        let l = layout(
            100_000,
            100_000,
            false,
            vec![placement(0, 0, 40_000, 10_000, rot)],
        );
        let r = resolve_transform(&l, MirrorOwner::None).unwrap();
        let p = r.placements[0].clone();
        (p.rect, p.map_source_um(0, 0), p.map_source_um(40_000, 0))
    };
    let (rect, tl, tr) = at(Rotation::Deg90);
    assert_eq!(
        rect,
        Rect {
            x: 0,
            y: 0,
            width: 10_000,
            height: 40_000
        }
    );
    assert_eq!((tl, tr), ((10_000, 0), (10_000, 40_000)));
    let (rect, tl, tr) = at(Rotation::Deg180);
    assert_eq!(
        rect,
        Rect {
            x: 0,
            y: 0,
            width: 40_000,
            height: 10_000
        }
    );
    assert_eq!((tl, tr), ((40_000, 10_000), (0, 10_000)));
    let (rect, tl, tr) = at(Rotation::Deg270);
    assert_eq!(
        rect,
        Rect {
            x: 0,
            y: 0,
            width: 10_000,
            height: 40_000
        }
    );
    assert_eq!((tl, tr), ((0, 40_000), (0, 0)));
}

#[test]
fn rotation_and_mirror_compose_with_the_mirror_last() {
    let l = layout(
        100_000,
        100_000,
        true,
        vec![placement(0, 0, 40_000, 10_000, Rotation::Deg90)],
    );
    let r = resolve_transform(&l, MirrorOwner::Rip).unwrap();
    let p = &r.placements[0];
    // Rotated top-left lands at (10_000, 0); mirror sends x to 100_000 - 10_000.
    assert_eq!(p.map_source_um(0, 0), (90_000, 0));
    assert_eq!(
        p.rect,
        Rect {
            x: 90_000,
            y: 0,
            width: 10_000,
            height: 40_000
        }
    );
}

#[test]
fn mirror_requires_exactly_one_owner() {
    let l = layout(
        100_000,
        100_000,
        true,
        vec![placement(0, 0, 1_000, 1_000, Rotation::Deg0)],
    );
    for owner in [MirrorOwner::Operator, MirrorOwner::None] {
        assert_eq!(
            resolve_transform(&l, owner),
            Err(LayoutError::MirrorOwnerConflict { owner })
        );
    }
    assert!(resolve_transform(&l, MirrorOwner::Rip).is_ok());
}

#[test]
fn resolution_is_deterministic() {
    let l = layout(
        200_000,
        100_000,
        true,
        vec![placement(1, 2, 3_000, 4_000, Rotation::Deg270)],
    );
    assert_eq!(
        resolve_transform(&l, MirrorOwner::Rip),
        resolve_transform(&l, MirrorOwner::Rip)
    );
}
