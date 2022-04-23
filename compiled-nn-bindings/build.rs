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

    println!("cargo:rustc-link-lib=hdf5");
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
        println!("cargo:rustc-link-lib={}", library);
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
