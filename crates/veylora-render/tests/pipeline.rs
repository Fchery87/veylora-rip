mod common;
use common::*;
use std::sync::{atomic::AtomicU32, Arc};
use veylora_compose::ComposeError;
use veylora_device::capability::MirrorOwner;
use veylora_domain::{errors::ErrorCode, preflight::Severity};
use veylora_domain::{provenance::ContentHash, recipe::WhiteSource};
use veylora_layout::LayoutError;
use veylora_render::AcknowledgedFinding;
use veylora_render::{
    render, ArtifactError, CancellationToken, DirectoryStore, ProcessBand, ProcessSeparator,
    RenderError, SeparatorError, Stage, WhiteOnly, MANIFEST_NAME,
};
use veylora_screen::{apply_calibration, screen_band, PageOrigin, ScreenSpec};
use veylora_separation::{
    allocate_logical_to_physical, alpha_plane, build_clear_mask, enforce_clear_mask,
    generate_white, AllocationLimits, LogicalPlanes, MaskInput, Plane, SuppliedMask,
};

fn run(job: &Job, band: usize) -> (TempDir, veylora_render::RenderArtifacts) {
    let dir = TempDir::new();
    let token = CancellationToken::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let artifacts = render(&request(job, &SyntheticSeparator, band, &token), &mut store)
        .unwrap_or_else(|e| panic!("render failed: {e:?}"));
    (dir, artifacts)
}

#[test]
fn stage_hashes_do_not_depend_on_band_height() {
    let job = standard_job();
    let (_d, reference) = run(&job, H);
    assert_eq!(reference.manifest.stats.halo_rows, 2);
    for band in [1, 2, 3, 5, 7, 11, 22, 23, 100] {
        let (_d, other) = run(&job, band);
        let (a, b) = (&reference.manifest, &other.manifest);
        assert_eq!(
            a.logical_planes.hash, b.logical_planes.hash,
            "logical, band {band}"
        );
        assert_eq!(
            a.physical_planes.hash, b.physical_planes.hash,
            "physical, band {band}"
        );
        assert_eq!(
            a.screened_planes.hash, b.screened_planes.hash,
            "screened, band {band}"
        );
    }
}

#[test]
fn banded_artifacts_equal_a_single_whole_page_computation() {
    let job = standard_job();
    let (dir, art) = run(&job, 4);
    let recipe = job.snapshot.snapshot().inputs().recipe.inputs().clone();

    // Whole page, no bands, no halos: the oracle.
    let eligible = SuppliedMask::from_asset(job.eligible.as_ref().unwrap()).unwrap();
    let clear = build_clear_mask(
        &MaskInput {
            source: &job.source,
            rows: None,
            eligible_mask: Some(&eligible),
            origin_px: (0, 0),
            pitch: PITCH,
        },
        &recipe.clear,
    )
    .unwrap();
    let alpha = alpha_plane(&job.source).unwrap();
    let white = generate_white(
        &veylora_separation::SeparationInput {
            alpha: alpha.clone(),
            tonal: None,
            supplied_mask: None,
            clear_mask: Some(clear.clone()),
        },
        &recipe.white,
        PITCH,
    )
    .unwrap();
    let ProcessBand {
        mut c,
        mut m,
        mut y,
        mut k,
    } = SyntheticSeparator.separate(&job.source, 0..H).unwrap();
    for plane in [&mut c, &mut m, &mut y, &mut k] {
        for (v, a) in plane.pixels.iter_mut().zip(&alpha.pixels) {
            *v *= a;
        }
    }
    let mut logical = LogicalPlanes {
        c,
        m,
        y,
        k,
        w: white,
    };
    enforce_clear_mask(&mut logical, &clear).unwrap();
    let files = &art.manifest.logical_planes.files;
    for (file, plane) in files
        .iter()
        .zip([&logical.c, &logical.m, &logical.y, &logical.k, &logical.w])
    {
        assert_eq!(f32s(&dir.read(&file.name)), plane.pixels, "{}", file.name);
    }
    apply_calibration(&mut logical, &recipe.calibration).unwrap();
    let physical = allocate_logical_to_physical(
        &logical,
        &job.map,
        &AllocationLimits {
            max_total_pixels: 6 * W * H,
        },
    )
    .unwrap();
    let spec = ScreenSpec {
        policy: recipe.screen.clone(),
        dot_level: 255,
    };
    for (protocol, slot) in &physical.slots {
        let p = f32s(&dir.read(&format!("physical-p{}.f32", protocol.0)));
        assert_eq!(p, slot.plane.pixels, "physical {protocol:?}");
        let screened = screen_band(&slot.plane, PageOrigin { x: 0, y: 0 }, &spec).unwrap();
        assert_eq!(
            u16s(&dir.read(&format!("screened-p{}.u16", protocol.0))),
            screened.pixels
        );
    }
}

