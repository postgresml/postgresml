/*!
 *  Copyright (c) 2015 by Contributors
 * \file base.h
 * \brief defines configuration macros
 */
#ifndef DMLC_BASE_H_
#define DMLC_BASE_H_

/*! \brief whether use glog for logging */
#ifndef DMLC_USE_GLOG
#define DMLC_USE_GLOG 0
#endif

/*
 * The preprocessor definition DMLC_USE_LOGGING_LIBRARY determines whether to
 * use a user-defined logging library. If defined, dmlc will not define the
 * macros CHECK() and LOG() and instead locate CHECK() and LOG() from the value
 * of DMLC_USE_LOGGING_LIBRARY. The DMLC_USE_LOGGING_LIBRARY macro shall be of
 * form <my_logging.h>:
 *
 * #define DMLC_USE_LOGGING_LIBRARY <my_logging.h>
 *
 * Make sure to define CHECK() and LOG() macros in the provided header;
 * otherwise the build will fail.
 */

/*!
 * \brief whether throw dmlc::Error instead of
 *  directly calling abort when FATAL error occured
 *  NOTE: this may still not be perfect.
 *  do not use FATAL and CHECK in destructors
 */
#ifndef DMLC_LOG_FATAL_THROW
#define DMLC_LOG_FATAL_THROW 1
#endif

/*!
 * \brief whether always log a message before throw
 * This can help identify the error that cannot be catched.
 */
#ifndef DMLC_LOG_BEFORE_THROW
#define DMLC_LOG_BEFORE_THROW 0
#endif

/*!
 * \brief Whether to use customized logger,
 * whose output can be decided by other libraries.
 */
#ifndef DMLC_LOG_CUSTOMIZE
#define DMLC_LOG_CUSTOMIZE 0
#endif

/*!
 * \brief Whether to enable debug logging feature.
 */
#ifndef DMLC_LOG_DEBUG
#ifdef NDEBUG
#define DMLC_LOG_DEBUG 0
#else
#define DMLC_LOG_DEBUG 1
#endif
#endif

/*!
 * \brief Whether to disable date message on the log.
 */
#ifndef DMLC_LOG_NODATE
#define DMLC_LOG_NODATE 0
#endif

/*! \brief whether compile with hdfs support */
#ifndef DMLC_USE_HDFS
#define DMLC_USE_HDFS 0
#endif

/*! \brief whether compile with s3 support */
#ifndef DMLC_USE_S3
#define DMLC_USE_S3 0
#endif

/*! \brief whether or not use parameter server */
#ifndef DMLC_USE_PS
#define DMLC_USE_PS 0
#endif

/*! \brief whether or not use c++11 support */
#ifndef DMLC_USE_CXX11
#if defined(__GXX_EXPERIMENTAL_CXX0X__) || defined(_MSC_VER)
#define DMLC_USE_CXX11 1
#else
#define DMLC_USE_CXX11 (__cplusplus >= 201103L)
#endif
#endif

/*! \brief strict CXX11 support */
#ifndef DMLC_STRICT_CXX11
#if defined(_MSC_VER)
#define DMLC_STRICT_CXX11 1
#else
#define DMLC_STRICT_CXX11 (__cplusplus >= 201103L)
#endif
#endif

/*! \brief Whether cxx11 thread local is supported */
#ifndef DMLC_CXX11_THREAD_LOCAL
#if defined(_MSC_VER)
#define DMLC_CXX11_THREAD_LOCAL (_MSC_VER >= 1900)
#elif defined(__clang__)
#define DMLC_CXX11_THREAD_LOCAL (__has_feature(cxx_thread_local))
#else
#define DMLC_CXX11_THREAD_LOCAL (__cplusplus >= 201103L)
#endif
#endif

/*! \brief Whether to use modern thread local construct */
#ifndef DMLC_MODERN_THREAD_LOCAL
#define DMLC_MODERN_THREAD_LOCAL 1
#endif



/*! \brief whether RTTI is enabled */
#ifndef DMLC_ENABLE_RTTI
#define DMLC_ENABLE_RTTI 1
#endif

/*! \brief whether use fopen64 */
#ifndef DMLC_USE_FOPEN64
#define DMLC_USE_FOPEN64 1
#endif

/// check for C++11 support
#if DMLC_USE_CXX11
#if (!defined(_MSC_VER) && __cplusplus < 201103L) || (defined(_MSC_VER) && _MSC_VER < 1900)
// MSVC doesn't support __cplusplus macro properly until MSVC 2017
// We want to also support MSVC 2015, so manually check _MSC_VER

#pragma message("Compiling without c++11, some features may be disabled")
#undef DMLC_USE_CXX11
#define DMLC_USE_CXX11 0

#endif  // (!defined(_MSC_VER) && __cplusplus < 201103L) || (defined(_MSC_VER) && _MSC_VER < 1900)
#endif  // DMLC_USE_CXX11

/*!
 * \brief Use little endian for binary serialization
 *  if this is set to 0, use big endian.
 */
#ifndef DMLC_IO_USE_LITTLE_ENDIAN
#define DMLC_IO_USE_LITTLE_ENDIAN 1
#endif

/*!
 * \brief Enable std::thread related modules,
 *  Used to disable some module in mingw compile.
 */
#ifndef DMLC_ENABLE_STD_THREAD
#define DMLC_ENABLE_STD_THREAD DMLC_USE_CXX11
#endif

/*! \brief whether enable regex support, actually need g++-4.9 or higher*/
#ifndef DMLC_USE_REGEX
#define DMLC_USE_REGEX DMLC_STRICT_CXX11
#endif

