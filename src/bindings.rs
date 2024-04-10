use core::ffi::{c_int, c_void};

// ideally we would use c_size_t but it's unstable

#[allow(dead_code)]
extern "C" {
	/* C standard */
	pub fn malloc(size: usize) -> *mut c_void;
	pub fn calloc(nmemb: usize, size: usize) -> *mut c_void;
	pub fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
	pub fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void;
	pub fn free(ptr: *mut c_void);

	/* POSIX */
	pub fn posix_memalign(memptr: *mut *mut c_void, alignment: usize, size: usize) -> c_int;

	/* glibc extensions */
	#[cfg(target_os = "android")]
	pub fn malloc_usable_size(ptr: *const c_void) -> usize;
	#[cfg(not(target_os = "android"))]
	pub fn malloc_usable_size(ptr: *mut c_void) -> usize;
	pub fn mallopt(param: c_int, value: c_int) -> c_int;
	pub fn malloc_trim(pad: usize) -> c_int;
	pub fn malloc_stats() -> c_void;

	/* TODO: add support for these
		#if defined(__GLIBC__) || defined(__ANDROID__)
	struct mallinfo h_mallinfo(void);
	#endif
	#ifndef __ANDROID__
	int h_malloc_info(int options, FILE *fp);
	#endif
	*/

	/* obsolete glbc extensions */
	pub fn memalign(alignment: usize, size: usize) -> *mut c_void;
	#[cfg(not(target_os = "android"))]
	pub fn valloc(size: usize) -> *mut c_void;
	#[cfg(not(target_os = "android"))]
	pub fn pvalloc(size: usize) -> *mut c_void;
	pub fn cfree(ptr: *mut c_void) -> c_void;
	pub fn malloc_get_state() -> *mut c_void;
	pub fn malloc_set_state(state: *mut c_void) -> c_int;

	/* TODO: add support for these android extensions
		// Android extensions
	#ifdef __ANDROID__
	size_t h_mallinfo_narenas(void);
	size_t h_mallinfo_nbins(void);
	struct mallinfo h_mallinfo_arena_info(size_t arena);
	struct mallinfo h_mallinfo_bin_info(size_t arena, size_t bin);
	int h_malloc_iterate(uintptr_t base, size_t size, void (*callback)(uintptr_t ptr, size_t size, void *arg),
				  void *arg);
	void h_malloc_disable(void);
	void h_malloc_enable(void);
	void h_malloc_disable_memory_tagging(void);
	#endif */

	/* hardened_malloc extensions */
	/// return an upper bound on object size for any pointer based on malloc
	/// metadata
	pub fn malloc_object_size(ptr: *const c_void) -> usize;

	/// similar to malloc_object_size, but avoiding locking so the results are
	/// much more limited
	pub fn malloc_object_size_fast(ptr: *const c_void) -> usize;

	/// The free function with an extra parameter for passing the size requested
	/// at allocation time.
	///
	/// This offers the same functionality as C++14 sized deallocation and can
	/// be used to implement it.
	///
	/// A performance-oriented allocator would use this as a performance
	/// enhancement with undefined behavior on a mismatch. Instead, this
	/// hardened allocator implementation uses it to improve security by
	/// checking that the passed size matches the allocated size.
	pub fn free_sized(ptr: *mut c_void, expected_size: usize) -> c_void;
}
