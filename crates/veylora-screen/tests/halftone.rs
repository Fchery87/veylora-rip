use veylora_domain::recipe::{ScreenAlgorithm, ScreenPolicy};
use veylora_screen::{screen_band, PageOrigin, ScreenError, ScreenSpec};
use veylora_separation::Plane;

fn spec(cell_w: u16, cell_h: u16, thresholds: Vec<u16>, phase: (u32, u32)) -> ScreenSpec {
    ScreenSpec {
        policy: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: phase.0,
            phase_y: phase.1,
            cell_width: cell_w,
            cell_height: cell_h,
            thresholds,
        },
        dot_level: 7,
    }
}
fn at(x: i64, y: i64) -> PageOrigin {
    PageOrigin { x, y }
}
fn dots(plane: &Plane, origin: PageOrigin, s: &ScreenSpec) -> usize {
    screen_band(plane, origin, s)
        .unwrap()
        .pixels
        .iter()
        .filter(|p| **p == 7)
        .count()
}

#[test]
fn flat_fields_print_the_fraction_of_thresholds_below_coverage() {
    let s = spec(4, 1, vec![0, 16384, 32768, 49152], (0, 0));
    let field = |v| Plane::new(8, 4, v);
    assert_eq!(dots(&field(0.0), at(0, 0), &s), 0);
    assert_eq!(dots(&field(1.0), at(0, 0), &s), 32);
    // 0.5 -> 32768: thresholds 0 and 16384 are below it, 32768 is not (strict).
    assert_eq!(dots(&field(32768.0 / 65535.0), at(0, 0), &s), 16);
    assert_eq!(dots(&field(0.3), at(0, 0), &s), 16); // 19661 > 0 and > 16384
    assert_eq!(dots(&field(0.2), at(0, 0), &s), 8); // 13107 > 0 only
}
#[test]
fn dot_density_never_decreases_along_a_ramp() {
    let s = spec(2, 2, vec![0, 32768, 49152, 16384], (0, 0));
    let mut last = 0;
    for step in 0..=20 {
        let n = dots(&Plane::new(8, 8, step as f32 / 20.0), at(0, 0), &s);
        assert!(n >= last, "step {step}");
        last = n;
    }
    assert_eq!(last, 64);
}
#[test]
fn output_codes_are_zero_or_the_dot_level() {
    let s = spec(2, 1, vec![0, 30000], (0, 0));
    let band = screen_band(&Plane::new(4, 3, 0.4), at(0, 0), &s).unwrap();
    assert!(band.pixels.iter().all(|p| *p == 0 || *p == 7));
    assert_eq!(band.origin, at(0, 0));
}
#[test]
fn phase_and_origin_use_absolute_page_coordinates() {
    let s = spec(3, 2, vec![0, 10000, 20000, 30000, 40000, 50000], (1, 1));
    let plane = Plane::new(6, 4, 0.5);
    let whole = screen_band(&plane, at(0, 0), &s).unwrap();
    // The same pixels, seen as a sub-band that starts mid-cell, match the whole page.
    let sub = Plane::new(3, 2, 0.5);
    let part = screen_band(&sub, at(2, 1), &s).unwrap();
    for y in 0..2 {
        for x in 0..3 {
            assert_eq!(part.pixels[y * 3 + x], whole.pixels[(y + 1) * 6 + x + 2]);
        }
    }
    // Negative origins wrap consistently.
    let neg = screen_band(&sub, at(-4, -3), &s).unwrap();
    let shifted = screen_band(&sub, at(-4 + 3 * 5, -3 + 2 * 4), &s).unwrap();
    assert_eq!(neg.pixels, shifted.pixels);
    // Changing the phase moves the pattern.
    let moved = spec(3, 2, s.policy.thresholds.clone(), (2, 1));
    assert_ne!(
        whole.pixels,
        screen_band(&plane, at(0, 0), &moved).unwrap().pixels
    );
}
#[test]
fn invalid_specs_and_planes_are_rejected() {
    let plane = Plane::new(2, 2, 0.5);
    let bad = |s: ScreenSpec| {
        assert_eq!(
            screen_band(&plane, at(0, 0), &s),
            Err(ScreenError::InvalidScreen)
        )
    };
    bad(spec(0, 1, vec![], (0, 0)));
    bad(spec(2, 2, vec![0, 1, 2], (0, 0))); // wrong threshold count
    bad(spec(2, 1, vec![0, 65535], (0, 0))); // could never print full coverage
    bad(spec(u16::MAX, u16::MAX, vec![0; 4], (0, 0))); // oversize cell
    let mut zero = spec(1, 1, vec![0], (0, 0));
    zero.dot_level = 0;
    assert_eq!(
        screen_band(&plane, at(0, 0), &zero),
        Err(ScreenError::InvalidLevels)
    );
    let ok = spec(1, 1, vec![0], (0, 0));
    let mut nan = plane.clone();
    nan.pixels[0] = f32::NAN;
    assert_eq!(
        screen_band(&nan, at(0, 0), &ok),
        Err(ScreenError::InvalidPlane)
    );
    assert_eq!(
        screen_band(&Plane::new(0, 0, 0.0), at(0, 0), &ok),
        Err(ScreenError::InvalidPlane)
    );
}
