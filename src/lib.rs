#![no_std]

extern crate hardened_malloc_sys as ffi;

use core::alloc::{GlobalAlloc, Layout, Allocator, AllocError};
use core::ffi::{c_int, c_void};
use ffi::*;

pub struct HardenedMalloc;

unsafe impl GlobalAlloc for HardenedMalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> Result<*mut u8, AllocError> {
        let ptr = h_malloc(layout.size());
        if ptr.is_null() {
            return Err(AllocError);
        }
        Ok(ptr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> Result<*mut u8, AllocError> {
        let ptr = h_calloc(layout.size(), 1);
        if ptr.is_null() {
            return Err(AllocError);
        }
        Ok(ptr)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        h_free(ptr as *mut c_void);
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> Result<*mut u8, AllocError> {
        let ptr = h_realloc(ptr, new_size);
        if ptr.is_null() {
            return Err(AllocError);
        }
        Ok(ptr)
    }
}
