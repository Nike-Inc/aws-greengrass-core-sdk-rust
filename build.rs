/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(feature = "coverage") {
        return ();
    };

    let mut builder = bindgen::Builder::default().header("wrapper.h");

    if env::var("AWS_GREENGRASS_STUBS").is_ok() {
        let dst = cmake::build("stubs");
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        builder = builder.clang_arg(format!("-I{}/include", dst.display()));
    }

    println!("cargo:rustc-link-lib=aws-greengrass-core-sdk-c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate c bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings");
}
