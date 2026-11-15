mod common;
use common::*;
use veylora_device::capability::MirrorOwner;
use veylora_domain::{errors::ErrorCode, layout::Rotation, preflight::Severity};
use veylora_layout::{resolve_transform, validate_layout, LayoutPolicy};

fn policy() -> LayoutPolicy {
    LayoutPolicy {
        min_effective_ppi: 150,
        margin_um: 3_000,
        supported_page_sizes_um: vec![],
    }
}
fn findings(
    page: (u32, u32),
    mirror: bool,
    p: veylora_domain::layout::Placement,
    pol: &LayoutPolicy,
) -> Vec<veylora_domain::preflight::Finding> {
    let l = layout(page.0, page.1, mirror, vec![p]);
    let owner = if mirror {
        MirrorOwner::Rip
    } else {
        MirrorOwner::None
    };
    validate_layout(&resolve_transform(&l, owner).unwrap(), pol)
}
fn codes(f: &[veylora_domain::preflight::Finding]) -> Vec<(ErrorCode, Severity)> {
    f.iter().map(|x| (x.code, x.severity)).collect()
}

#[test]
fn clean_layout_has_no_findings() {
    // 25.4 mm @ 1000 px = 1000 ppi, comfortably inside margins.
    let mut p = placement(10_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    p.crop_width_px = 1000;
    p.crop_height_px = 1000;
    assert!(findings((100_000, 100_000), false, p, &policy()).is_empty());
}

#[test]
fn clipping_is_blocking() {
    let p = placement(90_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    let f = findings((100_000, 100_000), false, p, &policy());
    assert!(codes(&f).contains(&(ErrorCode::Clipping, Severity::Blocking)));
}

#[test]
fn negative_origin_is_clipping() {
    let p = placement(-1, 10_000, 25_400, 25_400, Rotation::Deg0);
    let f = findings((100_000, 100_000), false, p, &policy());
    assert!(codes(&f).contains(&(ErrorCode::Clipping, Severity::Blocking)));
}

#[test]
fn clipping_is_checked_after_the_mirror_in_final_space() {
    // Inside the page unmirrored; still inside after mirror. Exactly on the edge is fine.
    let p = placement(0, 0, 100_000, 100_000, Rotation::Deg0);
    let mut pol = policy();
    pol.margin_um = 0;
    assert!(findings((100_000, 100_000), true, p, &pol).is_empty());
}

#[test]
fn unsafe_margin_requires_confirmation_but_is_not_clipping() {
    let p = placement(1_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    let f = findings((100_000, 100_000), false, p, &policy());
    assert_eq!(
        codes(&f),
        vec![(ErrorCode::UnsafeMargin, Severity::ConfirmationRequired)]
    );
}

#[test]
fn margin_is_measured_on_the_mirrored_side() {
    // Near the left edge before the mirror, so near the right edge after it.
    let p = placement(1_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    let f = findings((100_000, 100_000), true, p, &policy());
    assert_eq!(
        codes(&f),
        vec![(ErrorCode::UnsafeMargin, Severity::ConfirmationRequired)]
    );
    let p = placement(73_600, 10_000, 25_400, 25_400, Rotation::Deg0); // right edge at 99_000 unmirrored
    let f = findings((100_000, 100_000), true, p, &policy());
    assert_eq!(
        codes(&f),
        vec![(ErrorCode::UnsafeMargin, Severity::ConfirmationRequired)]
    );
}

#[test]
fn low_effective_resolution_uses_the_worse_axis() {
    // 100 px over 25.4 mm = 100 ppi horizontally; 1000 px vertically.
    let mut p = placement(10_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    p.crop_width_px = 100;
    let f = findings((100_000, 100_000), false, p, &policy());
    assert_eq!(
        codes(&f),
        vec![(
            ErrorCode::LowEffectiveResolution,
            Severity::ConfirmationRequired
        )]
    );
    // Exactly at the threshold passes: 150 px over 25.4 mm = 150 ppi.
    let mut p = placement(10_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    p.crop_width_px = 150;
    assert!(findings((100_000, 100_000), false, p, &policy()).is_empty());
}

#[test]
fn rotation_does_not_change_effective_resolution() {
    let mut p = placement(10_000, 10_000, 25_400, 50_800, Rotation::Deg90);
    p.crop_width_px = 100; // 100 ppi horizontally in source space
    p.crop_height_px = 2_000;
    let f = findings((100_000, 100_000), false, p, &policy());
    assert!(codes(&f).contains(&(
        ErrorCode::LowEffectiveResolution,
        Severity::ConfirmationRequired
    )));
}

#[test]
fn unsupported_page_size_is_blocking_when_a_supported_list_exists() {
    let mut pol = policy();
    pol.supported_page_sizes_um = vec![(210_000, 297_000)];
    let p = placement(10_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    let f = findings((100_000, 100_000), false, p.clone(), &pol);
    assert!(codes(&f).contains(&(ErrorCode::UnsupportedMediaSize, Severity::Blocking)));
    assert!(findings((210_000, 297_000), false, p, &pol).is_empty());
}

#[test]
fn findings_name_the_placement() {
    let p = placement(90_000, 10_000, 25_400, 25_400, Rotation::Deg0);
    let id = p.id.to_string();
    let f = findings((100_000, 100_000), false, p, &policy());
    assert_eq!(f[0].subject.as_deref(), Some(id.as_str()));
}
