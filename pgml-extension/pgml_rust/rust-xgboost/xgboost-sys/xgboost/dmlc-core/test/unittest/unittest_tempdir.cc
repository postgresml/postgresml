#include <dmlc/filesystem.h>
#include <gtest/gtest.h>
#include <fstream>
#include <string>
#include <queue>
#include <utility>

#ifdef _WIN32
#include <direct.h>
#else  // _WIN32
#include <sys/stat.h>
#endif  // _WIN32

static inline void MakeDirectory(const std::string& path) {
#ifdef _WIN32
  CHECK_EQ(_mkdir(path.c_str()), 0) << "Failed to make directory " << path;
#else  // _WIN32
  CHECK_EQ(mkdir(path.c_str(), 0777), 0) << "Failed to make directory " << path;
#endif  // _WIN32
}

TEST(TemporaryDirectory, test_basic) {
  std::string tempdir_path;
  {
    dmlc::TemporaryDirectory tempdir;
    tempdir_path = tempdir.path;
    const int num_file = 5;
    for (int i = 0; i < num_file; ++i) {
      std::ofstream fout(tempdir.path + "/" + std::to_string(i) + ".txt");
      fout << "0,1,1," << (i + 1) << "\n";
    }
    // Check if each file can be read back
    for (int i = 0; i < num_file; ++i) {
      std::ifstream fin(tempdir.path + "/" + std::to_string(i) + ".txt");
      std::string s;
      ASSERT_TRUE(static_cast<bool>(std::getline(fin, s)));
      ASSERT_EQ(s, std::string("0,1,1," + std::to_string(i + 1)));
      ASSERT_FALSE(static_cast<bool>(std::getline(fin, s)));
    }
  }
  // Test the directory is indeed deleted.
  const dmlc::io::URI uri(tempdir_path.c_str());
  ASSERT_ANY_THROW(dmlc::io::FileSystem::GetInstance(uri)->GetPathInfo(uri));
}

TEST(TemporaryDirectory, test_recursive) {
  std::string tempdir_path;
  {
    dmlc::TemporaryDirectory tempdir;
    tempdir_path = tempdir.path;
    const int recurse_depth = 5;

    std::queue<std::pair<int, std::string>> Q;  // (depth, directory)
    Q.emplace(0, tempdir.path);
    while (!Q.empty()) {
      auto e = Q.front(); Q.pop();
      const int current_depth = e.first;
      const std::string current_directory = e.second;
      if (current_depth < recurse_depth) {
        {
          std::ofstream of(current_directory + "/foobar.txt");
          of << "hello world\n";
        }
        MakeDirectory(current_directory + "/1");
        MakeDirectory(current_directory + "/2");
        Q.emplace(current_depth + 1, current_directory + "/1");
        Q.emplace(current_depth + 1, current_directory + "/2");
      } else {
        std::ofstream of(current_directory + "/foobar.txt");
        of << "hello world\n";
      }
    }
  }
  // Test the directory is indeed deleted.
  const dmlc::io::URI uri(tempdir_path.c_str());
  ASSERT_ANY_THROW(dmlc::io::FileSystem::GetInstance(uri)->GetPathInfo(uri));
}
