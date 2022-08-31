/*!
 *  Copyright (c) 2017 by Contributors
 * \file libfm_parser.h
 * \brief iterator parser to parse libfm format
 * \author formath
 */
#ifndef DMLC_DATA_LIBFM_PARSER_H_
#define DMLC_DATA_LIBFM_PARSER_H_

#include <dmlc/data.h>
#include <dmlc/strtonum.h>
#include <dmlc/parameter.h>
#include <map>
#include <string>
#include <limits>
#include <algorithm>
#include <cstring>
#include "./row_block.h"
#include "./text_parser.h"

namespace dmlc {
namespace data {

struct LibFMParserParam : public Parameter<LibFMParserParam> {
  std::string format;
  int indexing_mode;
  // declare parameters
  DMLC_DECLARE_PARAMETER(LibFMParserParam) {
    DMLC_DECLARE_FIELD(format).set_default("libfm")
        .describe("File format");
    DMLC_DECLARE_FIELD(indexing_mode).set_default(0)
        .describe(
          "If >0, treat all field and feature indices as 1-based. "
          "If =0, treat all field and feature indices as 0-based. "
          "If <0, use heuristic to automatically detect mode of indexing. "
          "See https://en.wikipedia.org/wiki/Array_data_type#Index_origin "
          "for more details on indexing modes.");
  }
};

/*!
 * \brief Text parser that parses the input lines
 * and returns rows in input data
 */
template <typename IndexType, typename DType = real_t>
class LibFMParser : public TextParserBase<IndexType, DType> {
 public:
  explicit LibFMParser(InputSplit *source, int nthread)
      : LibFMParser(source, std::map<std::string, std::string>(), nthread) {}
  explicit LibFMParser(InputSplit *source,
                       const std::map<std::string, std::string>& args,
                       int nthread)
      : TextParserBase<IndexType>(source, nthread) {
    param_.Init(args);
    CHECK_EQ(param_.format, "libfm");
  }

 protected:
  virtual void ParseBlock(const char *begin,
                          const char *end,
                          RowBlockContainer<IndexType, DType> *out);

 private:
  LibFMParserParam param_;
};

template <typename IndexType, typename DType>
void LibFMParser<IndexType, DType>::
ParseBlock(const char *begin,
           const char *end,
           RowBlockContainer<IndexType, DType> *out) {
  out->Clear();
  const char * lbegin = begin;
  const char * lend = lbegin;
  IndexType min_field_id = std::numeric_limits<IndexType>::max();
  IndexType min_feat_id = std::numeric_limits<IndexType>::max();
  while (lbegin != end) {
    // get line end
    lend = lbegin + 1;
    while (lend != end && *lend != '\n' && *lend != '\r') ++lend;
    // parse label[:weight]
    const char * p = lbegin;
    const char * q = NULL;
    real_t label;
    real_t weight;
    int r = ParsePair<real_t, real_t>(p, lend, &q, label, weight);
    if (r < 1) {
      // empty line
      lbegin = lend;
      continue;
    }
    if (r == 2) {
      // has weight
      out->weight.push_back(weight);
    }
    if (out->label.size() != 0) {
      out->offset.push_back(out->index.size());
    }
    out->label.push_back(label);
    // parse fieldid:feature:value
    p = q;
    while (p != lend) {
      IndexType fieldId;
      IndexType featureId;
      real_t value;
      int r = ParseTriple<IndexType, IndexType, real_t>(p, lend, &q, fieldId, featureId, value);
      if (r <= 1) {
        p = q;
        continue;
      }
      out->field.push_back(fieldId);
      out->index.push_back(featureId);
      min_field_id = std::min(fieldId, min_field_id);
      min_feat_id = std::min(featureId, min_feat_id);
      if (r == 3) {
        // has value
        out->value.push_back(value);
      }
      p = q;
    }
    // next line
    lbegin = lend;
  }
  if (out->label.size() != 0) {
    out->offset.push_back(out->index.size());
  }
  CHECK(out->field.size() == out->index.size());
  CHECK(out->label.size() + 1 == out->offset.size());

  // detect indexing mode
  // heuristic adopted from sklearn.datasets.load_svmlight_file
  // If all feature and field id's exceed 0, then detect 1-based indexing
  if (param_.indexing_mode > 0
      || (param_.indexing_mode < 0 && !out->index.empty() && min_feat_id > 0
          && !out->field.empty() && min_field_id > 0) ) {
    // convert from 1-based to 0-based indexing
    for (IndexType& e : out->index) {
      --e;
    }
    for (IndexType& e : out->field) {
      --e;
    }
  }
}

}  // namespace data
}  // namespace dmlc
#endif  // DMLC_DATA_LIBFM_PARSER_H_
