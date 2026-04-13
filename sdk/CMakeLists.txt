cmake_minimum_required(VERSION 3.16)
# C compiler is required by native dependencies.
project(geniex-bridge C CXX)

set(CMAKE_BUILD_TYPE Release)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED true)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED true)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Set RPATH configuration for all targets to use relative paths
set(CMAKE_BUILD_RPATH "$ORIGIN")
set(CMAKE_INSTALL_RPATH "$ORIGIN")

# Use install RPATH during build to avoid hardcoded build paths
set(CMAKE_BUILD_WITH_INSTALL_RPATH ON)

# Set UTF-8 encoding for MSVC
add_compile_options("$<$<AND:$<C_COMPILER_ID:MSVC>,$<NOT:$<COMPILE_LANGUAGE:CUDA>>>:/utf-8>")
add_compile_options("$<$<AND:$<CXX_COMPILER_ID:MSVC>,$<NOT:$<COMPILE_LANGUAGE:CUDA>>>:/utf-8>")
# Suppress warnings for NVCC
# Windows-specific compile definitions to prevent common header conflicts
# These definitions address the following issues:
# - WIN32_LEAN_AND_MEAN: Reduces Windows header bloat and speeds up compilation
# - NOMINMAX: Prevents min/max macro conflicts with std::min/std::max
# - _WINSOCKAPI_: Prevents winsock.h inclusion to avoid conflicts with winsock2.h
#   The main conflict is likely between winsock.h and httplib.h (which uses winsock2.h)
#   httplib.h is used by the validation library, which is used in production mode
#   This fixes compilation errors like "redefinition; different linkage" for
#   networking functions (inet_ntoa, listen, ntohl, ntohs) that occur when
#   both winsock.h and winsock2.h are included by different dependencies
if(WIN32)
    set(WINDOWS_COMPILE_DEFS WIN32_LEAN_AND_MEAN NOMINMAX)
    add_compile_definitions(${WINDOWS_COMPILE_DEFS})
    message(STATUS "Applied Windows compatibility definitions: ${WINDOWS_COMPILE_DEFS}")
endif()

# Convert CMAKE_TOOLCHAIN_FILE to absolute path if Set
if(DEFINED CMAKE_TOOLCHAIN_FILE AND NOT IS_ABSOLUTE "${CMAKE_TOOLCHAIN_FILE}")
    get_filename_component(ABSOLUTE_TOOLCHAIN_PATH ${CMAKE_TOOLCHAIN_FILE} ABSOLUTE)
    set(CMAKE_TOOLCHAIN_FILE ${ABSOLUTE_TOOLCHAIN_PATH})
endif()

set(GENIEX_BRIDGE_VERSION "v0.0.0" CACHE STRING "version of geniex_bridge library")

# ===============================================================================
# Build Options
# ===============================================================================

# Plugin Backend Options
option(GENIEX_PLUGIN_LLAMA_CPP            "Enable llama.cpp backend"                      ON)
option(GENIEX_PLUGIN_QNN                  "Enable qnn backend"                            OFF)

# Language Binding Options
option(GENIEX_BINDING_PYTHON              "Enable python binding"                         OFF)
# Build Control Options
option(GENIEX_SHARED                      "Build shared lib"                              ON)
option(GENIEX_DL                          "Enable dynamic plugin loading"                 ON)
option(GENIEX_DEBUG                       "Enable debug mode"                             ON)
option(GENIEX_TEST                        "Enable unit test"                              ON)
# Feature Options
option(GENIEX_PRODUCTION                  "Enable production mode"                        OFF)
option(GENIEX_VALIDATION                  "Enable validation"                             OFF)

# Print all backend options status
message(STATUS "=============== Options ===============")
message(STATUS "GENIEX_BRIDGE_VERSION:              ${GENIEX_BRIDGE_VERSION}")
message(STATUS "=======================================")
message(STATUS "GENIEX_PLUGIN_LLAMA_CPP:            ${GENIEX_PLUGIN_LLAMA_CPP}")
message(STATUS "GENIEX_PLUGIN_QNN:                  ${GENIEX_PLUGIN_QNN}")
message(STATUS "=======================================")
message(STATUS "GENIEX_BINDING_PYTHON:              ${GENIEX_BINDING_PYTHON}")
message(STATUS "=======================================")
message(STATUS "GENIEX_SHARED:                      ${GENIEX_SHARED}")
message(STATUS "GENIEX_DL:                          ${GENIEX_DL}")
message(STATUS "GENIEX_DEBUG:                       ${GENIEX_DEBUG}")
message(STATUS "GENIEX_TEST:                        ${GENIEX_TEST}")
message(STATUS "GENIEX_PRODUCTION:                  ${GENIEX_PRODUCTION}")
message(STATUS "GENIEX_VALIDATION:                  ${GENIEX_VALIDATION}")
message(STATUS "=======================================")

