# hardened_malloc-sys

the sys repo, rust wrapper

to build, just run `cargo build -r` which will build the light variant by default.
if you want default (called `standard` in this crate) variant, do `cargo build -r --features=standard --no-default-features`

### TODO:
- [ ] test if this even works
- [ ] add support for explicit make config args on top of choosing variant
- [ ] make build script better overall
- [ ] support C preprocessor macro definitions
- [ ] maybe add support for building both variants if both are specified, or dont use a default light variant
- [ ] add support for hardened_malloc `make clean` upon `cargo clean`
- [ ] potentially add support for cross-compiling so i can build on apple silicon for linux x86?
- [ ] add support for hardened_malloc's tests and our own tests
- [ ] add github CI/CD
- [ ] mirror to other places