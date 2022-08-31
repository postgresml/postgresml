/*!
 *  Copyright (c) 2015 by Contributors
 * \file csv_parser.h
 * \brief iterator parser to parse csv format
 * \author Tianqi Chen
 */
#ifndef DMLC_DATA_CSV_PARSER_H_
#define DMLC_DATA_CSV_PARSER_H_

#include <dmlc/data.h>
#include <dmlc/strtonum.h>
#include <dmlc/parameter.h>
#include <cmath>
#include <cstring>
#include <map>
#include <string>
#include <limits>
#include "./row_block.h"
#include "./text_parser.h"

namespace dmlc {
namespace data {

struct CSVParserParam : public Parameter<CSVParserParam> {
  std::string format;
  int label_column;
  std::string delimiter;
  int weight_column;
  // declare parameters
  DMLC_DECLARE_PARAMETER(CSVParserParam) {
    DMLC_DECLARE_FIELD(format).set_default("csv")
        .describe("File format.");
    DMLC_DECLARE_FIELD(label_column).set_default(-1)
        .describe("Column index (0-based) that will put into label.");
    DMLC_DECLARE_FIELD(delimiter).set_default(",")
      .describe("Delimiter used in the csv file.");
    DMLC_DECLARE_FIELD(weight_column).set_default(-1)
        .describe("Column index that will put into instance weights.");
  }
};


/*!
 * \brief CSVParser, parses a dense csv format.
 *  Currently is a dummy implementation, when label column is not specified.
 *  All columns are treated as real dense data.
 *  label will be assigned to 0.
 *
 *  This should be extended in future to accept arguments of column types.
 */
template <typename IndexType, typename DType = real_t>
class CSVParser : public TextParserBase<IndexType, DType> {
 public:
  explicit CSVParser(InputSplit *source,
                     const std::map<std::string, std::string>& args,
                     int nthread)
      : TextParserBase<IndexType, DType>(source, nthread) {
    param_.Init(args);
    CHECK_EQ(param_.format, "csv");
    CHECK(param_.label_column != param_.weight_column
          || param_.label_column < 0)
      << "Must have distinct columns for labels and instance weights";
  }

 protected:
  virtual void ParseBlock(const char *begin,
                          const char *end,
                          RowBlockContainer<IndexType, DType> *out);

 private:
  CSVParserParam param_;
};

template <typename IndexType, typename DType>
void CSVParser<IndexType, DType>::
ParseBlock(const char *begin,
           const char *end,
           RowBlockContainer<IndexType, DType> *out) {
  out->Clear();
  const char * lbegin = begin;
  const char * lend = lbegin;
  // advance lbegin if it points to newlines
  while ((lbegin != end) && (*lbegin == '\n' || *lbegin == '\r')) ++lbegin;
  while (lbegin != end) {
    // get line end
    this->IgnoreUTF8BOM(&lbegin, &end);
    lend = lbegin + 1;
    while (lend != end && *lend != '\n' && *lend != '\r') ++lend;

    const char* p = lbegin;
    int column_index = 0;
    IndexType idx = 0;
    DType label = DType(0.0f);
    real_t weight = std::numeric_limits<real_t>::quiet_NaN();

    while (p != lend) {
      char *endptr;
      DType v;
      // if DType is float32
      if (std::is_same<DType, real_t>::value) {
        v = strtof(p, &endptr);
      // If DType is int32
      } else if (std::is_same<DType, int32_t>::value) {
        v = static_cast<int32_t>(strtoll(p, &endptr, 0));
      // If DType is int64
      } else if (std::is_same<DType, int64_t>::value) {
        v = static_cast<int64_t>(strtoll(p, &endptr, 0));
      // If DType is all other types
      } else {
        LOG(FATAL) << "Only float32, int32, and int64 are supported for the time being";
      }

      if (column_index == param_.label_column) {
        label = v;
      } else if (std::is_same<DType, real_t>::value
                 && column_index == param_.weight_column) {
        weight = v;
      } else {
        if (std::distance(p, static_cast<char const*>(endptr)) != 0) {
          out->value.push_back(v);
          out->index.push_back(idx++);
        } else {
          idx++;
        }
      }
      p = (endptr >= lend) ? lend : endptr;
      ++column_index;
      while (*p != param_.delimiter[0] && p != lend) ++p;
      if (p == lend && idx == 0) {
        LOG(FATAL) << "Delimiter \'" << param_.delimiter << "\' is not found in the line. "
                   << "Expected \'" << param_.delimiter
                   << "\' as the delimiter to separate fields.";
      }
      if (p != lend) ++p;
    }
    // skip empty line
    while ((*lend == '\n' || *lend == '\r') && lend != end) ++lend;
    lbegin = lend;
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
#endif  // DMLC_DATA_CSV_PARSER_H_
