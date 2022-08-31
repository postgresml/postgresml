#include "../src/data/csv_parser.h"
#include "../src/data/libsvm_parser.h"
#include "../src/data/libfm_parser.h"
#include <cstdio>
#include <cstdlib>
#include <dmlc/io.h>
#include <gtest/gtest.h>

using namespace dmlc;
using namespace dmlc::data;

namespace parser_test {
template <typename IndexType, typename DType = real_t>
class CSVParserTest : public CSVParser<IndexType, DType> {
public:
  explicit CSVParserTest(InputSplit *source,
                         const std::map<std::string, std::string> &args,
                         int nthread)
      : CSVParser<IndexType, DType>(source, args, nthread) {}
  void CallParseBlock(char *begin, char *end,
                      RowBlockContainer<IndexType, DType> *out) {
    CSVParser<IndexType, DType>::ParseBlock(begin, end, out);
  }
};

template <typename IndexType, typename DType = real_t>
class LibSVMParserTest : public LibSVMParser<IndexType, DType> {
public:
  explicit LibSVMParserTest(InputSplit *source,
                            const std::map<std::string, std::string> &args,
                            int nthread)
      : LibSVMParser<IndexType, DType>(source, args, nthread) {}
  void CallParseBlock(char *begin, char *end,
                      RowBlockContainer<IndexType, DType> *out) {
    LibSVMParser<IndexType, DType>::ParseBlock(begin, end, out);
  }
};

template <typename IndexType, typename DType = real_t>
class LibFMParserTest : public LibFMParser<IndexType, DType> {
public:
  explicit LibFMParserTest(InputSplit *source,
                           const std::map<std::string, std::string> &args,
                           int nthread)
      : LibFMParser<IndexType, DType>(source, args, nthread) {}
  void CallParseBlock(char *begin, char *end,
                      RowBlockContainer<IndexType, DType> *out) {
    LibFMParser<IndexType, DType>::ParseBlock(begin, end, out);
  }
};

}  // namespace parser_test

namespace {

template <typename IndexType>
static inline void CountDimensions(RowBlockContainer<IndexType>* rctr,
                                   size_t* out_num_row, size_t* out_num_col) {
  size_t num_row = rctr->label.size();
  size_t num_col = 0;
  for (size_t i = rctr->offset[0]; i < rctr->offset[num_row]; ++i) {
    const IndexType index = rctr->index[i];
    num_col = std::max(num_col, static_cast<size_t>(index + 1));
  }
  *out_num_row = num_row;
  *out_num_col = num_col;
}

}  // namespace anonymous

TEST(CSVParser, test_ignore_bom) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::string data = "\xEF\xBB\xBF\x31\n\xEF\xBB\x32\n";
  char *out_data = const_cast<char *>(data.c_str());
  std::unique_ptr<RowBlockContainer<unsigned> > rctr {new RowBlockContainer<unsigned>()};
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  CHECK(rctr->value.size() == 1);
  CHECK(rctr->value.at(0) == 1);

  data = "\xEF\xBB\xBF\x31\n\xEF\xBB\xBF\x32\n";
  out_data = const_cast<char *>(data.c_str());
  rctr.reset(new RowBlockContainer<unsigned>());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  CHECK(rctr->value.size() == 2);
  CHECK(rctr->value.at(0) == 1);
  CHECK(rctr->value.at(1) == 2);
}

TEST(CSVParser, test_standard_case) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr { new RowBlockContainer<unsigned>() };
  std::string data = "0,1,2,3\n4,5,6,7\n8,9,10,11\n";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK(i == rctr->value[i]);
  }
}

TEST(CSVParser, missing_values) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr { new RowBlockContainer<unsigned>() };
  std::string data = "0,,,3\n4,5,6,7\n8,9,10,11\n";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  CHECK_EQ(rctr->value.size(), 10);
  CHECK(rctr->value[0] == 0);
  CHECK(rctr->index[0] == 0);
  CHECK_EQ(rctr->value[1], 3);
  CHECK(rctr->index[1] == 3);

  for (size_t i = 2; i < rctr->value.size(); ++i) {
    CHECK_EQ(rctr->value[i], i + 2);
  }
}

