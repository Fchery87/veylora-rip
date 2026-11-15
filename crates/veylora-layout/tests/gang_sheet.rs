mod common;
use common::*;
use veylora_domain::{ids::PlacementId, layout::Rotation};
use veylora_layout::{
    block_nest, duplicate_row, geometry::placement_rect, subset, GangSheet, NestConstraints,
    SheetError,
};

fn sheet(page: (u32, u32), items: Vec<veylora_domain::layout::Placement>) -> GangSheet {
    GangSheet::new(page.0, page.1, items)
}

#[test]
fn duplicates_keep_the_source_asset_crop_and_size_with_new_identities() {
    let mut p = placement(5_000, 5_000, 20_000, 10_000, Rotation::Deg0);
    p.crop_x_px = 12;
    p.crop_y_px = 34;
    p.crop_width_px = 500;
    p.crop_height_px = 250;
    let original = p.clone();
    let mut s = sheet((200_000, 100_000), vec![p]);
    let ids = duplicate_row(&mut s, original.id, 2, 1_000, &mut PlacementId::new).unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(s.placements.len(), 3);
    for id in &ids {
        let c = s.placements.iter().find(|p| p.id == *id).unwrap();
        assert_ne!(c.id, original.id);
        assert_eq!(c.asset_id, original.asset_id);
        assert_eq!(
            (c.crop_x_px, c.crop_y_px, c.crop_width_px, c.crop_height_px),
            (12, 34, 500, 250)
        );
        assert_eq!(
            (c.width_um, c.height_um, c.rotation),
            (20_000, 10_000, Rotation::Deg0)
        );
        assert_eq!(c.y_um, 5_000);
    }
    assert_eq!(s.placements[0], original, "source placement is untouched");
}

#[test]
fn row_copies_have_exact_gutters_using_rotated_extents() {
    let p = placement(0, 0, 20_000, 10_000, Rotation::Deg90); // bbox 10_000 wide
    let id = p.id;
    let mut s = sheet((200_000, 100_000), vec![p]);
    duplicate_row(&mut s, id, 3, 2_500, &mut PlacementId::new).unwrap();
    let xs: Vec<i64> = s.placements.iter().map(|p| placement_rect(p).x).collect();
    assert_eq!(xs, vec![0, 12_500, 25_000, 37_500]);
}

#[test]
fn editing_a_copy_never_changes_the_original() {
    let p = placement(0, 0, 20_000, 10_000, Rotation::Deg0);
    let id = p.id;
    let mut s = sheet((200_000, 100_000), vec![p.clone()]);
    let ids = duplicate_row(&mut s, id, 1, 0, &mut PlacementId::new).unwrap();
    let copy = s.placements.iter_mut().find(|p| p.id == ids[0]).unwrap();
    copy.width_um = 1;
    assert_eq!(s.placements[0], p);
}

#[test]
fn duplicating_an_unknown_placement_fails() {
    let mut s = sheet((100, 100), vec![placement(0, 0, 10, 10, Rotation::Deg0)]);
    let unknown = PlacementId::new();
    assert_eq!(
        duplicate_row(&mut s, unknown, 1, 0, &mut PlacementId::new),
        Err(SheetError::UnknownPlacement(unknown))
    );
}

#[test]
fn block_nest_is_deterministic_and_independent_of_input_order() {
    let a = placement(0, 0, 40_000, 30_000, Rotation::Deg0);
    let b = placement(0, 0, 40_000, 20_000, Rotation::Deg0);
    let c = placement(0, 0, 30_000, 30_000, Rotation::Deg0);
    let cons = NestConstraints {
        gutter_um: 2_000,
        margin_um: 3_000,
    };
    let mut s1 = sheet((100_000, 100_000), vec![a.clone(), b.clone(), c.clone()]);
    let mut s2 = sheet((100_000, 100_000), vec![c.clone(), a.clone(), b.clone()]);
    block_nest(&mut s1, &cons).unwrap();
    block_nest(&mut s2, &cons).unwrap();
    let pos = |s: &GangSheet, id| {
        let p = s.placements.iter().find(|p| p.id == id).unwrap();
        (p.x_um, p.y_um)
    };
    for id in [a.id, b.id, c.id] {
        assert_eq!(pos(&s1, id), pos(&s2, id));
    }
}

#[test]
fn block_nest_respects_margins_gutters_and_never_overlaps() {
    let items = vec![
        placement(0, 0, 40_000, 30_000, Rotation::Deg0),
        placement(0, 0, 40_000, 20_000, Rotation::Deg0),
        placement(0, 0, 30_000, 30_000, Rotation::Deg0),
        placement(0, 0, 25_000, 10_000, Rotation::Deg90),
    ];
    let mut s = sheet((100_000, 100_000), items);
    let cons = NestConstraints {
        gutter_um: 2_000,
        margin_um: 3_000,
    };
    block_nest(&mut s, &cons).unwrap();
    let rects: Vec<_> = s.placements.iter().map(placement_rect).collect();
    for (i, r) in rects.iter().enumerate() {
        assert!(r.x >= 3_000 && r.y >= 3_000);
        assert!(r.right() <= 97_000 && r.bottom() <= 97_000);
        for other in &rects[i + 1..] {
            assert!(!r.intersects(other));
            // any two neighbours on a shelf or between shelves keep at least the gutter
            let dx = (other.x - r.right()).max(r.x - other.right());
            let dy = (other.y - r.bottom()).max(r.y - other.bottom());
            assert!(
                dx >= 2_000 || dy >= 2_000,
                "gutter violated between {r:?} and {other:?}"
            );
        }
    }
}

