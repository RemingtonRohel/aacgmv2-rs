use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from("src")
        .canonicalize()
        .expect("Cannot canonicalize path");

    // Get the aacgm_v2 C code files
    let code_dir = "c_aacgm_v2.6";

    // Get the coefficient files
    let coeffs_dir = "aacgm_coeffs-13";

    // If necessary environment variables for AACGM_V2 not set, then set them accordingly
    if !env::var("AACGM_v2_DAT_PREFIX").is_ok() {
        env::set_var(
            "AACGM_v2_DAT_PREFIX",
            dir.join(format!("{coeffs_dir}/{coeffs_dir}-")),
        );
    }
    if !env::var("IGRF_COEFFS").is_ok() {
        env::set_var(
            "IGRF_COEFFS",
            dir.join(format!("{code_dir}/magmodel_1590-2020.txt")),
        );
    }

    // This is the path to the C header file
    let header_path = dir.join(format!("{code_dir}/aacgmlib_v2.h"));
    let header_path_str = header_path.to_str().expect("Path is not a valid string");

    // Path to the intermediate object file for the library
    let obj_path = dir.join(format!("{code_dir}/aacgmlib_v2.o"));

    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search={}/{}",
        dir.to_str().unwrap(),
        code_dir
    );

    // Tell cargo to tell rustc to link the aacgm shared library
    println!("cargo:rustc-link-lib=aacgmlib_v2");

    // Tell cargo to invalidate the built crate whenever the header file changes
    println!("cargo:rerun-if-changed={}", header_path_str);

    // Run `clang` to compile the `aacgmlib_v2.c` file into a `aacgmlib_v2.o` object file.
    // Unwrap if not possible to spawn the process.
    println!("clang -c -o {obj_path:?} {dir:?}/{code_dir}/aacgmlib_v2.c");

    if std::fs::File::open(dir.join(format!("{code_dir}/aacgmlib_v2.c"))).is_err() {
        panic!("C code file missing!")
    }

    // Compile the aacgmlib_v2 library to an archive file `libaacgmlib_v2.a`
    cc::Build::new()
        .file(dir.join(format!("{code_dir}/aacgmlib_v2.c")))
        .warnings(false)
        .compile("aacgmlib_v2");

    let bindings = bindgen::Builder::default()
        // Throw in stdio.h first so all types are available when generating aacgmlib_v2.h header
        .header_contents("_stdio.h", "#include<stdio.h>\n")
        // The input header to generate bindings for
        .header(header_path_str)
        // Tell cargo to invalidate the built crate whenever any of the included header files change
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings
        .generate()
        // Unwrap the result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to $OUT_DIR/bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("{out_path:?}");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