TEST(CSVParser, test_int32_parse) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned, int32_t>> parser(
      new CSVParserTest<unsigned, int32_t>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned, int32_t>> rctr {
    new RowBlockContainer<unsigned, int32_t>()};
  std::string data = "20000000,20000001,20000002,20000003\n"
                     "20000004,20000005,20000006,20000007\n"
                     "20000008,20000009,20000010,20000011\n";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK((i+20000000) == (size_t)rctr->value[i]);
  }
}

TEST(CSVParser, test_int64_parse) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned, int64_t>> parser(
    new CSVParserTest<unsigned, int64_t>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned, int64_t> > rctr {
    new RowBlockContainer<unsigned, int64_t>()};
  std::string data = "2147483648,2147483649,2147483650,2147483651\n"
                     "2147483652,2147483653,2147483654,2147483655\n"
                     "2147483656,2147483657,2147483658,2147483659\n";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK((i+2147483648) == (size_t)rctr->value[i]);
  }
}

TEST(CSVParser, test_different_newlines) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned> > rctr {new RowBlockContainer<unsigned>()};
  std::string data = "0,1,2,3\r\n4,5,6,7\r\n8,9,10,11\r\n";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK(i == rctr->value[i]);
  }
}

TEST(CSVParser, test_noeol) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned> > rctr {new RowBlockContainer<unsigned>()} ;
  std::string data = "0,1,2,3\r\n4,5,6,7\r\n8,9,10,11";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK(i == rctr->value[i]);
  }
}

TEST(CSVParser, test_delimiter) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{ {"delimiter", " "} };
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "0 1 2 3\n4 5 6 7\n8 9 10 11";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK(i == rctr->value[i]);
  }
}

TEST(CSVParser, test_weight_column) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{ {"weight_column", "2"} };
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "0,1,2,3\n4,5,6,7\n8,9,10,11";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  CHECK_EQ(rctr->weight.size(), 3U);
  for (size_t i = 0; i < rctr->weight.size(); i++) {
    CHECK_EQ(rctr->weight[i], 2.0f + 4.0f * i);
  }
  const std::vector<real_t>
    expected_values{0.0f, 1.0f, 3.0f, 4.0f, 5.0f, 7.0f, 8.0f, 9.0f, 11.0f};
  CHECK_EQ(rctr->value.size(), expected_values.size());
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK_EQ(rctr->value[i], expected_values[i]);
  }
}

TEST(CSVParser, test_weight_column_2) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<CSVParserTest<unsigned>> parser(
      new CSVParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "0,1,2,3\n4,5,6,7\n8,9,10,11";
  char *out_data = const_cast<char *>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  CHECK(rctr->weight.empty());
  CHECK_EQ(rctr->value.size(), 12U);
  for (size_t i = 0; i < rctr->value.size(); i++) {
    CHECK(i == rctr->value[i]);
  }
}

void test_qid(std::string data) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());
  const std::vector<size_t> expected_offset{
    0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60
  };
  const std::vector<real_t> expected_label{
    3, 2, 1, 1, 1, 2, 1, 1, 2, 3, 4, 1
  };
  const std::vector<uint64_t> expected_qid{
    1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3
  };
  const std::vector<unsigned> expected_index{
    1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
    1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
    1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5
  };
  const std::vector<real_t> expected_value{
    1.0f, 1.0f, 0.0f, 0.2f, 0.0f, 0.0f, 0.0f, 1.0f, 0.1f, 1.0f, 0.0f, 1.0f, 0.0f, 0.4f, 0.0f, 0.0f,
    0.0f, 1.0f, 0.3f, 0.0f, 0.0f, 0.0f, 1.0f, 0.2f, 0.0f, 1.0f, 0.0f, 1.0f, 0.4f, 0.0f, 0.0f, 0.0f,
    1.0f, 0.1f, 0.0f, 0.0f, 0.0f, 1.0f, 0.2f, 0.0f, 0.0f, 0.0f, 1.0f, 0.1f, 1.0f, 1.0f, 1.0f, 0.0f,
    0.3f, 0.0f, 1.0f, 0.0f, 0.0f, 0.4f, 1.0f, 0.0f, 1.0f, 1.0f, 0.5f, 0.0f
  };
  CHECK(rctr->offset == expected_offset);
  CHECK(rctr->label == expected_label);
  CHECK(rctr->qid == expected_qid);
  CHECK(rctr->index == expected_index);
  CHECK(rctr->value == expected_value);
}

