use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rustc-link-lib=z");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("include/wrapper.h")
        // Include gztool.c for local cargo builds
        // Note that nix build/run uses fetchGitHub source instead
        .clang_arg("-Ideps/gztool")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_path.join("wrap_static_fns"))
        .allowlist_var("verbosity_level")
        .allowlist_type("point")
        .allowlist_type("access")
        .allowlist_type("returned_output")
        .allowlist_type("VERBOSITY_LEVEL")
        .allowlist_type("INDEX_AND_EXTRACTION_OPTIONS")
        .allowlist_function("create_empty_index")
        .allowlist_function("free_index")
        .allowlist_function("empty_index_list")
        .allowlist_function("add_point")
        .allowlist_function("serialize_index_to_file")
        .allowlist_function("deserialize_index_from_file")
        .allowlist_function("check_index_file")
        .allowlist_function("compress_chunk")
        .allowlist_function("decompress_chunk")
        .allowlist_function("action_create_index")
        .allowlist_function("compress_file")
        .allowlist_function("decompress_file")
        .allowlist_function("decompress_and_build_index")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    cc::Build::new()
        .file(out_path.join("wrap_static_fns.c"))
        .includes([
            env!("CARGO_MANIFEST_DIR"),
            &format!("{}/deps/gztool", env!("CARGO_MANIFEST_DIR")),
        ])
        .compile("wrap_static_fns");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
