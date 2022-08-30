/*!
 *  Copyright (c) 2017 by Contributors
 * \file endian.h
 * \brief Endian testing, need c++11
 */
#ifndef DMLC_ENDIAN_H_
#define DMLC_ENDIAN_H_

#include "./base.h"

#ifdef DMLC_CMAKE_LITTLE_ENDIAN
  // If compiled with CMake, use CMake's endian detection logic
  #define DMLC_LITTLE_ENDIAN DMLC_CMAKE_LITTLE_ENDIAN
#else
  #if defined(__APPLE__) || defined(_WIN32)
    #define DMLC_LITTLE_ENDIAN 1
  #elif defined(__GLIBC__) || defined(__GNU_LIBRARY__) \
        || defined(__ANDROID__) || defined(__RISCV__)
    #include <endian.h>
    #define DMLC_LITTLE_ENDIAN (__BYTE_ORDER == __LITTLE_ENDIAN)
  #elif defined(__FreeBSD__) || defined(__OpenBSD__)
    #include <sys/endian.h>
    #define DMLC_LITTLE_ENDIAN (_BYTE_ORDER == _LITTLE_ENDIAN)
  #elif defined(__EMSCRIPTEN__) || defined(__hexagon__)
    #define DMLC_LITTLE_ENDIAN 1
  #elif defined(__sun) || defined(sun)
    #include <sys/isa_defs.h>
    #if defined(_LITTLE_ENDIAN)
      #define DMLC_LITTLE_ENDIAN 1
    #else
      #define DMLC_LITTLE_ENDIAN 0
    #endif
  #else
    #error "Unable to determine endianness of your machine; use CMake to compile"
  #endif
#endif

/*! \brief whether serialize using little endian */
#define DMLC_IO_NO_ENDIAN_SWAP (DMLC_LITTLE_ENDIAN == DMLC_IO_USE_LITTLE_ENDIAN)

namespace dmlc {

/*!
 * \brief A generic inplace byte swapping function.
 * \param data The data pointer.
 * \param elem_bytes The number of bytes of the data elements
 * \param num_elems Number of elements in the data.
 * \note Always try pass in constant elem_bytes to enable
 *       compiler optimization
 */
inline void ByteSwap(void* data, size_t elem_bytes, size_t num_elems) {
  for (size_t i = 0; i < num_elems; ++i) {
    uint8_t* bptr = reinterpret_cast<uint8_t*>(data) + elem_bytes * i;
    for (size_t j = 0; j < elem_bytes / 2; ++j) {
      uint8_t v = bptr[elem_bytes - 1 - j];
      bptr[elem_bytes - 1 - j] = bptr[j];
      bptr[j] = v;
    }
  }
}

}  // namespace dmlc
#endif  // DMLC_ENDIAN_H_