TEST(LibSVMParser, test_qid) {
  std::string data = R"qid(3 qid:1 1:1 2:1 3:0 4:0.2 5:0
                           2 qid:1 1:0 2:0 3:1 4:0.1 5:1
                           1 qid:1 1:0 2:1 3:0 4:0.4 5:0
                           1 qid:1 1:0 2:0 3:1 4:0.3 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.2 5:0
                           2 qid:2 1:1 2:0 3:1 4:0.4 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.1 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.2 5:0
                           2 qid:3 1:0 2:0 3:1 4:0.1 5:1
                           3 qid:3 1:1 2:1 3:0 4:0.3 5:0
                           4 qid:3 1:1 2:0 3:0 4:0.4 5:1
                           1 qid:3 1:0 2:1 3:1 4:0.5 5:0)qid";
  test_qid(data);
}

TEST(LibSVMParser, test_qid_with_comment) {
  std::string data = R"qid(# what does foo bar mean anyway
                           3 qid:1 1:1 2:1 3:0 4:0.2 5:0 # foo
                           2 qid:1 1:0 2:0 3:1 4:0.1 5:1
                           1 qid:1 1:0 2:1 3:0 4:0.4 5:0
                           1 qid:1 1:0 2:0 3:1 4:0.3 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.2 5:0 # bar
                           2 qid:2 1:1 2:0 3:1 4:0.4 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.1 5:0
                           1 qid:2 1:0 2:0 3:1 4:0.2 5:0
                           2 qid:3 1:0 2:0 3:1 4:0.1 5:1
                           3 qid:3 1:1 2:1 3:0 4:0.3 5:0
                           4 qid:3 1:1 2:0 3:0 4:0.4 5:1
                           1 qid:3 1:0 2:1 3:1 4:0.5 5:0)qid";
  test_qid(data);
}

TEST(LibSVMParser, test_excess_decimal_digits) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "0 1:17.065995780200002000000 4:17.0659957802 "
                     "6:0.00017065995780200002 8:0.000170659957802\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 1U);
  CHECK_EQ(num_col, 9U);

  const std::vector<unsigned> expected_index{1, 4, 6, 8};
  CHECK(rctr->index == expected_index);  // perform element-wise comparsion
  CHECK_EQ(rctr->value[0], rctr->value[1]);
  CHECK_EQ(rctr->value[2], rctr->value[3]);
}

TEST(LibSVMParser, test_indexing_mode_0_based) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "1 1:1 2:-1\n0 1:-1 2:1\n1 1:-1 2:-1\n0 1:1 2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 3U);

  const std::vector<unsigned> expected_index{1, 2, 1, 2, 1, 2, 1, 2};
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->index == expected_index);  // perform element-wise comparsion
  CHECK(rctr->value == expected_value);
}

TEST(LibSVMParser, test_indexing_mode_1_based) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "1"}};
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "1 1:1 2:-1\n0 1:-1 2:1\n1 1:-1 2:-1\n0 1:1 2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 2U);

  const std::vector<unsigned> expected_index{0, 1, 0, 1, 0, 1, 0, 1};
    // with indexing_mode=1, parser will subtract 1 from each feature index
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->index == expected_index);  // perform element-wise comparsion
  CHECK(rctr->value == expected_value);
}

TEST(LibSVMParser, test_indexing_mode_auto_detect) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "-1"}};
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "1 1:1 2:-1\n0 1:-1 2:1\n1 1:-1 2:-1\n0 1:1 2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 2U);

  const std::vector<unsigned> expected_index{0, 1, 0, 1, 0, 1, 0, 1};
    // expect to detect 1-based indexing, since the least feature id is 1
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->index == expected_index);  // perform element-wise comparsion
  CHECK(rctr->value == expected_value);
}