#[test]
fn artifacts_are_exact_bytes_and_the_manifest_describes_them() {
    let job = standard_job();
    let (dir, art) = run(&job, 5);
    let m = &art.manifest;
    assert_eq!(m.evidence, "software_only");
    for stage in [&m.logical_planes, &m.physical_planes, &m.screened_planes] {
        for file in &stage.files {
            let bytes = dir.read(&file.name);
            assert_eq!(bytes.len() as u64, file.bytes, "{}", file.name);
            assert_eq!(ContentHash::digest(&bytes), file.hash, "{}", file.name);
            assert_eq!(file.bytes, (W * H) as u64 * stage.format.bytes());
        }
        assert_eq!((stage.width, stage.height), (W, H));
    }
    assert_eq!(m.logical_planes.stage, Stage::LogicalPlanes);
    assert_eq!(m.logical_planes.files.len(), 5);
    assert_eq!(m.physical_planes.files.len(), 6); // 4 process + 2 white slots
    assert_eq!(m.screened_planes.files.len(), 6);
    // The manifest on disk is the returned manifest, and its recorded hash is of exactly those bytes.
    let on_disk = dir.read(MANIFEST_NAME);
    assert_eq!(ContentHash::digest(&on_disk), art.manifest_file.hash);
    let parsed: veylora_render::RenderManifest = serde_json::from_slice(&on_disk).unwrap();
    assert_eq!(&parsed, m);
    // Inputs are pinned by hash.
    assert_eq!(m.sources.len(), 1);
    assert_eq!(m.sources[0].content_hash, job.source.original_hash);
    assert_eq!(m.compose_algorithm, veylora_compose::ALGORITHM);
    assert_eq!(
        m.recipe_hash,
        job.snapshot.snapshot().inputs().recipe.content_hash()
    );
    assert_eq!(m.snapshot_hash, job.snapshot.content_hash());
    assert_eq!(
        m.eligible_mask.as_ref().unwrap().content_hash,
        job.eligible.as_ref().unwrap().original_hash
    );
    assert_eq!(m.channel_map_version, 1);
    // Two white slots split the white plane 60/40 without duplicating it.
    let (a, b) = (
        f32s(&dir.read("physical-p14.f32")),
        f32s(&dir.read("physical-p15.f32")),
    );
    let w = f32s(&dir.read("logical-w.f32"));
    assert!(w.iter().any(|v| *v > 0.0));
    for i in 0..w.len() {
        assert!(
            (a[i] - 0.6 * w[i]).abs() < 1e-6 && (b[i] - 0.4 * w[i]).abs() < 1e-6,
            "pixel {i}"
        );
    }
}

#[test]
fn rendering_is_deterministic() {
    let job = standard_job();
    let (_a, one) = run(&job, 3);
    let (_b, two) = run(&job, 3);
    assert_eq!(one, two);
}

#[test]
fn hidden_rgb_under_transparency_never_prints() {
    let job = standard_job();
    let (dir, _) = run(&job, 6);
    // Pixels x < 3, y > 12 are alpha 0 with saturated RGB.
    for name in [
        "logical-c.f32",
        "logical-m.f32",
        "logical-y.f32",
        "logical-k.f32",
    ] {
        let plane = f32s(&dir.read(name));
        for y in 13..H {
            for x in 0..3 {
                assert_eq!(plane[y * W + x], 0.0, "{name} ({x},{y})");
            }
        }
    }
    // White spreads 2 px from opaque pixels (including the row-17 black strip, which is cleared
    // only after spreading), but not farther.
    let white = f32s(&dir.read("logical-w.f32"));
    for y in 20..H {
        assert_eq!(
            white[y * W],
            0.0,
            "white far inside the transparent region, row {y}"
        );
    }
    // Partial alpha edge prints, scaled by its coverage (not full strength).
    let c = f32s(&dir.read("logical-c.f32"));
    let edge: Vec<f32> = (0..H)
        .filter(|y| y % 7 != 3)
        .map(|y| c[y * W + 11])
        .collect();
    assert!(edge.iter().any(|v| *v > 0.0));
}

