// Copyright by Contributors

#include <dmlc/filesystem.h>
#include <queue>

namespace dmlc {
namespace io {

void FileSystem::ListDirectoryRecursive(const URI &path,
                                        std::vector<FileInfo> *out_list) {
  std::queue<URI> queue;
  queue.push(path);
  while (!queue.empty()) {
    std::vector<FileInfo> dfiles;
    ListDirectory(queue.front(), &dfiles);
    queue.pop();
    for (auto dfile : dfiles) {
      if (dfile.type == kDirectory) {
        queue.push(dfile.path);
      } else {
        out_list->push_back(dfile);
      }
    }
  }
}

}  // namespace io

void TemporaryDirectory::RecursiveDelete(const std::string &path) {
  io::URI uri(path.c_str());
  io::FileSystem* fs = io::FileSystem::GetInstance(uri);
  std::vector<io::FileInfo> file_list;
  fs->ListDirectory(uri, &file_list);
  for (io::FileInfo info : file_list) {
    CHECK(!IsSymlink(info.path.name))
        << "Symlink not supported in TemporaryDirectory";
    if (info.type == io::FileType::kDirectory) {
      RecursiveDelete(info.path.name);
    } else {
      if (std::remove(info.path.name.c_str()) != 0) {
        LOG(INFO) << "Couldn't remove file " << info.path.name
                  << "; you may want to remove it manually";
      }
    }
  }
#if _WIN32
  const bool rmdir_success = (RemoveDirectoryA(path.c_str()) != 0);
#else
  const bool rmdir_success = (rmdir(path.c_str()) == 0);
#endif
  if (rmdir_success) {
    if (verbose_) {
      LOG(INFO) << "Successfully deleted temporary directory " << path;
    }
  } else {
    LOG(INFO) << "~TemporaryDirectory(): "
              << "Could not remove temporary directory " << path
              << "; you may want to remove it manually";
  }
}
}  // namespace dmlc
