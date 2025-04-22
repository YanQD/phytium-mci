use core::{ptr::{copy_nonoverlapping, write_bytes, NonNull}, slice::{from_raw_parts, from_raw_parts_mut}};

use alloc::vec::Vec;
use log::warn;

use super::osa_dealloc;

pub struct PoolBuffer {
    size: usize,
    addr: NonNull<u8>,
}

impl PoolBuffer {
    pub fn new(size: usize, addr: NonNull<u8>) -> Self {
        Self {
            size,
            addr,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.addr.as_ptr(), self.size) }
    }

    pub fn as_slice_u32(&self) -> &[u32] {
        unsafe { from_raw_parts(self.addr.as_ptr() as *const u32, self.size / size_of::<u32>()) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.addr.as_ptr(), self.size) }
    }

    pub fn to_vec_u32(&self) -> Vec<u32> {
        let slice = self.as_slice_u32();
        warn!("now size is {}", self.size());
        slice.to_vec()
    }

    pub fn clear(&mut self) {
        unsafe { write_bytes(self.addr.as_ptr(), 0, self.size); }
    }

    pub fn copy_from_slice<T: Copy>(&mut self, src: &[T]) -> Result<(), &'static str> {
        let len = src.len() * size_of::<T>();
        if self.size < len {
            return Err("Too small to receive data!");
        }

        unsafe {
            copy_nonoverlapping(
                src.as_ptr() as *mut u8,
                self.addr.as_ptr(),
                len
            );
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn addr(&self) -> NonNull<u8> {
        self.addr.clone()
    }
}

impl Drop for PoolBuffer {
    fn drop(&mut self) {
        osa_dealloc(self.addr, self.size);
    }
}

impl Into<Vec<u32>> for PoolBuffer {
    fn into(self) -> Vec<u32> {
        unsafe {
           let slice = from_raw_parts(self.addr.as_ptr() as *const u32, self.size / 4);
           slice.to_vec()
        }
    }
}