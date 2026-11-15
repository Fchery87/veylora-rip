mod common;
use common::{asset, rgb};
use veylora_device::channels::{
    ChannelBinding, LogicalChannel, LogicalChannelMap, PhysicalSlotId, ProtocolChannel,
};
use veylora_domain::{
    ids::AssetId,
    recipe::{
        ClearPolicy, KnockoutPolicy, KnockoutTarget, MaskCombination, PerforationPolicy,
        WhitePolicy, WhiteSource,
    },
};
use veylora_image::SampleLayout;
use veylora_separation::{
    allocate_logical_to_physical, build_clear_mask, enforce_clear_mask, generate_white,
    AllocationLimits, ClearMask, LogicalPlanes, MaskInput, PixelPitch, Plane, SeparationError,
    SeparationInput, SuppliedMask,
};

const PITCH: PixelPitch = PixelPitch {
    x_um: 100,
    y_um: 100,
};

fn input(a: &veylora_image::SourceAsset) -> MaskInput<'_> {
    MaskInput {
        source: a,
        eligible_mask: None,
        origin_px: (0, 0),
        pitch: PITCH,
    }
}
fn knockout(
    target: KnockoutTarget,
    tolerance_permille: u16,
    transition_permille: u16,
) -> KnockoutPolicy {
    KnockoutPolicy {
        target,
        tolerance_permille,
        transition_permille,
    }
}
fn policy(knockouts: Vec<KnockoutPolicy>) -> ClearPolicy {
    ClearPolicy {
        knockouts,
        perforation: None,
    }
}
fn retained(a: &veylora_image::SourceAsset, p: &ClearPolicy) -> Vec<f32> {
    build_clear_mask(&input(a), p)
        .unwrap()
        .retained()
        .pixels
        .clone()
}
fn close(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b) {
        assert!((x - y).abs() < 1e-4, "{a:?} != {b:?}");
    }
}

#[test]
fn black_knockout_clears_black_and_near_black_but_not_navy() {
    let a = rgb(4, &[[0, 0, 0], [10, 10, 10], [0, 0, 128], [255, 255, 255]]);
    // 80 per-mille is 20.4 of 255: clears 10, keeps navy (128) and white.
    close(
        &retained(&a, &policy(vec![knockout(KnockoutTarget::Black, 80, 0)])),
        &[0.0, 0.0, 1.0, 1.0],
    );
}
#[test]
fn tolerance_boundary_is_exact_integer_arithmetic() {
    // 200 per-mille is exactly 51/255: 51 clears, 52 does not (hard clip).
    let a = rgb(2, &[[51, 51, 51], [52, 52, 52]]);
    close(
        &retained(&a, &policy(vec![knockout(KnockoutTarget::Black, 200, 0)])),
        &[0.0, 1.0],
    );
}
#[test]
fn transition_ramps_between_cleared_and_retained() {
    let a = rgb(
        4,
        &[[0, 0, 0], [51, 51, 51], [102, 102, 102], [204, 204, 204]],
    );
    // tolerance 200 (51), transition 400 (102): 51 -> 0, 102 -> 0.5, 204 -> 1.
    close(
        &retained(&a, &policy(vec![knockout(KnockoutTarget::Black, 200, 400)])),
        &[0.0, 0.0, 0.5, 1.0],
    );
}
#[test]
fn selected_color_knockout_uses_source_encoded_distance() {
    let a = rgb(3, &[[200, 30, 30], [205, 25, 35], [30, 30, 200]]);
    let p = policy(vec![knockout(
        KnockoutTarget::Color { rgb: [200, 30, 30] },
        40,
        0,
    )]);
    close(&retained(&a, &p), &[0.0, 0.0, 1.0]);
}
#[test]
fn hidden_color_under_transparency_is_not_knocked_out() {
    // Black under alpha 0 is invisible artwork; opaque black is cleared.
    let a = asset(2, 1, SampleLayout::Rgba8, vec![0, 0, 0, 0, 0, 0, 0, 255]);
    close(
        &retained(&a, &policy(vec![knockout(KnockoutTarget::Black, 100, 0)])),
        &[1.0, 0.0],
    );
}
#[test]
fn building_a_mask_never_modifies_the_artwork() {
    let a = rgb(2, &[[0, 0, 0], [255, 255, 255]]);
    let before = a.decoded.samples.clone();
    let mask = build_clear_mask(
        &input(&a),
        &policy(vec![knockout(KnockoutTarget::Black, 100, 0)]),
    )
    .unwrap();
    assert_eq!(a.decoded.samples, before);
    // An empty policy is an open mask; disabling knockout is just not applying the rule.
    let open = build_clear_mask(&input(&a), &ClearPolicy::default()).unwrap();
    assert_eq!(open, ClearMask::open(2, 1));
    assert_ne!(open, mask);
}
#[test]
fn multiple_rules_only_clear_more() {
    let a = rgb(3, &[[0, 0, 0], [200, 30, 30], [255, 255, 255]]);
    let p = policy(vec![
        knockout(KnockoutTarget::Black, 50, 0),
        knockout(KnockoutTarget::Color { rgb: [200, 30, 30] }, 50, 0),
    ]);
    close(&retained(&a, &p), &[0.0, 0.0, 1.0]);
}
#[test]
fn out_of_range_knockout_values_are_rejected_even_if_built_by_hand() {
    let a = rgb(1, &[[0, 0, 0]]);
    for (t, tr) in [(1001, 0), (0, 1001)] {
        assert_eq!(
            build_clear_mask(
                &input(&a),
                &policy(vec![knockout(KnockoutTarget::Black, t, tr)])
            ),
            Err(SeparationError::InvalidRule)
        );
    }
}

