add_test(
  NAME cub.example.cmake.add_subdir
  COMMAND "${CMAKE_COMMAND}"
    --log-level=VERBOSE
    -G "${CMAKE_GENERATOR}"
    -S "${CMAKE_CURRENT_SOURCE_DIR}/add_subdir"
    -B "${CMAKE_CURRENT_BINARY_DIR}/add_subdir"
    -D "CUB_ROOT=${CUB_SOURCE_DIR}"
    -D "CMAKE_CUDA_COMPILER=${CMAKE_CUDA_COMPILER}"
    -D "CMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}"
)