# Detect platform-specific features
include(./cmake/platforms.cmake)

include(./cmake/util.cmake)

enable_testing()

set(COMMON_INCLUDE ${CMAKE_CURRENT_SOURCE_DIR}/include)

# Find common libraries (will be copied after plugins are configured)
find_common_libs(GENIEX_COMMON_LIB_FILES)

# ==== plugin id map start ====
set(GENIEX_PLUGIN_ID_LLAMA_CPP          "cpu_gpu")
add_compile_definitions(GENIEX_PLUGIN_ID_LLAMA_CPP="${GENIEX_PLUGIN_ID_LLAMA_CPP}")

set(GENIEX_PLUGIN_ID_QNN                "npu")
add_compile_definitions(GENIEX_PLUGIN_ID_QNN="${GENIEX_PLUGIN_ID_QNN}")
# ==== plugin id map end ====

# Create placeholder targets that plugins can depend on
# These will be populated with actual dependencies after plugins are configured
add_custom_target(copy_common_libs)
add_custom_target(copy_llama_libs)

# Include llama_cpp.cmake to set up LLAMA_LIBS variable
include(./cmake/llama_cpp.cmake)


if (GENIEX_DL)
    add_compile_definitions(GENIEX_DL)
endif()

if (GENIEX_DEBUG)
    add_compile_definitions(GENIEX_DEBUG)
    add_compile_definitions(PROJECT_SOURCE_DIR="${CMAKE_SOURCE_DIR}")
    set(CMAKE_BUILD_TYPE RelWithDebInfo)
endif()

if (GENIEX_VALIDATION)
    add_compile_definitions(GENIEX_VALIDATION)
endif()

# Handle production mode
if (GENIEX_PRODUCTION)
    add_compile_definitions(GENIEX_PRODUCTION)
endif()

# Handle static vs shared build
if (NOT GENIEX_SHARED)
    add_compile_definitions(GENIEX_STATIC)
endif()

# Validation interface target
# Always create the interface target so other targets can link it unconditionally.
add_library(geniex_validation INTERFACE)
target_include_directories(geniex_validation INTERFACE ${COMMON_INCLUDE})
# Only add OpenSSL and related definitions when validation is enabled
if (GENIEX_VALIDATION)
    include(./cmake/openssl.cmake)
endif()

add_subdirectory(./src)
add_subdirectory(./plugins)

# After plugins are configured, set up actual copying for centralized libraries
set(CENTRALIZED_LIBS ${GENIEX_COMMON_LIB_FILES} ${LLAMA_LIBS})
if(CENTRALIZED_LIBS)
    copy_output(${COMMON_LIB_OUTPUT_DIR} ${CENTRALIZED_LIBS})
    # Add dependencies to the placeholder targets created earlier
    add_dependencies(copy_common_libs copy_output_out_common_lib)
    add_dependencies(copy_llama_libs copy_output_out_common_lib)
    message(STATUS "Centralized libraries will be copied to build/${COMMON_LIB_OUTPUT_DIR} at build time")

    # Create major version symlinks for versioned libraries on Unix-like systems
    # This ensures that plugins can find the SOVERSION symlinks they need at runtime
    # Examples:
    #   Linux: libmtmd.so.0 -> libmtmd.so.0.0.7294
    if(UNIX)
        add_custom_command(TARGET copy_output_out_common_lib POST_BUILD
            COMMAND ${CMAKE_COMMAND}
                -DLIB_DIR=${CMAKE_BINARY_DIR}/${COMMON_LIB_OUTPUT_DIR}
                -P ${CMAKE_SOURCE_DIR}/cmake/create_dylib_symlinks.cmake
            COMMENT "Creating major version symlinks for shared libraries"
        )
    endif()
else()
    message(WARNING "No centralized libraries found, please check libs/")
endif()

add_subdirectory(./bindings)
add_subdirectory(./tests)
