if(GENIEX_PLUGIN_LLAMA_CPP)
    add_subdirectory(./llama_cpp)
endif()

if(GENIEX_PLUGIN_QAIRT)
    add_subdirectory(./qairt)
endif()

if(GENIEX_OPENMP_RUNTIME_DLL)
    # Keep OpenMP runtime next to ggml-cpu.dll so backend loading can resolve
    # transitive dependencies in runtime-only packaging.
    install(FILES "${GENIEX_OPENMP_RUNTIME_DLL}" DESTINATION lib/llama_cpp)

    if(WIN32 AND CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64")
        get_filename_component(_geniex_openmp_runtime_name "${GENIEX_OPENMP_RUNTIME_DLL}" NAME)
        if(NOT _geniex_openmp_runtime_name STREQUAL "libomp140.aarch64.dll")
            install(FILES "${GENIEX_OPENMP_RUNTIME_DLL}" DESTINATION lib/llama_cpp RENAME "libomp140.aarch64.dll")
        endif()
    endif()
endif()