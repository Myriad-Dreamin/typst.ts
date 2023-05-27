use std::num::NonZeroUsize;

use tiny_skia as sk;

/// Number of bytes per pixel.
pub const BYTES_PER_PIXEL: usize = 4;

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

pub struct PixmapBuffer {
    data: Vec<u8>,
    size: IntSize,
}

impl PixmapBuffer {
    pub fn for_size(size: typst::geom::Size, pixel_per_pt: f32) -> Option<Self> {
        let (mut prealloc, size) = {
            let pxw = (pixel_per_pt * (size.x.to_pt() as f32)).round().max(1.0) as u32;
            let pxh = (pixel_per_pt * (size.y.to_pt() as f32)).round().max(1.0) as u32;
            let size = IntSize {
                width: pxw,
                height: pxh,
            };
            let data_len = data_len_for_size(size)?;

            (vec![0; data_len], size)
        };

        let _ = sk::PixmapMut::from_bytes(&mut prealloc, size.width, size.height)?;
        Some(Self {
            data: prealloc,
            size,
        })
    }

    pub fn size(&self) -> IntSize {
        self.size
    }

    pub fn as_canvas_mut(&mut self) -> sk::PixmapMut {
        sk::PixmapMut::from_bytes(&mut self.data, self.size.width, self.size.height).unwrap()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

/// Returns minimum bytes per row as usize.
///
/// Pixmap's maximum value for row bytes must fit in 31 bits.
fn min_row_bytes(size: IntSize) -> Option<NonZeroUsize> {
    let w = i32::try_from(size.width).ok()?;
    let w = w.checked_mul(BYTES_PER_PIXEL as i32)?;
    NonZeroUsize::new(w as usize)
}

/// Returns storage size required by pixel array.
fn compute_data_len(size: IntSize, row_bytes: usize) -> Option<usize> {
    let h = size.height.checked_sub(1)?;
    let h = (h as usize).checked_mul(row_bytes)?;

    let w = (size.width as usize).checked_mul(BYTES_PER_PIXEL)?;

    h.checked_add(w)
}

fn data_len_for_size(size: IntSize) -> Option<usize> {
    let row_bytes = min_row_bytes(size)?;
    compute_data_len(size, row_bytes.get())
}
