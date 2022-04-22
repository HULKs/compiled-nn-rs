extern crate bindgen;
extern crate pkg_config;

use std::{env, path::PathBuf, process::Command};

use cmake::Config;
use walkdir::WalkDir;

fn build_vendored_compiled_nn(out_path: &PathBuf) -> PathBuf {
    let source_path = out_path.join("CompiledNN/");
    let status = Command::new("rsync")
        .args(["-a", "CompiledNN/", source_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute rsync process");
    if !status.success() {
        panic!("rsync process exited with {:?}", status.code());
    }

    let install_path = Config::new(source_path)
        .define("BUILD_SHARED_LIBS", "OFF")
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
        .define("WITH_ONNX", "OFF")
        .build();

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
    let library64_path = install_path.join("lib64/");
    let include_path = install_path.join("include/");
    println!("cargo:rustc-link-search=native={}", library_path.display());
    println!(
        "cargo:rustc-link-search=native={}",
        library64_path.display()
    );
    println!("cargo:rustc-link-lib=static=CompiledNN");

    println!("cargo:rustc-link-lib=dylib=hdf5");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=c++");
        }
        _ => {
            panic!("We don't seem to be compiling on a known OS, aborting...")
        }
    }

    include_path
}

fn pkg_config_config(config: &pkg_config::Library) {
    for library_path in config.link_paths.iter() {
        println!("cargo:rustc-link-search=native={}", library_path.display());
    }

    for library in config.libs.iter() {
        println!("cargo:rustc-link-lib=dylib={}", library);
    }
}

fn generate_bindings(include_path: &[PathBuf], out_path: &PathBuf) {
    let clang_args = {
        let mut args = vec!["-x", "c++", "-std=c++11"];
        for buf in include_path {
            if let Some(s) = buf.to_str() {
                args.push("-I");
                args.push(s);
            }
        }
        args
    };

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(clang_args)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let pkg_config = pkg_config::Config::new()
        .atleast_version("1.0.0")
        .probe("compilednn");

    if let Ok(config) = pkg_config {
        let include_path = config.include_paths.clone();
        pkg_config_config(&config);
        generate_bindings(include_path.as_slice(), &out_path);
    } else {
        let include_path = vec![build_vendored_compiled_nn(&out_path)];
        generate_bindings(&include_path, &out_path);
    }

    println!("cargo:rerun-if-changed=wrapper.h");
}
