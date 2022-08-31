/*!
 *  Copyright (c) 2015 by Contributors
 * \file s3_filesys.h
 * \brief S3 access module
 * \author Tianqi Chen
 */
#ifndef DMLC_IO_S3_FILESYS_H_
#define DMLC_IO_S3_FILESYS_H_

#include <dmlc/filesystem.h>
#include <vector>
#include <string>

namespace dmlc {
namespace io {
/*! \brief S3 filesystem */
class S3FileSystem : public FileSystem {
 public:
  /*! \brief destructor */
  virtual ~S3FileSystem() {}

  /*!
   * \brief Sets S3 access credentials
   * \param s3_access_id The S3 Access Key ID
   * \param s3_secret_key The S3 Secret Key
   * \return the information about the file
   */
  void SetCredentials(const std::string& s3_access_id,
                      const std::string& s3_secret_key);

  /*!
   * \brief get information about a path
   * \param path the path to the file
   * \return the information about the file
   */
  virtual FileInfo GetPathInfo(const URI &path);
  /*!
   * \brief list files in a directory
   * \param path to the file
   * \param out_list the output information about the files
   */
  virtual void ListDirectory(const URI &path, std::vector<FileInfo> *out_list);
  /*!
   * \brief open a stream, will report error and exit if bad thing happens
   * NOTE: the Stream can continue to work even when filesystem was destructed
   * \param path path to file
   * \param uri the uri of the input
   * \param flag can be "w", "r", "a"
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  virtual Stream *Open(const URI &path, const char* const flag, bool allow_null);
  /*!
   * \brief open a seekable stream for read
   * \param path the path to the file
   * \param allow_null whether NULL can be returned, or directly report error
   * \return the created stream, can be NULL when allow_null == true and file do not exist
   */
  virtual SeekStream *OpenForRead(const URI &path, bool allow_null);
  /*!
   * \brief get a singleton of S3FileSystem when needed
   * \return a singleton instance
   */
  inline static S3FileSystem *GetInstance(void) {
    static S3FileSystem instance;
    return &instance;
  }

 private:
  /*! \brief constructor */
  S3FileSystem();
  /*! \brief S3 access id */
  std::string s3_access_id_;
  /*! \brief S3 secret key */
  std::string s3_secret_key_;
  /*! \brief S3 session token */
  std::string s3_session_token_;
  /*! \brief S3 region*/
  std::string s3_region_;
  /*! \brief S3 endpoint*/
  std::string s3_endpoint_;
  /*! \brief S3 verify ssl*/
  bool s3_verify_ssl_;
  bool s3_is_aws_;

  /*!
   * \brief try to get information about a path
   * \param path the path to the file
   * \param out_info holds the path info
   * \return return false when path do not exist
   */
  bool TryGetPathInfo(const URI &path, FileInfo *info);

  /*!
  * \brief list the objects in the bucket with prefix specified by path.name
  * \param path the path to query
  * \param out_list stores the output results which match given prefix
  */
  void ListObjects(const URI &path, std::vector<FileInfo> *out_list);
};
}  // namespace io
}  // namespace dmlc
#endif  // DMLC_IO_S3_FILESYS_H_