#[test]
fn clear_mask_zeroes_every_plane_at_every_stage() {
    let job = standard_job();
    let (dir, art) = run(&job, 4);
    // The black strip (y % 7 == 3) is knocked out everywhere.
    let stages: Vec<(&str, Vec<String>)> = vec![
        (
            "logical",
            art.manifest
                .logical_planes
                .files
                .iter()
                .map(|f| f.name.clone())
                .collect(),
        ),
        (
            "physical",
            art.manifest
                .physical_planes
                .files
                .iter()
                .map(|f| f.name.clone())
                .collect(),
        ),
    ];
    for (stage, names) in stages {
        for name in names {
            let plane = f32s(&dir.read(&name));
            for y in (0..H).filter(|y| y % 7 == 3) {
                for x in 0..W {
                    assert_eq!(plane[y * W + x], 0.0, "{stage} {name} ({x},{y})");
                }
            }
        }
    }
    for file in &art.manifest.screened_planes.files {
        let plane = u16s(&dir.read(&file.name));
        for y in (0..H).filter(|y| y % 7 == 3) {
            for x in 0..W {
                assert_eq!(plane[y * W + x], 0, "{} ({x},{y})", file.name);
            }
        }
    }
}

#[test]
fn changing_only_the_white_policy_leaves_process_planes_unchanged() {
    let build = |density: u8| {
        let eligible = eligible_mask();
        let mut white = white_policy(WhiteSource::Alpha, 200);
        white.density_percent = density;
        let recipe = recipe(clear_policy(Some(eligible.id)), white);
        let job = job_with(artwork(), None, Some(eligible), recipe, |_| {});
        let (dir, art) = run(&job, 5);
        (dir, art.manifest)
    };
    let (_d1, a) = build(90);
    let (_d2, b) = build(40);
    for i in 0..4 {
        assert_eq!(
            a.logical_planes.files[i].hash, b.logical_planes.files[i].hash,
            "process plane {i}"
        );
    }
    assert_ne!(
        a.logical_planes.files[4].hash,
        b.logical_planes.files[4].hash
    );
}

#[test]
fn supplied_white_mask_is_used_banded_and_pinned_in_the_manifest() {
    let mask = white_mask();
    let recipe = recipe(
        veylora_domain::recipe::ClearPolicy::default(),
        white_policy(WhiteSource::Supplied { mask_id: mask.id }, 100),
    );
    let job = job_with(artwork(), Some(mask), None, recipe, |_| {});
    let (_d, whole) = run(&job, H);
    let (dir, banded) = run(&job, 3);
    assert_eq!(
        whole.manifest.logical_planes.hash,
        banded.manifest.logical_planes.hash
    );
    assert_eq!(
        banded.manifest.supplied_mask.as_ref().unwrap().id,
        job.supplied.as_ref().unwrap().id
    );
    // Explicit white artwork prints even where alpha is zero.
    let w = f32s(&dir.read("logical-w.f32"));
    assert!(w[14 * W] > 0.0);
}

#[test]
fn white_only_mode_prints_no_process_ink() {
    let job = standard_job();
    let dir = TempDir::new();
    let token = CancellationToken::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let art = render(&request(&job, &WhiteOnly, 5, &token), &mut store).unwrap();
    for file in &art.manifest.logical_planes.files[..4] {
        assert!(
            f32s(&dir.read(&file.name)).iter().all(|v| *v == 0.0),
            "{}",
            file.name
        );
    }
    assert!(f32s(&dir.read("logical-w.f32")).iter().any(|v| *v > 0.0));
}

#[test]
fn stats_record_bounded_working_set_and_deterministic_counters() {
    let job = standard_job();
    let (_d, art) = run(&job, 4);
    let s = &art.manifest.stats;
    assert_eq!(s.bands, 6); // ceil(23 / 4)
    assert_eq!(s.band_height, 4);
    assert_eq!(s.halo_rows, 2);
    assert_eq!(s.page_pixels, (W * H) as u64);
    assert_eq!(s.peak_working_pixels, (W * (4 + 2 * 2)) as u64);
    assert!(s.peak_working_pixels < s.page_pixels);
    assert_eq!(
        s.bytes_written,
        [
            &art.manifest.logical_planes,
            &art.manifest.physical_planes,
            &art.manifest.screened_planes
        ]
        .iter()
        .flat_map(|st| st.files.iter())
        .map(|f| f.bytes)
        .sum::<u64>()
    );
}

