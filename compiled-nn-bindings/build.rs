extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=bz2");

    println!("cargo:rerun-if-changed=wrapper.h");

    println!("cargo:rustc-link-search=/home/mystery/hulks/compiled-nnng/compiled-nn-bindings/CompiledNN/install/lib/");
    println!("cargo:rustc-link-lib=CompiledNN");
    println!("cargo:rustc-env=LD_LIBRARY_PATH=/home/mystery/hulks/compiled-nnng/compiled-nn-bindings/CompiledNN/install/lib/");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(vec![
            "-x",
            "c++",
            "-I",
            "/home/mystery/hulks/compiled-nnng/compiled-nn-bindings/CompiledNN/install/include/",
        ])
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
