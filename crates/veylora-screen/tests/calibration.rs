use veylora_domain::recipe::CalibrationPolicy;
use veylora_screen::{apply_calibration, CalibrationError};
use veylora_separation::{LogicalPlanes, Plane};

fn policy() -> CalibrationPolicy {
    CalibrationPolicy {
        revision: 1,
        process_curves: std::array::from_fn(|_| vec![0, 65535]),
        channel_limits_percent: [100; 4],
        total_coverage_percent: 400,
    }
}
fn planes(c: f32, m: f32, y: f32, k: f32, white: f32) -> LogicalPlanes {
    let p = |v| Plane::new(2, 1, v);
    LogicalPlanes {
        c: p(c),
        m: p(m),
        y: p(y),
        k: p(k),
        w: p(white),
    }
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{a} != {b}");
}

#[test]
fn identity_policy_changes_nothing_and_white_is_never_touched() {
    let mut p = planes(0.2, 0.4, 0.6, 0.8, 0.9);
    let before = p.clone();
    apply_calibration(&mut p, &policy()).unwrap();
    for (a, b) in [
        (&p.c, &before.c),
        (&p.m, &before.m),
        (&p.y, &before.y),
        (&p.k, &before.k),
    ] {
        close(a.pixels[0], b.pixels[0]);
    }
    assert_eq!(p.w, before.w);
}
#[test]
fn each_channel_follows_its_own_curve_by_linear_interpolation() {
    let mut pol = policy();
    pol.process_curves[0] = vec![0, 0, 65535]; // C: dead zone then ramp
    pol.process_curves[1] = vec![0, 32768, 65535]; // M: lifted midtone
    let mut p = planes(0.5, 0.5, 0.5, 0.5, 0.0);
    apply_calibration(&mut p, &pol).unwrap();
    close(p.c.pixels[0], 0.0);
    close(p.m.pixels[0], 32768.0 / 65535.0);
    close(p.y.pixels[0], 0.5);
    let mut q = planes(0.75, 0.0, 0.0, 0.0, 0.0);
    apply_calibration(&mut q, &pol).unwrap();
    close(q.c.pixels[0], 0.5);
}
#[test]
fn channel_limits_cap_the_calibrated_value() {
    let mut pol = policy();
    pol.channel_limits_percent = [100, 80, 60, 40];
    let mut p = planes(1.0, 1.0, 1.0, 1.0, 1.0);
    apply_calibration(&mut p, &pol).unwrap();
    close(p.c.pixels[0], 1.0);
    close(p.m.pixels[0], 0.8);
    close(p.y.pixels[0], 0.6);
    close(p.k.pixels[0], 0.4);
    close(p.w.pixels[0], 1.0);
}
#[test]
fn combined_limit_scales_all_process_channels_proportionally() {
    let mut pol = policy();
    pol.total_coverage_percent = 200;
    let mut p = planes(1.0, 1.0, 1.0, 1.0, 1.0);
    apply_calibration(&mut p, &pol).unwrap();
    for plane in [&p.c, &p.m, &p.y, &p.k] {
        close(plane.pixels[0], 0.5);
    }
    // Proportions are preserved and pixels already under the limit are untouched.
    let mut q = planes(0.6, 0.3, 0.0, 0.0, 0.0);
    pol.total_coverage_percent = 45;
    apply_calibration(&mut q, &pol).unwrap();
    close(q.c.pixels[0], 0.3);
    close(q.m.pixels[0], 0.15);
    let mut r = planes(0.1, 0.1, 0.0, 0.0, 0.0);
    apply_calibration(&mut r, &pol).unwrap();
    close(r.c.pixels[0], 0.1);
}
#[test]
fn limits_hold_across_a_full_ramp_of_every_channel() {
    let mut pol = policy();
    pol.channel_limits_percent = [90, 70, 50, 100];
    pol.total_coverage_percent = 180;
    let ramp: Vec<f32> = (0..=255).map(|i| i as f32 / 255.0).collect();
    let plane = |reverse: bool| {
        let mut v = ramp.clone();
        if reverse {
            v.reverse();
        }
        Plane::from_pixels(256, 1, v).unwrap()
    };
    let mut p = LogicalPlanes {
        c: plane(false),
        m: plane(true),
        y: plane(false),
        k: plane(true),
        w: plane(false),
    };
    apply_calibration(&mut p, &pol).unwrap();
    for i in 0..256 {
        let v = [p.c.pixels[i], p.m.pixels[i], p.y.pixels[i], p.k.pixels[i]];
        for (value, cap) in v.iter().zip([0.9, 0.7, 0.5, 1.0]) {
            assert!(*value <= cap + 1e-5 && *value >= 0.0, "pixel {i}: {v:?}");
        }
        assert!(v.iter().sum::<f32>() <= 1.8 + 1e-4, "pixel {i}: {v:?}");
    }
}
#[test]
fn empty_coverage_stays_empty() {
    let mut pol = policy();
    pol.process_curves = std::array::from_fn(|_| vec![0, 40000, 65535]);
    let mut p = planes(0.0, 0.0, 0.0, 0.0, 0.0);
    apply_calibration(&mut p, &pol).unwrap();
    for plane in [&p.c, &p.m, &p.y, &p.k] {
        assert_eq!(plane.pixels, vec![0.0, 0.0]);
    }
}
#[test]
fn calibration_is_deterministic() {
    let mut pol = policy();
    pol.process_curves[2] = vec![0, 20000, 65535];
    pol.total_coverage_percent = 150;
    let run = || {
        let mut p = planes(0.7, 0.55, 0.35, 0.9, 0.2);
        apply_calibration(&mut p, &pol).unwrap();
        p
    };
    assert_eq!(run(), run());
}
#[test]
fn invalid_policies_and_planes_are_rejected_without_modifying_anything() {
    let reject = |pol: CalibrationPolicy, expected: CalibrationError| {
        let mut p = planes(0.5, 0.5, 0.5, 0.5, 0.5);
        let before = p.clone();
        assert_eq!(apply_calibration(&mut p, &pol), Err(expected));
        assert_eq!(p, before);
    };
    let mut pol = policy();
    pol.process_curves[3] = vec![100, 65535]; // lifts zero
    reject(pol, CalibrationError::InvalidCurve);
    let mut pol = policy();
    pol.process_curves[0] = vec![0];
    reject(pol, CalibrationError::InvalidCurve);
    let mut pol = policy();
    pol.channel_limits_percent[1] = 101;
    reject(pol, CalibrationError::InvalidLimit);
    let mut pol = policy();
    pol.total_coverage_percent = 401;
    reject(pol, CalibrationError::InvalidLimit);

    let mut p = planes(0.5, 0.5, 0.5, 0.5, 0.5);
    p.y = Plane::new(3, 1, 0.5);
    let before = p.clone();
    assert_eq!(
        apply_calibration(&mut p, &policy()),
        Err(CalibrationError::DimensionMismatch)
    );
    assert_eq!(p, before);
    let mut p = planes(0.5, 0.5, 0.5, 0.5, 0.5);
    p.k.pixels[0] = f32::NAN;
    assert_eq!(
        apply_calibration(&mut p, &policy()),
        Err(CalibrationError::InvalidPlane)
    );
}
