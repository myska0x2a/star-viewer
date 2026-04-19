// build.rs

// use std::env;
// use std::io::{self, Write};
// use std::path::Path;
// use std::process::{Command, Stdio};

fn main() {
    // let out_dir = env::var("OUT_DIR").unwrap();

    // let glsl_output = Command::new("glslc").args(&["shaders/stars/stars.vert", "-o", "shaders/stars/stars.vert.spv"])
    //                     .current_dir(&Path::new(&out_dir))
    //                     .stdout(Stdio::piped())
    //                     .output();

    // let output = Command::new("ls")
    //     .stdout(Stdio::piped())
    //     .output()
    //     .expect("Failed to execute command");

    // let msg = String::from_utf8(glsl_output.unwrap().stdout).unwrap();
    // let msg2 = String::from_utf8_lossy(&output.stderr);

    // println!("cargo::error=\"{}\"", msg2);
    // print!("");
}
