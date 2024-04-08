#![no_std]

mod bindings;
use core::{
	alloc::{GlobalAlloc, Layout},
	ffi::c_void,
};

pub use bindings::{calloc, free, malloc, realloc};

pub struct HardenedMalloc;

unsafe impl GlobalAlloc for HardenedMalloc {
	#[inline]
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 { malloc(layout.size()) as *mut u8 }

	#[inline]
	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { calloc(layout.size(), 1) as *mut u8 }

	#[inline]
	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) { free(ptr as *mut c_void); }

	#[inline]
	unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, size: usize) -> *mut u8 {
		realloc(ptr as *mut c_void, size) as *mut u8
	}
}
