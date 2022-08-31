/*!
 *  Copyright (c) 2015 by Contributors
 * \file io.h
 * \brief defines serializable interface of dmlc
 */
#ifndef DMLC_IO_H_
#define DMLC_IO_H_
#include <cstdio>
#include <string>
#include <cstring>
#include <vector>
#include <istream>
#include <ostream>
#include <streambuf>
#include "./logging.h"

// include uint64_t only to make io standalone
#ifdef _MSC_VER
/*! \brief uint64 */
typedef unsigned __int64 uint64_t;
#else
#include <inttypes.h>
#endif

/*! \brief namespace for dmlc */
namespace dmlc {
/*!
 * \brief interface of stream I/O for serialization
 */
class Stream {  // NOLINT(*)
 public:
  /*!
   * \brief reads data from a stream
   * \param ptr pointer to a memory buffer
   * \param size block size
   * \return the size of data read
   */
  virtual size_t Read(void *ptr, size_t size) = 0;
  /*!
   * \brief writes data to a stream
   * \param ptr pointer to a memory buffer
   * \param size block size
   */
  virtual void Write(const void *ptr, size_t size) = 0;
  /*! \brief virtual destructor */
  virtual ~Stream(void) {}
  /*!
   * \brief generic factory function
   *  create an stream, the stream will close the underlying files upon deletion
   *
   * \param uri the uri of the input currently we support
   *            hdfs://, s3://, and file:// by default file:// will be used
   * \param flag can be "w", "r", "a"
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  static Stream *Create(const char *uri,
                        const char* const flag,
                        bool allow_null = false);
  // helper functions to write/read different data structures
  /*!
   * \brief writes a data to stream.
   *
   * dmlc::Stream support Write/Read of most STL composites and base types.
   * If the data type is not supported, a compile time error will be issued.
   *
   * This function is endian-aware,
   * the output endian defined by DMLC_IO_USE_LITTLE_ENDIAN
   *
   * \param data data to be written
   * \tparam T the data type to be written
   */
  template<typename T>
  inline void Write(const T &data);
  /*!
   * \brief loads a data from stream.
   *
   * dmlc::Stream support Write/Read of most STL composites and base types.
   * If the data type is not supported, a compile time error will be issued.
   *
   * This function is endian-aware,
   * the input endian defined by DMLC_IO_USE_LITTLE_ENDIAN
   *
   * \param out_data place holder of data to be deserialized
   * \return whether the load was successful
   */
  template<typename T>
  inline bool Read(T *out_data);
  /*!
   * \brief Endian aware write array of data.
   * \param data The data pointer
   * \param num_elems Number of elements
   * \tparam T the data type.
   */
  template<typename T>
  inline void WriteArray(const T* data, size_t num_elems);
  /*!
   * \brief Endian aware read array of data.
   * \param data The data pointer
   * \param num_elems Number of elements
   * \tparam T the data type.
   * \return whether the load was successful
   */
  template<typename T>
  inline bool ReadArray(T* data, size_t num_elems);
};

/*! \brief interface of i/o stream that support seek */
class SeekStream: public Stream {
 public:
  // virtual destructor
  virtual ~SeekStream(void) {}
  /*! \brief seek to certain position of the file */
  virtual void Seek(size_t pos) = 0;
  /*! \brief tell the position of the stream */
  virtual size_t Tell(void) = 0;
  /*!
   * \brief generic factory function
   *  create an SeekStream for read only,
   *  the stream will close the underlying files upon deletion
   *  error will be reported and the system will exit when create failed
   * \param uri the uri of the input currently we support
   *            hdfs://, s3://, and file:// by default file:// will be used
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  static SeekStream *CreateForRead(const char *uri,
                                   bool allow_null = false);
};

/*! \brief interface for serializable objects */
class Serializable {
 public:
  /*! \brief virtual destructor */
  virtual ~Serializable() {}
  /*!
  * \brief load the model from a stream
  * \param fi stream where to load the model from
  */
  virtual void Load(Stream *fi) = 0;
  /*!
  * \brief saves the model to a stream
  * \param fo stream where to save the model to
  */
  virtual void Save(Stream *fo) const = 0;
};

/*!
 * \brief input split creates that allows reading
 *  of records from split of data,
 *  independent part that covers all the dataset
 *
 *  see InputSplit::Create for definition of record
 */
class InputSplit {
 public:
  /*! \brief a blob of memory region */
  struct Blob {
    /*! \brief points to start of the memory region */
    void *dptr;
    /*! \brief size of the memory region */
    size_t size;
  };
  /*!
   * \brief hint the inputsplit how large the chunk size
   *  it should return when implementing NextChunk
   *  this is a hint so may not be enforced,
   *  but InputSplit will try adjust its internal buffer
   *  size to the hinted value
   * \param chunk_size the chunk size
   */
  virtual void HintChunkSize(size_t chunk_size) {}
  /*! \brief get the total size of the InputSplit */
  virtual size_t GetTotalSize(void) = 0;
  /*! \brief reset the position of InputSplit to beginning */
  virtual void BeforeFirst(void) = 0;
  /*!
   * \brief get the next record, the returning value
   *   is valid until next call to NextRecord, NextChunk or NextBatch
   *   caller can modify the memory content of out_rec
   *
   *   For text, out_rec contains a single line
   *   For recordio, out_rec contains one record content(with header striped)
   *
   * \param out_rec used to store the result
   * \return true if we can successfully get next record
   *     false if we reached end of split
   * \sa InputSplit::Create for definition of record
   */
  virtual bool NextRecord(Blob *out_rec) = 0;
  /*!
   * \brief get a chunk of memory that can contain multiple records,
   *  the caller needs to parse the content of the resulting chunk,
   *  for text file, out_chunk can contain data of multiple lines
   *  for recordio, out_chunk can contain multiple records(including headers)
   *
   *  This function ensures there won't be partial record in the chunk
   *  caller can modify the memory content of out_chunk,
   *  the memory is valid until next call to NextRecord, NextChunk or NextBatch
   *
   *  Usually NextRecord is sufficient, NextChunk can be used by some
   *  multi-threaded parsers to parse the input content
   *
   * \param out_chunk used to store the result
   * \return true if we can successfully get next record
   *     false if we reached end of split
   * \sa InputSplit::Create for definition of record
   * \sa RecordIOChunkReader to parse recordio content from out_chunk
   */
  virtual bool NextChunk(Blob *out_chunk) = 0;
  /*!
   * \brief get a chunk of memory that can contain multiple records,
   *  with hint for how many records is needed,
   *  the caller needs to parse the content of the resulting chunk,
   *  for text file, out_chunk can contain data of multiple lines
   *  for recordio, out_chunk can contain multiple records(including headers)
   *
   *  This function ensures there won't be partial record in the chunk
   *  caller can modify the memory content of out_chunk,
   *  the memory is valid until next call to NextRecord, NextChunk or NextBatch
   *
   *
   * \param out_chunk used to store the result
   * \param n_records used as a hint for how many records should be returned, may be ignored
   * \return true if we can successfully get next record
   *     false if we reached end of split
   * \sa InputSplit::Create for definition of record
   * \sa RecordIOChunkReader to parse recordio content from out_chunk
   */
  virtual bool NextBatch(Blob *out_chunk, size_t n_records) {
    return NextChunk(out_chunk);
  }
  /*! \brief destructor*/
  virtual ~InputSplit(void) DMLC_THROW_EXCEPTION {}
  /*!
   * \brief reset the Input split to a certain part id,
   *  The InputSplit will be pointed to the head of the new specified segment.
   *  This feature may not be supported by every implementation of InputSplit.
   * \param part_index The part id of the new input.
   * \param num_parts The total number of parts.
   */
  virtual void ResetPartition(unsigned part_index, unsigned num_parts) = 0;
  /*!
   * \brief factory function:
   *  create input split given a uri
   * \param uri the uri of the input, can contain hdfs prefix
   * \param part_index the part id of current input
   * \param num_parts total number of splits
   * \param type type of record
   *   List of possible types: "text", "recordio", "indexed_recordio"
   *     - "text":
   *         text file, each line is treated as a record
   *         input split will split on '\\n' or '\\r'
   *     - "recordio":
   *         binary recordio file, see recordio.h
   *     - "indexed_recordio":
   *         binary recordio file with index, see recordio.h
   * \return a new input split
   * \sa InputSplit::Type
   */
  static InputSplit* Create(const char *uri,
                            unsigned part_index,
                            unsigned num_parts,
                            const char *type);
  /*!
   * \brief factory function:
   *  create input split given a uri for input and index
   * \param uri the uri of the input, can contain hdfs prefix
   * \param index_uri the uri of the index, can contain hdfs prefix
   * \param part_index the part id of current input
   * \param num_parts total number of splits
   * \param type type of record
   *   List of possible types: "text", "recordio", "indexed_recordio"
   *     - "text":
   *         text file, each line is treated as a record
   *         input split will split on '\\n' or '\\r'
   *     - "recordio":
   *         binary recordio file, see recordio.h
   *     - "indexed_recordio":
   *         binary recordio file with index, see recordio.h
   * \param shuffle whether to shuffle the output from the InputSplit,
   *                supported only by "indexed_recordio" type.
   *                Defaults to "false"
   * \param seed random seed to use in conjunction with the "shuffle"
   *             option. Defaults to 0
   * \param batch_size a hint to InputSplit what is the intended number
   *                   of examples return per batch. Used only by
   *                   "indexed_recordio" type
   * \param recurse_directories whether to recursively traverse directories
   * \return a new input split
   * \sa InputSplit::Type
   */
  static InputSplit* Create(const char *uri,
                            const char *index_uri,
                            unsigned part_index,
                            unsigned num_parts,
                            const char *type,
                            const bool shuffle = false,
                            const int seed = 0,
                            const size_t batch_size = 256,
                            const bool recurse_directories = false);
};

#ifndef _LIBCPP_SGX_NO_IOSTREAMS
/*!
 * \brief a std::ostream class that can can wrap Stream objects,
 *  can use ostream with that output to underlying Stream
 *
 * Usage example:
 * \code
 *
 *   Stream *fs = Stream::Create("hdfs:///test.txt", "w");
 *   dmlc::ostream os(fs);
 *   os << "hello world" << std::endl;
 *   delete fs;
 * \endcode
 */
class ostream : public std::basic_ostream<char> {
 public:
  /*!
   * \brief construct std::ostream type
   * \param stream the Stream output to be used
   * \param buffer_size internal streambuf size
   */
  explicit ostream(Stream *stream,
                   size_t buffer_size = (1 << 10))
      : std::basic_ostream<char>(NULL), buf_(buffer_size) {
    this->set_stream(stream);
  }
  // explictly synchronize the buffer
  virtual ~ostream() DMLC_NO_EXCEPTION {
    buf_.pubsync();
  }
  /*!
   * \brief set internal stream to be stream, reset states
   * \param stream new stream as output
   */
  inline void set_stream(Stream *stream) {
    buf_.set_stream(stream);
    this->rdbuf(&buf_);
  }

