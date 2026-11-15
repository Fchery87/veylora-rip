mod common;
use common::*;
use std::sync::{atomic::AtomicU32, Arc};
use veylora_domain::{provenance::ContentHash, recipe::WhiteSource};
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
    assert_eq!(m.source.content_hash, job.source.original_hash);
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
        attempt(
            &|j| j.eligible = Some(eligible_mask_with_id(j.eligible.as_ref().unwrap().id, 5, 5))
        ),
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

#[test]
fn only_identity_layouts_are_rendered() {
    let token = CancellationToken::new();
    let try_layout = |mutate: fn(&mut veylora_domain::layout::LayoutInput)| {
        let eligible = eligible_mask();
        let recipe = recipe(
            clear_policy(Some(eligible.id)),
            white_policy(WhiteSource::Alpha, 200),
        );
        let job = job_with(artwork(), None, Some(eligible), recipe, mutate);
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
    assert!(matches!(
        try_layout(|l| l.mirror = true),
        Err(RenderError::UnsupportedLayout(_))
    ));
    assert!(matches!(
        try_layout(|l| l.placements[0].rotation = veylora_domain::layout::Rotation::Deg90),
        Err(RenderError::UnsupportedLayout(_))
    ));
    assert!(matches!(
        try_layout(|l| l.placements[0].x_um = 100),
        Err(RenderError::UnsupportedLayout(_))
    ));
    assert!(matches!(
        try_layout(|l| l.placements[0].crop_x_px = 1),
        Err(RenderError::UnsupportedLayout(_))
    ));
    assert!(matches!(
        try_layout(|l| l.page_width_um += 100),
        Err(RenderError::UnsupportedLayout(_))
    ));
    assert!(matches!(
        try_layout(|l| l.placements[0].width_um += 1),
        Err(RenderError::UnsupportedLayout(_))
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
