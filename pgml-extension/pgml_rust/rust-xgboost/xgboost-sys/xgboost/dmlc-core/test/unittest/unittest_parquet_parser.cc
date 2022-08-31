/*!
 *  Copyright (c) 2021 by Contributors
 * \file unittest_parquet_parser.cc
 * \brief test parquet parser loads the same data as csv parser; first generate some data,
 * write into a csv and a parquet file, then load from them and check the entries are the same
 * \author Chengyang Gu
 */

#include <dmlc/build_config.h>

#ifdef DMLC_USE_PARQUET

#include <dmlc/filesystem.h>
#include <gtest/gtest.h>
#include <arrow/api.h>
#include <arrow/io/api.h>
#include <parquet/arrow/reader.h>
#include <parquet/arrow/writer.h>
#include <iostream>
#include <fstream>
#include <vector>
#include <string>
#include <memory>
#include "../src/data/csv_parser.h"
#include "../src/data/parquet_parser.h"

namespace {

void write_to_csv(const std::vector<std::vector<float>>& entries, const std::string& filename) {
  std::ofstream csv_writer(filename);
  for (auto& row : entries) {
    for (auto e : row) {
      csv_writer << e << ',';
    }
    csv_writer << std::endl;
  }
  csv_writer.close();
}

void write_to_parquet(const std::vector<std::vector<float>>& entries, const std::string& filename) {
  int n_obs = entries.size();
  int n_feature = entries.at(0).size();
  std::vector<arrow::FloatBuilder> column_builders(n_feature);
  std::vector<std::shared_ptr<arrow::Array>> arrays(n_feature);
  std::vector<std::shared_ptr<arrow::Field>> fields;
  for (int j = 0; j < n_feature; ++j) {
    for (int i = 0; i < n_obs; ++i) {
      PARQUET_THROW_NOT_OK(column_builders.at(j).AppendValues({entries.at(i).at(j)}));
    }
    PARQUET_THROW_NOT_OK(column_builders.at(j).Finish(&arrays[j]));
    fields.emplace_back(arrow::field((std::to_string(j)), arrow::float32()));
  }
  std::shared_ptr<arrow::Schema> schema = arrow::schema(fields);

  std::shared_ptr<arrow::Table> table = arrow::Table::Make(schema, arrays);

  // save to a file
  std::shared_ptr<arrow::io::FileOutputStream> outfile;
  PARQUET_ASSIGN_OR_THROW(
      outfile,
      arrow::io::FileOutputStream::Open(filename));
  // The last argument to the function call is the size of the RowGroup in
  // the parquet file. Normally you would choose this to be rather large but
  // for the example, we use a small value to have multiple RowGroups.
  PARQUET_THROW_NOT_OK(
      parquet::arrow::WriteTable(*table.get(), arrow::default_memory_pool(), outfile,
        n_obs * n_feature));

  outfile->Close();
}

}  // anonymous namespace

TEST(ParquetParser, test_end_to_end) {
  srand(static_cast<unsigned>(time(0)));
  int n_obs = 10;
  int n_feature = 5;

  // create a n_obs x n_feature matrix with random entries in (0, 1)
  std::vector<std::vector<float>> entries(n_obs, std::vector<float>(n_feature));
  for (int i = 0; i < n_obs; ++i) {
    for (int j = 0; j < n_feature; ++j) {
      entries.at(i).at(j) = static_cast<float>(rand()) / static_cast<float>(RAND_MAX);
    }
  }

  dmlc::TemporaryDirectory tempdir;

  const std::string csv_filename = tempdir.path + "/test_parquet.csv";
  const std::string parquet_filename = tempdir.path + "/test_parquet.parquet";

  write_to_csv(entries, csv_filename);
  write_to_parquet(entries, parquet_filename);

  // read both csv and parquet
  dmlc::data::CSVParser<unsigned> csv_parser(
      dmlc::InputSplit::Create(csv_filename.c_str(), 0, 1, "text"),
      {{"label_column", "-1"}},
      1
  );

  dmlc::data::ParquetParser<unsigned> parquet_parser(
      parquet_filename,
      {{"nthreads", "1"}, {"label_column", "-1"}}
  );

  std::vector<dmlc::data::RowBlockContainer<unsigned>> csv_data(1), parquet_data(1);

  csv_parser.ParseNext(&csv_data);
  parquet_parser.ParseNext(&parquet_data);
  EXPECT_EQ(csv_data.size(), 1);
  EXPECT_EQ(parquet_data.size(), 1);

  // check all entries are equal
  for (int i = 0; i < n_obs * n_feature; ++i) {
    EXPECT_NEAR(csv_data.at(0).value.at(i), parquet_data.at(0).value.at(i), 1e-6);
  }
}

#endif  // DMLC_USE_PARQUET
