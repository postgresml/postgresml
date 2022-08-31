# This example demonstrates / tests adding CUB via a CMake add_subdirectory
# call from a parent project.

cmake_minimum_required(VERSION 3.15)

# Silence warnings about empty CUDA_ARCHITECTURES properties on example targets:
if (CMAKE_VERSION VERSION_GREATER_EQUAL 3.18)
  cmake_policy(SET CMP0104 OLD)
endif()

project(CubAddSubDirExample CUDA)

# Use your project's checkout of CUB here, for most cases
# `add_subdirectory(cub)` will be sufficient.
add_subdirectory("${CUB_ROOT}" cub)

# Link the CUB::CUB target to your project's targets
add_executable(HelloCUB dummy.cu)
target_link_libraries(HelloCUB CUB::CUB)

#
# Validation
#

function(assert_target target_name)
  if (NOT TARGET "${target_name}")
    message(FATAL_ERROR "Target '${target_name}' not defined.")
  endif()
endfunction()

assert_target(CUB::CUB)
assert_target(HelloCUB)
