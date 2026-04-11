use rustc_version::{version_meta, Channel};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(USE_RUSTC_FEATURES)");

    let meta = version_meta().unwrap();

    let use_feature = meta.channel == Channel::Nightly || option_env!("RUSTC_BOOTSTRAP").is_some();
    if use_feature {
        println!("cargo:rustc-cfg=USE_RUSTC_FEATURES");
    }
}
