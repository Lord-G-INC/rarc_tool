fn main() {
    #[cfg(feature = "cxx")] {
        println!("cargo::rerun-if-changed=src/cpp_exports.rs");
        cxx_build::bridge("src/cpp_exports.rs")
        .compile("librust");
    }
}