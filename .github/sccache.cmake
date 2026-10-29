# CI compiler caching only; the packaged crate does not require sccache.
find_program(MNN_SCCACHE sccache REQUIRED)
# All native CI builds use Ninja. CMake's per-language compiler launchers do
# not support ASM, so use its compile-rule launcher to cover assembly as well
# as C/C++ and Metal's Objective-C++. Linking is deliberately not wrapped.
set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE "\"${MNN_SCCACHE}\"")
if(CMAKE_HOST_SYSTEM_NAME STREQUAL "Windows")
    # Upstream hardcodes /Zi after project(); convert it before sccache parses
    # the command, preserving debug info without a shared compiler PDB.
    find_program(MNN_PYTHON python REQUIRED)
    set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE
        "\"${MNN_PYTHON}\" \"${CMAKE_CURRENT_LIST_DIR}/sccache-msvc.py\" \"${MNN_SCCACHE}\"")
endif()

# MNN uses FindCUDA's cuda_add_library, which bypasses the CUDA launcher.
# Keep nvcc in its SDK and give FindCUDA an executable wrapper under OUT_DIR.
find_program(MNN_NVCC nvcc)
if(MNN_NVCC AND CMAKE_HOST_SYSTEM_NAME STREQUAL "Linux")
    get_filename_component(cuda_bin "${MNN_NVCC}" DIRECTORY)
    get_filename_component(CUDA_TOOLKIT_ROOT_DIR "${cuda_bin}" DIRECTORY)
    set(CUDA_TOOLKIT_ROOT_DIR "${CUDA_TOOLKIT_ROOT_DIR}" CACHE PATH "CUDA SDK")
    set(nvcc_wrapper "${CMAKE_BINARY_DIR}/sccache-nvcc")
    file(WRITE "${nvcc_wrapper}"
        "#!/bin/sh\nexec \"${MNN_SCCACHE}\" \"${MNN_NVCC}\" \"$@\"\n")
    file(CHMOD "${nvcc_wrapper}" PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE
        GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE)
    # A normal variable survives FindCUDA clearing its cache on SDK discovery.
    set(CUDA_NVCC_EXECUTABLE "${nvcc_wrapper}")
endif()
