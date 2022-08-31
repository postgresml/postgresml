/*!
 *  Copyright (c) 2021 by Contributors
 * \file parquet_parser.h
 * \brief iterator parser to parse parquet format
 * \author Chengyang Gu
 */
#ifndef DMLC_DATA_PARQUET_PARSER_H_
#define DMLC_DATA_PARQUET_PARSER_H_

#include <dmlc/data.h>
#include <dmlc/strtonum.h>
#include <dmlc/parameter.h>
#include <cmath>
#include <cstring>
#include <map>
#include <string>
#include <limits>
#include <future>
#include <algorithm>
#include <memory>
#include <vector>
#include "../data/row_block.h"
#include "../data/parser.h"
#include "arrow/io/api.h"
#include "parquet/api/reader.h"


namespace dmlc {
namespace data {

struct ParquetParserParam : public Parameter<ParquetParserParam> {
  std::string format;
  int label_column;
  int weight_column;
  int nthreads;

  DMLC_DECLARE_PARAMETER(ParquetParserParam) {
    DMLC_DECLARE_FIELD(format).set_default("parquet")
      .describe("File format.");
    DMLC_DECLARE_FIELD(label_column).set_default(0)
      .describe("Column index (0-based) that will put into label.");
    DMLC_DECLARE_FIELD(weight_column).set_default(-1)
      .describe("Column index that will put into instance weights.");
    DMLC_DECLARE_FIELD(nthreads).set_default(1)
      .describe("Column index that will put into instance weights.");
  }
};

template <typename IndexType, typename DType = real_t>
class ParquetParser : public ParserImpl<IndexType, DType> {
 public:
  ParquetParser(const std::string& filename,
                const std::map<std::string, std::string>& args) : row_groups_read_(0) {
    param_.Init(args);
    nthread_ = param_.nthreads;
    CHECK_EQ(param_.format, "parquet");

    parquet_reader_ = parquet::ParquetFileReader::OpenFile(filename, false);
    metadata_ = parquet_reader_->metadata();
    num_rows_ = metadata_->num_rows();
    num_cols_ = metadata_->num_columns();
    num_row_groups_ = metadata_->num_row_groups();

    have_next_ = (num_rows_ != 0);
  }

  /*!
   * \brief read in next several blocks of data
   * \param data vector of data to be returned
   * \return true if the data is loaded, false if reach end
   */
  virtual bool ParseNext(std::vector<RowBlockContainer<IndexType, DType> > *data);

 protected:
  virtual void ParseRowGroup(int row_group_id,
                            RowBlockContainer<IndexType, DType> *out);

  virtual size_t BytesRead(void) const {
    return -1;
  }

  virtual void BeforeFirst(void) {}

 private:
  ParquetParserParam param_;
  // handle for reading parquet files
  std::unique_ptr<parquet::ParquetFileReader> parquet_reader_;
  std::shared_ptr<parquet::FileMetaData> metadata_;
  // number of rows having read
  int num_rows_;
  int num_cols_;
  int num_row_groups_;
  int row_groups_read_;
  // whether we have reached end of parquet file
  bool have_next_;
  // number of threads; hardcoded 4 for now
  int nthread_;
};

template <typename IndexType, typename DType>
bool ParquetParser<IndexType, DType>::
ParseNext(std::vector<RowBlockContainer<IndexType, DType> > *data) {
  if (!have_next_) {
    parquet_reader_->Close();
    return false;
  }
  std::vector<std::future<void>> futures;

  int next_row_groups = std::min(nthread_, num_row_groups_ - row_groups_read_);
  data->resize(next_row_groups);
  futures.resize(next_row_groups);

  for (int tid = 0; tid < next_row_groups; ++tid) {
    int row_group_id = row_groups_read_ + tid;
    futures[tid] = std::async(std::launch::async, [&, row_group_id, data, tid] {
      ParseRowGroup(row_group_id, &(*data)[tid]);
    });
  }

  for (int i = 0; i < next_row_groups; ++i) {
    futures[i].wait();
  }

  row_groups_read_ += next_row_groups;
  have_next_ = (row_groups_read_ < num_row_groups_);
  return true;
}

template <typename IndexType, typename DType>
void ParquetParser<IndexType, DType>::
ParseRowGroup(int row_group_id,
              RowBlockContainer<IndexType, DType> *out) {
  out->Clear();
  DType v;

  std::shared_ptr<parquet::RowGroupReader> row_group_reader
      = parquet_reader_->RowGroup(row_group_id);
  std::vector<std::shared_ptr<parquet::ColumnReader>> all_column_readers;
  std::vector<parquet::FloatReader*> all_float_readers;

  // get all the column readers; will iterate each column row-wise later
  for (int i_col = 0; i_col < num_cols_; ++i_col) {
    all_column_readers.push_back(row_group_reader->Column(i_col));
    all_float_readers.push_back(
        static_cast<parquet::FloatReader*>(all_column_readers[i_col].get()));
  }

  int num_rows_this_group = metadata_->RowGroup(row_group_id)->num_rows();
  constexpr int chunk_size = 1;
  int64_t values_read;

  for (int i_row = 0; i_row < num_rows_this_group; i_row++) {
    IndexType idx = 0;
    DType label = DType(0.0f);
    real_t weight = std::numeric_limits<real_t>::quiet_NaN();

    for (int i_col = 0; i_col < num_cols_; i_col++) {
      all_float_readers[i_col]->ReadBatch(chunk_size, nullptr, nullptr, &v, &values_read);
      CHECK_EQ(values_read, chunk_size);
      if (i_col == param_.label_column) {
        label = v;
      } else if (std::is_same<DType, real_t>::value
                 && i_col == param_.weight_column) {
        weight = v;
      } else {
        out->value.push_back(v);
        out->index.push_back(idx++);
      }
    }

    out->label.push_back(label);
    if (!std::isnan(weight)) {
      out->weight.push_back(weight);
    }
    out->offset.push_back(out->index.size());
  }
  CHECK(out->label.size() + 1 == out->offset.size());
  CHECK(out->weight.size() == 0 || out->weight.size() + 1 == out->offset.size());
}

}  // namespace data
}  // namespace dmlc
#endif  // DMLC_DATA_PARQUET_PARSER_H_
