fn main() {
    static_vcruntime::metabuild();

    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_MODE=server");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE=my_precompiled_passphrase");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_NO_ENVELOPE=true");
}