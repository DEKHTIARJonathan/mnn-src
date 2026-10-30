# mnn-src

Pristine Alibaba MNN sources built with upstream CMake. Cargo.toml owns the
package version; prerelease suffixes identify packaging candidates.

Requires Rust, CMake and a C/C++ compiler. Maintainer preparation additionally
requires Python 3.11 and network access for pinned KleidiAI/CUTLASS archives.
Consumer builds extract bundled sources under OUT_DIR without downloading them.

`static` is the default CPU configuration. For shared MNN use
`default-features = false, features = ["shared"]`. Exactly one linkage mode is
required. Optional features map to upstream options: `metal`, `opencl`,
`vulkan`, `cuda`, `kleidiai`, `bf16`, `low-memory`, and `transformer-fuse`.

Initial target scope is Linux x86_64, macOS ARM64, and Windows x64/MSVC. Metal
and KleidiAI require macOS ARM64; OpenCL/Vulkan target Linux and Windows. CUDA
targets Linux x86_64 with a compatible CUDA 12.x toolkit. Backend compilation
does not establish GPU numerical qualification. Other SDK backends, converters,
training, and application toolchains are outside this crate's scope.

CUDA builds select C++17 for host and device compilation: MNN 3.5's non-fused
CUDA kernels use newer language features than its default C++11 setting allows.
CUDA configurations using cuBLAS also require its compatible runtime library.

Use a direct normal dependency and `use mnn_src as _;`. The dependent build
script receives DEP_MNN_ROOT, INCLUDE, LIB, RUNTIME_DIR, LICENSE_DIR, VERSION
and LINK_KIND metadata. Compile your C++ adapter against INCLUDE; native linking
is owned by this crate. Shared Windows adapters define USING_MNN_DLL. Keep
compiler and C++ runtime choices consistent; FrameForge uses /MD. Respect the
application's MACOSX_DEPLOYMENT_TARGET and SDKROOT.

Static CPU requires no MNN runtime library. Shared builds require the installed
shared library; Linux CUDA also requires libMNN_Cuda_Main.so even with static
MNN. OpenCL/Vulkan need system drivers. Metal links Foundation, Metal and
CoreGraphics. Applications own runtime search paths, redistribution, signing,
MSVC CRT staging, and notices from LICENSE_DIR.

## Local verification

Run `make setup`, `prek run -a`, formatting checks, then `make package`.
Packaging rejects archives of 10,000,000 bytes or larger. Only after this gate,
run `make build` and `make test`. Test a fresh extraction through the
independent consumer as well; checkout tests do not establish archive
completeness.

## FrameForge migration

FrameForge requires MNN 3.6.1; select the matching crate version. Add the source
crate as an optional normal dependency of insta360-rs's AI features and consume
DEP_MNN_INCLUDE instead of MNN_ROOT and private cache markers. Preserve its
adapter, models, exact runtime version checks, CPU high precision, and thread
budgets. Remove MNN acquisition/prewarming only after numerical and packaged
application checks pass. Keep Windows CRT and license staging: the GStreamer
helper also consumes those CRT files.

The process-global MNN thread pool retains its first capacity. Run thread-budget
qualification in fresh processes with the largest budget initialized first.

## Release procedure

Use immutable packaging candidates X.Y.Z-rcN with the native pin unchanged.
Checks, packaging and packaged-consumer qualification precede authorized tag
publication. Transfer the exact crate between CI jobs. Reconstruct it, restore
Cargo.toml.orig, repackage with the lockfile and compare bytes before trusted
publication. Download the exact registry version afterward and compare bytes. A
stable version requires a new archive and all checks again. Never move a
published tag. Pushes and publications require explicit authorization.
