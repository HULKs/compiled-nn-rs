extern crate bindgen;

use std::{env, path::PathBuf, process::Command};

use cmake::Config;
use glob::glob;
use walkdir::WalkDir;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let hdf5_source_path = out_path.join("hdf5/");
    let status = Command::new("rsync")
        .args(["-a", "hdf5/", hdf5_source_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute rsync process");
    if !status.success() {
        panic!("rsync process exited with {:?}", status.code());
    }

    let hdf5_install_path = Config::new(hdf5_source_path)
        .define("BUILD_TESTING", "OFF")
        .define("HDF5_BUILD_TOOLS", "OFF")
        .define("HDF5_BUILD_EXAMPLES", "OFF")
        .define("HDF5_BUILD_UTILS", "OFF")
        .define("HDF5_BUILD_HL_LIB", "OFF")
        .define("HDF5_BUILD_CPP_LIB", "OFF")
        .define("TEST_LFS_WORKS_RUN", "0")
        .define("TEST_LFS_WORKS_RUN__TRYRUN_OUTPUT", "0")
        .define("RUN_RESULT_VAR", "0")
        .define("RUN_RESULT_VAR__TRYRUN_OUTPUT", "")
        .define("H5_PRINTF_LL_TEST_RUN", "1")
        .define("H5_PRINTF_LL_TEST_RUN__TRYRUN_OUTPUT", "8")
        .define("H5_LDOUBLE_TO_LONG_SPECIAL_RUN", "0")
        .define("H5_LDOUBLE_TO_LONG_SPECIAL_RUN__TRYRUN_OUTPUT", "")
        .define("H5_LONG_TO_LDOUBLE_SPECIAL_RUN", "0")
        .define("H5_LONG_TO_LDOUBLE_SPECIAL_RUN__TRYRUN_OUTPUT", "")
        .define("H5_LDOUBLE_TO_LLONG_ACCURATE_RUN", "0")
        .define("H5_LDOUBLE_TO_LLONG_ACCURATE_RUN__TRYRUN_OUTPUT", "")
        .define("H5_LLONG_TO_LDOUBLE_CORRECT_RUN", "0")
        .define("H5_LLONG_TO_LDOUBLE_CORRECT_RUN__TRYRUN_OUTPUT", "")
        .define("H5_DISABLE_SOME_LDOUBLE_CONV_RUN", "0")
        .define("H5_DISABLE_SOME_LDOUBLE_CONV_RUN__TRYRUN_OUTPUT", "")
        .define("H5_NO_ALIGNMENT_RESTRICTIONS_RUN", "0")
        .define("H5_NO_ALIGNMENT_RESTRICTIONS_RUN__TRYRUN_OUTPUT", "")
        .build();
    let hdf5_include_directory_path = hdf5_install_path.join("include");

    let compiled_nn_install_path = Config::new("CompiledNN")
        .define("BUILD_TESTING", "OFF")
        .define("WITH_ONNX", "OFF")
        .define("HDF5_ROOT", &hdf5_install_path)
        .cxxflag(format!("-I{}", hdf5_include_directory_path.display()))
        .build();

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
    println!(
        "cargo:rustc-link-search=native={}",
        hdf5_library_path.display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        compiled_nn_library_path.display()
    );
    if glob(hdf5_library_path.join("*hdf5*debug*").to_str().unwrap())
        .expect("Failed to glob for hdf5 debug library")
        .next()
        .is_some()
    {
        println!("cargo:rustc-link-lib=static=hdf5_debug");
    } else {
        println!("cargo:rustc-link-lib=static=hdf5");
    }
    println!("cargo:rustc-link-lib=static=CompiledNN");
    println!("cargo:rustc-link-lib=dylib=stdc++");

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
