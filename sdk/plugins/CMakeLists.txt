if(GENIEX_PLUGIN_LLAMA_CPP)
    add_subdirectory(./llama_cpp)
endif()

if(GENIEX_PLUGIN_QAIRT)
    add_subdirectory(./qairt)
endif()

if(GENIEX_OPENMP_RUNTIME_DLL)
    install(FILES "${GENIEX_OPENMP_RUNTIME_DLL}" DESTINATION lib)
endif()