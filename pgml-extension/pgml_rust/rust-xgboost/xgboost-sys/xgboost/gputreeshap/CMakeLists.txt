cmake_minimum_required(VERSION 3.14 FATAL_ERROR)
include(FetchContent)
project(GPUTreeShap LANGUAGES CXX CUDA)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CUDA_STANDARD 14)
option(BUILD_GTEST "Build google tests" OFF)
option(BUILD_EXAMPLES "Build examples" OFF)
option(BUILD_BENCHMARKS "Build benchmarks" OFF)

# Use release build by default
if (NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE "Release" CACHE STRING "" FORCE)
endif()

# GPUTreeSHAP target is header-only
add_library(GPUTreeShap INTERFACE)
add_library(GPUTreeShap::GPUTreeShap ALIAS GPUTreeShap)
target_sources(GPUTreeShap INTERFACE ${GPUTreeShap_SOURCE_DIR}/GPUTreeShap/gpu_treeshap.h)
target_include_directories(GPUTreeShap INTERFACE ${GPUTreeShap_SOURCE_DIR})

set(COMPILE_OPTIONS --expt-extended-lambda -lineinfo)
set(GCC_COMPILE_OPTIONS -Xcompiler -Werror,-Wall,-Wextra)
if(BUILD_GTEST)
  set(gtest_force_shared_crt ON CACHE BOOL "Always use msvcrt.dll")
  FetchContent_Declare(
    googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG        release-1.8.1
    GIT_SHALLOW       true
  )
  FetchContent_MakeAvailable(googletest)
  add_executable(TestGPUTreeShap tests/test_gpu_treeshap.cu)
  target_link_libraries(TestGPUTreeShap PRIVATE GPUTreeShap)
  target_compile_options(TestGPUTreeShap PRIVATE $<$<COMPILE_LANGUAGE:CUDA>: ${COMPILE_OPTIONS}>)
  if(NOT MSVC)
    target_compile_options(TestGPUTreeShap PRIVATE $<$<COMPILE_LANGUAGE:CUDA>:${GCC_COMPILE_OPTIONS}>)
  else()
  endif()
  target_link_libraries(TestGPUTreeShap  PRIVATE gtest gtest_main)
endif()

if(BUILD_EXAMPLES)
  add_executable(GPUTreeShapExample example/example.cu)
  target_link_libraries(GPUTreeShapExample PRIVATE GPUTreeShap)
  target_compile_options(GPUTreeShapExample PRIVATE $<$<COMPILE_LANGUAGE:CUDA>:${COMPILE_OPTIONS}>)
  if(NOT MSVC)
    target_compile_options(GPUTreeShapExample PRIVATE $<$<COMPILE_LANGUAGE:CUDA>:${GCC_COMPILE_OPTIONS}>)
  endif()
endif()

if(BUILD_BENCHMARKS)
  FetchContent_Declare(
    benchmark
    GIT_REPOSITORY https://github.com/google/benchmark.git
    GIT_TAG        v1.5.2
    GIT_SHALLOW       true
  )
  set(BENCHMARK_ENABLE_TESTING OFF CACHE INTERNAL "Google benchmark tests off")
  FetchContent_MakeAvailable(benchmark)
  add_executable(BenchmarkGPUTreeShap benchmark/benchmark_gpu_treeshap.cu)
  target_link_libraries(BenchmarkGPUTreeShap PRIVATE GPUTreeShap)
  target_compile_options(BenchmarkGPUTreeShap PRIVATE $<$<COMPILE_LANGUAGE:CUDA>: ${COMPILE_OPTIONS}>)
  target_link_libraries(BenchmarkGPUTreeShap  PRIVATE benchmark::benchmark)
endif()
