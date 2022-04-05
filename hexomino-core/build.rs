use std::{env, fs, path::Path, process::Command};

fn main() {
    let output = Command::new("python3")
        .args(["gen_hexos/gen.py"])
        .output()
        .expect("failed to run python script to generate hexominos");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hexos.rs");
    fs::write(&dest_path, output.stdout).expect("failed to write to \"hexos.rs\"");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=gen_hexos/gen.py");
}
