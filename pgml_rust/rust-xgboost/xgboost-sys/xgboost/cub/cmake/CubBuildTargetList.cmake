# This file provides utilities for building and working with CUB
# configuration targets.
#
# CUB_TARGETS
#  - Built by the calling the `cub_build_target_list()` function.
#  - Each item is the name of a CUB interface target that is configured for a
#    certain build configuration. Currently only C++ standard dialect is
#    considered.
#
# cub_build_target_list()
# - Creates the CUB_TARGETS list.
#
# The following functions can be used to test/set metadata on a CUB target:
#
# cub_get_target_property(<prop_var> <target_name> <prop>)
#   - Checks the ${prop} target property on CUB target ${target_name}
#     and sets the ${prop_var} variable in the caller's scope.
#   - <prop_var> is any valid cmake identifier.
#   - <target_name> is the name of a CUB target.
#   - <prop> is one of the following:
#     - DIALECT: The C++ dialect. Valid values: 11, 14, 17.
#     - PREFIX: A unique prefix that should be used to name all
#       targets/tests/examples that use this configuration.
#
# cub_get_target_properties(<target_name>)
#   - Defines ${target_name}_${prop} in the caller's scope, for `prop` in:
#     {DIALECT, PREFIX}. See above for details.
#
# cub_clone_target_properties(<dst_target> <src_target>)
#   - Set the {DIALECT, PREFIX} metadata on ${dst_target} to match
#     ${src_target}. See above for details.
#   - This *MUST* be called on any targets that link to another CUB target
#     to ensure that dialect information is updated correctly, e.g.
#     `cub_clone_target_properties(${my_cub_test} ${some_cub_target})`

# Dialects:
set(CUB_CPP_DIALECT_OPTIONS
  11 14 17
  CACHE INTERNAL "C++ dialects supported by CUB." FORCE
)

define_property(TARGET PROPERTY _CUB_DIALECT
  BRIEF_DOCS "A target's C++ dialect: 11, 14, or 17."
  FULL_DOCS "A target's C++ dialect: 11, 14, or 17."
)
define_property(TARGET PROPERTY _CUB_PREFIX
  BRIEF_DOCS "A prefix describing the config, eg. 'cub.cpp14'."
  FULL_DOCS "A prefix describing the config, eg. 'cub.cpp14'."
)

function(cub_set_target_properties target_name dialect prefix)
  set_target_properties(${target_name}
    PROPERTIES
      _CUB_DIALECT ${dialect}
      _CUB_PREFIX ${prefix}
  )

  get_target_property(type ${target_name} TYPE)
  if (NOT ${type} STREQUAL "INTERFACE_LIBRARY")
    set_target_properties(${target_name}
      PROPERTIES
        CXX_STANDARD ${dialect}
        CUDA_STANDARD ${dialect}
        ARCHIVE_OUTPUT_DIRECTORY "${CUB_LIBRARY_OUTPUT_DIR}"
        LIBRARY_OUTPUT_DIRECTORY "${CUB_LIBRARY_OUTPUT_DIR}"
        RUNTIME_OUTPUT_DIRECTORY "${CUB_EXECUTABLE_OUTPUT_DIR}"
    )

    # CMake still emits errors about empty CUDA_ARCHITECTURES when CMP0104
    # is set to OLD. This suppresses the errors for good.
    if (CMAKE_VERSION VERSION_GREATER_EQUAL 3.18)
      set_target_properties(${target_name}
        PROPERTIES
          CUDA_ARCHITECTURES OFF
      )
    endif()
  endif()
endfunction()

# Get a cub property from a target and store it in var_name
# cub_get_target_property(<var_name> <target_name> [DIALECT|PREFIX]
macro(cub_get_target_property prop_var target_name prop)
  get_property(${prop_var} TARGET ${target_name} PROPERTY _CUB_${prop})
endmacro()

# Defines the following string variables in the caller's scope:
# - ${target_name}_DIALECT
# - ${target_name}_PREFIX
macro(cub_get_target_properties target_name)
  cub_get_target_property(${target_name}_DIALECT ${target_name} DIALECT)
  cub_get_target_property(${target_name}_PREFIX ${target_name} PREFIX)
endmacro()

# Set one target's _CUB_* properties to match another target
function(cub_clone_target_properties dst_target src_target)
  cub_get_target_properties(${src_target})
  cub_set_target_properties(${dst_target}
    ${${src_target}_DIALECT}
    ${${src_target}_PREFIX}
  )
endfunction()

# Set ${var_name} to TRUE or FALSE in the caller's scope
function(_cub_is_config_valid var_name dialect)
  if (CUB_ENABLE_DIALECT_CPP${dialect})
    set(${var_name} TRUE PARENT_SCOPE)
  else()
    set(${var_name} FALSE PARENT_SCOPE)
  endif()
endfunction()

function(_cub_init_target_list)
  set(CUB_TARGETS "" CACHE INTERNAL "" FORCE)
endfunction()

function(_cub_add_target_to_target_list target_name dialect prefix)
  cub_set_target_properties(${target_name} ${dialect} ${prefix})

  target_link_libraries(${target_name} INTERFACE
    CUB::CUB
    cub.compiler_interface
  )

  if (TARGET cub.thrust)
    target_link_libraries(${target_name} INTERFACE cub.thrust)
  endif()

  set(CUB_TARGETS ${CUB_TARGETS} ${target_name} CACHE INTERNAL "" FORCE)

  set(label "cpp${dialect}")
  string(TOLOWER "${label}" label)
  message(STATUS "Enabling CUB configuration: ${label}")
