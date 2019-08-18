use std::path::PathBuf;
use std::{env, error::Error, io::Write as _};
use std::fs::{File, create_dir};

extern crate glob;
extern crate bindgen;
extern crate cc;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Collect sources
    let mut sources = vec![];
    for entry in glob::glob("src/libsodium/**/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => sources.push(path),
            Err(e) => println!("{:?}", e),
        }
    }

    // Create version file
    let _ = create_dir(out_path.join("sodium"));
    let version_path = out_path.join("sodium/version.h");
    match File::create(&version_path) {
        Err(e) => panic!("Error opening file {}: {}", version_path.display(), e.description()),
        Ok(mut f) => {
            f.write(format!("
                #ifndef sodium_version_H
                #define sodium_version_H

                #define SODIUM_VERSION_STRING        \"{}\"
                #define SODIUM_LIBRARY_VERSION_MAJOR {}
                #define SODIUM_LIBRARY_VERSION_MINOR {}

                #endif
            ", 
            env::var("CARGO_PKG_VERSION_MAJOR").unwrap(),
            env::var("CARGO_PKG_VERSION_MAJOR").unwrap(),
            env::var("CARGO_PKG_VERSION_MINOR").unwrap(),
            ).as_bytes()).unwrap();
        },
    }

    // Configure bindings
    let mut bindings = bindgen::Builder::default()
        .generate_comments(false)
        //.parse_callbacks(Box::new(ignored_macros))
        .use_core()
        .ctypes_prefix("cty")
        .clang_arg("-Isrc/libsodium/include")
        .clang_arg("-Isrc/libsodium/include/sodium")
        .clang_arg(format!("-I{}", &out_path.to_str().unwrap()))
        .clang_arg(format!("-I{}/sodium", &out_path.to_str().unwrap()))
        .header("src/libsodium/include/sodium.h");

    // Override sysroot if required
    if let Ok(s) = env::var("SYSROOT") {
        bindings = bindings.clang_arg(format!("--sysroot={}", s));
    } else if let Ok(t) = env::var("TARGET") {
        if t.starts_with("thumb") {
            bindings = bindings.clang_arg("--sysroot=/usr/lib/arm-none-eabi");
        } else if t.starts_with("arm") && t.ends_with("hf") {
            bindings = bindings.clang_arg("--sysroot=/usr/arm-linux-gnueabihf");
        }
    }

    // Generate bindings
    let bindings = bindings
        .generate()
        .expect("Unable to generate bindings");

    // Open a file for writing
    let binding_path = out_path.join("libsodium.rs");
    let file = match File::create(&binding_path) {
        Err(e) => panic!("Error opening file {}: {}", binding_path.display(), e.description()),
        Ok(f) => f,
    };

    // Write bindings
    bindings
        .write(Box::new(file))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .files(sources)
        .include("src/libsodium/include")
        .include("src/libsodium/include/sodium")
        .include(format!("{}", &out_path.to_str().unwrap()))
        .include(format!("{}/sodium", &out_path.to_str().unwrap()))
        .debug(true)
        .warnings(false)
        .extra_warnings(false)
        .flag("-std=c11")
        .flag("-DDEV_MODE=1")       // YOLO building from master
        .flag("-DCONFIGURED=1")     // YOLO CC is unsupported
        .flag("-Wno-everything")
        .compile("libsodium");

    println!("cargo:rustc-link-lib=static=libsodium");
}
