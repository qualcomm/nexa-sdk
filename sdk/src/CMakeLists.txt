file(GLOB BRIDGE_SRC *.cpp)

add_library(geniex SHARED ${BRIDGE_SRC})

target_compile_definitions(geniex PRIVATE GENIEX_VERSION="${GENIEX_VERSION}")
target_compile_definitions(geniex PRIVATE ML_SHARED ML_BUILD)

target_include_directories(geniex PUBLIC ${COMMON_INCLUDE})

if(GENIEX_MODEL_MANAGER)
    add_dependencies(geniex model_manager_build)
    target_link_libraries(geniex PRIVATE
        "${RUST_TARGET_DIR}/libgeniex_model.a"
    )
    target_include_directories(geniex PUBLIC
        "${CMAKE_SOURCE_DIR}/model-manager/include"
    )
    install(FILES "${CMAKE_SOURCE_DIR}/model-manager/include/ml_model.h"
        DESTINATION include)
endif()

install(TARGETS geniex
    LIBRARY DESTINATION lib
    RUNTIME DESTINATION lib)

install(FILES ${CMAKE_SOURCE_DIR}/include/ml.h DESTINATION include)
