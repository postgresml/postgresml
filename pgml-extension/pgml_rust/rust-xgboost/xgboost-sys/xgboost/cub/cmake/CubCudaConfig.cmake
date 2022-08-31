if (NOT ("${CMAKE_CUDA_HOST_COMPILER}" STREQUAL "" OR
         "${CMAKE_CUDA_HOST_COMPILER}" STREQUAL "${CMAKE_CXX_COMPILER}"))
  message(FATAL_ERROR
    "CUB tests and examples require the C++ compiler and the CUDA host "
    "compiler to be the same; to set this compiler, please use the "
    "CMAKE_CXX_COMPILER variable, not the CMAKE_CUDA_HOST_COMPILER variable."
  )
endif()
set(CMAKE_CUDA_HOST_COMPILER "${CMAKE_CXX_COMPILER}")

#
# Architecture options:
#

set(all_archs 35 37 50 52 53 60 61 62 70 72 75 80)
set(arch_message "CUB: Enabled CUDA architectures:")
set(enabled_archs)

# Thrust sets up the architecture flags in CMAKE_CUDA_FLAGS already. Just
# reuse them if possible. After we transition to CMake 3.18 CUDA_ARCHITECTURE
# target properties this will need to be updated.
if (CUB_IN_THRUST)
  # Configure to use all flags from thrust:
  set(CMAKE_CUDA_FLAGS "${THRUST_CUDA_FLAGS_BASE} ${THRUST_CUDA_FLAGS_NO_RDC}")

  # Update the enabled architectures list from thrust
  foreach (arch IN LISTS all_archs)
    if (THRUST_ENABLE_COMPUTE_${arch})
      set(CUB_ENABLE_COMPUTE_${arch} True)
      list(APPEND enabled_archs ${arch})
      string(APPEND arch_message " sm_${arch}")
    else()
      set(CUB_ENABLE_COMPUTE_${arch} False)
    endif()
  endforeach()

  # Otherwise create cache options and build the flags ourselves:
else() # NOT CUB_IN_THRUST

  # Find the highest arch:
  list(SORT all_archs)
  list(LENGTH all_archs max_idx)
  math(EXPR max_idx "${max_idx} - 1")
  list(GET all_archs ${max_idx} highest_arch)

  option(CUB_DISABLE_ARCH_BY_DEFAULT
    "If ON, then all CUDA architectures are disabled on the initial CMake run."
    OFF
  )

  set(option_init ON)
  if (CUB_DISABLE_ARCH_BY_DEFAULT)
    set(option_init OFF)
  endif()

  set(arch_flags)
  foreach (arch IN LISTS all_archs)
    option(CUB_ENABLE_COMPUTE_${arch}
      "Enable code generation for sm_${arch}."
      ${option_init}
    )
    if (CUB_ENABLE_COMPUTE_${arch})
      list(APPEND enabled_archs ${arch})
      string(APPEND arch_flags " -gencode arch=compute_${arch},code=sm_${arch}")
      string(APPEND arch_message " sm_${arch}")
    endif()
  endforeach()

  option(CUB_ENABLE_COMPUTE_FUTURE
    "Enable code generation for tests for compute_${highest_arch}"
    ${option_init}
  )
  if (CUB_ENABLE_COMPUTE_FUTURE)
    string(APPEND arch_flags
      " -gencode arch=compute_${highest_arch},code=compute_${highest_arch}"
    )
    string(APPEND arch_message " compute_${highest_arch}")
  endif()

  # TODO Once CMake 3.18 is required, use the CUDA_ARCHITECTURE target props
  string(APPEND CMAKE_CUDA_FLAGS "${arch_flags}")
endif()

message(STATUS ${arch_message})

# Create a variable containing the minimal target arch for tests
list(SORT enabled_archs)
list(GET enabled_archs 0 CUB_MINIMAL_ENABLED_ARCH)

#
# RDC options:
#

option(CUB_ENABLE_TESTS_WITH_RDC
  "Build all CUB tests with RDC; tests that require RDC are not affected by this option."
  OFF
)

option(CUB_ENABLE_EXAMPLES_WITH_RDC
  "Build all CUB examples with RDC; examples which require RDC are not affected by this option."
  OFF
)

# Check for RDC/SM compatibility and error/warn if necessary
set(no_rdc_archs 53 62 72)
set(rdc_supported True)
foreach (arch IN LISTS no_rdc_archs)
  if (CUB_ENABLE_COMPUTE_${arch})
    set(rdc_supported False)
    break()
  endif()
endforeach()

set(rdc_opts
  CUB_ENABLE_TESTS_WITH_RDC
  CUB_ENABLE_EXAMPLES_WITH_RDC
)
set(rdc_requested False)
foreach (rdc_opt IN LISTS rdc_opts)
  if (${rdc_opt})
    set(rdc_requested True)
    break()
  endif()
endforeach()

if (rdc_requested AND NOT rdc_supported)
  string(JOIN ", " no_rdc ${no_rdc_archs})
  string(JOIN "\n" opts ${rdc_opts})
  message(FATAL_ERROR
    "Architectures {${no_rdc}} do not support RDC and are incompatible with "
    "these options:\n${opts}"
  )
endif()