struct CancelOnSecondBand<'a>(&'a CancellationToken, std::sync::atomic::AtomicU32);
impl ProcessSeparator for CancelOnSecondBand<'_> {
    fn separate(
        &self,
        source: &veylora_image::SourceAsset,
        rows: std::ops::Range<usize>,
    ) -> Result<ProcessBand, SeparatorError> {
        if self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1 {
            self.0.cancel();
        }
        SyntheticSeparator.separate(source, rows)
    }
}

#[test]
fn cancellation_stops_the_render_and_removes_partial_artifacts() {
    let job = standard_job();
    // Already cancelled: nothing is created.
    let dir = TempDir::new();
    let token = CancellationToken::new();
    token.cancel();
    let mut store = DirectoryStore::new(dir.path(), None);
    let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
    assert!(matches!(r, Err(RenderError::Cancelled)));
    assert!(dir.files().is_empty());
    // Cancelled while rendering: partial files are rolled back.
    let dir = TempDir::new();
    let token = CancellationToken::new();
    let sep = CancelOnSecondBand(&token, AtomicU32::new(0));
    let mut store = DirectoryStore::new(dir.path(), None);
    let r = render(&request(&job, &sep, 4, &token), &mut store);
    assert!(matches!(r, Err(RenderError::Cancelled)), "{r:?}");
    assert!(dir.files().is_empty(), "left behind: {:?}", dir.files());
}

#[test]
fn a_full_disk_fails_early_or_mid_render_and_leaves_nothing_behind() {
    let job = standard_job();
    let token = CancellationToken::new();
    // Known quota too small: refused before any file is created.
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), Some(1000));
    let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
    assert!(
        matches!(r, Err(RenderError::Artifact(ArtifactError::NoSpace))),
        "{r:?}"
    );
    assert!(dir.files().is_empty());
    // Disk fills part-way through, with no warning in advance.
    let dir = TempDir::new();
    let mut store = FlakyStore {
        inner: DirectoryStore::new(dir.path(), None),
        fail_after: 2000,
        written: Arc::new(AtomicU32::new(0)),
    };
    let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
    assert!(
        matches!(r, Err(RenderError::Artifact(ArtifactError::NoSpace))),
        "{r:?}"
    );
    assert!(dir.files().is_empty(), "left behind: {:?}", dir.files());
}

#[test]
fn existing_artifacts_are_never_overwritten_or_removed() {
    let job = standard_job();
    let dir = TempDir::new();
    std::fs::write(dir.path().join("logical-k.f32"), b"keep me").unwrap();
    let token = CancellationToken::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
    assert!(
        matches!(r, Err(RenderError::Artifact(ArtifactError::Exists))),
        "{r:?}"
    );
    assert_eq!(dir.read("logical-k.f32"), b"keep me");
    assert_eq!(dir.files(), vec!["logical-k.f32".to_string()]);
}

#[test]
fn limits_are_enforced_before_rendering() {
    let job = standard_job();
    let token = CancellationToken::new();
    let try_limits = |f: &dyn Fn(&mut veylora_render::RenderLimits)| {
        let dir = TempDir::new();
        let mut req = request(&job, &SyntheticSeparator, 4, &token);
        f(&mut req.limits);
        let mut store = DirectoryStore::new(dir.path(), None);
        let r = render(&req, &mut store);
        assert!(
            r.is_ok() || dir.files().is_empty(),
            "failed render left {:?}",
            dir.files()
        );
        r
    };
    assert!(matches!(
        try_limits(&|l| l.max_page_pixels = W * H - 1),
        Err(RenderError::LimitExceeded(_))
    ));
    assert!(matches!(
        try_limits(&|l| l.band_height = 0),
        Err(RenderError::InvalidBandHeight)
    ));
    // Band of 4 rows plus a 2-row halo each side is 8 rows of 12 pixels.
    assert!(matches!(
        try_limits(&|l| l.max_working_pixels = W * 8 - 1),
        Err(RenderError::LimitExceeded(_))
    ));
    assert!(try_limits(&|l| l.max_working_pixels = W * 8).is_ok());
}

