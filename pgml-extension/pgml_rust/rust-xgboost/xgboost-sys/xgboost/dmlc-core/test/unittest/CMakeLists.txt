# ---[ Google Test
if(MSVC)
  if(MSVC_VERSION LESS 1900)
    message(FATAL_ERROR "Need Visual Studio 2015 or newer to compile unit tests")
  endif()
endif()

if (UNIX)
  SET(CMAKE_EXE_LINKER_FLAGS "-pthread")
endif(UNIX)

# Compiler definitions needed to use GNU/POSIX extensions
set(ENABLE_GNU_EXTENSION_FLAGS -D_XOPEN_SOURCE=700
  -D_POSIX_SOURCE -D_POSIX_C_SOURCE=200809L -D_DARWIN_C_SOURCE)

enable_testing()
find_package(Threads REQUIRED)

file(GLOB_RECURSE UNIT_TEST_SOURCE "*.cc")
add_executable(dmlc_unit_tests ${UNIT_TEST_SOURCE})
set_property(TARGET dmlc_unit_tests
  PROPERTY RUNTIME_OUTPUT_DIRECTORY ${PRIVATE_RUNTIME_DIR})

message(STATUS "${CMAKE_CURRENT_SOURCE_DIR}/build_config.h.in -> ${CMAKE_CURRENT_SOURCE_DIR}/build_config.h")
configure_file("${CMAKE_CURRENT_SOURCE_DIR}/build_config.h.in" "${CMAKE_CURRENT_SOURCE_DIR}/build_config.h")

target_compile_definitions(dmlc_unit_tests PRIVATE -DDMLC_UNIT_TESTS_USE_CMAKE -DDMLC_CORE_USE_CMAKE ${ENABLE_GNU_EXTENSION_FLAGS})
target_include_directories(dmlc_unit_tests PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

find_package(GTest)
if (NOT GTEST_FOUND)
  message(STATUS "GTest not found, downloading GTest.")
  # Download and unpack googletest at configure time
  message("${CMAKE_LOCAL}/gtest_cmake.in -> ${CMAKE_BINARY_DIR}/googletest-download/CMakeLists.txt")
  configure_file("${CMAKE_LOCAL}/gtest_cmake.in" "${CMAKE_BINARY_DIR}/googletest-download/CMakeLists.txt")
  execute_process(COMMAND "${CMAKE_COMMAND}" -G "${CMAKE_GENERATOR}" .
    WORKING_DIRECTORY "${CMAKE_BINARY_DIR}/googletest-download" )
  execute_process(COMMAND "${CMAKE_COMMAND}" --build .
    WORKING_DIRECTORY "${CMAKE_BINARY_DIR}/googletest-download" )
  # Prevent GoogleTest from overriding our compiler/linker options
  # when building with Visual Studio
  set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)

  # Add googletest directly to our build. This adds
  # the following targets: gtest, gtest_main, gmock
  # and gmock_main
  add_subdirectory("${CMAKE_BINARY_DIR}/googletest-src"
    "${CMAKE_BINARY_DIR}/googletest-build")

  target_compile_definitions(gtest PRIVATE ${ENABLE_GNU_EXTENSION_FLAGS})
  target_compile_definitions(gmock PRIVATE ${ENABLE_GNU_EXTENSION_FLAGS})
  target_compile_definitions(gtest_main PRIVATE ${ENABLE_GNU_EXTENSION_FLAGS})
  target_compile_definitions(gmock_main PRIVATE ${ENABLE_GNU_EXTENSION_FLAGS})
  target_include_directories(dmlc_unit_tests PRIVATE
    "${gtest_SOURCE_DIR}/include" "${gmock_SOURCE_DIR}/include")
  target_link_libraries(dmlc_unit_tests
    gtest dmlc Threads::Threads)
else()
  target_include_directories(dmlc_unit_tests PRIVATE ${GTEST_INCLUDE_DIRS})
  target_link_libraries(dmlc_unit_tests
    ${GTEST_LIBRARIES} dmlc Threads::Threads)
endif()

if(USE_OPENMP)
  target_link_libraries(dmlc_unit_tests OpenMP::OpenMP_CXX)
endif()

add_test(AllTestsInDMLCUnitTests ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/dmlc_unit_tests)
