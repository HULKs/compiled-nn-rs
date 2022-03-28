extern crate bindgen;

use std::{env, path::PathBuf, process::Command};

use walkdir::WalkDir;

fn run_cmake(build_path: &PathBuf, arguments: &[&str]) {
    let status = Command::new("cmake")
        .args(arguments)
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }
    let status = Command::new("cmake")
        .args(["--build", build_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }
    let status = Command::new("cmake")
        .args(["--install", build_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute cmake process");
    if !status.success() {
        panic!("cmake process exited with {:?}", status.code());
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let hdf5_source_path = out_path.join("hdf5/source/");
    let hdf5_build_path = out_path.join("hdf5/build/");
    let hdf5_install_path = out_path.join("hdf5/install/");
    let hdf5_cmake_install_prefix =
        format!("-DCMAKE_INSTALL_PREFIX={}", hdf5_install_path.display());
    let status = Command::new("rsync")
        .args(["-a", "--mkpath", "hdf5/", hdf5_source_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute rsync process");
    if !status.success() {
        panic!("rsync process exited with {:?}", status.code());
    }
    run_cmake(
        &hdf5_build_path,
        &[
            "-S",
            hdf5_source_path.to_str().unwrap(),
            "-B",
            hdf5_build_path.to_str().unwrap(),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DBUILD_TESTING=OFF",
            "-DHDF5_BUILD_TOOLS=OFF",
            "-DHDF5_BUILD_EXAMPLES=OFF",
            "-DHDF5_BUILD_UTILS=OFF",
            "-DHDF5_BUILD_HL_LIB=OFF",
            "-DHDF5_BUILD_CPP_LIB=OFF",
            hdf5_cmake_install_prefix.as_str(),
        ],
    );

    let compiled_nn_build_path = out_path.join("CompiledNN/build/");
    let compiled_nn_install_path = out_path.join("CompiledNN/install/");
    let compiled_nn_hdf5_root = format!("-DHDF5_ROOT={}", hdf5_install_path.display());
    let compiled_nn_cmake_install_prefix = format!(
        "-DCMAKE_INSTALL_PREFIX={}",
        compiled_nn_install_path.display()
    );
    run_cmake(
        &compiled_nn_build_path,
        &[
            "-S",
            "CompiledNN",
            "-B",
            compiled_nn_build_path.to_str().unwrap(),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DWITH_ONNX=OFF",
            compiled_nn_hdf5_root.as_str(),
            compiled_nn_cmake_install_prefix.as_str(),
        ],
    );

    println!("cargo:rerun-if-changed=wrapper.h");
    for entry in WalkDir::new("hdf5")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| match entry.metadata().ok() {
            Some(metadata) if metadata.is_file() => Some(entry),
            _ => None,
        })
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }
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

    let hdf5_library_path = hdf5_install_path.join("lib/");
    let compiled_nn_library_path = compiled_nn_install_path.join("lib/");
    let include_path = compiled_nn_install_path.join("include/");
    println!("cargo:rustc-link-search={}", hdf5_library_path.display());
    println!(
        "cargo:rustc-link-search={}",
        compiled_nn_library_path.display()
    );
    println!("cargo:rustc-link-lib=hdf5");
    println!("cargo:rustc-link-lib=CompiledNN");
    println!(
        "cargo:rustc-env=LD_LIBRARY_PATH={}",
        compiled_nn_library_path.display()
    );

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(vec!["-x", "c++", "-I", include_path.to_str().unwrap()])
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