#[test]
fn inputs_must_match_the_job_snapshot() {
    let job = standard_job();
    let token = CancellationToken::new();
    let attempt = |mutate: &dyn Fn(&mut Job)| {
        let mut changed = standard_job();
        mutate(&mut changed);
        let dir = TempDir::new();
        let mut store = DirectoryStore::new(dir.path(), None);
        let r = render(
            &request(&changed, &SyntheticSeparator, 4, &token),
            &mut store,
        );
        assert!(dir.files().is_empty());
        r
    };
    let _ = &job;
    // Different source bytes than the snapshot retained.
    assert!(matches!(
        attempt(&|j| j.source.original_hash = ContentHash::digest(b"x")),
        Err(RenderError::InputMismatch(_))
    ));
    // Referenced mask missing, or a mask the recipe does not reference.
    assert!(matches!(
        attempt(&|j| j.eligible = None),
        Err(RenderError::InputMismatch(_))
    ));
    assert!(matches!(
        attempt(&|j| j.supplied = Some(white_mask())),
        Err(RenderError::InputMismatch(_))
    ));
    // Mask bytes or size differ from the retained mask.
    assert!(matches!(
        attempt(&|j| j.eligible.as_mut().unwrap().original_hash = ContentHash::digest(b"y")),
        Err(RenderError::InputMismatch(_))
    ));
    assert!(matches!(
        attempt(&|j| {
            // Same id and retained hash, wrong size: only the dimension check can catch it.
            let old = j.eligible.as_ref().unwrap();
            let (id, hash) = (old.id, old.original_hash);
            let mut wrong = eligible_mask_with_id(id, 5, 5);
            wrong.original_hash = hash;
            j.eligible = Some(wrong);
        }),
        Err(RenderError::InputMismatch(_))
    ));
}
fn eligible_mask_with_id(
    id: veylora_domain::ids::AssetId,
    w: u32,
    h: u32,
) -> veylora_image::SourceAsset {
    let mut m = asset(
        w,
        h,
        veylora_image::SampleLayout::Gray8,
        vec![255; (w * h) as usize],
    );
    m.id = id;
    m
}

fn plain_job(mutate: impl FnOnce(&mut veylora_domain::layout::LayoutInput)) -> Job {
    // No perforation: its eligible mask is page-sized and untransformed, so transform
    // equivalences below compare only the operations that are symmetric under flips and turns.
    let recipe = recipe(
        veylora_domain::recipe::ClearPolicy::default(),
        white_policy(WhiteSource::Alpha, 200),
    );
    job_with(artwork(), None, None, recipe, mutate)
}
const PLANES: [&str; 5] = [
    "logical-c.f32",
    "logical-m.f32",
    "logical-y.f32",
    "logical-k.f32",
    "logical-w.f32",
];
fn logical(dir: &TempDir, name: &str) -> Vec<f32> {
    f32s(&dir.read(name))
}

#[test]
fn sheet_mirror_flips_every_plane_exactly_once() {
    let (plain_dir, _) = run(&plain_job(|_| {}), 4);
    let (mirror_dir, art) = run(&plain_job(|l| l.mirror = true), 4);
    assert_eq!(art.manifest.page_width_px, W);
    for name in PLANES {
        let (a, b) = (logical(&plain_dir, name), logical(&mirror_dir, name));
        for y in 0..H {
            for x in 0..W {
                assert_eq!(b[y * W + (W - 1 - x)], a[y * W + x], "{name} ({x},{y})");
            }
        }
    }
    assert_ne!(
        logical(&plain_dir, PLANES[4]),
        logical(&mirror_dir, PLANES[4])
    );
}

#[test]
fn mirror_without_rip_ownership_is_refused_and_leaves_nothing() {
    let job = plain_job(|l| l.mirror = true);
    let token = CancellationToken::new();
    for owner in [MirrorOwner::Operator, MirrorOwner::None] {
        let dir = TempDir::new();
        let mut store = DirectoryStore::new(dir.path(), None);
        let mut req = request(&job, &SyntheticSeparator, 4, &token);
        req.mirror_owner = owner;
        assert!(matches!(
            render(&req, &mut store),
            Err(RenderError::Layout(LayoutError::MirrorOwnerConflict { .. }))
        ));
        assert!(dir.files().is_empty(), "left {:?}", dir.files());
    }
}

