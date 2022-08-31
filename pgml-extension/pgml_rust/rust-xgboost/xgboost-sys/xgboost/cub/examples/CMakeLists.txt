# Create meta targets that build all examples for a single configuration:
foreach(cub_target IN LISTS CUB_TARGETS)
  cub_get_target_property(config_prefix ${cub_target} PREFIX)
  set(config_meta_target ${config_prefix}.examples)
  add_custom_target(${config_meta_target})
  add_dependencies(${config_prefix}.all ${config_meta_target})
endforeach()

## cub_add_example
#
# Add an example executable and register it with ctest.
#
# target_name_var: Variable name to overwrite with the name of the example
#   target. Useful for post-processing target information per-backend.
# example_name: The name of the example minus "<config_prefix>.example." For
#   instance, examples/vector.cu will be "vector", and examples/cuda/copy.cu
#   would be "cuda.copy".
# example_src: The source file that implements the example.
# cub_target: The reference cub target with configuration information.
#
function(cub_add_example target_name_var example_name example_src cub_target)
  cub_get_target_property(config_prefix ${cub_target} PREFIX)

  # The actual name of the test's target:
  set(example_target ${config_prefix}.example.${example_name})
  set(${target_name_var} ${example_target} PARENT_SCOPE)

  # Related target names:
  set(config_meta_target ${config_prefix}.examples)
  set(example_meta_target cub.all.example.${example_name})

  add_executable(${example_target} "${example_src}")
  target_link_libraries(${example_target} ${cub_target})
  cub_clone_target_properties(${example_target} ${cub_target})
  target_include_directories(${example_target} PRIVATE "${CUB_SOURCE_DIR}/examples")

  # Add to the active configuration's meta target
  add_dependencies(${config_meta_target} ${example_target})

  # Meta target that builds examples with this name for all configurations:
  if (NOT TARGET ${example_meta_target})
    add_custom_target(${example_meta_target})
  endif()
  add_dependencies(${example_meta_target} ${example_target})

  if (CUB_ENABLE_EXAMPLES_WITH_RDC)
    set_target_properties(${example_target} PROPERTIES
      CUDA_SEPARABLE_COMPILATION ON
    )
  endif()

  add_test(NAME ${example_target}
    COMMAND "$<TARGET_FILE:${example_target}>"
  )
endfunction()

add_subdirectory(cmake)
add_subdirectory(block)
add_subdirectory(device)
