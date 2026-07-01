use std::path::PathBuf;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    if target_os == "android" {
        build_android(&manifest_dir, &out_dir);
    } else {
        build_host(&out_dir);
    }
}

fn build_host(out_dir: &PathBuf) {
    let lib = pkg_config::probe_library("hamlib")
        .expect("hamlib not found — install libhamlib-dev (Debian/Ubuntu) or hamlib-devel (Fedora)");

    let header = lib
        .include_paths
        .iter()
        .map(|p| p.join("hamlib/rig.h"))
        .find(|p| p.exists())
        .expect("hamlib/rig.h not found in pkg-config include paths");

    let mut builder = bindgen::Builder::default().header(header.to_string_lossy());
    for inc in &lib.include_paths {
        builder = builder.clang_arg(format!("-I{}", inc.display()));
    }

    let bindings = builder
        .allowlist_file(".*hamlib.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed to generate hamlib bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings");
}

fn build_android(manifest_dir: &PathBuf, out_dir: &PathBuf) {
    // hamlib source lives two levels up from hamlib-sys/
    let hamlib_src = manifest_dir.parent().unwrap().join("hamlib");
    let header = hamlib_src.join("include/hamlib/rig.h");

    assert!(
        header.exists(),
        "hamlib submodule not initialized — run: git submodule update --init lib/hamlib"
    );

    // Map Rust target arch to Android ABI directory
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let abi = arch_to_abi(&target_arch);

    let prebuilt_lib = manifest_dir
        .parent()
        .unwrap()
        .join("hamlib-prebuilt/android")
        .join(abi)
        .join("lib");

    assert!(
        prebuilt_lib.join("libhamlib.a").exists(),
        "hamlib not pre-built for Android ABI {abi}.\n\
         Run: lib/scripts/build-hamlib-android.sh"
    );

    println!("cargo:rustc-link-search=native={}", prebuilt_lib.display());
    println!("cargo:rustc-link-lib=static=hamlib");

    // NDK sysroot for cross-targeted bindgen (so struct layouts match the target ABI)
    let ndk_home = std::env::var("ANDROID_NDK_HOME")
        .or_else(|_| std::env::var("ANDROID_NDK_ROOT"))
        .expect("ANDROID_NDK_HOME must be set when building for Android");
    let sysroot = format!(
        "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
        ndk_home
    );

    // Full Rust target triple (e.g., aarch64-linux-android); append API level for clang
    let rust_target = std::env::var("TARGET").unwrap();
    let clang_target = android_clang_target(&rust_target);

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", hamlib_src.join("include").display()))
        .clang_arg(format!("-I{}", hamlib_src.display()))
        .clang_arg(format!("--target={}", clang_target))
        .clang_arg(format!("--sysroot={}", sysroot))
        .allowlist_file(".*hamlib.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed to generate hamlib bindings for Android");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings");
}

fn arch_to_abi(arch: &str) -> &'static str {
    match arch {
        "aarch64" => "arm64-v8a",
        "arm" => "armeabi-v7a",
        "x86_64" => "x86_64",
        "x86" => "x86",
        _ => panic!("unsupported Android arch: {arch}"),
    }
}

fn android_clang_target(rust_target: &str) -> String {
    // armv7-linux-androideabi → armv7a-linux-androideabi28 (NDK uses armv7a)
    let base = rust_target.replace("armv7-", "armv7a-");
    format!("{}28", base)
}