endfunction()

# Build a ${CUB_TARGETS} list containing target names for all
# requested configurations
function(cub_build_target_list)
  # Clear the list of targets:
  _cub_init_target_list()

  # Handle dialect options:
  foreach (dialect IN LISTS CUB_CPP_DIALECT_OPTIONS)
    if (CUB_IN_THRUST)
      # Just use Thrust's settings:
      if (THRUST_ENABLE_MULTICONFIG)
        set(CUB_ENABLE_DIALECT_CPP${dialect}
            ${THRUST_MULTICONFIG_ENABLE_DIALECT_CPP${dialect}}
        )
      else()
        set(val OFF)
        if (dialect EQUAL ${THRUST_CPP_DIALECT})
          set(val ON)
        endif()
        set(CUB_ENABLE_DIALECT_CPP${dialect} ${val})
      endif()
    else()
      # Create CMake options:
      set(default_value OFF)
      if (dialect EQUAL 14) # Default to just 14 on:
        set(default_value ON)
      endif()
      option(CUB_ENABLE_DIALECT_CPP${dialect}
        "Generate C++${dialect} build configurations."
        ${default_value}
      )
    endif()
  endforeach()

  # CMake added C++17 support for CUDA targets in 3.18:
  if (CUB_ENABLE_DIALECT_CPP17)
    cmake_minimum_required(VERSION 3.18)
  endif()

  # Supported versions of MSVC do not distinguish between C++11 and C++14.
  # Warn the user that they may be generating a ton of redundant targets.
  if ("MSVC" STREQUAL "${CMAKE_CXX_COMPILER_ID}" AND
      CUB_ENABLE_DIALECT_CPP11)
    message(WARNING
      "Supported versions of MSVC (2017+) do not distinguish between C++11 "
      "and C++14. The requested C++11 targets will be built with C++14."
    )
  endif()

  # Generic config flags:
  macro(add_flag_option flag docstring default)
    set(cub_opt "CUB_${flag}")
    if (CUB_IN_THRUST)
      set(thrust_opt "THRUST_${flag}")
      # Use thrust's settings:
      set(${cub_opt} ${${thrust_opt}})
    else()
      option(${cub_opt} "${docstring}" "${default}")
      mark_as_advanced(${cub_opt})
    endif()
  endmacro()
  add_flag_option(IGNORE_DEPRECATED_CPP_DIALECT "Don't warn about any deprecated C++ standards and compilers." OFF)
  add_flag_option(IGNORE_DEPRECATED_CPP_11 "Don't warn about deprecated C++11." OFF)
  add_flag_option(IGNORE_DEPRECATED_COMPILER "Don't warn about deprecated compilers." OFF)

  # Build cub.compiler_interface with warning flags, etc
  # This must be called before _cub_add_target_to_target_list.
  cub_build_compiler_targets()

  # Set up the CUB target while testing out our find_package scripts.
  find_package(CUB REQUIRED CONFIG
    NO_DEFAULT_PATH # Only check the explicit path in HINTS:
    HINTS "${CUB_SOURCE_DIR}"
  )

  # TODO
  # Some of the iterators and unittests depend on thrust. We should break the
  # cyclical dependency by migrating CUB's Thrust bits into Thrust.
  find_package(Thrust ${CUB_VERSION} EXACT CONFIG
    HINTS "../../" # Check if we are in thrust/dependencies/cub
  )

  if (Thrust_FOUND)
    thrust_set_CUB_target(CUB::CUB)
    thrust_create_target(cub.thrust HOST CPP DEVICE CUDA)
  else()
    message(STATUS
      "Thrust was not found. Set CMake variable 'Thrust_DIR' to the "
      "thrust-config.cmake file of a Thrust ${CUB_VERSION} installation to "
      "enable additional testing."
    )
  endif()

  # Build CUB_TARGETS
  foreach(dialect IN LISTS CUB_CPP_DIALECT_OPTIONS)
    _cub_is_config_valid(config_valid ${dialect})
    if (config_valid)
      set(prefix "cub.cpp${dialect}")
      string(TOLOWER "${prefix}" prefix)
      set(target_name "${prefix}")

      add_library(${target_name} INTERFACE)

      # Set configuration metadata for this cub interface target:
      _cub_add_target_to_target_list(${target_name} ${dialect} ${prefix})
    endif()
  endforeach() # dialects

  list(LENGTH CUB_TARGETS count)
  message(STATUS "${count} unique cub.dialect configurations generated")

  # Top level meta-target. Makes it easier to just build CUB targets when
  # building both CUB and Thrust. Add all project files here so IDEs will be
  # aware of them. This will not generate build rules.
  file(GLOB_RECURSE all_sources
    RELATIVE "${CMAKE_CURRENT_LIST_DIR}"
    "${CUB_SOURCE_DIR}/cub/*.cuh"
  )
  add_custom_target(cub.all SOURCES ${all_sources})

  # Create meta targets for each config:
  foreach(cub_target IN LISTS CUB_TARGETS)
    cub_get_target_property(config_prefix ${cub_target} PREFIX)
    add_custom_target(${config_prefix}.all)
    add_dependencies(cub.all ${config_prefix}.all)
  endforeach()
endfunction()
