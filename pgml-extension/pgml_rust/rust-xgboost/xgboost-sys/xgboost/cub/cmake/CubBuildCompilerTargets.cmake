#
# This file defines the `cub_build_compiler_targets()` function, which
# creates the following interface targets:
#
# cub.compiler_interface
# - Interface target providing compiler-specific options needed to build
#   Thrust's tests, examples, etc.

function(cub_build_compiler_targets)
  set(cxx_compile_definitions)
  set(cxx_compile_options)

  if ("MSVC" STREQUAL "${CMAKE_CXX_COMPILER_ID}")
    # TODO Enable /Wall
    append_option_if_available("/WX" cxx_compile_options)

    # Disabled loss-of-data conversion warnings.
    # TODO Re-enable.
    append_option_if_available("/wd4244" cxx_compile_options)
    append_option_if_available("/wd4267" cxx_compile_options)

    # Suppress numeric conversion-to-bool warnings.
    # TODO Re-enable.
    append_option_if_available("/wd4800" cxx_compile_options)

    # Disable warning about applying unary operator- to unsigned type.
    append_option_if_available("/wd4146" cxx_compile_options)

    # Some tests require /bigobj to fit everything into their object files:
    append_option_if_available("/bigobj" cxx_compile_options)
  else()
    append_option_if_available("-Werror" cxx_compile_options)
    append_option_if_available("-Wall" cxx_compile_options)
    append_option_if_available("-Wextra" cxx_compile_options)
    append_option_if_available("-Winit-self" cxx_compile_options)
    append_option_if_available("-Woverloaded-virtual" cxx_compile_options)
    append_option_if_available("-Wcast-qual" cxx_compile_options)
    append_option_if_available("-Wno-cast-align" cxx_compile_options)
    append_option_if_available("-Wno-long-long" cxx_compile_options)
    append_option_if_available("-Wno-variadic-macros" cxx_compile_options)
    append_option_if_available("-Wno-unused-function" cxx_compile_options)
    append_option_if_available("-Wno-unused-variable" cxx_compile_options)

    # CUB uses deprecated texture functions (cudaBindTexture, etc). These
    # need to be replaced, but silence the warnings for now.
    append_option_if_available("-Wno-deprecated-declarations" cxx_compile_options)
  endif()

  if ("GNU" STREQUAL "${CMAKE_CXX_COMPILER_ID}")
    if (CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL 4.5)
      # This isn't available until GCC 4.3, and misfires on TMP code until
      # GCC 4.5.
      append_option_if_available("-Wlogical-op" cxx_compile_options)
    endif()

    if (CMAKE_CXX_COMPILER_VERSION VERSION_GREATER_EQUAL 7.3)
      # GCC 7.3 complains about name mangling changes due to `noexcept`
      # becoming part of the type system; we don't care.
      append_option_if_available("-Wno-noexcept-type" cxx_compile_options)
    endif()
  endif()

  if (("Clang" STREQUAL "${CMAKE_CXX_COMPILER_ID}") OR
      ("XL" STREQUAL "${CMAKE_CXX_COMPILER_ID}"))
    # xlC and Clang warn about unused parameters in uninstantiated templates.
    # This causes xlC to choke on the OMP backend, which is mostly #ifdef'd out
    # (and thus has unused parameters) when you aren't using it.
    append_option_if_available("-Wno-unused-parameters" cxx_compile_options)
  endif()

  if ("Clang" STREQUAL "${CMAKE_CXX_COMPILER_ID}")
    # -Wunneeded-internal-declaration misfires in the unit test framework
    # on older versions of Clang.
    append_option_if_available("-Wno-unneeded-internal-declaration" cxx_compile_options)
  endif()

  add_library(cub.compiler_interface INTERFACE)

  foreach (cxx_option IN LISTS cxx_compile_options)
    target_compile_options(cub.compiler_interface INTERFACE
      $<$<COMPILE_LANGUAGE:CXX>:${cxx_option}>
      # Only use -Xcompiler with NVCC, not Feta.
      #
      # CMake can't split genexs, so this can't be formatted better :(
      # This is:
      # if (using CUDA and CUDA_COMPILER is NVCC) add -Xcompiler=opt:
      $<$<AND:$<COMPILE_LANGUAGE:CUDA>,$<CUDA_COMPILER_ID:NVIDIA>>:-Xcompiler=${cxx_option}>
    )
  endforeach()

  # Add these for both CUDA and CXX targets:
  target_compile_definitions(cub.compiler_interface INTERFACE
    ${cxx_compile_definitions}
  )

  # Promote warnings and display diagnostic numbers for nvcc:
  target_compile_options(cub.compiler_interface INTERFACE
    # If using CUDA w/ NVCC...
    $<$<AND:$<COMPILE_LANGUAGE:CUDA>,$<CUDA_COMPILER_ID:NVIDIA>>:-Xcudafe=--display_error_number>
    $<$<AND:$<COMPILE_LANGUAGE:CUDA>,$<CUDA_COMPILER_ID:NVIDIA>>:-Xcudafe=--promote_warnings>
  )
endfunction()