fn perforation() -> PerforationPolicy {
    PerforationPolicy {
        diameter_um: 200,
        spacing_um: 400,
        phase_x_px: 0,
        phase_y_px: 0,
        eligible_mask_id: None,
        edge_protection_um: 0,
    }
}
fn perforated(p: PerforationPolicy) -> ClearPolicy {
    ClearPolicy {
        knockouts: vec![],
        perforation: Some(p),
    }
}
fn solid(w: u32, h: u32) -> veylora_image::SourceAsset {
    asset(w, h, SampleLayout::Gray8, vec![128; (w * h) as usize])
}
#[test]
fn perforation_places_holes_on_a_deterministic_physical_grid() {
    // 100 um pixels, grid every 400 um (4 px) anchored at the page origin, 200 um holes:
    // pixel centers sit 50 or 150 um from a grid point, so holes are the 2x2 clusters of
    // pixels at offsets {3, 0} mod 4 on each axis.
    let a = solid(8, 8);
    let r = retained(&a, &perforated(perforation()));
    for y in 0..8usize {
        for x in 0..8usize {
            let near = |v: usize| matches!(v % 4, 0 | 3);
            let expected = if near(x) && near(y) { 0.0 } else { 1.0 };
            assert_eq!(r[y * 8 + x], expected, "({x},{y})");
        }
    }
    assert_eq!(r.iter().filter(|v| **v == 0.0).count(), 16);
}
#[test]
fn perforation_is_stable_across_bands_of_the_same_page() {
    let full = retained(&solid(8, 8), &perforated(perforation()));
    let band = solid(8, 4);
    // Row 2 is not a multiple of the 4 px grid period, so ignoring the origin would differ.
    let lower = build_clear_mask(
        &MaskInput {
            origin_px: (0, 2),
            ..input(&band)
        },
        &perforated(perforation()),
    )
    .unwrap();
    assert_eq!(lower.retained().pixels, full[16..48].to_vec());
}
#[test]
fn phase_shifts_the_grid() {
    let a = solid(8, 8);
    let shifted = PerforationPolicy {
        phase_x_px: 1,
        ..perforation()
    };
    let base = retained(&a, &perforated(perforation()));
    let moved = retained(&a, &perforated(shifted));
    assert_ne!(base, moved);
    // Shifting the grid by one pixel moves each hole one pixel to the right.
    for y in 0..8 {
        for x in 1..8 {
            assert_eq!(moved[y * 8 + x], base[y * 8 + x - 1]);
        }
    }
}
fn eligible_left_half(id: AssetId) -> SuppliedMask {
    let mut plane = Plane::new(8, 8, 0.0);
    for y in 0..8 {
        for x in 0..4 {
            plane.set(x, y, 1.0);
        }
    }
    SuppliedMask { id, plane }
}
#[test]
fn perforation_respects_eligible_areas_and_edge_protection() {
    let a = solid(8, 8);
    let id = AssetId::new();
    let eligible = eligible_left_half(id);
    let with = |p: PerforationPolicy| {
        build_clear_mask(
            &MaskInput {
                eligible_mask: Some(&eligible),
                ..input(&a)
            },
            &perforated(p),
        )
        .unwrap()
        .retained()
        .pixels
        .clone()
    };
    let r = with(PerforationPolicy {
        eligible_mask_id: Some(id),
        ..perforation()
    });
    // Right half is ineligible so nothing is punched there.
    for y in 0..8 {
        for x in 4..8 {
            assert_eq!(r[y * 8 + x], 1.0);
        }
    }
    assert_eq!(r[0], 0.0);
    // One pixel of edge protection erodes eligibility (including the page edge).
    let g = with(PerforationPolicy {
        eligible_mask_id: Some(id),
        edge_protection_um: 100,
        ..perforation()
    });
    assert!(g.iter().all(|v| *v == 1.0), "{g:?}");
}
#[test]
fn the_eligible_mask_must_be_the_one_the_policy_references() {
    let a = solid(8, 8);
    let id = AssetId::new();
    let policy = perforated(PerforationPolicy {
        eligible_mask_id: Some(id),
        ..perforation()
    });
    assert_eq!(
        build_clear_mask(&input(&a), &policy),
        Err(SeparationError::MissingSuppliedMask)
    );
    let other = eligible_left_half(AssetId::new());
    assert_eq!(
        build_clear_mask(
            &MaskInput {
                eligible_mask: Some(&other),
                ..input(&a)
            },
            &policy
        ),
        Err(SeparationError::SuppliedMaskMismatch)
    );
    let wrong_size = SuppliedMask {
        id,
        plane: Plane::new(3, 3, 1.0),
    };
    assert_eq!(
        build_clear_mask(
            &MaskInput {
                eligible_mask: Some(&wrong_size),
                ..input(&a)
            },
            &policy
        ),
        Err(SeparationError::DimensionMismatch)
    );
}
#[test]
fn unsafe_perforation_values_are_rejected_even_if_built_by_hand() {
    let a = solid(8, 8);
    let bad = |p: PerforationPolicy| {
        assert_eq!(
            build_clear_mask(&input(&a), &perforated(p)),
            Err(SeparationError::InvalidRule)
        )
    };
    bad(PerforationPolicy {
        diameter_um: 0,
        ..perforation()
    });
    bad(PerforationPolicy {
        spacing_um: 200,
        ..perforation()
    }); // spacing not above diameter
    bad(PerforationPolicy {
        edge_protection_um: 100,
        ..perforation()
    }); // needs a mask
    let skew = PixelPitch {
        x_um: 100,
        y_um: 200,
    };
    assert_eq!(
        build_clear_mask(
            &MaskInput {
                pitch: skew,
                ..input(&a)
            },
            &perforated(perforation())
        ),
        Err(SeparationError::AnisotropicPitch)
    );
}

