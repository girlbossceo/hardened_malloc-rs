use core::ffi::c_void;

#[allow(dead_code)]
extern "C" {
	/* C standard */
	pub fn malloc(size: usize) -> *mut c_void;
	pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
	pub fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
	pub fn free(ptr: *mut c_void);
}
