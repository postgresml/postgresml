# Thrust manages its own copy of these rules. Update ThrustInstallRules.cmake
# if modifying this file.
if (CUB_IN_THRUST)
  return()
endif()

# CUB is a header library; no need to build anything before installing:
set(CMAKE_SKIP_INSTALL_ALL_DEPENDENCY TRUE)

install(DIRECTORY "${CUB_SOURCE_DIR}/cub"
  TYPE INCLUDE
  FILES_MATCHING
    PATTERN "*.cuh"
    PATTERN "*.cmake"
)
