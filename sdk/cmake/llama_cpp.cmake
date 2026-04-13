# llama_cpp.cmake - Centralized llama.cpp build configuration
#
# This module handles the building of third-party/llama.cpp and provides
# stable alias targets for plugins to consume.

# Guard against multiple inclusions
if(TARGET llama)
    return()
endif()

# Configure GGML options
set(GGML_BLAS OFF)
set(GGML_NATIVE OFF)

if (GENIEX_DL)
    if (NOT GENIEX_SHARED)
        message(FATAL_ERROR "Disabling GENIEX_SHARED is not supported with GENIEX_DL")
    endif()
    set(GGML_BACKEND_DL ON CACHE BOOL "Enable ggml dynamic loading")
else()
    set(GGML_BACKEND_DL OFF CACHE BOOL "Disable ggml dynamic loading")
endif()

# Configure llama.cpp options
set(LLAMA_BUILD_COMMON ON)
set(LLAMA_CURL OFF)
set(LLAMA_BUILD_TESTS OFF)
set(LLAMA_BUILD_TOOLS ON)
set(LLAMA_BUILD_EXAMPLES OFF)
set(LLAMA_BUILD_SERVER OFF)
set(BUILD_SHARED_LIBS ${GENIEX_SHARED})

# Custom llama.cpp build marker
set(GENIEX_LLAMA_CPP ON)

# Add llama.cpp as subdirectory
include(${CMAKE_SOURCE_DIR}/third-party/opencl/CMakeLists.txt)
add_subdirectory(${CMAKE_SOURCE_DIR}/third-party/llama.cpp ${CMAKE_BINARY_DIR}/third-party/llama.cpp)

message(STATUS "===== Llama.cpp Configuration =====")
message(STATUS "GGML_BACKEND_DL:    ${GGML_BACKEND_DL}")
message(STATUS "GGML_OPENCL:        ${GGML_OPENCL}")
message(STATUS "BUILD_SHARED_LIBS:  ${BUILD_SHARED_LIBS}")
message(STATUS "===================================")

# Export list of llama.cpp libraries for centralized copying
set(LLAMA_LIBS
    mtmd llama ggml ggml-base ggml-cpu
    ggml-opencl ggml-hexagon)
if(NOT GENIEX_SHARED)
    list(APPEND LLAMA_LIBS common)
endif()

# qualcomm htp files
foreach(htp_ver v73 v75 v79 v81)
    if(TARGET htp-${htp_ver})
        ExternalProject_Get_Property(htp-${htp_ver} BINARY_DIR)
        message(STATUS "htp-${htp_ver} binary build dir: ${BINARY_DIR}")
        add_custom_target(copy_output_out_htp_${htp_ver}
            COMMAND ${CMAKE_COMMAND} -E copy
                ${BINARY_DIR}/libggml-htp-${htp_ver}.so
                ${CMAKE_BINARY_DIR}/out/common/lib/libggml-htp-${htp_ver}.so
            DEPENDS htp-${htp_ver}
        )
        add_dependencies(copy_llama_libs copy_output_out_htp_${htp_ver})
    endif()
endforeach()

message(STATUS "Llama.cpp libraries will be copied to build/${COMMON_LIB_OUTPUT_DIR} at build time")
