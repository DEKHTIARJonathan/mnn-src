use std::{env, fs, path::PathBuf};

fn feature(name: &str) -> bool {
    env::var_os(format!(
        "CARGO_FEATURE_{}",
        name.replace('-', "_").to_uppercase()
    ))
    .is_some()
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    assert_ne!(
        feature("static"),
        feature("shared"),
        "select exactly one of static/shared"
    );
    assert!(
        matches!(os.as_str(), "linux" | "macos" | "windows"),
        "unsupported target"
    );
    assert!(!feature("metal") || os == "macos", "Metal requires macOS");
    assert!(
        !feature("kleidiai") || (os == "macos" && arch == "aarch64"),
        "KleidiAI requires macOS ARM64"
    );
    assert!(
        !feature("cuda") || (os == "linux" && arch == "x86_64"),
        "CUDA requires Linux x86_64"
    );
    assert!(
        !(feature("opencl") || feature("vulkan")) || os != "macos",
        "OpenCL/Vulkan supported on Linux/Windows"
    );
    println!("cargo:rerun-if-changed=mnn-sources.tar.xz");
    if root.join("mnn").is_dir() {
        println!("cargo:rerun-if-changed=mnn");
    }
    let sources = out.join("sources");
    if sources.exists() {
        fs::remove_dir_all(&sources).unwrap();
    }
    let file = fs::File::open(root.join("mnn-sources.tar.xz"))
        .expect("run make setup to prepare pinned source archives");
    let decoder = lzma_rust2::XzReader::new(std::io::BufReader::new(file), false);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(&sources).expect("extract MNN sources");
    std::io::copy(&mut archive.into_inner(), &mut std::io::sink())
        .expect("validate complete XZ stream");
    let mut config = cmake::Config::new(sources.join("mnn"));
    config
        .profile("Release")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
    for key in [
        "MNN_BUILD_TOOLS",
        "MNN_BUILD_CONVERTER",
        "MNN_BUILD_DEMO",
        "MNN_BUILD_TEST",
        "MNN_BUILD_TRAIN",
        "MNN_SEP_BUILD",
        "MNN_SME2",
        "MNN_OPENMP",
    ] {
        config.define(key, "OFF");
    }
    for (flag, key) in [
        ("shared", "MNN_BUILD_SHARED_LIBS"),
        ("metal", "MNN_METAL"),
        ("opencl", "MNN_OPENCL"),
        ("vulkan", "MNN_VULKAN"),
        ("cuda", "MNN_CUDA"),
        ("kleidiai", "MNN_KLEIDIAI"),
        ("bf16", "MNN_SUPPORT_BF16"),
        ("low-memory", "MNN_LOW_MEMORY"),
        ("transformer-fuse", "MNN_SUPPORT_TRANSFORMER_FUSE"),
    ] {
        config.define(key, if feature(flag) { "ON" } else { "OFF" });
    }
    config.define(
        "MNN_WIN_RUNTIME_MT",
        if env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .split(',')
            .any(|v| v == "crt-static")
        {
            "ON"
        } else {
            "OFF"
        },
    );
    config.define("KLEIDIAI_SRC_DIR", sources.join("dependencies/kleidiai"));
    let cutlass = sources
        .join("dependencies/cutlass")
        .join(if feature("transformer-fuse") {
            "v4.0.0"
        } else {
            "v2.9.0"
        });
    if feature("cuda") {
        let expected = sources.join("mnn/3rd_party/cutlass");
        fs::create_dir_all(&expected).unwrap();
        fs::rename(&cutlass, expected.join(cutlass.file_name().unwrap())).unwrap();
        config.define(
            "FETCHCONTENT_SOURCE_DIR_CUTLASS",
            expected.join(cutlass.file_name().unwrap()),
        );
    }
    let prefix = config.build();
    let licenses = prefix.join("licenses");
    fs::create_dir_all(&licenses).unwrap();
    fs::copy(
        sources.join("mnn/LICENSE.txt"),
        licenses.join("MNN-LICENSE.txt"),
    )
    .unwrap();
    copy_notices(&sources, &licenses, &sources);
    if feature("cuda") {
        fs::copy(
            prefix.join("build/source/backend/cuda/libMNN_Cuda_Main.so"),
            prefix.join("lib/libMNN_Cuda_Main.so"),
        )
        .expect("install CUDA runtime");
        println!("cargo:rustc-link-lib=dylib=MNN_Cuda_Main");
    }
    for (key, path) in [
        ("root", prefix.clone()),
        ("include", prefix.join("include")),
        ("lib", prefix.join("lib")),
        (
            "runtime_dir",
            prefix.join(if os == "windows" { "bin" } else { "lib" }),
        ),
        ("license_dir", licenses),
    ] {
        println!("cargo:{key}={}", path.display());
    }
    println!(
        "cargo:version={}",
        env!("CARGO_PKG_VERSION").split('-').next().unwrap()
    );
    println!(
        "cargo:link_kind={}",
        if feature("static") {
            "static"
        } else {
            "shared"
        }
    );
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib").display()
    );
    println!(
        "cargo:rustc-link-lib={}MNN",
        if feature("static") {
            "static:+whole-archive="
        } else {
            "dylib="
        }
    );
    if os == "linux" {
        for lib in ["stdc++", "pthread", "dl", "m"] {
            println!("cargo:rustc-link-lib={lib}");
        }
    }
    if os == "macos" {
        println!("cargo:rustc-link-lib=c++");
        if feature("metal") {
            for lib in ["Foundation", "Metal", "CoreGraphics"] {
                println!("cargo:rustc-link-lib=framework={lib}");
            }
        }
    }
}

fn copy_notices(dir: &std::path::Path, dest: &std::path::Path, root: &std::path::Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            copy_notices(&path, dest, root);
        } else {
            let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
            if name.contains("license")
                || name.contains("notice")
                || path.components().any(|p| p.as_os_str() == "lic")
            {
                let target = dest.join(path.strip_prefix(root).unwrap());
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(path, target).unwrap();
            }
        }
    }
}
