use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=data/");
    let profile = env::var_os("PROFILE").unwrap();
    if profile.eq("release") {
        let test = Command::new("meurglys3").output();
        if let Err(err) = test {
            println!("cargo::warning=Could not execute meurglys3, make sure it is available on the PATH.");
            println!("Otherwise the built binary will depend on the game data being present in the `data` directory and not a meurglys3 package.");
            println!("cargo::warning=Error was: `{err}`");
            println!("cargo::rustc-env=PH_NO_MEU3=1");
        } else {
            let out_dir = env::var_os("OUT_DIR").unwrap();
            let dest_path = PathBuf::from(out_dir).join("../../data");
            Command::new("meurglys3")
                .arg("pack")
                .arg("data/")
                .arg(dest_path)
                .output()
                .expect("failed to execute meurglys3");
        }
    }
    println!("cargo::rerun-if-changed=build.rs");
}
