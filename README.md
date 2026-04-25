# gztool-rs

Rust wrapper of <https://github.com/circulosmeos/gztool>.

Currently supports building .gzi index (with a custom window span), reading it and decompressing data from chunks that cover start of a window that is in the generated index file.

# Building

`cargo build` should work if `zlib` (or `zlib-ng` in `zlib` compat mode) is present in the environment.

Use `nix develop` to enter the environment with all necessary dependencies fetched.
Note that this also includes `gztool` source (with a small patch to help with unwarranted stdout closing), but currently `build.rs` overrides that source include with `deps/gztool` present in this repository (for ease of using without nix).
If the package is built using `nix run` or `nix build`, it should use source fetched from gztool's official github (and patched automatically) as crane does not include `deps/gztool` in the build process (see `commonArgs.src` defined in `flake.nix`) 