fn bindings() -> LogicalChannelMap {
    use LogicalChannel::*;
    let b = |l, slot, protocol, fraction| ChannelBinding {
        logical: l,
        slot: PhysicalSlotId(slot),
        protocol: ProtocolChannel(protocol),
        fraction,
    };
    LogicalChannelMap::new(
        1,
        vec![
            b(Cyan, 0, 10, 1.0),
            b(Magenta, 1, 11, 1.0),
            b(Yellow, 2, 12, 1.0),
            b(Black, 3, 13, 1.0),
            b(White, 4, 14, 0.5),
            b(White, 5, 15, 0.5),
        ],
    )
    .unwrap()
}

#[test]
fn enforcement_clears_every_logical_and_physical_plane_and_white_spread_cannot_return() {
    // Opaque artwork, black text knocked out in the middle pixel, with white spread 1 px.
    let art = rgb(3, &[[255, 255, 255], [0, 0, 0], [255, 255, 255]]);
    let clear = build_clear_mask(
        &input(&art),
        &policy(vec![knockout(KnockoutTarget::Black, 50, 0)]),
    )
    .unwrap();
    assert_eq!(clear.retained().pixels, vec![1.0, 0.0, 1.0]);

    let alpha = Plane::new(3, 1, 1.0);
    let white = generate_white(
        &SeparationInput {
            alpha: alpha.clone(),
            tonal: None,
            supplied_mask: None,
            clear_mask: Some(clear.clone()),
        },
        &WhitePolicy {
            source: WhiteSource::Alpha,
            combine: MaskCombination::Replace,
            density_percent: 100,
            spread_um: 100,
            response: vec![0, 65535],
        },
        PITCH,
    )
    .unwrap();
    assert_eq!(white.pixels, vec![1.0, 0.0, 1.0]);

    let mut logical = LogicalPlanes {
        c: Plane::new(3, 1, 0.5),
        m: Plane::new(3, 1, 0.5),
        y: Plane::new(3, 1, 0.5),
        k: Plane::new(3, 1, 0.5),
        w: Plane::new(3, 1, 1.0), // deliberately not pre-cleared
    };
    enforce_clear_mask(&mut logical, &clear).unwrap();
    for p in [&logical.c, &logical.m, &logical.y, &logical.k, &logical.w] {
        assert_eq!(p.pixels[1], 0.0);
        assert_ne!(p.pixels[0], 0.0);
    }
    let mut physical = allocate_logical_to_physical(
        &logical,
        &bindings(),
        &AllocationLimits {
            max_total_pixels: 100,
        },
    )
    .unwrap();
    for slot in physical.slots.values() {
        assert_eq!(slot.plane.pixels[1], 0.0, "slot {:?}", slot.slot);
    }
    // Enforcing again is idempotent.
    let before = physical.clone();
    enforce_clear_mask(&mut physical, &clear).unwrap();
    assert_eq!(physical, before);
}
#[test]
fn enforcement_checks_sizes_before_changing_anything() {
    let clear = ClearMask::from_plane(Plane::new(2, 1, 0.0)).unwrap();
    let mut planes = LogicalPlanes {
        c: Plane::new(2, 1, 1.0),
        m: Plane::new(2, 1, 1.0),
        y: Plane::new(2, 1, 1.0),
        k: Plane::new(2, 1, 1.0),
        w: Plane::new(3, 1, 1.0),
    };
    let before = planes.clone();
    assert_eq!(
        enforce_clear_mask(&mut planes, &clear),
        Err(SeparationError::DimensionMismatch)
    );
    assert_eq!(planes, before);
}
#[test]
fn clear_masks_reject_out_of_range_or_invalid_planes() {
    assert!(ClearMask::from_plane(Plane::new(1, 1, 1.5)).is_err());
    assert!(ClearMask::from_plane(Plane::new(1, 1, f32::NAN)).is_err());
    assert!(ClearMask::from_plane(Plane::new(0, 0, 0.0)).is_err());
}
