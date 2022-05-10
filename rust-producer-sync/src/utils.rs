use std::collections::HashMap;
use std::iter::Sum;

// A trait so that we can ignore this Clippy here, and here only
pub(crate) trait AsF64Lossy {
    fn as_f64(&self) -> f64;
}

impl AsF64Lossy for i64 {
    #[allow(clippy::cast_precision_loss)]
    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl AsF64Lossy for usize {
    #[allow(clippy::cast_precision_loss)]
    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

pub(crate) trait Avgf64 {
    fn avg_f64(&self) -> f64;
}

pub(crate) fn sum_by<K, T, R: Sum<R>>(stats: &HashMap<K, T>, by: fn(&T) -> R) -> R {
    stats
        .iter()
        .map(|(_, val)| by(val))
        .sum::<R>()
}

pub(crate) fn avg_by<K, T>(stats: &HashMap<K, T>, by: fn(&T) -> f64) -> f64 {
    sum_by(stats, by) / stats.len().as_f64()
}

