extern crate spirv_builder;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for shader_name in ["shader-slime"] {
        let crate_path = [manifest_dir, "..", shader_name]
            .iter()
            .copied()
            .collect::<std::path::PathBuf>();

        SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
            .capability(Capability::StorageImageWriteWithoutFormat)
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    }
    Ok(())
}

// use std::env;
// use std::error::Error;
// use std::path::PathBuf;
//
// fn main() -> Result<(), Box<dyn Error>> {
//     let target_os = std::env::var("CARGO_CFG_TARGET_OS")?;
//     let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH")?;
//     println!("cargo:rerun-if-changed=build.rs");
//     println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
//     println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
//     // While OUT_DIR is set for both build.rs and compiling the crate, PROFILE is only set in
//     // build.rs. So, export it to crate compilation as well.
//     let profile = env::var("PROFILE").unwrap();
//     println!("cargo:rustc-env=PROFILE={profile}");
//     let mut dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
//     // Strip `$profile/build/*/out`.
//     let ok = dir.ends_with("out")
//         && dir.pop()
//         && dir.pop()
//         && dir.ends_with("build")
//         && dir.pop()
//         && dir.ends_with(profile)
//         && dir.pop();
//     assert!(ok);
//     // NOTE(eddyb) this needs to be distinct from the `--target-dir` value that
//     // `spirv-builder` generates in a similar way from `$OUT_DIR` and `$PROFILE`,
//     // otherwise repeated `cargo build`s will cause build script reruns and the
//     // rebuilding of `rustc_codegen_spirv` (likely due to common proc macro deps).
//     let dir = dir.join("builder");
//     let status = std::process::Command::new("cargo")
//         .args([
//             "run",
//             "--release",
//             "-p",
//             "builder",
//             "--target-dir",
//         ])
//         .arg(dir)
//         .env_remove("CARGO_ENCODED_RUSTFLAGS")
//         .stderr(std::process::Stdio::inherit())
//         .stdout(std::process::Stdio::inherit())
//         .status()?;
//     if !status.success() {
//         if let Some(code) = status.code() {
//             std::process::exit(code);
//         } else {
//             std::process::exit(1);
//         }
//     }
//     Ok(())
// }
