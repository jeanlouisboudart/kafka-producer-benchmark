use std::collections::HashMap;
use std::iter::Sum;
use crate::utils::conversion_utils::AsF64Lossy;

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

