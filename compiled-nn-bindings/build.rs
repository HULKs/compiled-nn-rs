extern crate bindgen;

use std::{env, path::PathBuf, process::Command};

use walkdir::WalkDir;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = out_path.join("CompiledNN/build/");
    let install_path = out_path.join("CompiledNN/install/");
    let cmake_install_prefix = format!("-DCMAKE_INSTALL_PREFIX={}", install_path.display());
    let status = Command::new("cmake")
        .args([
            "-S",
            "CompiledNN",
            "-B",
            build_path.to_str().unwrap(),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_SHARED_LIBS=ON",
            cmake_install_prefix.as_str(),
            "-DWITH_ONNX=OFF",
        ])
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }
    let status = Command::new("cmake")
        .args([
            "--build",
            build_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }
    let status = Command::new("cmake")
        .args([
            "--install",
            build_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    for entry in WalkDir::new("CompiledNN")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| match entry.metadata().ok() {
            Some(metadata) if metadata.is_file() => Some(entry),
            _ => None,
        })
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    let library_path = install_path.join("lib/");
    let include_path = install_path.join("include/");
    println!("cargo:rustc-link-search={}", library_path.display());
    println!("cargo:rustc-link-lib=CompiledNN");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", library_path.display());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(vec![
            "-x",
            "c++",
            "-I",
            include_path.to_str().unwrap(),
        ])
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