#[test]
fn block_nest_places_exact_gutters_on_a_shelf() {
    let mut s = sheet(
        (100_000, 100_000),
        vec![
            placement(0, 0, 20_000, 10_000, Rotation::Deg0),
            placement(0, 0, 20_000, 10_000, Rotation::Deg0),
        ],
    );
    block_nest(
        &mut s,
        &NestConstraints {
            gutter_um: 5_000,
            margin_um: 1_000,
        },
    )
    .unwrap();
    let mut xs: Vec<i64> = s.placements.iter().map(|p| p.x_um).collect();
    xs.sort();
    assert_eq!(xs, vec![1_000, 26_000]);
    assert!(s.placements.iter().all(|p| p.y_um == 1_000));
}

#[test]
fn block_nest_starts_a_new_shelf_below_the_tallest_item_plus_gutter() {
    let mut s = sheet(
        (50_000, 100_000),
        vec![
            placement(0, 0, 30_000, 20_000, Rotation::Deg0),
            placement(0, 0, 30_000, 10_000, Rotation::Deg0),
        ],
    );
    block_nest(
        &mut s,
        &NestConstraints {
            gutter_um: 4_000,
            margin_um: 0,
        },
    )
    .unwrap();
    let mut ys: Vec<i64> = s.placements.iter().map(|p| p.y_um).collect();
    ys.sort();
    assert_eq!(ys, vec![0, 24_000]);
}

#[test]
fn block_nest_reports_the_item_that_does_not_fit_and_leaves_the_sheet_unchanged() {
    let big = placement(7, 7, 120_000, 10_000, Rotation::Deg0);
    let id = big.id;
    let mut s = sheet((100_000, 100_000), vec![big]);
    let before = s.clone();
    assert_eq!(
        block_nest(
            &mut s,
            &NestConstraints {
                gutter_um: 0,
                margin_um: 0
            }
        ),
        Err(SheetError::DoesNotFit(id))
    );
    assert_eq!(s, before);
    // Height overflow, too.
    let tall = placement(0, 0, 10_000, 60_000, Rotation::Deg0);
    let tall2 = placement(0, 0, 10_000, 60_000, Rotation::Deg0);
    let mut s = sheet((15_000, 100_000), vec![tall, tall2]);
    assert!(matches!(
        block_nest(
            &mut s,
            &NestConstraints {
                gutter_um: 0,
                margin_um: 0
            }
        ),
        Err(SheetError::DoesNotFit(_))
    ));
}

#[test]
fn nesting_preserves_identity_crop_and_rotation() {
    let mut p = placement(0, 0, 20_000, 10_000, Rotation::Deg270);
    p.crop_x_px = 9;
    let before = p.clone();
    let mut s = sheet((100_000, 100_000), vec![p]);
    block_nest(
        &mut s,
        &NestConstraints {
            gutter_um: 0,
            margin_um: 0,
        },
    )
    .unwrap();
    let q = &s.placements[0];
    assert_eq!(
        (q.id, q.asset_id, q.rotation, q.crop_x_px),
        (before.id, before.asset_id, before.rotation, 9)
    );
}

#[test]
fn selective_reprint_keeps_identity_and_position() {
    let a = placement(1_000, 2_000, 10_000, 10_000, Rotation::Deg0);
    let b = placement(30_000, 2_000, 10_000, 10_000, Rotation::Deg0);
    let (ia, ib) = (a.id, b.id);
    let mut s = sheet((100_000, 100_000), vec![a, b.clone()]);
    s.set_note(ib, "customer B").unwrap();
    let sub = subset(&s, &[ib]).unwrap();
    assert_eq!(sub.placements, vec![b]);
    assert_eq!(sub.page_width_um, 100_000);
    assert_eq!(sub.notes.get(&ib).map(String::as_str), Some("customer B"));
    assert!(!sub.notes.contains_key(&ia));
    assert!(matches!(
        subset(&s, &[PlacementId::new()]),
        Err(SheetError::UnknownPlacement(_))
    ));
}

#[test]
fn sheet_converts_to_a_validated_layout_snapshot() {
    let a = placement(1_000, 2_000, 10_000, 10_000, Rotation::Deg0);
    let s = sheet((100_000, 100_000), vec![a.clone()]);
    let snap = s.to_snapshot(true).unwrap();
    assert_eq!(snap.placements(), &[a]);
    assert!(snap.inputs().mirror);
    assert!(sheet((100_000, 100_000), vec![])
        .to_snapshot(false)
        .is_err());
}

#[test]
fn notes_do_not_change_the_layout_hash() {
    let a = placement(1_000, 2_000, 10_000, 10_000, Rotation::Deg0);
    let id = a.id;
    let mut s = sheet((100_000, 100_000), vec![a]);
    let h1 = s.to_snapshot(false).unwrap().content_hash();
    s.set_note(id, "label only").unwrap();
    assert_eq!(s.to_snapshot(false).unwrap().content_hash(), h1);
}

#[test]
fn shelf_wraps_at_the_exact_boundary() {
    let cons = NestConstraints {
        gutter_um: 1_000,
        margin_um: 0,
    };
    let row = |second_w| {
        let mut s = sheet(
            (21_000, 100_000),
            vec![
                placement(0, 0, 10_000, 10_000, Rotation::Deg0),
                placement(0, 0, second_w, 5_000, Rotation::Deg0),
            ],
        );
        block_nest(&mut s, &cons).unwrap();
        s.placements.iter().map(|p| p.y_um).max().unwrap()
    };
    assert_eq!(
        row(10_000),
        0,
        "10_000 + 1_000 + 10_000 fills the page exactly"
    );
    assert_eq!(
        row(10_001),
        11_000,
        "one micrometre more wraps to a new shelf"
    );
}
