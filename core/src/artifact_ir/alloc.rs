use core::fmt;
use core::ptr::NonNull;
use std::alloc;

// this is not exact the item aligen, but a constant for the all possible items
// on all platforms.
pub(super) const fn item_align_up(x: usize) -> usize {
    const ALIGN: usize = 8;
    const MASK: usize = !(ALIGN - 1);

    (x + ALIGN - 1) & MASK
}

pub(super) fn get_sizes_array(origin_sizes: &[u8]) -> (&[u64], Vec<u8>) {
    if cfg!(target_endian = "little") {
        let sizes = unsafe {
            std::slice::from_raw_parts(
                origin_sizes.as_ptr() as *const u64,
                origin_sizes.len() / std::mem::size_of::<u64>(),
            )
        };
        (sizes, Vec::default())
    } else {
        let swaped_sizes = origin_sizes.to_vec();
        let sizes = unsafe {
            std::slice::from_raw_parts_mut(
                origin_sizes.as_ptr() as *mut u64,
                origin_sizes.len() / std::mem::size_of::<u64>(),
            )
        };
        for sz in sizes.iter_mut() {
            *sz = u64::from_le(*sz);
        }
        let sizes = unsafe {
            std::slice::from_raw_parts(
                origin_sizes.as_ptr() as *const u64,
                origin_sizes.len() / std::mem::size_of::<u64>(),
            )
        };
        (sizes, swaped_sizes)
    }
}

pub(super) struct AlignedBuffer {
    ptr: core::ptr::NonNull<u8>,
    len: usize,
}

impl Drop for AlignedBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.ptr.as_ptr(), self.layout());
        }
    }
}

impl fmt::Debug for AlignedBuffer {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl Clone for AlignedBuffer {
    #[inline]
    fn clone(&self) -> Self {
        let mut cloned = Self::with_size(self.len);
        cloned.as_mut_slice().copy_from_slice(self.as_slice());
        cloned
    }
}

impl AlignedBuffer {
    pub const ALIGNMENT: usize = 16;

    #[inline]
    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(self.len, Self::ALIGNMENT) }
    }

    #[inline]
    pub fn with_size(len: usize) -> Self {
        let ptr = unsafe {
            let layout = alloc::Layout::from_size_align_unchecked(len, Self::ALIGNMENT);
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            NonNull::new_unchecked(ptr)
        };
        Self { ptr, len }
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

// Safety: AlignedBuffer is used read-only
unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}
