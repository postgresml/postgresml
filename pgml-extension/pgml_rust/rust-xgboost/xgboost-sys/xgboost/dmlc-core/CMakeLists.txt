cmake_minimum_required(VERSION 3.2)

project(dmlc VERSION 0.3 LANGUAGES C CXX)

if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/build/private/local_config.cmake)
  include(${CMAKE_CURRENT_SOURCE_DIR}/build/private/local_config.cmake)
endif()

set(CMAKE_LOCAL "${PROJECT_SOURCE_DIR}/cmake")
list(APPEND CMAKE_MODULE_PATH ${CMAKE_LOCAL}/Modules)

include(CheckCXXSymbolExists)
include(cmake/Utils.cmake)

# Options
dmlccore_option(USE_HDFS "Build with HDFS support" OFF)
dmlccore_option(DMLC_HDFS_SHARED "Build with dynamic HDFS library" OFF)
dmlccore_option(USE_AZURE "Build with AZURE support" OFF)
dmlccore_option(USE_S3 "Build with S3 support" OFF)
dmlccore_option(USE_PARQUET "Build with Arrow Parquet" OFF)
dmlccore_option(USE_OPENMP "Build with OpenMP" ON)
dmlccore_option(USE_CXX14_IF_AVAILABLE "Build with C++14 if the compiler supports it" OFF)
dmlccore_option(GOOGLE_TEST "Build google tests" OFF)
dmlccore_option(INSTALL_DOCUMENTATION "Install documentation" OFF)
dmlccore_option(DMLC_FORCE_SHARED_CRT "Build with dynamic CRT on Windows (/MD)" OFF)
dmlccore_option(DMLC_USE_SANITIZER "Use santizer flags; to specify a custom path for sanitizers, set this variable a value that's not ON or OFF" OFF)
set(DMLC_ENABLED_SANITIZERS "address" "leak" CACHE STRING
  "Semicolon separated list of sanitizer names. E.g 'address;leak'. Supported sanitizers are
  address, leak and thread.")

include(CheckCXXCompilerFlag)
if(USE_CXX14_IF_AVAILABLE)
  check_cxx_compiler_flag("-std=c++14" SUPPORT_CXX14)
endif()
if(SUPPORT_CXX14)
  set(CMAKE_CXX_STANDARD 14)
else()
  set(CMAKE_CXX_STANDARD 11)
endif()
set(CMAKE_CXX_EXTENSIONS OFF)

FILE(GLOB SOURCE "src/*.cc")
FILE(GLOB_RECURSE SOURCE_INCLUDE "include/*")
list(APPEND SOURCE ${SOURCE_INCLUDE})
list(APPEND SOURCE "src/io/line_split.cc")
list(APPEND SOURCE "src/io/recordio_split.cc")
list(APPEND SOURCE "src/io/indexed_recordio_split.cc")
list(APPEND SOURCE "src/io/input_split_base.cc")
list(APPEND SOURCE "src/io/filesys.cc")
list(APPEND SOURCE "src/io/local_filesys.cc")
if(USE_HDFS)
  list(APPEND SOURCE "src/io/hdfs_filesys.cc")
endif()
if(USE_S3)
  list(APPEND SOURCE "src/io/s3_filesys.cc")
endif()
if(USE_AZURE)
  list(APPEND SOURCE "src/io/azure_filesys.cc")
endif()

add_library(dmlc ${SOURCE})

# Sanitizer
if (DMLC_USE_SANITIZER)
  # Older CMake versions have had troubles with Sanitizer
  cmake_minimum_required(VERSION 3.12)
  include(cmake/Sanitizer.cmake)
  enable_sanitizers("${DMLC_ENABLED_SANITIZERS}")
endif (DMLC_USE_SANITIZER)

if(USE_PARQUET)
  find_package(Arrow REQUIRED)
  find_package(Parquet REQUIRED)
  if(BUILD_SHARED_LIBS)
    target_link_libraries(dmlc PRIVATE arrow_shared parquet_shared)
  else()
    # No need to use arrow_static / parquet_static here, since
    # we don't have a compelling need to include Arrow and Parquet
    # in the generaed static lib libdmlc.a. This is similar to
    # how we don't statically link with OpenSSL and cURL.
    target_link_libraries(dmlc PUBLIC arrow_shared parquet_shared)
  endif()
  set(DMLC_USE_PARQUET 1)
endif()

# HDFS configurations
if(USE_HDFS)
  find_package(HDFS REQUIRED)
  find_package(JNI REQUIRED)
  target_include_directories(dmlc PRIVATE ${HDFS_INCLUDE_DIR})
  if (DMLC_HDFS_SHARED)
    target_link_libraries(dmlc PRIVATE ${HDFS_LIBRARIES} ${JAVA_JVM_LIBRARY})
  else()
    target_link_libraries(dmlc PRIVATE ${HDFS_STATIC_LIB} ${JAVA_JVM_LIBRARY})
  endif()
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_HDFS=1)
else()
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_HDFS=0)
endif()
# S3 configurations
if(USE_S3)
  find_package(CURL REQUIRED)
  target_include_directories(dmlc SYSTEM PRIVATE ${CURL_INCLUDE_DIR})
  target_link_libraries(dmlc PRIVATE ${CURL_LIBRARY})

  find_package(OpenSSL REQUIRED)
  target_include_directories(dmlc SYSTEM PRIVATE ${OPENSSL_INCLUDE_DIR})
  target_link_libraries(dmlc PRIVATE ${OPENSSL_LIBRARY} ${OPENSSL_LIBRARIES} ${OPENSSL_CRYPTO_LIBRARY})
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_S3=1)
else()
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_S3=0)
endif()
# Azure configurations
if(USE_AZURE)
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_AZURE=1)
else()
  target_compile_definitions(dmlc PRIVATE -DDMLC_USE_AZURE=0)
