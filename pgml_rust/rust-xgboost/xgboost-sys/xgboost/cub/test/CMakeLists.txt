# TODO investigate whether this is really needed:
math(EXPR CUB_TEST_ARCH "${CUB_MINIMAL_ENABLED_ARCH} * 10")
message(STATUS "CUB Test architecture (TEST_ARCH): ${CUB_TEST_ARCH}")

# Create meta targets that build all tests for a single configuration:
foreach(cub_target IN LISTS CUB_TARGETS)
  cub_get_target_property(config_prefix ${cub_target} PREFIX)
  set(config_meta_target ${config_prefix}.tests)
  add_custom_target(${config_meta_target})
  add_dependencies(${config_prefix}.all ${config_meta_target})
endforeach()

file(GLOB test_srcs
  RELATIVE ${CUB_SOURCE_DIR}/test
  CONFIGURE_DEPENDS
  test_*.cu
)

## cub_add_test
#
# Add a test executable and register it with ctest.
#
# target_name_var: Variable name to overwrite with the name of the test
#   target. Useful for post-processing target information.
# test_name: The name of the test minus "<config_prefix>.test." For example,
#   testing/vector.cu will be "vector", and testing/cuda/copy.cu will be
#   "cuda.copy".
# test_src: The source file that implements the test.
# cub_target: The reference cub target with configuration information.
#
function(cub_add_test target_name_var test_name test_src cub_target)
  cub_get_target_property(config_prefix ${cub_target} PREFIX)

  # The actual name of the test's target:
  set(test_target ${config_prefix}.test.${test_name})
  set(${target_name_var} ${test_target} PARENT_SCOPE)

  # Related target names:
  set(config_meta_target ${config_prefix}.tests)
  set(test_meta_target cub.all.test.${test_name})

  add_executable(${test_target} "${test_src}")
  target_link_libraries(${test_target} ${cub_target})
  cub_clone_target_properties(${test_target} ${cub_target})
  target_compile_definitions(${test_target} PRIVATE TEST_ARCH=${CUB_TEST_ARCH})
  target_include_directories(${test_target} PRIVATE "${CUB_SOURCE_DIR}/test")

  # Add to the active configuration's meta target
  add_dependencies(${config_meta_target} ${test_target})

  # Meta target that builds tests with this name for all configurations:
  if (NOT TARGET ${test_meta_target})
    add_custom_target(${test_meta_target})
  endif()
  add_dependencies(${test_meta_target} ${test_target})

  if (CUB_ENABLE_TESTS_WITH_RDC)
    set_target_properties(${test_target} PROPERTIES
      CUDA_SEPARABLE_COMPILATION ON
    )
  endif()

  add_test(NAME ${test_target}
    COMMAND "$<TARGET_FILE:${test_target}>"
  )
endfunction()

# Sets HAS_QUICK_VARIANT / HAS_QUICKER_VARIANT / NO_VARIANTS to True/False in
# the calling scope.
# Used to detect variants of unit tests depending on whether a source file
# contains the strings "QUICK_TEST" or "QUICKER_TEST".
function(cub_check_for_test_variants src)
  file(READ "${src}" data)

  string(FIND "${data}" "QUICK_TEST" quick_loc)
  set(HAS_QUICK_VARIANT False PARENT_SCOPE)
  if (NOT quick_loc EQUAL -1)
    set(HAS_QUICK_VARIANT True PARENT_SCOPE)
  endif()

  string(FIND "${data}" "QUICKER_TEST" quicker_loc)
  set(HAS_QUICKER_VARIANT False PARENT_SCOPE)
  if (NOT quicker_loc EQUAL -1)
    set(HAS_QUICKER_VARIANT True PARENT_SCOPE)
  endif()

  set(NO_VARIANTS False PARENT_SCOPE)
  if (NOT (HAS_QUICK_VARIANT OR HAS_QUICKER_VARIANT))
    set(NO_VARIANTS True PARENT_SCOPE)
  endif()
endfunction()

foreach (test_src IN LISTS test_srcs)
  # TODO: Per-test flags.

  get_filename_component(test_name "${test_src}" NAME_WE)
  string(REGEX REPLACE "^test_" "" test_name "${test_name}")

  # Some tests change behavior based on whether the compiler defs QUICK_TEST
  # and/or QUICKER_TEST are defined. Detect these and build variants for each
  # configuration:
  cub_check_for_test_variants("${test_src}")

  foreach(cub_target IN LISTS CUB_TARGETS)
    if (NO_VARIANTS)
      # Only one version of this test.
      cub_add_test(test_target ${test_name} "${test_src}" ${cub_target})
    else()
      # By default (no flags), the "thorough" version of the test is built:
      cub_add_test(test_target_thorough
        ${test_name}.thorough
        "${test_src}"
        ${cub_target}
      )

      # Add the other variants with appropriate suffixes:
      if (HAS_QUICK_VARIANT)
        cub_add_test(test_target_quick
          ${test_name}.quick
          "${test_src}"
          ${cub_target}
        )
        target_compile_definitions(${test_target_quick} PRIVATE QUICK_TEST)
      endif()

      if (HAS_QUICKER_VARIANT)
        cub_add_test(test_target_quicker
          ${test_name}.quicker
          "${test_src}"
          ${cub_target}
        )
        target_compile_definitions(${test_target_quicker} PRIVATE QUICKER_TEST)
      endif()
    endif()
  endforeach()
endforeach()
