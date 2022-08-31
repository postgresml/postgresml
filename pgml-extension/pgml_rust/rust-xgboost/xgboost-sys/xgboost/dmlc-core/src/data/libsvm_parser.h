/*!
 *  Copyright (c) 2015 by Contributors
 * \file libsvm_parser.h
 * \brief iterator parser to parse libsvm format
 * \author Tianqi Chen
 */
#ifndef DMLC_DATA_LIBSVM_PARSER_H_
#define DMLC_DATA_LIBSVM_PARSER_H_

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

struct LibSVMParserParam : public Parameter<LibSVMParserParam> {
  std::string format;
  int indexing_mode;
  // declare parameters
  DMLC_DECLARE_PARAMETER(LibSVMParserParam) {
    DMLC_DECLARE_FIELD(format).set_default("libsvm")
        .describe("File format");
    DMLC_DECLARE_FIELD(indexing_mode).set_default(0)
        .describe(
          "If >0, treat all feature indices as 1-based. "
          "If =0, treat all feature indices as 0-based. "
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
class LibSVMParser : public TextParserBase<IndexType> {
 public:
  explicit LibSVMParser(InputSplit *source, int nthread)
      : LibSVMParser(source, std::map<std::string, std::string>(), nthread) {}
  explicit LibSVMParser(InputSplit *source,
                        const std::map<std::string, std::string>& args,
                        int nthread)
      : TextParserBase<IndexType>(source, nthread) {
    param_.Init(args);
    CHECK_EQ(param_.format, "libsvm");
  }

 protected:
  virtual void ParseBlock(const char *begin,
                          const char *end,
                          RowBlockContainer<IndexType, DType> *out);

 private:
  LibSVMParserParam param_;
};

template <char kSymbol = '#'>
std::ptrdiff_t IgnoreCommentAndBlank(char const* beg,
                                     char const* line_end) {
  char const* p = beg;
  std::ptrdiff_t length = std::distance(beg, line_end);
  while (p != line_end) {
    if (*p == kSymbol) {
      // advance to line end, `ParsePair' will return empty line.
      return length;
    }
    if (!isblank(*p)) {
      return std::distance(beg, p);  // advance to p
    }
    p++;
  }
  // advance to line end, `ParsePair' will return empty line.
  return length;
}

template <typename IndexType, typename DType>
void LibSVMParser<IndexType, DType>::
ParseBlock(const char *begin,
           const char *end,
           RowBlockContainer<IndexType, DType> *out) {
  out->Clear();
  const char * lbegin = begin;
  const char * lend = lbegin;
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
    std::ptrdiff_t advanced = IgnoreCommentAndBlank(p, lend);
    p += advanced;
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
    // parse qid:id
    uint64_t qid;
    p = q;
    while (p != end && *p == ' ') ++p;
    if (p != lend && (strncmp(p, "qid:", 4) == 0)) {
      p += 4;
      qid = static_cast<uint64_t>(atoll(p));
      while (p != lend && isdigitchars(*p)) ++p;
      out->qid.push_back(qid);
    }
    // parse feature[:value]
    while (p != lend) {
      IndexType featureId;
      real_t value;
      std::ptrdiff_t advanced = IgnoreCommentAndBlank(p, lend);
      p += advanced;
      int r = ParsePair<IndexType, real_t>(p, lend, &q, featureId, value);
      if (r < 1) {
        // q is set to line end by `ParsePair', here is p. The latter terminates
        // while loop of parsing features.
        p = q;
        continue;
      }
      out->index.push_back(featureId);
      min_feat_id = std::min(featureId, min_feat_id);
      if (r == 2) {
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
  CHECK(out->label.size() + 1 == out->offset.size());

  // detect indexing mode
  // heuristic adopted from sklearn.datasets.load_svmlight_file
  // If all feature id's exceed 0, then detect 1-based indexing
  if (param_.indexing_mode > 0
      || (param_.indexing_mode < 0 && !out->index.empty() && min_feat_id > 0)) {
    // convert from 1-based to 0-based indexing
    for (IndexType& e : out->index) {
      --e;
    }
  }
}

}  // namespace data
}  // namespace dmlc
#endif  // DMLC_DATA_LIBSVM_PARSER_H_
