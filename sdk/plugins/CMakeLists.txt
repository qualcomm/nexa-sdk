# llama-cpp
if (GENIEX_PLUGIN_LLAMA_CPP)
    add_subdirectory(./llama_cpp)
endif()

# qnn
if (GENIEX_PLUGIN_QNN)
    add_subdirectory(./qnn)
endif()