endif()

# OpenMP
if(USE_OPENMP)
  if(APPLE AND (NOT CMAKE_COMPILER_IS_GNUCC))
    # Require CMake 3.16+ for Mac to ensure that OpenMP can be located
    # (Exception: it's okay if Homebrew GCC is used)
    cmake_minimum_required(VERSION 3.16)
  endif()

  find_package(OpenMP REQUIRED)

  # For CMake < 3.9, we need to make target OpenMP::OpenMP_CXX ourselves
  if(NOT TARGET OpenMP::OpenMP_CXX)
    find_package(Threads REQUIRED)
    add_library(OpenMP::OpenMP_CXX IMPORTED INTERFACE)
    set_property(TARGET OpenMP::OpenMP_CXX
                 PROPERTY INTERFACE_COMPILE_OPTIONS ${OpenMP_CXX_FLAGS})
    set_property(TARGET OpenMP::OpenMP_CXX
                 PROPERTY INTERFACE_LINK_LIBRARIES ${OpenMP_CXX_FLAGS} Threads::Threads)
  endif()
  target_link_libraries(dmlc PRIVATE OpenMP::OpenMP_CXX)
endif()

if(WIN32 AND (NOT MSVC))  # On Windows, link Shlwapi.lib for non-MSVC compilers
  target_link_libraries(dmlc PRIVATE Shlwapi)
endif()

# Check location of clock_gettime; if it's in librt, link it
include(CheckLibraryExists)
CHECK_LIBRARY_EXISTS(rt clock_gettime "time.h" HAVE_CLOCK_GETTIME_IN_LIBRT)
if(HAVE_CLOCK_GETTIME_IN_LIBRT)
  target_link_libraries(dmlc PRIVATE rt)
endif()

# Check headers and symbols
include(CheckSymbolExists)
include(CheckIncludeFile)
include(CheckIncludeFileCXX)
check_symbol_exists(fopen64 stdio.h DMLC_FOPEN_64_PRESENT)
check_include_file_cxx(cxxabi.h DMLC_CXXABI_H_PRESENT)
check_symbol_exists(nanosleep time.h DMLC_NANOSLEEP_PRESENT)

# Check existence of backtrace(3)
find_package(Backtrace)
if(Backtrace_FOUND)
  set(DMLC_EXECINFO_H_PRESENT 1)
  set(DMLC_EXECINFO_H ${Backtrace_HEADER})
  target_include_directories(dmlc SYSTEM PRIVATE ${Backtrace_INCLUDE_DIRS})
  target_link_libraries(dmlc PRIVATE ${Backtrace_LIBRARIES})
else()
  set(DMLC_EXECINFO_H_PRESENT 0)
endif()

# Check endianness
include(TestBigEndian)
test_big_endian(BIG_ENDIAN)
if(BIG_ENDIAN)
  set(DMLC_CMAKE_LITTLE_ENDIAN 0)
else()
  set(DMLC_CMAKE_LITTLE_ENDIAN 1)
endif()

message(STATUS "${CMAKE_LOCAL}/build_config.h.in -> include/dmlc/build_config.h")
configure_file("cmake/build_config.h.in" "include/dmlc/build_config.h")

target_include_directories(dmlc PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<BUILD_INTERFACE:${CMAKE_CURRENT_BINARY_DIR}/include>
  $<INSTALL_INTERFACE:$<INSTALL_PREFIX>/include>)
