/*!
 *  Copyright (c) 2018 by Contributors
 * \file filesystem.h
 * \brief Utilities to manipulate files
 * \author Hyunsu Philip Cho
 */
#ifndef DMLC_FILESYSTEM_H_
#define DMLC_FILESYSTEM_H_

#include <dmlc/logging.h>
#include <dmlc/io.h>
#include <algorithm>
#include <string>
#include <vector>
#include <random>

/* platform specific headers */
#ifdef _WIN32
#define NOMINMAX
#include <windows.h>
#include <Shlwapi.h>
#pragma comment(lib, "Shlwapi.lib")
#else  // _WIN32
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#endif  // _WIN32

namespace dmlc {

/*!
 * \brief Manager class for temporary directories. Whenever a new
 *        TemporaryDirectory object is constructed, a temporary directory is
 *        created. The directory is deleted when the object is deleted or goes
 *        out of scope. Note: no symbolic links are allowed inside the
 *        temporary directory.
 *
 * Usage example:
 * \code
 *
 *   void foo() {
 *     dmlc::TemporaryDirectory tempdir;
 *     // Create a file my_file.txt inside the temporary directory
 *     std::ofstream of(tempdir.path + "/my_file.txt");
 *     // ... write to my_file.txt ...
 *
 *     // ... use my_file.txt
 *
 *     // When tempdir goes out of scope, the temporary directory is deleted
 *   }
 *
 * \endcode
 */
class TemporaryDirectory {
 public:
  /*!
   * \brief Default constructor.
   *        Creates a new temporary directory with a unique name.
   * \param verbose whether to emit extra messages
   */
  explicit TemporaryDirectory(bool verbose = false)
    : verbose_(verbose) {
#if _WIN32
    /* locate the root directory of temporary area */
    char tmproot[MAX_PATH] = {0};
    const DWORD dw_retval = GetTempPathA(MAX_PATH, tmproot);
    if (dw_retval > MAX_PATH || dw_retval == 0) {
      LOG(FATAL) << "TemporaryDirectory(): "
                 << "Could not create temporary directory";
    }
    /* generate a unique 8-letter alphanumeric string */
    const std::string letters = "abcdefghijklmnopqrstuvwxyz0123456789_";
    std::string uniqstr(8, '\0');
    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_int_distribution<int> dis(0, letters.length() - 1);
    std::generate(uniqstr.begin(), uniqstr.end(),
      [&dis, &gen, &letters]() -> char {
        return letters[dis(gen)];
      });
    /* combine paths to get the name of the temporary directory */
    char tmpdir[MAX_PATH] = {0};
    PathCombineA(tmpdir, tmproot, uniqstr.c_str());
    if (!CreateDirectoryA(tmpdir, NULL)) {
      LOG(FATAL) << "TemporaryDirectory(): "
                 << "Could not create temporary directory";
    }
    path = std::string(tmpdir);
#else  // _WIN32
    std::string tmproot; /* root directory of temporary area */
    std::string dirtemplate; /* template for temporary directory name */
    /* Get TMPDIR env variable or fall back to /tmp/ */
    {
      const char* tmpenv = getenv("TMPDIR");
      if (tmpenv) {
        tmproot = std::string(tmpenv);
        // strip trailing forward slashes
        while (tmproot.length() != 0 && tmproot[tmproot.length() - 1] == '/') {
          tmproot.resize(tmproot.length() - 1);
        }
      } else {
        tmproot = "/tmp";
      }
    }
    dirtemplate = tmproot + "/tmpdir.XXXXXX";
    std::vector<char> dirtemplate_buf(dirtemplate.begin(), dirtemplate.end());
    dirtemplate_buf.push_back('\0');
    char* tmpdir = mkdtemp(&dirtemplate_buf[0]);
    if (!tmpdir) {
      LOG(FATAL) << "TemporaryDirectory(): "
                 << "Could not create temporary directory";
    }
    path = std::string(tmpdir);
#endif  // _WIN32
    if (verbose_) {
      LOG(INFO) << "Created temporary directory " << path;
    }
  }

  /*! \brief Destructor. Will perform recursive deletion via RecursiveDelete() */
  ~TemporaryDirectory() {
    RecursiveDelete(path);
  }

  /*! \brief Full path of the temporary directory */
  std::string path;

 private:
  /*! \brief Whether to emit extra messages */
  bool verbose_;

  /*!
   * \brief Determine whether a given path is a symbolic link
   * \param path String representation of path
   */
  inline bool IsSymlink(const std::string& path) {
#ifdef _WIN32
    DWORD attr = GetFileAttributesA(path.c_str());
    CHECK_NE(attr, INVALID_FILE_ATTRIBUTES)
      << "dmlc::TemporaryDirectory::IsSymlink(): Unable to read file attributes";
    return attr & FILE_ATTRIBUTE_REPARSE_POINT;
#else  // _WIN32
    struct stat sb;
    CHECK_EQ(lstat(path.c_str(), &sb), 0)
      << "dmlc::TemporaryDirectory::IsSymlink(): Unable to read file attributes";
    return S_ISLNK(sb.st_mode);
#endif  // _WIN32
  }

  /*!
   * \brief Delete a directory recursively, along with sub-directories and files.
   * \param path String representation of path. It must refer to a directory.
   */
  void RecursiveDelete(const std::string& path);
};

}  // namespace dmlc
#endif  // DMLC_FILESYSTEM_H_
