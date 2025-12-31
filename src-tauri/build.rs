fn main() {
    println!("cargo:rerun-if-changed=miniaudio.c");
    println!("cargo:rerun-if-changed=miniaudio.h");

    cc::Build::new().file("miniaudio.c").compile("miniaudio");

    let bindings = bindgen::Builder::default()
        .header("miniaudio.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("unable to generate bindings to miniaudio");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("couldn't write miniaudio bindings");

    tauri_build::build();
}
