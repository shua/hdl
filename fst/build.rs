fn main() {
    println!("cargo:rerun-if-changed=fstapi/*");

    // build object files from C source
    cc::Build::new()
        .include("./fstapi")
        .file("fstapi/lz4.c")
        .file("fstapi/fastlz.c")
        .file("fstapi/fstapi.c")
        .static_flag(true)
        .warnings(false) // nothing we can do, just noise
        .compile("fstapi");

    // this has to go after cc::Build? not sure what is happening
    println!("cargo:rustc-link-lib=static=z");

    // generate rust bindings
    let bindings = bindgen::Builder::default()
        .header("fstapi/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Couldn't generate fstapi bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write fstapi bindings");
}
