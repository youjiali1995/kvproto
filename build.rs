// Copyright 2019 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use protobuf_build::*;
use std::fs::{read_dir, remove_file, File};
use std::io::Write;

fn main() {
    // This build script creates files in the `src` directory. Since that is
    // outside Cargo's OUT_DIR it will cause an error when this crate is used
    // as a dependency. Therefore, the user must opt-in to regenerating the
    // Rust files.
    if !cfg!(feature = "gen") {
        println!("cargo:rerun-if-changed=build.rs");
        return;
    }

    check_protoc_version();

    let file_names: Vec<_> = read_dir("proto")
        .expect("Couldn't read proto directory")
        .map(|e| {
            format!(
                "proto/{}",
                e.expect("Couldn't list file").file_name().to_string_lossy()
            )
        })
        .collect();

    for f in &file_names {
        println!("cargo:rerun-if-changed={}", f);
    }

    // Generate Prost files.
    generate_prost_files(&file_names, "src/prost");
    remove_file("src/prost/gogoproto.rs").unwrap();
    remove_file("src/prost/google.protobuf.rs").unwrap();
    remove_file("src/prost/eraftpb.rs").unwrap();
    let mod_names = module_names_for_dir("src/prost");
    generate_wrappers(
        &mod_names
            .iter()
            .map(|m| format!("src/prost/{}.rs", m))
            .collect::<Vec<_>>(),
        "src/prost",
        GenOpt::All,
    );
    generate_prost_rs(&mod_names);
}

fn generate_prost_rs(mod_names: &[String]) {
    let mut text = "#![allow(dead_code)]\n\npub use raft::eraftpb;\n\n".to_owned();

    for mod_name in mod_names {
        text.push_str("pub mod ");
        text.push_str(mod_name);
        text.push_str("{\n");
        text.push_str("include!(\"prost/");
        text.push_str(mod_name);
        text.push_str(".rs\");");
        text.push_str("include!(\"prost/wrapper_");
        text.push_str(mod_name);
        text.push_str(".rs\");");
        text.push_str("}\n");
    }

    let mut lib = File::create("src/prost.rs").expect("Could not create prost.rs");
    lib.write_all(text.as_bytes())
        .expect("Could not write prost.rs");
}
