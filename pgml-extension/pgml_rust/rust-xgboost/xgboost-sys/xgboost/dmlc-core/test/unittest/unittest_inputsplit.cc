#include <dmlc/data.h>
#include <dmlc/filesystem.h>
#include <string>
#include <iostream>
#include <fstream>
#include <vector>
#include <algorithm>
#include <random>
#include <future>
#include <cstdlib>
#include <gtest/gtest.h>

namespace {

inline void CountDimensions(dmlc::Parser<uint32_t>* parser,
                            size_t* out_num_row, size_t* out_num_col) {
  size_t num_row = 0;
  size_t num_col = 0;
  while (parser->Next()) {
    const dmlc::RowBlock<uint32_t>& batch = parser->Value();
    num_row += batch.size;
    for (size_t i = batch.offset[0]; i < batch.offset[batch.size]; ++i) {
      const uint32_t index = batch.index[i];
      num_col = std::max(num_col, static_cast<size_t>(index + 1));
    }
  }
  *out_num_row = num_row;
  *out_num_col = num_col;
}

struct RecordIOHeader {
  uint32_t flag;
  float label;
  uint64_t image_id[2];
};

}  // namespace anonymous

TEST(InputSplit, test_split_csv_noeol) {
  size_t num_row, num_col;
  {
    /* Create a test case for partitioned csv with NOEOL */
    dmlc::TemporaryDirectory tempdir;
    {
      std::ofstream of(tempdir.path + "/train_0.csv", std::ios::binary);
      of << "0,1,1,1";  // NOEOL (no '\n' at end of file)
    }
    {
      std::ofstream of(tempdir.path + "/train_1.csv", std::ios::binary);
      of << "0,1,1,2\n";
    }
    {
      std::ofstream of(tempdir.path + "/train_2.csv", std::ios::binary);
      of << "0,1,1,2\n";
    }
    /* Load the test case with InputSplit and obtain matrix dimensions */
    {
      std::unique_ptr<dmlc::Parser<uint32_t> > parser(
        dmlc::Parser<uint32_t>::Create(tempdir.path.c_str(), 0, 1, "csv"));
      CountDimensions(parser.get(), &num_row, &num_col);
    }
  }
  /* Check matrix dimensions: must be 3x4 */
  ASSERT_EQ(num_row, 3U);
  ASSERT_EQ(num_col, 4U);
}

TEST(InputSplit, test_split_libsvm_noeol) {
  {
    /* Create a test case for partitioned libsvm with NOEOL */
    dmlc::TemporaryDirectory tempdir;
    const char* line
      = "1 3:1 10:1 11:1 21:1 30:1 34:1 36:1 40:1 41:1 53:1 58:1 65:1 69:1 "
        "77:1 86:1 88:1 92:1 95:1 102:1 105:1 117:1 124:1";
    {
      std::ofstream of(tempdir.path + "/train_0.libsvm", std::ios::binary);
      of << line << "\n";
    }
    {
      std::ofstream of(tempdir.path + "/train_1.libsvm", std::ios::binary);
      of << line;  // NOEOL (no '\n' at end of file)
    }
    std::unique_ptr<dmlc::Parser<uint32_t> > parser(
      dmlc::Parser<uint32_t>::Create(tempdir.path.c_str(), 0, 1, "libsvm"));
    size_t num_row, num_col;
    CountDimensions(parser.get(), &num_row, &num_col);
    ASSERT_EQ(num_row, 2);
    ASSERT_EQ(num_col, 125);
  }
}