#[test]
fn quarter_turn_rotates_every_plane() {
    let (plain_dir, _) = run(&plain_job(|_| {}), 4);
    let (turn_dir, art) = run(
        &plain_job(|l| {
            l.placements[0].rotation = veylora_domain::layout::Rotation::Deg90;
            std::mem::swap(&mut l.page_width_um, &mut l.page_height_um);
        }),
        5,
    );
    assert_eq!(
        (art.manifest.page_width_px, art.manifest.page_height_px),
        (H, W)
    );
    for name in PLANES {
        let (a, b) = (logical(&plain_dir, name), logical(&turn_dir, name));
        for sy in 0..H {
            for sx in 0..W {
                // Clockwise: source (sx, sy) lands at (H - 1 - sy, sx).
                assert_eq!(
                    b[sx * H + (H - 1 - sy)],
                    a[sy * W + sx],
                    "{name} ({sx},{sy})"
                );
            }
        }
    }
}

#[test]
fn layout_transforms_do_not_depend_on_the_band_height() {
    let make = || {
        plain_job(|l| {
            l.mirror = true;
            l.placements[0].rotation = veylora_domain::layout::Rotation::Deg270;
            std::mem::swap(&mut l.page_width_um, &mut l.page_height_um);
        })
    };
    let (_a, whole) = run(&make(), 100);
    let (_b, banded) = run(&make(), 3);
    assert_eq!(
        whole.manifest.logical_planes.hash,
        banded.manifest.logical_planes.hash
    );
    assert_eq!(
        whole.manifest.physical_planes.hash,
        banded.manifest.physical_planes.hash
    );
    assert_eq!(
        whole.manifest.screened_planes.hash,
        banded.manifest.screened_planes.hash
    );
    assert_eq!(
        whole.manifest.composed_page_hash,
        banded.manifest.composed_page_hash
    );
}

#[test]
fn placement_offset_and_crop_change_the_page_and_the_manifest() {
    // A page one pixel wider with the artwork shifted right by one pixel: column 0 is empty.
    let (dir, art) = run(
        &plain_job(|l| {
            l.page_width_um += 100;
            l.placements[0].x_um = 100;
        }),
        4,
    );
    assert_eq!(art.manifest.page_width_px, W + 1);
    // Process ink never prints where nothing is placed. White may spread into the margin.
    for name in &PLANES[..4] {
        let plane = logical(&dir, name);
        for y in 0..H {
            assert_eq!(plane[y * (W + 1)], 0.0, "{name} row {y}");
        }
    }
    let layout_hash = art.manifest.layout_hash;
    let (_d, other) = run(&plain_job(|_| {}), 4);
    assert_ne!(other.manifest.layout_hash, layout_hash);
}

#[test]
fn layouts_the_composer_refuses_are_errors_and_leave_nothing() {
    let token = CancellationToken::new();
    let try_layout = |mutate: fn(&mut veylora_domain::layout::LayoutInput)| {
        let job = plain_job(mutate);
        let dir = TempDir::new();
        let mut store = DirectoryStore::new(dir.path(), None);
        let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
        assert!(
            r.is_ok() || dir.files().is_empty(),
            "failed render left {:?}",
            dir.files()
        );
        r
    };
    assert!(try_layout(|_| {}).is_ok());
    // Clipped: shifted right without widening the page.
    assert!(matches!(
        try_layout(|l| l.placements[0].x_um = 100),
        Err(RenderError::LayoutBlocked(_))
    ));
    // Rotated into a page of the old shape.
    assert!(matches!(
        try_layout(|l| l.placements[0].rotation = veylora_domain::layout::Rotation::Deg90),
        Err(RenderError::LayoutBlocked(_))
    ));
    // Crop beyond the source.
    assert!(matches!(
        try_layout(|l| l.placements[0].crop_x_px = 1),
        Err(RenderError::Compose(ComposeError::CropOutsideSource(_)))
    ));
}

