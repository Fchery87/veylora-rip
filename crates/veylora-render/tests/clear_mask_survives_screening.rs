use veylora_domain::recipe::{CalibrationPolicy, ScreenAlgorithm, ScreenPolicy};
use veylora_render::{render_bands, CancellationToken, RenderLimits};
use veylora_screen::{apply_calibration, ScreenSpec};
use veylora_separation::{enforce_clear_mask, ClearMask, LogicalPlanes, Plane};

#[test]
fn cleared_pixels_stay_empty_through_calibration_and_banded_screening() {
    let (w, h) = (6, 7);
    let mut keep = Plane::new(w, h, 1.0);
    for (x, y) in [(0, 0), (2, 3), (5, 6), (3, 3)] {
        keep.set(x, y, 0.0);
    }
    let clear = ClearMask::from_plane(keep.clone()).unwrap();
    let mut planes = LogicalPlanes {
        c: Plane::new(w, h, 1.0),
        m: Plane::new(w, h, 0.9),
        y: Plane::new(w, h, 0.6),
        k: Plane::new(w, h, 0.3),
        w: Plane::new(w, h, 1.0),
    };
    enforce_clear_mask(&mut planes, &clear).unwrap();
    // Calibration lifts midtones and limits load, but cannot bring a cleared pixel back.
    let calibration = CalibrationPolicy {
        revision: 1,
        process_curves: std::array::from_fn(|_| vec![0, 50000, 65535]),
        channel_limits_percent: [100; 4],
        total_coverage_percent: 250,
    };
    apply_calibration(&mut planes, &calibration).unwrap();
    let spec = ScreenSpec {
        policy: ScreenPolicy {
            algorithm: ScreenAlgorithm::OrderedThreshold,
            revision: 1,
            seed: 0,
            phase_x: 0,
            phase_y: 0,
            cell_width: 2,
            cell_height: 2,
            thresholds: vec![0, 32768, 49152, 16384],
        },
        dot_level: 255,
    };
    let limits = RenderLimits {
        max_pixels: 1000,
        band_height: 3,
    };
    for plane in [&planes.c, &planes.m, &planes.y, &planes.k, &planes.w] {
        let bands = render_bands(plane, &spec, limits, &CancellationToken::new()).unwrap();
        let screened: Vec<u16> = bands.into_iter().flat_map(|b| b.pixels).collect();
        for (i, keep) in keep.pixels.iter().enumerate() {
            if *keep == 0.0 {
                assert_eq!(screened[i], 0, "pixel {i} reappeared after screening");
            }
        }
    }
}