/*! \brief helper macro to supress unused warning */
#if defined(__GNUC__)
#define DMLC_ATTRIBUTE_UNUSED __attribute__((unused))
#else
#define DMLC_ATTRIBUTE_UNUSED
#endif

/*! \brief helper macro to supress Undefined Behavior Sanitizer for a specific function */
#if defined(__clang__)
#define DMLC_SUPPRESS_UBSAN __attribute__((no_sanitize("undefined")))
#elif defined(__GNUC__) && (__GNUC__ * 100 + __GNUC_MINOR__ >= 409)
#define DMLC_SUPPRESS_UBSAN __attribute__((no_sanitize_undefined))
#else
#define DMLC_SUPPRESS_UBSAN
#endif

/*! \brief helper macro to generate string concat */
#define DMLC_STR_CONCAT_(__x, __y) __x##__y
#define DMLC_STR_CONCAT(__x, __y) DMLC_STR_CONCAT_(__x, __y)

/*!
 * \brief Disable copy constructor and assignment operator.
 *
 * If C++11 is supported, both copy and move constructors and
 * assignment operators are deleted explicitly. Otherwise, they are
 * only declared but not implemented. Place this macro in private
 * section if C++11 is not available.
 */
#ifndef DISALLOW_COPY_AND_ASSIGN
#  if DMLC_USE_CXX11
#    define DISALLOW_COPY_AND_ASSIGN(T) \
       T(T const&) = delete; \
       T(T&&) = delete; \
       T& operator=(T const&) = delete; \
       T& operator=(T&&) = delete
#  else
#    define DISALLOW_COPY_AND_ASSIGN(T) \
       T(T const&); \
       T& operator=(T const&)
#  endif
#endif

#ifdef __APPLE__
#  define off64_t off_t
#endif

#ifdef _MSC_VER
#if _MSC_VER < 1900
// NOTE: sprintf_s is not equivalent to snprintf,
// they are equivalent when success, which is sufficient for our case
#define snprintf sprintf_s
#define vsnprintf vsprintf_s
#endif
#else
#ifdef _FILE_OFFSET_BITS
#if _FILE_OFFSET_BITS == 32
#pragma message("Warning: FILE OFFSET BITS defined to be 32 bit")
#endif
#endif

extern "C" {
#include <sys/types.h>
}
#endif

#ifdef _MSC_VER
//! \cond Doxygen_Suppress
typedef signed char int8_t;
typedef __int16 int16_t;
typedef __int32 int32_t;
typedef __int64 int64_t;
typedef unsigned char uint8_t;
typedef unsigned __int16 uint16_t;
typedef unsigned __int32 uint32_t;
typedef unsigned __int64 uint64_t;
//! \endcond
#else
#include <inttypes.h>
#endif
#include <string>
#include <vector>

#if defined(_MSC_VER) && _MSC_VER < 1900
#define noexcept_true throw ()
#define noexcept_false
#define noexcept(a) noexcept_##a
#endif

#if defined(_MSC_VER)
#define DMLC_NO_INLINE __declspec(noinline)
#else
#define DMLC_NO_INLINE __attribute__((noinline))
#endif

#if defined(__GNUC__) || defined(__clang__)
#define DMLC_ALWAYS_INLINE inline __attribute__((__always_inline__))
#elif defined(_MSC_VER)
#define DMLC_ALWAYS_INLINE __forceinline
#else
#define DMLC_ALWAYS_INLINE inline
#endif

#if DMLC_USE_CXX11
#define DMLC_THROW_EXCEPTION noexcept(false)
#define DMLC_NO_EXCEPTION  noexcept(true)
#else
#define DMLC_THROW_EXCEPTION
#define DMLC_NO_EXCEPTION
#endif

/*! \brief namespace for dmlc */
namespace dmlc {
/*!
 * \brief safely get the beginning address of a vector
 * \param vec input vector
 * \return beginning address of a vector
 */
template<typename T>
inline T *BeginPtr(std::vector<T> &vec) {  // NOLINT(*)
  if (vec.size() == 0) {
    return NULL;
  } else {
    return &vec[0];
  }
}
/*!
 * \brief get the beginning address of a const vector
 * \param vec input vector
 * \return beginning address of a vector
 */
template<typename T>
inline const T *BeginPtr(const std::vector<T> &vec) {
  if (vec.size() == 0) {
    return NULL;
  } else {
    return &vec[0];
  }
}
/*!
 * \brief get the beginning address of a string
 * \param str input string
 * \return beginning address of a string
 */
inline char* BeginPtr(std::string &str) {  // NOLINT(*)
  if (str.length() == 0) return NULL;
  return &str[0];
}
/*!
 * \brief get the beginning address of a const string
 * \param str input string
 * \return beginning address of a string
 */
inline const char* BeginPtr(const std::string &str) {
  if (str.length() == 0) return NULL;
  return &str[0];
}
}  // namespace dmlc

#if defined(_MSC_VER) && _MSC_VER < 1900
#define constexpr const
#define alignof __alignof
#endif

/* If fopen64 is not defined by current machine,
   replace fopen64 with std::fopen. Also determine ability to print stack trace
   for fatal error and define DMLC_LOG_STACK_TRACE if stack trace can be
   produced. Always keep this include directive at the bottom of dmlc/base.h */
#ifdef DMLC_CORE_USE_CMAKE
#include <dmlc/build_config.h>
#else
#include <dmlc/build_config_default.h>
#endif

#endif  // DMLC_BASE_H_
