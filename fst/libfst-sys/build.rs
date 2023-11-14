fn main() {
    println!("cargo:rerun-if-changed=fstapi/*");

    // build object files from C source
    cc::Build::new()
        .include(".")
        .file("lz4.c")
        .file("fastlz.c")
        .file("fstapi.c")
        // .static_flag(true)
        .warnings(false) // nothing we can do, just noise
        .compile("fstapi");

    // this has to go after cc::Build? not sure what is happening
    // tried adding libz-sys as a dependency, but it resulted in linker errors
    println!("cargo:rustc-link-lib=z");

    // generate rust bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Couldn't generate fstapi bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write fstapi bindings");
}