  /*! \return how many bytes we written so far */
  inline size_t bytes_written(void) const {
    return buf_.bytes_out();
  }

 private:
  // internal streambuf
  class OutBuf : public std::streambuf {
   public:
    explicit OutBuf(size_t buffer_size)
        : stream_(NULL), buffer_(buffer_size), bytes_out_(0) {
      if (buffer_size == 0) buffer_.resize(2);
    }
    // set stream to the buffer
    inline void set_stream(Stream *stream);

    inline size_t bytes_out() const { return bytes_out_; }
   private:
    /*! \brief internal stream by StreamBuf */
    Stream *stream_;
    /*! \brief internal buffer */
    std::vector<char> buffer_;
    /*! \brief number of bytes written so far */
    size_t bytes_out_;
    // override sync
    inline int_type sync(void);
    // override overflow
    inline int_type overflow(int c);
  };
  /*! \brief buffer of the stream */
  OutBuf buf_;
};

/*!
 * \brief a std::istream class that can can wrap Stream objects,
 *  can use istream with that output to underlying Stream
 *
 * Usage example:
 * \code
 *
 *   Stream *fs = Stream::Create("hdfs:///test.txt", "r");
 *   dmlc::istream is(fs);
 *   is >> mydata;
 *   delete fs;
 * \endcode
 */
class istream : public std::basic_istream<char> {
 public:
  /*!
   * \brief construct std::ostream type
   * \param stream the Stream output to be used
   * \param buffer_size internal buffer size
   */
  explicit istream(Stream *stream,
                   size_t buffer_size = (1 << 10))
      : std::basic_istream<char>(NULL), buf_(buffer_size) {
    this->set_stream(stream);
  }
  virtual ~istream() DMLC_NO_EXCEPTION {}
  /*!
   * \brief set internal stream to be stream, reset states
   * \param stream new stream as output
   */
  inline void set_stream(Stream *stream) {
    buf_.set_stream(stream);
    this->rdbuf(&buf_);
  }
  /*! \return how many bytes we read so far */
  inline size_t bytes_read(void) const {
    return buf_.bytes_read();
  }

