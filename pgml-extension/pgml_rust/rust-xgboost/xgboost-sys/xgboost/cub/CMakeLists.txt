# Will be increased to 3.18 when C++17 is enabled:
cmake_minimum_required(VERSION 3.15)

# Remove this when we use the new CUDA_ARCHITECTURES properties.
if (CMAKE_VERSION VERSION_GREATER_EQUAL 3.18)
  cmake_policy(SET CMP0104 OLD)
endif()

# CXX is only needed for AppendOptionIfAvailable.
project(CUB CUDA CXX)

# Determine whether CUB is the top-level project or included into
# another project via add_subdirectory().
if ("${CMAKE_SOURCE_DIR}" STREQUAL "${CMAKE_CURRENT_LIST_DIR}")
  set(CUB_TOPLEVEL_PROJECT ON)
else()
  set(CUB_TOPLEVEL_PROJECT OFF)
endif()

# Thrust has its own copy of CUB install rules to handle packaging usecases
# where we want to install CUB headers but aren't actually building anything.
# In these cases the add_subdirectory(dependencies/cub) line in Thrust won't get
# called so we can't rely on CUB providing its own rules.
if (NOT CUB_IN_THRUST)
  option(CUB_ENABLE_INSTALL_RULES "Enable installation of CUB" ${CUB_TOPLEVEL_PROJECT})
  if (CUB_ENABLE_INSTALL_RULES)
    include(cmake/CubInstallRules.cmake)
  endif()
endif()

# Support adding CUB to a parent project via add_subdirectory.
# See examples/cmake/add_subdir/CMakeLists.txt for details.
if (NOT CUB_TOPLEVEL_PROJECT AND NOT CUB_IN_THRUST)
  include(cmake/CubAddSubdir.cmake)
  return()
endif()

include(cmake/AppendOptionIfAvailable.cmake)
include(cmake/CubBuildCompilerTargets.cmake)
include(cmake/CubBuildTargetList.cmake)
include(cmake/CubCudaConfig.cmake)

option(CUB_ENABLE_HEADER_TESTING "Test that all public headers compile." ON)
option(CUB_ENABLE_TESTING "Build CUB testing suite." ON)
option(CUB_ENABLE_EXAMPLES "Build CUB examples." ON)

# Check if we're actually building anything before continuing. If not, no need
# to search for deps, etc. This is a common approach for packagers that just
# need the install rules. See GH issue NVIDIA/thrust#1211.
if (NOT (CUB_ENABLE_HEADER_TESTING OR
         CUB_ENABLE_TESTING OR
         CUB_ENABLE_EXAMPLES))
  return()
endif()

if ("" STREQUAL "${CMAKE_BUILD_TYPE}")
  set(CMAKE_BUILD_TYPE "RelWithDebInfo" CACHE STRING "Choose the type of build." FORCE)

  set_property(
    CACHE CMAKE_BUILD_TYPE
    PROPERTY STRINGS Debug Release RelWithDebInfo MinSizeRel
  )
endif ()

set(CMAKE_CXX_EXTENSIONS OFF)

# Where to put build outputs. Use CMAKE_BINARY_DIR so they'll show up alongside
# Thrust targets when building as part of Thrust.
set(CUB_LIBRARY_OUTPUT_DIR "${CMAKE_BINARY_DIR}/lib")
set(CUB_EXECUTABLE_OUTPUT_DIR "${CMAKE_BINARY_DIR}/bin")

cub_build_target_list()

if (CUB_ENABLE_HEADER_TESTING)
  include(cmake/CubHeaderTesting.cmake)
endif()

# Both testing and examples use ctest
if (CUB_ENABLE_TESTING OR CUB_ENABLE_EXAMPLES)
  include(CTest)
  enable_testing()
endif()

if (CUB_ENABLE_TESTING)
  add_subdirectory(test)
endif()

if (CUB_ENABLE_EXAMPLES)
  add_subdirectory(examples)
endif()
