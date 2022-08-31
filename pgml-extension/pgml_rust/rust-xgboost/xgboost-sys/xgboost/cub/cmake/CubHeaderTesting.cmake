# For every public header, build a translation unit containing `#include <header>`
# to let the compiler try to figure out warnings in that header if it is not otherwise
# included in tests, and also to verify if the headers are modular enough.
# .inl files are not globbed for, because they are not supposed to be used as public
# entrypoints.

# Meta target for all configs' header builds:
add_custom_target(cub.all.headers)

file(GLOB_RECURSE headers
  RELATIVE "${CUB_SOURCE_DIR}/cub"
  CONFIGURE_DEPENDS
  cub/*.cuh
)

set(headertest_srcs)
foreach (header IN LISTS headers)
  set(headertest_src "headers/${header}.cu")
  configure_file("${CUB_SOURCE_DIR}/cmake/header_test.in" "${headertest_src}")
  list(APPEND headertest_srcs "${headertest_src}")
endforeach()

foreach(cub_target IN LISTS CUB_TARGETS)
  cub_get_target_property(config_prefix ${cub_target} PREFIX)

  set(headertest_target ${config_prefix}.headers)
  add_library(${headertest_target} OBJECT ${headertest_srcs})
  target_link_libraries(${headertest_target} PUBLIC ${cub_target})
  cub_clone_target_properties(${headertest_target} ${cub_target})

  add_dependencies(cub.all.headers ${headertest_target})
  add_dependencies(${config_prefix}.all ${headertest_target})
endforeach()
