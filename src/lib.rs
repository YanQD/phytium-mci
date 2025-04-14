#![no_std]

extern crate alloc;

use core::{ptr::NonNull, time::Duration};

#[macro_use]
mod regs;
pub mod mci;
pub mod iopad;
pub mod mci_host;
mod tools;

pub use iopad::*;
pub use mci_host::*;

pub trait Kernel {
    fn sleep(duration: Duration);
}

pub(crate) fn sleep(duration: Duration) {
    extern "Rust" {
        fn _phytium_mci_sleep(duration: Duration);
    }

    unsafe {
        _phytium_mci_sleep(duration);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ToDevice,
    FromDevice,
    Bidirectional,
}

pub trait Impl {
    /// map virt address to physical address
    fn map(addr: NonNull<u8>, size: usize, direction: Direction) -> u64;
    /// unmap virt address
    fn unmap(addr: NonNull<u8>, size: usize);
    /// write cache back to memory
    fn flush(addr: NonNull<u8>, size: usize);
    /// invalidate cache
    fn invalidate(addr: NonNull<u8>, size: usize);
}

fn map(addr: NonNull<u8>, size: usize, direction: Direction) -> u64 {
    unsafe { __phytium_map(addr, size, direction) }
}

fn unmap(addr: NonNull<u8>, size: usize) {
    unsafe { __phytium_unmap(addr, size) }
}

fn flush(addr: NonNull<u8>, size: usize) {
    unsafe { __phytium_flush(addr, size) }
}

fn invalidate(addr: NonNull<u8>, size: usize) {
    unsafe { __phytium_invalidate(addr, size) }
}

extern "Rust" {
    fn __phytium_map(addr: NonNull<u8>, size: usize, direction: Direction) -> u64;
    fn __phytium_unmap(addr: NonNull<u8>, size: usize);
    fn __phytium_flush(addr: NonNull<u8>, size: usize);
    fn __phytium_invalidate(addr: NonNull<u8>, size: usize);
}

#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _phytium_mci_sleep(duration: core::time::Duration) {
            <$t as $crate::Kernel>::sleep(duration)
        }
        #[no_mangle]
        fn __dma_api_map(
            addr: core::ptr::NonNull<u8>,
            size: usize,
            direction: $crate::Direction,
        ) -> u64 {
            <$t as $crate::Impl>::map(addr, size, direction)
        }
        #[no_mangle]
        fn __dma_api_unmap(addr: core::ptr::NonNull<u8>, size: usize) {
            <$t as $crate::Impl>::unmap(addr, size)
        }
        #[no_mangle]
        fn __dma_api_flush(addr: core::ptr::NonNull<u8>, size: usize) {
            <$t as $crate::Impl>::flush(addr, size)
        }
        #[no_mangle]
        fn __dma_api_invalidate(addr: core::ptr::NonNull<u8>, size: usize) {
            <$t as $crate::Impl>::invalidate(addr, size)
        }
    };
}