#[test]
fn recipes_the_pipeline_cannot_honestly_render_are_errors() {
    // A tonal white source needs a color engine; there is no tonal plane to give it.
    let token = CancellationToken::new();
    let eligible = eligible_mask();
    let recipe = recipe(
        clear_policy(Some(eligible.id)),
        white_policy(WhiteSource::Tonal, 0),
    );
    let job = job_with(artwork(), None, Some(eligible), recipe, |_| {});
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let r = render(&request(&job, &SyntheticSeparator, 4, &token), &mut store);
    assert!(
        matches!(
            r,
            Err(RenderError::Separation(
                veylora_separation::SeparationError::MissingTonal
            ))
        ),
        "{r:?}"
    );
    assert!(dir.files().is_empty());
    let _ = Plane::new(1, 1, 0.0);
}

#[test]
fn edge_protection_needs_its_own_halo_when_white_does_not_spread() {
    // No white spread, but perforation edge protection of 3 px: only the clear mask reads
    // neighboring rows, so the halo must come from it.
    let eligible = eligible_mask();
    let mut clear = clear_policy(Some(eligible.id));
    clear.perforation.as_mut().unwrap().edge_protection_um = 300;
    let recipe = recipe(clear, white_policy(WhiteSource::Alpha, 0));
    let job = job_with(artwork(), None, Some(eligible), recipe, |_| {});
    let (_d, whole) = run(&job, H);
    assert_eq!(whole.manifest.stats.halo_rows, 3);
    for band in [1, 2, 4, 5, 9] {
        let (_d, other) = run(&job, band);
        assert_eq!(
            whole.manifest.logical_planes.hash, other.manifest.logical_planes.hash,
            "logical, band {band}"
        );
        assert_eq!(
            whole.manifest.screened_planes.hash, other.manifest.screened_planes.hash,
            "screened, band {band}"
        );
    }
}

#[test]
fn scaled_and_repeated_placements_render_and_pin_each_source_once() {
    // Half-size copy at the origin plus a second half-size copy of the same asset beside it.
    let (dir, art) = run(
        &plain_job(|l| {
            let mut first = l.placements[0].clone();
            first.width_um /= 2;
            first.height_um /= 2;
            let mut second = first.clone();
            second.id = veylora_domain::ids::PlacementId::new();
            second.x_um = i64::from(first.width_um);
            l.page_width_um = first.width_um * 2;
            l.page_height_um = first.height_um;
            l.placements = vec![first, second];
        }),
        4,
    );
    let m = &art.manifest;
    assert_eq!((m.page_width_px, m.page_height_px), (12, 12));
    assert_eq!(m.sources.len(), 1, "one asset used twice is pinned once");
    // Both halves are the same resampled artwork.
    let c = logical(&dir, "logical-c.f32");
    for y in 0..12 {
        for x in 0..6 {
            assert_eq!(c[y * 12 + x], c[y * 12 + x + 6], "({x},{y})");
        }
    }
    assert!(c.iter().any(|v| *v > 0.0));
}

#[test]
fn every_placement_asset_must_be_retained_and_supplied_unmodified() {
    let token = CancellationToken::new();
    let job = plain_job(|_| {});
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    // Not supplied at all.
    let mut req = request(&job, &SyntheticSeparator, 4, &token);
    req.images = &[];
    assert!(matches!(
        render(&req, &mut store),
        Err(RenderError::InputMismatch(_))
    ));
    assert!(dir.files().is_empty());
}

#[test]
fn a_supplied_white_mask_must_ride_with_every_placement() {
    let token = CancellationToken::new();
    // Recipe names a mask but the placement does not carry it.
    let job = {
        let mask = white_mask();
        let recipe = recipe(
            veylora_domain::recipe::ClearPolicy::default(),
            white_policy(WhiteSource::Supplied { mask_id: mask.id }, 100),
        );
        job_with(artwork(), Some(mask), None, recipe, |l| {
            l.placements[0].mask_id = None
        })
    };
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    assert!(matches!(
        render(&request(&job, &SyntheticSeparator, 4, &token), &mut store),
        Err(RenderError::InputMismatch(_))
    ));
    assert!(dir.files().is_empty());
    // A placement names a mask but the recipe does not use one.
    let job = job_with(
        artwork(),
        Some(white_mask()),
        None,
        recipe(
            veylora_domain::recipe::ClearPolicy::default(),
            white_policy(WhiteSource::Alpha, 100),
        ),
        |_| {},
    );
    let mut store = DirectoryStore::new(dir.path(), None);
    for supply in [true, false] {
        let mut req = request(&job, &SyntheticSeparator, 4, &token);
        if !supply {
            req.supplied_masks = &[];
        }
        assert!(matches!(
            render(&req, &mut store),
            Err(RenderError::InputMismatch(_))
        ));
        assert!(dir.files().is_empty());
    }
}