TEST(InputSplit, test_split_libsvm) {
  size_t num_row, num_col;
  {
    /* Create a test case for partitioned libsvm */
    dmlc::TemporaryDirectory tempdir;
    const int nfile = 5;
    for (int file_id = 0; file_id < nfile; ++file_id) {
      std::ofstream of(tempdir.path + "/test_" + std::to_string(file_id) + ".libsvm",
                       std::ios::binary);
      of << "1 3:1 10:1 11:1 21:1 30:1 34:1 36:1 40:1 41:1 53:1 58:1 65:1 69:1 "
         << "77:1 86:1 88:1 92:1 95:1 102:1 105:1 117:1 124:1\n";
    }
    /* Load the test case with InputSplit and obtain matrix dimensions */
    {
      std::unique_ptr<dmlc::Parser<uint32_t> > parser(
        dmlc::Parser<uint32_t>::Create(tempdir.path.c_str(), 0, 1, "libsvm"));
      CountDimensions(parser.get(), &num_row, &num_col);
    }
  }
  /* Check matrix dimensions: must be 5x125 */
  ASSERT_EQ(num_row, 5U);
  ASSERT_EQ(num_col, 125U);
}

TEST(InputSplit, test_split_libsvm_distributed) {
  {
    /* Create a test case for partitioned libsvm */
    dmlc::TemporaryDirectory tempdir;
    const char* line
      = "1 3:1 10:1 11:1 21:1 30:1 34:1 36:1 40:1 41:1 53:1 58:1 65:1 69:1 "
        "77:1 86:1 88:1 92:1 95:1 102:1 105:1 117:1 124:1\n";
    const int nfile = 5;
    for (int file_id = 0; file_id < nfile; ++file_id) {
      std::ofstream of(tempdir.path + "/test_" + std::to_string(file_id) + ".libsvm",
                       std::ios::binary);
      const int nrepeat = (file_id == 0 ? 6 : 1);
      for (int i = 0; i < nrepeat; ++i) {
        of << line;
      }
    }

    /* Load the test case with InputSplit and obtain matrix dimensions */
    const int npart = 2;
    const size_t expected_dims[npart][2] = { {6, 125}, {4, 125} };
    for (int part_id = 0; part_id < npart; ++part_id) {
      std::unique_ptr<dmlc::Parser<uint32_t> > parser(
        dmlc::Parser<uint32_t>::Create(tempdir.path.c_str(), part_id, npart, "libsvm"));
      size_t num_row, num_col;
      CountDimensions(parser.get(), &num_row, &num_col);
      ASSERT_EQ(num_row, expected_dims[part_id][0]);
      ASSERT_EQ(num_col, expected_dims[part_id][1]);
    }
  }
}

#ifdef DMLC_UNIT_TESTS_USE_CMAKE
/* Don't run the following when CMake is not used */

#include "./build_config.h"
#include <dmlc/build_config.h>

#ifndef DMLC_CMAKE_LITTLE_ENDIAN
  #error "DMLC_CMAKE_LITTLE_ENDIAN not defined"
#endif // DMLC_CMAKE_LITTLE_ENDIAN

#if DMLC_CMAKE_LITTLE_ENDIAN

TEST(InputSplit, test_recordio) {
  dmlc::TemporaryDirectory tempdir;

  std::unique_ptr<dmlc::InputSplit> source(
    dmlc::InputSplit::Create(CMAKE_CURRENT_SOURCE_DIR "/sample.rec", 0, 1, "recordio"));

  source->BeforeFirst();
  dmlc::InputSplit::Blob rec;
  char* content;
  RecordIOHeader header;
  size_t content_size;

  int idx = 1;

  while (source->NextRecord(&rec)) {
    ASSERT_GT(rec.size, sizeof(header));
    std::memcpy(&header, rec.dptr, sizeof(header));
    content = reinterpret_cast<char*>(rec.dptr) + sizeof(header);
    content_size = rec.size - sizeof(header);

    std::string expected;
    for (int i = 0; i < 10; ++i) {
      expected += std::to_string(idx) + "\n";
    }

    ASSERT_EQ(header.label, static_cast<float>(idx % 2));
    ASSERT_EQ(header.image_id[0], idx);
    ASSERT_EQ(std::string(content, content_size), expected);

    ++idx;
  }
}

#endif  // DMLC_CMAKE_LITTLE_ENDIAN

#endif  // DMLC_UNIT_TESTS_USE_CMAKE