target_compile_definitions(dmlc PRIVATE -D_XOPEN_SOURCE=700
  -D_POSIX_SOURCE -D_POSIX_C_SOURCE=200809L -D_DARWIN_C_SOURCE)
# Older stdc++ enable c++11 items
target_compile_definitions(dmlc PUBLIC -D__USE_XOPEN2K8)
# DMLC_CORE_USE_CMAKE macro constant indicates the use of CMake
target_compile_definitions(dmlc PUBLIC -DDMLC_CORE_USE_CMAKE)

# compiler flags
if(MSVC)
  target_compile_definitions(dmlc PUBLIC -DDMLC_USE_CXX11=1)
  if(NOT BUILD_SHARED_LIBS AND NOT DMLC_FORCE_SHARED_CRT)
    foreach(flag_var
          CMAKE_CXX_FLAGS CMAKE_CXX_FLAGS_DEBUG CMAKE_CXX_FLAGS_RELEASE
          CMAKE_CXX_FLAGS_MINSIZEREL CMAKE_CXX_FLAGS_RELWITHDEBINFO)
      if(${flag_var} MATCHES "/MD")
        string(REGEX REPLACE "/MD" "/MT" ${flag_var} "${${flag_var}}")
      endif(${flag_var} MATCHES "/MD")
    endforeach(flag_var)
  endif()
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /EHsc")
else()
  set(CMAKE_POSITION_INDEPENDENT_CODE ON)
  check_cxx_compiler_flag("-msse2" SUPPORT_MSSE2)
  if(SUPPORT_MSSE2)
    target_compile_options(dmlc PRIVATE -msse2)
  endif()
  target_compile_options(dmlc PRIVATE -Wall -Wno-unknown-pragmas -fPIC)
  if(CMAKE_BUILD_TYPE STREQUAL "DEBUG" OR CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_options(dmlc PRIVATE -g -O0)
  else()
    target_compile_options(dmlc PRIVATE -O3)
  endif()

  target_compile_definitions(dmlc PUBLIC -DDMLC_USE_CXX11=1)
  if(SUPPORT_CXX14)
    target_compile_definitions(dmlc PUBLIC -DDMLC_USE_CXX14=1)
  endif()
endif()


include(GNUInstallDirs)
# ---[ Install Includes
install(DIRECTORY ${PROJECT_SOURCE_DIR}/include/dmlc
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/include/dmlc/build_config.h
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/dmlc)

# ---[ Install the archive static lib and header files
install(TARGETS dmlc
  EXPORT DMLCTargets
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR})
install(EXPORT DMLCTargets
  FILE DMLCTargets.cmake
  NAMESPACE dmlc::
  EXPORT_LINK_INTERFACE_LIBRARIES
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/dmlc)

# ---[ Install documentation
if(INSTALL_DOCUMENTATION)
  install(DIRECTORY doc DESTINATION ${CMAKE_INSTALL_DATADIR})
endif()

# ---[ Package configurations
include(CMakePackageConfigHelpers)
configure_package_config_file(
  ${CMAKE_LOCAL}/dmlc-config.cmake.in
  ${CMAKE_BINARY_DIR}/cmake/dmlc-config.cmake
  INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/dmlc)
write_basic_package_version_file(
  ${CMAKE_BINARY_DIR}/cmake/dmlc-config-version.cmake
  VERSION ${DMLC_VERSION}
  COMPATIBILITY AnyNewerVersion)
install(
  FILES
  ${CMAKE_BINARY_DIR}/cmake/dmlc-config.cmake
  ${CMAKE_BINARY_DIR}/cmake/dmlc-config-version.cmake
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/dmlc)

# ---[ Linter target
if(MSVC)
  find_package(PythonInterp)
  set(PYTHON_EXECUTABLE ${PYTHON_EXECUTABLE} CACHE FILEPATH "Path to the python 2.x executable")
endif()
set(LINT_DIRS include src scripts)
add_custom_target(dmlc_lint COMMAND ${CMAKE_COMMAND} -DMSVC=${MSVC} -DPYTHON_EXECUTABLE=${PYTHON_EXECUTABLE}  -DPROJECT_SOURCE_DIR=${PROJECT_SOURCE_DIR} -DLINT_DIRS=${LINT_DIRS} -DPROJECT_NAME=dmlc -P ${PROJECT_SOURCE_DIR}/cmake/lint.cmake)

# Setup testing
if(GOOGLE_TEST)
  include(CTest)
  add_subdirectory(test/unittest)
endif()