fn refusal(
    job: &Job,
    policy: impl FnOnce(&mut veylora_layout::LayoutPolicy),
    acknowledged: &[AcknowledgedFinding],
) -> (
    Result<veylora_render::RenderArtifacts, RenderError>,
    TempDir,
) {
    let token = CancellationToken::new();
    let dir = TempDir::new();
    let mut store = DirectoryStore::new(dir.path(), None);
    let mut req = request(job, &SyntheticSeparator, 4, &token);
    policy(&mut req.layout_policy);
    req.acknowledged = acknowledged;
    (render(&req, &mut store), dir)
}

#[test]
fn blocking_layout_findings_refuse_the_render_and_cannot_be_acknowledged() {
    let job = plain_job(|l| l.placements[0].x_um = 100); // clipped
    let ack = [AcknowledgedFinding {
        code: ErrorCode::Clipping,
        subject: None,
    }];
    for acknowledged in [&[][..], &ack[..]] {
        let (r, dir) = refusal(&job, |_| {}, acknowledged);
        let Err(RenderError::LayoutBlocked(findings)) = r else {
            panic!("{r:?}")
        };
        assert!(findings.iter().all(|f| f.severity == Severity::Blocking));
        assert!(findings.iter().any(|f| f.code == ErrorCode::Clipping));
        assert!(dir.files().is_empty());
    }
    // An unsupported page size blocks too.
    let job = plain_job(|_| {});
    let (r, dir) = refusal(&job, |p| p.supported_page_sizes_um = vec![(1, 1)], &[]);
    let Err(RenderError::LayoutBlocked(f)) = r else {
        panic!("{r:?}")
    };
    assert_eq!(f[0].code, ErrorCode::UnsupportedMediaSize);
    assert!(dir.files().is_empty());
}

#[test]
fn confirmation_findings_need_an_explicit_matching_acknowledgement() {
    let job = plain_job(|_| {});
    // Margin larger than the artwork-to-edge distance: the placement touches every edge.
    let strict = |p: &mut veylora_layout::LayoutPolicy| p.margin_um = 100;
    let (r, dir) = refusal(&job, strict, &[]);
    let Err(RenderError::LayoutConfirmationRequired(found)) = r else {
        panic!("{r:?}")
    };
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, ErrorCode::UnsafeMargin);
    assert!(dir.files().is_empty());
    let subject = found[0].subject.clone();

    // The wrong code, or the right code for another placement, does not count.
    for wrong in [
        AcknowledgedFinding {
            code: ErrorCode::LowEffectiveResolution,
            subject: subject.clone(),
        },
        AcknowledgedFinding {
            code: ErrorCode::UnsafeMargin,
            subject: Some("other".into()),
        },
        AcknowledgedFinding {
            code: ErrorCode::UnsafeMargin,
            subject: None,
        },
    ] {
        let list = [wrong];
        let (r, _d) = refusal(&job, strict, &list);
        assert!(matches!(r, Err(RenderError::LayoutConfirmationRequired(_))));
    }

    // The exact finding, acknowledged, renders and is recorded in the manifest.
    let list = [AcknowledgedFinding {
        code: ErrorCode::UnsafeMargin,
        subject,
    }];
    let (r, _d) = refusal(&job, strict, &list);
    let art = r.unwrap();
    assert_eq!(art.manifest.acknowledged_layout_findings.len(), 1);
    assert_eq!(
        art.manifest.acknowledged_layout_findings[0].code,
        ErrorCode::UnsafeMargin
    );
}

#[test]
fn low_effective_resolution_is_checked_against_the_policy() {
    let job = plain_job(|_| {}); // 254 ppi
    let (r, _d) = refusal(&job, |p| p.min_effective_ppi = 300, &[]);
    let Err(RenderError::LayoutConfirmationRequired(f)) = r else {
        panic!("{r:?}")
    };
    assert_eq!(f[0].code, ErrorCode::LowEffectiveResolution);
    let (r, _d) = refusal(&job, |p| p.min_effective_ppi = 254, &[]);
    assert!(r.unwrap().manifest.acknowledged_layout_findings.is_empty());
}
