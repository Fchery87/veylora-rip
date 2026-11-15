use veylora_domain::recipe::{ScreenAlgorithm, ScreenPolicy};
use veylora_screen::{screen_band, PageOrigin, ScreenSpec};
use veylora_separation::Plane;

fn spec() -> ScreenSpec {
    ScreenSpec {
        policy: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: 2,
            phase_y: 3,
            cell_width: 3,
            cell_height: 2,
            thresholds: vec![0, 11000, 22000, 33000, 44000, 55000],
        },
        dot_level: 255,
    }
}
/// Deterministic pseudo-random coverage so the test covers arbitrary tones, not a ramp.
fn page(w: usize, h: usize) -> Plane {
    let mut state = 0x2545_f491_u32;
    let pixels = (0..w * h)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 8) as f32 / (1u32 << 24) as f32
        })
        .collect();
    Plane::from_pixels(w, h, pixels).unwrap()
}
fn crop(p: &Plane, x0: usize, y0: usize, w: usize, h: usize) -> Plane {
    let mut v = Vec::new();
    for y in y0..y0 + h {
        v.extend_from_slice(&p.pixels[y * p.width + x0..y * p.width + x0 + w]);
    }
    Plane::from_pixels(w, h, v).unwrap()
}

#[test]
fn any_tiling_of_a_page_screens_identically_to_the_whole_page() {
    let (w, h) = (13, 9);
    let source = page(w, h);
    let whole = screen_band(&source, PageOrigin { x: 0, y: 0 }, &spec()).unwrap();
    for (tw, th) in [(1, 1), (2, 3), (5, 4), (13, 1), (1, 9), (7, 7), (13, 9)] {
        let mut assembled = vec![0u16; w * h];
        let mut y0 = 0;
        while y0 < h {
            let bh = th.min(h - y0);
            let mut x0 = 0;
            while x0 < w {
                let bw = tw.min(w - x0);
                let band = screen_band(
                    &crop(&source, x0, y0, bw, bh),
                    PageOrigin {
                        x: x0 as i64,
                        y: y0 as i64,
                    },
                    &spec(),
                )
                .unwrap();
                for y in 0..bh {
                    for x in 0..bw {
                        assembled[(y0 + y) * w + x0 + x] = band.pixels[y * bw + x];
                    }
                }
                x0 += bw;
            }
            y0 += bh;
        }
        assert_eq!(assembled, whole.pixels, "tile {tw}x{th}");
    }
}
