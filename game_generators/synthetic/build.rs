extern crate bindgen;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

fn make_gurobi_bindings() {
    if let Ok(gurobi_lib_path) = env::var("GUROBI_LIB") {
        println!("cargo:rustc-link-search=all={}", gurobi_lib_path);
    }
    println!("cargo:rustc-link-lib=gurobi81");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("gurobi_wrapper.h")
        // Finish the builder and generate the bindings.
        .clang_arg("-I/Library/Developer/CommandLineTools/SDKs/MacOSX10.14.sdk/usr/include/") // For my mac. xcode BS working here...
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("gurobi_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    // make_cbc_bindings();
    make_gurobi_bindings();
}