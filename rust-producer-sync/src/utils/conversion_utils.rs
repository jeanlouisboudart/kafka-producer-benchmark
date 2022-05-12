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
