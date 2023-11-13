#![no_std]

use core::ffi::{c_int, c_void};

extern crate libc;

mod hardened_malloc_bindings;
pub use hardened_malloc_bindings::*;
