#
# find_package(CUB) config file.
#
# Defines a CUB::CUB target that may be linked from user projects to include
# CUB.

if (TARGET CUB::CUB)
  return()
endif()

function(_cub_declare_interface_alias alias_name ugly_name)
  # 1) Only IMPORTED and ALIAS targets can be placed in a namespace.
  # 2) When an IMPORTED library is linked to another target, its include
  #    directories are treated as SYSTEM includes.
  # 3) nvcc will automatically check the CUDA Toolkit include path *before* the
  #    system includes. This means that the Toolkit CUB will *always* be used
  #    during compilation, and the include paths of an IMPORTED CUB::CUB
  #    target will never have any effect.
  # 4) This behavior can be fixed by setting the property NO_SYSTEM_FROM_IMPORTED
  #    on EVERY target that links to CUB::CUB. This would be a burden and a
  #    footgun for our users. Forgetting this would silently pull in the wrong CUB!
  # 5) A workaround is to make a non-IMPORTED library outside of the namespace,
  #    configure it, and then ALIAS it into the namespace (or ALIAS and then
  #    configure, that seems to work too).
  add_library(${ugly_name} INTERFACE)
  add_library(${alias_name} ALIAS ${ugly_name})
endfunction()

#
# Setup targets
#

_cub_declare_interface_alias(CUB::CUB _CUB_CUB)
# Strip out the 'cub/cmake/' from 'cub/cmake/cub-config.cmake':
get_filename_component(_CUB_INCLUDE_DIR "../.." ABSOLUTE BASE_DIR "${CMAKE_CURRENT_LIST_DIR}")
target_include_directories(_CUB_CUB INTERFACE "${_CUB_INCLUDE_DIR}")

if (CUB_IGNORE_DEPRECATED_CPP_DIALECT OR
    THRUST_IGNORE_DEPRECATED_CPP_DIALECT)
  target_compile_definitions(_CUB_CUB INTERFACE "CUB_IGNORE_DEPRECATED_CPP_DIALECT")
endif()

if (CUB_IGNORE_DEPRECATED_CPP_11 OR
    THRUST_IGNORE_DEPRECATED_CPP_11)
  target_compile_definitions(_CUB_CUB INTERFACE "CUB_IGNORE_DEPRECATED_CPP_11")
endif()

if (CUB_IGNORE_DEPRECATED_COMPILER OR
    THRUST_IGNORE_DEPRECATED_COMPILER)
  target_compile_definitions(_CUB_CUB INTERFACE "CUB_IGNORE_DEPRECATED_COMPILER")
endif()

#
# Standardize version info
#

set(CUB_VERSION ${${CMAKE_FIND_PACKAGE_NAME}_VERSION} CACHE INTERNAL "")
set(CUB_VERSION_MAJOR ${${CMAKE_FIND_PACKAGE_NAME}_VERSION_MAJOR} CACHE INTERNAL "")
set(CUB_VERSION_MINOR ${${CMAKE_FIND_PACKAGE_NAME}_VERSION_MINOR} CACHE INTERNAL "")
set(CUB_VERSION_PATCH ${${CMAKE_FIND_PACKAGE_NAME}_VERSION_PATCH} CACHE INTERNAL "")
set(CUB_VERSION_TWEAK ${${CMAKE_FIND_PACKAGE_NAME}_VERSION_TWEAK} CACHE INTERNAL "")
set(CUB_VERSION_COUNT ${${CMAKE_FIND_PACKAGE_NAME}_VERSION_COUNT} CACHE INTERNAL "")

include(FindPackageHandleStandardArgs)
if (NOT CUB_CONFIG)
  set(CUB_CONFIG "${CMAKE_CURRENT_LIST_FILE}")
endif()
find_package_handle_standard_args(CUB CONFIG_MODE)
