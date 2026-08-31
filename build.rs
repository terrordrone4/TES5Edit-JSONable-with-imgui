use std::{env, fs, path::PathBuf};

fn main() {
    const GUIDE: &str = "esp-structure-explaination.md";
    println!("cargo:rerun-if-changed={GUIDE}");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR must be below target/<profile>/build/<package>/out");
    fs::copy(GUIDE, profile_dir.join(GUIDE)).expect("copying the ESP structure guide");
}
