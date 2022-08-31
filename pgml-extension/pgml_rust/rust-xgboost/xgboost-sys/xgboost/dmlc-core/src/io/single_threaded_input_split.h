// Copyright by contributors
#ifndef DMLC_IO_SINGLE_THREADED_INPUT_SPLIT_H_
#define DMLC_IO_SINGLE_THREADED_INPUT_SPLIT_H_

#include <dmlc/threadediter.h>
#include <dmlc/base.h>
#include <algorithm>
#include "./input_split_base.h"

namespace dmlc {
namespace io {
/*!
 * \brief provides a single threaded input split
 *  Useful for debugging purposes. Be cautious of use
 *  for production use cases, as this is much less performant
 *  compared to ThreadedInputSplit
 */
class SingleThreadedInputSplit : public InputSplit {
 public:
  explicit SingleThreadedInputSplit(InputSplitBase *base,
                                    const size_t batch_size)
      : buffer_size_(InputSplitBase::kBufferSize), batch_size_(batch_size),
        base_(base), tmp_chunk_(NULL) {}
  bool NextProducer(InputSplitBase::Chunk **dptr) {
    if (*dptr == NULL) {
      *dptr = new InputSplitBase::Chunk(buffer_size_);
    }
    return base_->NextBatchEx(*dptr, batch_size_);
  }
  void BeforeFirstProducer() { base_->BeforeFirst(); }
  virtual ~SingleThreadedInputSplit(void) {
    delete tmp_chunk_;
    delete base_;
  }
  virtual void BeforeFirst() {
    BeforeFirstProducer();
    if (tmp_chunk_ != NULL) {
      tmp_chunk_ = NULL;
    }
  }
  virtual void HintChunkSize(size_t chunk_size) {
    buffer_size_ = std::max(chunk_size / sizeof(uint32_t), buffer_size_);
  }

  virtual bool NextRecord(Blob *out_rec) {
    if (tmp_chunk_ == NULL) {
      if (!NextProducer(&tmp_chunk_))
        return false;
    }
    while (!base_->ExtractNextRecord(out_rec, tmp_chunk_)) {
      tmp_chunk_ = NULL;
      if (!NextProducer(&tmp_chunk_))
        return false;
    }
    return true;
  }

  virtual bool NextChunk(Blob *out_chunk) {
    if (tmp_chunk_ == NULL) {
      if (!NextProducer(&tmp_chunk_))
        return false;
    }
    while (!base_->ExtractNextChunk(out_chunk, tmp_chunk_)) {
      tmp_chunk_ = NULL;
      if (!NextProducer(&tmp_chunk_))
        return false;
    }
    return true;
  }

  virtual size_t GetTotalSize(void) { return base_->GetTotalSize(); }

  virtual void ResetPartition(unsigned part_index, unsigned num_parts) {
    base_->ResetPartition(part_index, num_parts);
    this->BeforeFirst();
  }

 private:
  size_t buffer_size_;
  size_t batch_size_;
  InputSplitBase *base_;
  InputSplitBase::Chunk *tmp_chunk_;
};
}  //  namespace io
}  //  namespace dmlc

#endif  // DMLC_IO_SINGLE_THREADED_INPUT_SPLIT_H_
