file(GLOB_RECURSE example_srcs
  RELATIVE "${CMAKE_CURRENT_LIST_DIR}"
  CONFIGURE_DEPENDS
  example_*.cu
)

foreach (cub_target IN LISTS CUB_TARGETS)
  foreach (example_src IN LISTS example_srcs)
    get_filename_component(example_name "${example_src}" NAME_WE)
    string(REGEX REPLACE
      "^example_device_" "device."
      example_name "${example_name}"
    )
    cub_add_example(target_name ${example_name} "${example_src}" ${cub_target})
  endforeach()
endforeach()
