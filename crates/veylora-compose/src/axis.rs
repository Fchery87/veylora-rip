//! One-axis area weights. Exact integer geometry; only the final fractions are floating point.

/// Which source axis a destination axis reads. Quarter turns swap the axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceAxis {
    X,
    Y,
}

/// How much of one destination pixel each source pixel covers along a single axis.
///
/// `[lo, hi)` is the destination pixel's footprint along the source axis in nanometres, measured
/// from the source's origin. It is clipped to `[0, extent_nm]`; the uncovered part simply has no
/// weight, so weights sum to the covered fraction of the destination pixel. The `n` source pixels
/// divide `extent_nm` evenly, and `pixel_nm` is the destination pixel's size on this axis.
pub(crate) fn axis_weights(
    lo: i128,
    hi: i128,
    extent_nm: i128,
    n: u32,
    pixel_nm: i128,
) -> Vec<(u32, f64)> {
    let (lo, hi) = (lo.max(0), hi.min(extent_nm));
    if lo >= hi {
        return Vec::new();
    }
    let n = i128::from(n);
    let first = lo * n / extent_nm;
    let last = (hi * n + extent_nm - 1) / extent_nm;
    let denominator = (n * pixel_nm) as f64;
    (first..last)
        .filter_map(|i| {
            let overlap = (hi * n).min((i + 1) * extent_nm) - (lo * n).max(i * extent_nm);
            (overlap > 0).then(|| (i as u32, overlap as f64 / denominator))
        })
        .collect()
}
