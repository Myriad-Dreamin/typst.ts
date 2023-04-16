use std::num::NonZeroUsize;
use tiny_skia as sk;

/// An integer size.
///
/// # Guarantees
///
/// - Width and height are positive and non-zero.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct IntSize {
    pub width: u32,
    pub height: u32,
}

/// Returns minimum bytes per row as usize.
///
/// Pixmap's maximum value for row bytes must fit in 31 bits.
pub fn min_row_bytes(size: IntSize) -> Option<NonZeroUsize> {
    let w = i32::try_from(size.width).ok()?;
    let w = w.checked_mul(sk::BYTES_PER_PIXEL as i32)?;
    NonZeroUsize::new(w as usize)
}

/// Returns storage size required by pixel array.
pub fn compute_data_len(size: IntSize, row_bytes: usize) -> Option<usize> {
    let h = size.height.checked_sub(1)?;
    let h = (h as usize).checked_mul(row_bytes)?;

    let w = (size.width as usize).checked_mul(sk::BYTES_PER_PIXEL)?;

    h.checked_add(w)
}

pub fn data_len_for_size(size: IntSize) -> Option<usize> {
    let row_bytes = min_row_bytes(size)?;
    compute_data_len(size, row_bytes.get())
}
