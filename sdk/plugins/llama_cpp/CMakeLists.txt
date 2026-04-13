message(STATUS "Configuring geniex_llama_cpp plugin")

add_subdirectory(./src)

# Ensure llama.cpp libs are copied to centralized location before building this plugin
add_dependencies(geniex_llama_cpp copy_llama_libs)

copy_output(out/${GENIEX_PLUGIN_ID_LLAMA_CPP}
    geniex_llama_cpp
)
