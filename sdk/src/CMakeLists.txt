file(GLOB BRIDGE_SRC *.cpp)

if (GENIEX_SHARED)
    add_library(geniex_bridge SHARED ${BRIDGE_SRC})
else()
    add_library(geniex_bridge STATIC ${BRIDGE_SRC})
endif()

target_compile_definitions(geniex_bridge PRIVATE BRIDGE_VERSION="${GENIEX_BRIDGE_VERSION}")
target_compile_definitions(geniex_bridge PRIVATE ML_SHARED ML_BUILD)

target_include_directories(geniex_bridge PUBLIC ${COMMON_INCLUDE})

# Always link the validation interface target; it is empty unless GENIEX_VALIDATION is ON
target_link_libraries(geniex_bridge PRIVATE geniex_validation)

set_target_properties(geniex_bridge PROPERTIES OUTPUT_NAME "geniex_bridge")

copy_output(out geniex_bridge OpenSSL::SSL OpenSSL::Crypto)
if(WIN32)
    add_custom_target(copy_geniex_bridge_lib ALL
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_LINKER_FILE:geniex_bridge> ${CMAKE_BINARY_DIR}/out/)
endif()

add_custom_target(copy_output_ml_h ALL
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_SOURCE_DIR}/include/ml.h
        ${CMAKE_BINARY_DIR}/out/ml.h
)
