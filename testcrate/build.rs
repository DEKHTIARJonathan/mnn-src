fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if os != "windows" {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            if os == "macos" {
                "@loader_path"
            } else {
                "$ORIGIN"
            }
        );
    }
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .file("smoke.cpp")
        .include(std::env::var_os("DEP_MNN_INCLUDE").unwrap());
    if std::env::var("DEP_MNN_LINK_KIND").unwrap() == "shared" {
        build.define("USING_MNN_DLL", None);
    }
    build.compile("mnn_smoke");
    println!(
        "cargo:rustc-env=MNN_EXPECTED_VERSION={}",
        std::env::var("DEP_MNN_VERSION").unwrap()
    );
    println!("cargo:rerun-if-changed=smoke.cpp");
}
