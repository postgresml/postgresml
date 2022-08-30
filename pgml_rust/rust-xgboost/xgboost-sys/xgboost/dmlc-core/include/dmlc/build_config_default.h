/*!
 * Copyright (c) 2018 by Contributors
 * \file build_config_default.h
 * \brief Default detection logic for fopen64 and other symbols.
 *        May be overriden by CMake
 * \author KOLANICH
 */
#ifndef DMLC_BUILD_CONFIG_DEFAULT_H_
#define DMLC_BUILD_CONFIG_DEFAULT_H_

/* default logic for fopen64 */
#if DMLC_USE_FOPEN64 && \
  (!defined(__GNUC__) || (defined __ANDROID__) || (defined __FreeBSD__) \
  || (defined __APPLE__) || ((defined __MINGW32__) && !(defined __MINGW64__)) \
  || (defined __CYGWIN__) )
  #define fopen64 std::fopen
#endif

/* default logic for stack trace */
#if (defined(__GNUC__) && !defined(__MINGW32__)\
     && !defined(__sun) && !defined(__SVR4)\
     && !(defined __MINGW64__) && !(defined __ANDROID__))\
     && !defined(__CYGWIN__) && !defined(__EMSCRIPTEN__)\
     && !defined(__RISCV__) && !defined(__hexagon__)
  #if !defined(DMLC_LOG_STACK_TRACE)
    #define DMLC_LOG_STACK_TRACE 1
    #define DMLC_EXECINFO_H <execinfo.h>
  #else
    #if DMLC_LOG_STACK_TRACE
      #define DMLC_EXECINFO_H <execinfo.h>
    #else
      #define DMLC_EXECINFO_H
    #endif
  #endif
  #ifndef DMLC_LOG_STACK_TRACE_SIZE
  #define DMLC_LOG_STACK_TRACE_SIZE 10
  #endif
#endif

/* default logic for detecting existence of nanosleep() */
#if !(defined _WIN32) || (defined __CYGWIN__)
  #define DMLC_NANOSLEEP_PRESENT
#endif

#endif  // DMLC_BUILD_CONFIG_DEFAULT_H_
