// Embed rpath entries so the built binary can locate its dynamic dependencies
// at runtime without relying on LD_LIBRARY_PATH.
use std::process::Command;

fn main() {
    let output = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
        .args(["--print", "sysroot"])
        .output()
        .expect("failed to run `rustc --print sysroot`");

    let sysroot = String::from_utf8(output.stdout)
        .expect("rustc sysroot path was not valid UTF-8");
    let sysroot = sysroot.trim();

    // The active toolchain's lib dir, for librustc_driver and the other
    println!("cargo:rustc-link-arg=-Wl,-rpath,{sysroot}/lib");
    // $ORIGIN/deps, for dylib crate-dependencies like decls-gen
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
}