 private:
  // internal streambuf
  class InBuf : public std::streambuf {
   public:
    explicit InBuf(size_t buffer_size)
        : stream_(NULL), bytes_read_(0),
          buffer_(buffer_size) {
      if (buffer_size == 0) buffer_.resize(2);
    }
    // set stream to the buffer
    inline void set_stream(Stream *stream);
    // return how many bytes read so far
    inline size_t bytes_read(void) const {
      return bytes_read_;
    }
   private:
    /*! \brief internal stream by StreamBuf */
    Stream *stream_;
    /*! \brief how many bytes we read so far */
    size_t bytes_read_;
    /*! \brief internal buffer */
    std::vector<char> buffer_;
    // override underflow
    inline int_type underflow();
  };
  /*! \brief input buffer */
  InBuf buf_;
};
#endif
}  // namespace dmlc

#include "./serializer.h"

namespace dmlc {
// implementations of inline functions
template<typename T>
inline void Stream::Write(const T &data) {
  serializer::Handler<T>::Write(this, data);
}
template<typename T>
inline bool Stream::Read(T *out_data) {
  return serializer::Handler<T>::Read(this, out_data);
}

template<typename T>
inline void Stream::WriteArray(const T* data, size_t num_elems) {
  for (size_t i = 0; i < num_elems; ++i) {
    this->Write<T>(data[i]);
  }
}

template<typename T>
inline bool Stream::ReadArray(T* data, size_t num_elems) {
  for (size_t i = 0; i < num_elems; ++i) {
    if (!this->Read<T>(data + i)) return false;
  }
  return true;
}

#ifndef _LIBCPP_SGX_NO_IOSTREAMS
// implementations for ostream
inline void ostream::OutBuf::set_stream(Stream *stream) {
  if (stream_ != NULL) this->pubsync();
  this->stream_ = stream;
  this->setp(&buffer_[0], &buffer_[0] + buffer_.size() - 1);
}
inline int ostream::OutBuf::sync(void) {
  if (stream_ == NULL) return -1;
  std::ptrdiff_t n = pptr() - pbase();
  stream_->Write(pbase(), n);
  this->pbump(-static_cast<int>(n));
  bytes_out_ += n;
  return 0;
}
inline int ostream::OutBuf::overflow(int c) {
  *(this->pptr()) = c;
  std::ptrdiff_t n = pptr() - pbase();
  this->pbump(-static_cast<int>(n));
  if (c == EOF) {
    stream_->Write(pbase(), n);
    bytes_out_ += n;
  } else {
    stream_->Write(pbase(), n + 1);
    bytes_out_ += n + 1;
  }
  return c;
}

// implementations for istream
inline void istream::InBuf::set_stream(Stream *stream) {
  stream_ = stream;
  this->setg(&buffer_[0], &buffer_[0], &buffer_[0]);
}
inline int istream::InBuf::underflow() {
  char *bhead = &buffer_[0];
  if (this->gptr() == this->egptr()) {
    size_t sz = stream_->Read(bhead, buffer_.size());
    this->setg(bhead, bhead, bhead + sz);
    bytes_read_ += sz;
  }
  if (this->gptr() == this->egptr()) {
    return traits_type::eof();
  } else {
    return traits_type::to_int_type(*gptr());
  }
}
#endif

namespace io {
/*! \brief common data structure for URI */
struct URI {
  /*! \brief protocol */
  std::string protocol;
  /*!
   * \brief host name, namenode for HDFS, bucket name for s3
   */
  std::string host;
  /*! \brief name of the path */
  std::string name;
  /*! \brief enable default constructor */
  URI(void) {}
  /*!
   * \brief construct from URI string
   */
  explicit URI(const char *uri) {
    const char *p = std::strstr(uri, "://");
    if (p == NULL) {
      name = uri;
    } else {
      protocol = std::string(uri, p - uri + 3);
      uri = p + 3;
      p = std::strchr(uri, '/');
      if (p == NULL) {
        host = uri; name = '/';
      } else {
        host = std::string(uri, p - uri);
        name = p;
      }
    }
  }
  /*! \brief string representation */
  inline std::string str(void) const {
    return protocol + host + name;
  }
};

/*! \brief type of file */
enum FileType {
  /*! \brief the file is file */
  kFile,
  /*! \brief the file is directory */
  kDirectory
};

/*! \brief use to store file information */
struct FileInfo {
  /*! \brief full path to the file */
  URI path;
  /*! \brief the size of the file */
  size_t size;
  /*! \brief the type of the file */
  FileType type;
  /*! \brief default constructor */
  FileInfo() : size(0), type(kFile) {}
};

/*! \brief file system system interface */
class FileSystem {
 public:
  /*!
   * \brief get singleton of filesystem instance according to URI
   * \param path can be s3://..., hdfs://..., file://...,
   *            empty string(will return local)
   * \return a corresponding filesystem, report error if
   *         we cannot find a matching system
   */
  static FileSystem *GetInstance(const URI &path);
  /*! \brief virtual destructor */
  virtual ~FileSystem() {}
  /*!
   * \brief get information about a path
   * \param path the path to the file
   * \return the information about the file
   */
  virtual FileInfo GetPathInfo(const URI &path) = 0;
  /*!
   * \brief list files in a directory
   * \param path to the file
   * \param out_list the output information about the files
   */
  virtual void ListDirectory(const URI &path, std::vector<FileInfo> *out_list) = 0;
  /*!
   * \brief list files in a directory recursively using ListDirectory
   * \param path to the file
   * \param out_list the output information about the files
   */
  virtual void ListDirectoryRecursive(const URI &path,
                                      std::vector<FileInfo> *out_list);
  /*!
   * \brief open a stream
   * \param path path to file
   * \param flag can be "w", "r", "a
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  virtual Stream *Open(const URI &path,
                       const char* const flag,
                       bool allow_null = false) = 0;
  /*!
   * \brief open a seekable stream for read
   * \param path the path to the file
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  virtual SeekStream *OpenForRead(const URI &path,
                                  bool allow_null = false) = 0;
};

}  // namespace io
}  // namespace dmlc
#endif  // DMLC_IO_H_
