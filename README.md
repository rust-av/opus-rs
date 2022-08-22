# libopus bindings

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Actions Status](https://github.com/rust-av/opus-rs/workflows/opus/badge.svg)](https://github.com/rust-av/opus-rs/actions)


It is a simple [binding][1] and safe abstraction over [libopus][2].

## Building

To build the code, always have a look at [CI](https://github.com/rust-av/opus-rs/blob/master/.github/workflows/opus.yml) to install the necessary dependencies on all
supported operating systems.


## Building with vcpkg for Windows x64

To build with [vcpkg](https://vcpkg.io/en/index.html), you need to follow these
steps:

1. Install `pkg-config` through [chocolatey](https://chocolatey.org/)

       choco install pkgconfiglite

2. Install `opus`

       vcpkg install opus:x64-windows

3. Add to the `PKG_CONFIG_PATH` environment variable the path `$VCPKG_INSTALLATION_ROOT\installed\x64-windows\lib\pkgconfig`

4. Build code

       cargo build --workspace

To speed up the computation, you can build your packages only in `Release` mode
adding the `set(VCPKG_BUILD_TYPE release)` line to the
`$VCPKG_INSTALLATION_ROOT\triplets\x64-windows.cmake` file.

Building for Windows x86 is the same, just replace `x64` with `x86` in the
steps above.

## TODO
- [ ] Simple bindings
- [ ] Safe abstraction
- [ ] Examples

[1]: https://github.com/servo/rust-bindgen
[2]: https://opus-codec.org/
