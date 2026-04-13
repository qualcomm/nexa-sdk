file(GLOB_RECURSE LLAMA_CPP_SRCS *.cpp)

if (GENIEX_SHARED)
    add_library(geniex_llama_cpp SHARED ${LLAMA_CPP_SRCS})
else()
    add_library(geniex_llama_cpp STATIC ${LLAMA_CPP_SRCS})
endif()

target_compile_definitions(geniex_llama_cpp PUBLIC ML_SHARED)

target_include_directories(geniex_llama_cpp
    PUBLIC
        ${COMMON_INCLUDE}
    PRIVATE
        ../include/)

# Always link the validation interface target; it is empty unless GENIEX_VALIDATION is ON
target_link_libraries(geniex_llama_cpp
    PUBLIC llama common
    PRIVATE geniex_bridge mtmd geniex_validation)

set_target_properties(geniex_llama_cpp PROPERTIES OUTPUT_NAME "geniex_plugin")

# Set proper RPATH for the plugin to find centralized libraries
if(UNIX)
    set_target_properties(geniex_llama_cpp PROPERTIES
        INSTALL_RPATH "$ORIGIN:$ORIGIN/../${COMMON_LIB_REL_PATH}"
        BUILD_WITH_INSTALL_RPATH TRUE
    )
endif()
