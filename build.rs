use rustc_version::{version_meta, Channel, Version};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(USE_RUSTC_FEATURES)");
    println!("cargo::rustc-check-cfg=cfg(CONFIG_RUSTC_HAS_UNSAFE_PINNED)");
    println!("cargo:rustc-check-cfg=cfg(RUSTC_RAW_REF_OP_IS_STABLE)");

    let meta = version_meta().unwrap();

    let use_feature = meta.channel == Channel::Nightly || std::env::var("RUSTC_BOOTSTRAP").is_ok();
    if use_feature {
        // Use this cfg option to control whether we should enable features that are already stable
        // in some new Rust versions, but are available as unstable features in older Rust versions
        // that needs to be supported by the Linux kernel.
        println!("cargo:rustc-cfg=USE_RUSTC_FEATURES");
    }

    if meta.semver >= Version::parse("1.89.0-nightly").unwrap() && use_feature {
        println!("cargo:rustc-cfg=CONFIG_RUSTC_HAS_UNSAFE_PINNED");
    }
    if meta.semver >= Version::parse("1.82.0").unwrap() && use_feature {
        println!("cargo:rustc-cfg=RUSTC_RAW_REF_OP_IS_STABLE");
    }
}
