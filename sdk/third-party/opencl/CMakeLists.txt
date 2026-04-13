# add opencl lib for windows arm64
if (CMAKE_SYSTEM_NAME STREQUAL "Windows" AND CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64")
    set(OpenCL_INCLUDE_DIR "${CMAKE_CURRENT_LIST_DIR}/windows-arm64/include")
    set(OpenCL_LIBRARY "${CMAKE_CURRENT_LIST_DIR}/windows-arm64/lib/OpenCL.lib")
endif()
# add opencl lib for linux arm64
if (CMAKE_SYSTEM_NAME STREQUAL "Linux" AND CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(OpenCL_INCLUDE_DIR "${CMAKE_CURRENT_LIST_DIR}/linux-arm64/include")
    set(OpenCL_LIBRARY "${CMAKE_CURRENT_LIST_DIR}/linux-arm64/lib/OpenCL.so")
endif()
# add opencl lib for android aarch64
if (CMAKE_SYSTEM_NAME STREQUAL "Android" AND CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(OpenCL_INCLUDE_DIR "${CMAKE_CURRENT_LIST_DIR}/android-arm64/include")
    set(OpenCL_LIBRARY "${CMAKE_CURRENT_LIST_DIR}/android-arm64/lib/libOpenCL.so")
endif()
