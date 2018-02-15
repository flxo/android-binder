use std::path::PathBuf;
use std::env::var;
use std::env;

fn main() {
    let path = var("PATH").expect("Failed to read PATH");
    let ndk_toolchain = var("NDK_TOOLCHAIN")
        .map(|n| PathBuf::from(n).join("bin"))
        .expect("Failed to read NDK_TOOLCHAIN");

    // Inject toolchain bin into PATH
    let mut paths = vec!(ndk_toolchain);
    paths.extend(env::split_paths(&path).collect::<Vec<_>>().into_iter());
    let path = PathBuf::from(env::join_paths(paths).expect("Failed to join pathes"));
    println!("cargo:rustc-env=PATH={}", path.display());
}
