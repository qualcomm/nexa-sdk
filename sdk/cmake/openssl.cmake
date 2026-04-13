# Find OpenSSL first without CMAKE_SYSTEM_PATH to prefer local installations
find_package(OpenSSL NO_CMAKE_SYSTEM_PATH)
if(NOT OpenSSL_FOUND)
    find_package(OpenSSL REQUIRED)
endif()

message(STATUS "Found OpenSSL: ${OPENSSL_VERSION}")
message(STATUS "Found OpenSSL Libraries: ${OPENSSL_LIBRARIES}")

target_link_libraries(geniex_validation INTERFACE OpenSSL::SSL OpenSSL::Crypto)
target_compile_definitions(geniex_validation INTERFACE CPPHTTPLIB_OPENSSL_SUPPORT)