TEST(LibSVMParser, test_indexing_mode_auto_detect_2) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "-1"}};
  std::unique_ptr<LibSVMParserTest<unsigned>> parser(
      new LibSVMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data = "1 1:1 2:-1\n0 0:-2 1:-1 2:1\n1 1:-1 2:-1\n0 1:1 2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 3U);

  const std::vector<unsigned> expected_index{1, 2, 0, 1, 2, 1, 2, 1, 2};
    // expect to detect 0-based indexing, since the least feature id is 0
  const std::vector<real_t> expected_value{1, -1, -2, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->index == expected_index);  // perform element-wise comparsion
  CHECK(rctr->value == expected_value);
}

TEST(LibFMParser, test_indexing_mode_0_based) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args;
  std::unique_ptr<LibFMParserTest<unsigned>> parser(
      new LibFMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data
    = "1 1:1:1 1:2:-1\n0 1:1:-1 2:2:1\n1 2:1:-1 1:2:-1\n0 2:1:1 2:2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 3U);

  const std::vector<unsigned> expected_field{1, 1, 1, 2, 2, 1, 2, 2};
  const std::vector<unsigned> expected_index{1, 2, 1, 2, 1, 2, 1, 2};
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->field == expected_field);
  CHECK(rctr->index == expected_index);
  CHECK(rctr->value == expected_value);  // perform element-wise comparsion
}

TEST(LibFMParser, test_indexing_mode_1_based) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "1"}};
  std::unique_ptr<LibFMParserTest<unsigned>> parser(
      new LibFMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data
    = "1 1:1:1 1:2:-1\n0 1:1:-1 2:2:1\n1 2:1:-1 1:2:-1\n0 2:1:1 2:2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 2U);

  const std::vector<unsigned> expected_field{0, 0, 0, 1, 1, 0, 1, 1};
  const std::vector<unsigned> expected_index{0, 1, 0, 1, 0, 1, 0, 1};
    // with indexing_mode=1, parser will subtract 1 from field/feature indices
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->field == expected_field);
  CHECK(rctr->index == expected_index);
  CHECK(rctr->value == expected_value);  // perform element-wise comparsion
}

TEST(LibFMParser, test_indexing_mode_auto_detect) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "-1"}};
  std::unique_ptr<LibFMParserTest<unsigned>> parser(
      new LibFMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data
    = "1 1:1:1 1:2:-1\n0 1:1:-1 2:2:1\n1 2:1:-1 1:2:-1\n0 2:1:1 2:2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 2U);

  const std::vector<unsigned> expected_field{0, 0, 0, 1, 1, 0, 1, 1};
  const std::vector<unsigned> expected_index{0, 1, 0, 1, 0, 1, 0, 1};
    // expect to detect 1-based indexing, since all field/feature id's exceed 0
  const std::vector<real_t> expected_value{1, -1, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->field == expected_field);
  CHECK(rctr->index == expected_index);
  CHECK(rctr->value == expected_value);  // perform element-wise comparsion
}

TEST(LibFMParser, test_indexing_mode_auto_detect_2) {
  using namespace parser_test;
  InputSplit *source = nullptr;
  const std::map<std::string, std::string> args{{"indexing_mode", "-1"}};
  std::unique_ptr<LibFMParserTest<unsigned>> parser(
      new LibFMParserTest<unsigned>(source, args, 1));
  std::unique_ptr<RowBlockContainer<unsigned>> rctr {new RowBlockContainer<unsigned>()};
  std::string data
    = "1 1:1:1 1:2:-1\n0 0:0:-2 1:1:-1 2:2:1\n1 2:1:-1 1:2:-1\n0 2:1:1 2:2:1\n";
  char* out_data = const_cast<char*>(data.c_str());
  parser->CallParseBlock(out_data, out_data + data.size(), rctr.get());

  size_t num_row, num_col;
  CountDimensions(rctr.get(), &num_row, &num_col);
  CHECK_EQ(num_row, 4U);
  CHECK_EQ(num_col, 3U);

  const std::vector<unsigned> expected_field{1, 1, 0, 1, 2, 2, 1, 2, 2};
  const std::vector<unsigned> expected_index{1, 2, 0, 1, 2, 1, 2, 1, 2};
    // expect to detect 0-based indexing, since second row has feature id 0
  const std::vector<real_t> expected_value{1, -1, -2, -1, 1, -1, -1, 1, 1};
  CHECK(rctr->field == expected_field);
  CHECK(rctr->index == expected_index);
  CHECK(rctr->value == expected_value);  // perform element-wise comparsion
}
