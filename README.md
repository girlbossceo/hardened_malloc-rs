# hardened_malloc-rs

Rust wrapper library for GrapheneOS's [hardened_malloc](https://github.com/GrapheneOS/hardened_malloc) that can be integrated as the [global memory allocator](https://doc.rust-lang.org/std/alloc/index.html) in your Rust crate.

### Why?

The default memory allocator apart of your C library (glibc, musl, etc) is still used in your Rust crate unless you build with the other various memory allocators out there such as [jemalloc](https://crates.io/crates/tikv-jemalloc-sys), [mimalloc](https://crates.io/crates/mimalloc), etc. However the majority of memory allocators out there are too performance-focused (jemalloc) or are focused on a balance between security and performance.

A memory allocator like GrapheneOS's hardened_malloc is purely security focused and is perfect for a security focused usecase, but the light variant makes hardened_malloc significantly suitable for replacing your default C library's malloc while still retaining a lot of the security properties and ends up speeding up the performance of your application.

Additionally, building hardened_malloc in your binary instead of relying on LD_PRELOAD'ing creates is more secure as it can be used to create position independent code (`-fPIE`/`-fPIC`) and prevents interposition of exported symbols (aka using LD_PRELOAD to drop in your malloc) with `-fno-semantic-interposition`. And it can benefit from optimisations by the compiler and linker that would not be otherwise available from a dynamic shared library.

https://github.com/GrapheneOS/hardened_malloc?tab=readme-ov-file#individual-applications

> It can offer slightly better performance when integrated into the C standard library and there are other opportunities for similar hardening within C standard library and dynamic linker implementations. For example, a library region can be implemented to offer similar isolation for dynamic libraries as this allocator offers across different size classes. The intention is that this will be offered as part of hardened variants of the Bionic and musl C standard libraries.

### Building

The default features used are "static", "gcc", "light".

To configure, you MUST build without default features. The list of features to configure are:

- `static` - creates a static library of libhardened_malloc for static linking your crate
- `dynamic` - creates a dynamically-linked library of libhardened_malloc
- `gcc` - builds hardened_malloc with gcc as `$CC`
- `clang` - builds hardened_malloc with clang as `$CC`
- `light` - builds hardened_malloc with the light variant/config (balance between performance and security)
- `standard` - builds hardened_malloc with the default variant/config (more secure)

You cannot enable both of the same type of feature at the moment (e.g. cannot enable gcc and clang at the same time).

### Usage

In your Cargo.toml's dependencies (example):

```toml
hardened_malloc-rs = { version = "0.1", features = ["static", "clang", "light"], default-features = false }
```

In your crate's main.rs:

```rs
#[cfg(all(not(target_env = "msvc"), not(target_os = "macos")))]
use hardened_malloc_rs::HardenedMalloc;

#[cfg(all(not(target_env = "msvc"), not(target_os = "macos")))]
#[global_allocator]
static GLOBAL: HardenedMalloc = HardenedMalloc;
```

### Note

This [requires a fork of hardened_malloc](https://github.com/girlbossceo/hardened_malloc/commits/main/) to skip the LTO linking stage if doing static Clang/LLVM builds. LTO on Clang/LLVM will produce LLVM IR bitcode which is not compatible with GNU linker (`ld`) and `ar` without requiring the top level crate to use LLVM linker (`lld`) across the **entire** dependency graph and to use `llvm-ar`. GCC is unaffected.

See https://github.com/girlbossceo/hardened_malloc-rs/issues/5 for more details.

[FatLTO](https://llvm.org/docs/FatLTO.html) using `-ffat-lto-objects` seems like it would solve this compatibility issue, but this is a lld and gold plugin feature only.

### Minimum Supported Rust Version (MSRV)

hardened_malloc states the most ancient set of dependencies that can be used to build hardened_malloc is Debian 12 (bookworm), aka the latest stable. Debian 12 has Rust 1.63 in their repos which will be the MSRV of hardened_malloc-rs.

See https://github.com/GrapheneOS/hardened_malloc?tab=readme-ov-file#dependencies

##### TODO:

- [ ] support building this crate as is to output the .so/.a file
- [ ] test if this even works
- [ ] add support for explicit make config args on top of choosing variant
- [ ] make build script better overall
- [ ] support C preprocessor macro definitions
- [ ] maybe add support for building both variants if both are specified, or dont use a default light variant
- [ ] potentially add support for cross-compiling so i can build on apple silicon for linux x86?
- [ ] add support for hardened_malloc's tests and our own tests
- [ ] add github CI/CD
- [ ] mirror to other places