fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // println!("cargo:rustc-link-lib=static=SDL2_mixer");
    println!("cargo:rustc-link-search=native=./lib/");
}