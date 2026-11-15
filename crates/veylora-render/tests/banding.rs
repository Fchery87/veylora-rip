use veylora_domain::recipe::{ScreenAlgorithm, ScreenPolicy};
use veylora_render::{render_bands, CancellationToken, RenderError, RenderLimits};
use veylora_screen::{PageOrigin, ScreenSpec};
use veylora_separation::Plane;

pub fn spec() -> ScreenSpec {
    ScreenSpec {
        policy: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: 1,
            phase_y: 1,
            cell_width: 2,
            cell_height: 2,
            thresholds: vec![0, 32768, 49152, 16384],
        },
        dot_level: 255,
    }
}
fn plane(w: usize, h: usize) -> Plane {
    Plane::from_pixels(w, h, (0..w * h).map(|i| (i % 7) as f32 / 7.0).collect()).unwrap()
}
fn limits(max_pixels: usize, band_height: usize) -> RenderLimits {
    RenderLimits {
        max_pixels,
        band_height,
    }
}

#[test]
fn band_height_does_not_change_pixels_for_any_split() {
    let p = plane(5, 11);
    let flat = |h| -> Vec<u16> {
        render_bands(&p, &spec(), limits(100, h), &CancellationToken::new())
            .unwrap()
            .into_iter()
            .flat_map(|b| b.pixels)
            .collect()
    };
    let whole = flat(11);
    for h in 1..=11 {
        assert_eq!(whole, flat(h), "band height {h}");
    }
}
#[test]
fn bands_carry_absolute_page_origins() {
    let bands = render_bands(
        &plane(4, 5),
        &spec(),
        limits(100, 2),
        &CancellationToken::new(),
    )
    .unwrap();
    let origins: Vec<_> = bands.iter().map(|b| b.origin).collect();
    assert_eq!(
        origins,
        vec![
            PageOrigin { x: 0, y: 0 },
            PageOrigin { x: 0, y: 2 },
            PageOrigin { x: 0, y: 4 }
        ]
    );
}
#[test]
fn rejects_oversize_zero_band_and_invalid_planes() {
    let t = CancellationToken::new();
    assert_eq!(
        render_bands(&plane(4, 4), &spec(), limits(15, 2), &t),
        Err(RenderError::LimitExceeded)
    );
    assert_eq!(
        render_bands(&plane(4, 4), &spec(), limits(100, 0), &t),
        Err(RenderError::InvalidBandHeight)
    );
    let mut nan = plane(2, 2);
    nan.pixels[0] = f32::NAN;
    assert_eq!(
        render_bands(&nan, &spec(), limits(100, 1), &t),
        Err(RenderError::InvalidPlane)
    );
    assert_eq!(
        render_bands(&Plane::new(0, 0, 0.0), &spec(), limits(100, 1), &t),
        Err(RenderError::InvalidPlane)
    );
}
#[test]
fn cancellation_from_a_clone_stops_rendering() {
    let t = CancellationToken::new();
    let owner = t.clone();
    owner.cancel();
    assert_eq!(
        render_bands(&plane(4, 4), &spec(), limits(100, 1), &t),
        Err(RenderError::Cancelled)
    );
}
