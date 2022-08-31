/*!
 *  Copyright (c) 2015 by Contributors
 * \file threadediter.h
 * \brief thread backed iterator that can be used to implement
 *   general thread-based pipeline such as prefetch and pre-computation
 * To use the functions in this header, C++11 is required
 * \author Tianqi Chen
 */
#ifndef DMLC_THREADEDITER_H_
#define DMLC_THREADEDITER_H_
// defines DMLC_USE_CXX11
#include "./base.h"
// this code depends on c++11
#if DMLC_ENABLE_STD_THREAD
#include <condition_variable>
#include <functional>
#include <mutex>
#include <queue>
#include <atomic>
#include <thread>
#include <utility>
#include <memory>
#include "./data.h"
#include "./logging.h"

namespace dmlc {

/*!
 * \brief Wrapper class to manage std::thread; uses RAII pattern to automatically
 *        join std::thread upon destruction
 */
class ScopedThread {
 public:
  /*!
   * \brief constructor
   * \param thread thread to manage
   */
  explicit ScopedThread(std::thread thread)
      : thread_(std::move(thread)) {
    if (!thread_.joinable()) {
      throw std::logic_error("No thread");
    }
  }
  // destructor: join upon destruction
  virtual ~ScopedThread() {
    thread_.join();
  }
  // copy assignment and construction are not allowed
  ScopedThread(ScopedThread const&) = delete;
  ScopedThread& operator=(ScopedThread const&) = delete;

 private:
  std::thread thread_;
};

/*!
 * \brief a iterator that was backed by a thread
 *  to pull data eagerly from a single producer into a bounded buffer
 *  the consumer can pull the data at its own rate
 *
 * NOTE: thread concurrency cost time, make sure to store big blob of data in DType
 *
 * Usage example:
 * \code
 * ThreadedIter<DType> iter;
 * iter.Init(&producer);
 * // the following code can be in parallel
 * DType *dptr;
 * while (iter.Next(&dptr)) {
 *   // do something on dptr
 *   // recycle the space
 *   iter.Recycle(&dptr);
 * }
 * \endcode
 * \tparam DType the type of data blob we support
 */
template<typename DType>
class ThreadedIter : public DataIter<DType> {
 public:
  /*!
   * \brief producer class interface
   *  that threaditer used as source to
   *  preduce the content
   */
  class Producer {
   public:
    // virtual destructor
    virtual ~Producer() = default;
    /*! \brief reset the producer to beginning */
    virtual void BeforeFirst(void) {
      NotImplemented();
    }
    /*!
     * \brief load the data content into DType,
     * the caller can pass in NULL or an existing address
     * when inout_dptr is NULL:
     *    producer need to allocate a DType and fill the content
     * when inout_dptr is specified
     *    producer takes need to fill the content into address
     *    specified inout_dptr, or delete the one and create a new one
     *
     * \param inout_dptr used to pass in the data holder cell
     *        and return the address of the cell filled
     * \return true if there is next record, false if we reach the end
     */
    virtual bool Next(DType **inout_dptr) = 0;
  };
  /*!
   * \brief constructor
   * \param max_capacity maximum capacity of the queue
   */
  explicit ThreadedIter(size_t max_capacity = 8)
      : producer_(nullptr),
        producer_thread_(nullptr),
        max_capacity_(max_capacity),
        nwait_consumer_(0),
        nwait_producer_(0),
        out_data_(NULL) {}
  /*! \brief destructor */
  virtual ~ThreadedIter(void) {
    this->Destroy();
  }
  /*!
   * \brief destroy all the related resources
   *  this is equivalent to destructor, can be used
   *  to destroy the threaditer when user think it is
   *  appropriate, it is safe to call this multiple times
   */
  inline void Destroy(void);
  /*!
   * \brief set maximum capacity of the queue
   * \param max_capacity maximum capacity of the queue
   */
  inline void set_max_capacity(size_t max_capacity) {
    max_capacity_ = max_capacity;
  }
  /*!
   * \brief initialize the producer and start the thread can only be
   *   called once
   * \param producer pointer to the producer
   */
  inline void Init(std::shared_ptr<Producer> producer);
  /*!
   * \brief initialize the producer and start the thread
   *  pass in two function(closure) of producer to represent the producer
   *  the beforefirst function is optional, and defaults to not implemented
   *   NOTE: the closure must remain valid until the ThreadedIter destructs
   * \param next the function called to get next element, see Producer.Next
   * \param beforefirst the function to call to reset the producer, see Producer.BeforeFirst
   */
  inline void Init(std::function<bool(DType **)> next,
                   std::function<void()> beforefirst = NotImplemented);
  /*!
   * \brief get the next data, this function is threadsafe
   * \param out_dptr used to hold the pointer to the record
   *  after the function call, the caller takes ownership of the pointer
   *  the caller can call recycle to return ownership back to the threaditer
   *  so that the pointer can be re-used
   * \return true if there is next record, false if we reach the end
   * \sa Recycle
   */
  inline bool Next(DType **out_dptr);
  /*!
   * \brief recycle the data cell, this function is threadsafe
   * the threaditer can reuse the data cell for future data loading
   * \param inout_dptr pointer to the dptr to recycle, after the function call
   *        the content of inout_dptr will be set to NULL
   */
  inline void Recycle(DType **inout_dptr);

  /*!
   * \brief Rethrows exception which is set by the producer
   */
  inline void ThrowExceptionIfSet(void);

  /*!
   * \brief clears exception_ptr, called from Init
   */
  inline void ClearException(void);

  /*!
   * \brief adapt the iterator interface's Next
   *  NOTE: the call to this function is not threadsafe
   *  use the other Next instead
   * \return true if there is next record, false if we reach the end
   */
  virtual bool Next(void) {
    if (out_data_ != NULL) {
      this->Recycle(&out_data_);
    }
    if (Next(&out_data_)) {
      return true;
    } else {
      return false;
    }
  }
  /*!
   * \brief adapt the iterator interface's Value
   *  NOTE: the call to this function is not threadsafe
   *  use the other Next instead
   */
  virtual const DType &Value(void) const {
    CHECK(out_data_ != NULL) << "Calling Value at beginning or end?";
    return *out_data_;
  }
  /*! \brief set the iterator before first location */
  virtual void BeforeFirst(void) {
    ThrowExceptionIfSet();
    std::unique_lock<std::mutex> lock(mutex_);
    if (out_data_ != NULL) {
      free_cells_.push(out_data_);
      out_data_ = NULL;
    }
    if (producer_sig_.load(std::memory_order_acquire) == kDestroy)  return;

    producer_sig_.store(kBeforeFirst, std::memory_order_release);
    CHECK(!producer_sig_processed_.load(std::memory_order_acquire));
    if (nwait_producer_ != 0) {
      producer_cond_.notify_one();
    }
    CHECK(!producer_sig_processed_.load(std::memory_order_acquire));
    // wait until the request has been processed
    consumer_cond_.wait(lock, [this]() {
        return producer_sig_processed_.load(std::memory_order_acquire);
      });
    producer_sig_processed_.store(false, std::memory_order_release);
    bool notify = nwait_producer_ != 0 && !produce_end_;
    lock.unlock();
    // notify producer, in case they are waiting for the condition.
    if (notify) producer_cond_.notify_one();
    ThrowExceptionIfSet();
  }

 private:
  /*! \brief not support BeforeFirst */
  inline static void NotImplemented(void) {
    LOG(FATAL) << "BeforeFirst is not supported";
  }
  /*! \brief signals send to producer */
  enum Signal {
    kProduce,
    kBeforeFirst,
    kDestroy
  };
  /*! \brief producer class */
  // Producer *producer_owned_;
  std::shared_ptr<Producer> producer_;

  /*! \brief signal to producer */
  std::atomic<Signal> producer_sig_;
  /*! \brief whether the special signal other than kProduce is procssed */
  std::atomic<bool> producer_sig_processed_;
  /*! \brief thread that runs the producer */
  std::unique_ptr<ScopedThread> producer_thread_;
  /*! \brief whether produce ends */
  std::atomic<bool> produce_end_;
  /*! \brief maximum queue size */
  size_t max_capacity_;
  /*! \brief internal mutex */
  std::mutex mutex_;
  /*! brief internal mutex for exceptions */
  std::mutex mutex_exception_;
  /*! \brief number of consumer waiting */
  unsigned nwait_consumer_;
  /*! \brief number of producer waiting */
  unsigned nwait_producer_;
  /*! \brief conditional variable for producer thread */
  std::condition_variable producer_cond_;
  /*! \brief conditional variable for consumer threads */
  std::condition_variable consumer_cond_;
  /*! \brief the current output cell */
  DType *out_data_;
  /*! \brief internal queue of producer */
  std::queue<DType*> queue_;
  /*! \brief free cells that can be used */
  std::queue<DType*> free_cells_;
  /*! \brief holds a reference to iterator exception thrown in spawned threads */
  std::exception_ptr iter_exception_{nullptr};
};

// implementation of functions
template <typename DType> inline void ThreadedIter<DType>::Destroy(void) {
  if (producer_thread_) {
    {
      // lock the mutex
      std::lock_guard<std::mutex> lock(mutex_);
      // send destroy signal
      producer_sig_.store(kDestroy, std::memory_order_release);
      if (nwait_producer_ != 0) {
        producer_cond_.notify_one();
      }
    }
    producer_thread_.reset(nullptr);
  }
  // end of critical region
  // now the slave thread should exit
  while (free_cells_.size() != 0) {
    delete free_cells_.front();
    free_cells_.pop();
  }
  while (queue_.size() != 0) {
    delete queue_.front();
    queue_.pop();
  }
  if (producer_ != NULL) {
    producer_.reset();
  }
  if (out_data_ != NULL) {
    delete out_data_;
    out_data_ = NULL;
  }
}

template<typename DType>
inline void ThreadedIter<DType>::
Init(std::shared_ptr<Producer> producer) {
  CHECK(producer_ == NULL) << "can only call Init once";
  auto next = [producer](DType **dptr) {
      return producer->Next(dptr);
  };
  auto beforefirst = [producer]() {
    producer->BeforeFirst();
  };
  this->Init(next, beforefirst);
}

template <typename DType>
inline void ThreadedIter<DType>::Init(std::function<bool(DType **)> next,
                                      std::function<void()> beforefirst) {
  producer_sig_.store(kProduce, std::memory_order_release);
  producer_sig_processed_.store(false, std::memory_order_release);
  produce_end_.store(false, std::memory_order_release);
  ClearException();
  // procedure running in prodcuer
  // run producer thread
  auto producer_fun = [this, next, beforefirst]() {
    while (true) {
      try {
        DType *cell = NULL;
        {
          // lockscope
          std::unique_lock<std::mutex> lock(mutex_);
          ++this->nwait_producer_;
          producer_cond_.wait(lock, [this]() {
            if (producer_sig_.load(std::memory_order_acquire) == kProduce) {
              bool ret = !produce_end_.load(std::memory_order_acquire)
                         && (queue_.size() < max_capacity_ ||
                             free_cells_.size() != 0);
              return ret;
            } else {
              return true;
            }
          });
          --this->nwait_producer_;
          if (producer_sig_.load(std::memory_order_acquire) == kProduce) {
            if (free_cells_.size() != 0) {
              cell = free_cells_.front();
              free_cells_.pop();
            }
          } else if (producer_sig_.load(std::memory_order_acquire) == kBeforeFirst) {
            // reset the producer
            beforefirst();
            // cleanup the queue
            while (queue_.size() != 0) {
              free_cells_.push(queue_.front());
              queue_.pop();
            }
            // reset the state
            produce_end_.store(false, std::memory_order_release);
            producer_sig_processed_.store(true, std::memory_order_release);
            producer_sig_.store(kProduce, std::memory_order_release);
            // notify consumer that all the process as been done.
            lock.unlock();
            consumer_cond_.notify_all();
            continue;
          } else {
            // destroy the thread
            DCHECK(producer_sig_.load(std::memory_order_acquire) == kDestroy);
            producer_sig_processed_.store(true, std::memory_order_release);
            produce_end_.store(true, std::memory_order_release);
            lock.unlock();
            consumer_cond_.notify_all();
            return;
          }
        }  // end of lock scope
        // now without lock
        produce_end_.store(!next(&cell), std::memory_order_release);
        DCHECK(cell != NULL || produce_end_.load(std::memory_order_acquire));
        bool notify;
        {
          // lockscope
          std::lock_guard<std::mutex> lock(mutex_);
          if (!produce_end_.load(std::memory_order_acquire)) {
            queue_.push(cell);
          } else {
            if (cell != NULL)
              free_cells_.push(cell);
          }
          // put things into queue
          notify = nwait_consumer_ != 0;
        }
        if (notify)
          consumer_cond_.notify_all();
      } catch (std::exception &e) {
        // Shouldn't throw exception in destructor
        DCHECK(producer_sig_.load(std::memory_order_acquire) != kDestroy);
        {
          std::lock_guard<std::mutex> lock(mutex_exception_);
          if (!iter_exception_) {
            iter_exception_ = std::current_exception();
          }
        }
        bool next_notify = false;
        {
          std::unique_lock<std::mutex> lock(mutex_);
          if (producer_sig_.load(std::memory_order_acquire) == kBeforeFirst) {
            while (queue_.size() != 0) {
              free_cells_.push(queue_.front());
              queue_.pop();
            }
            produce_end_.store(true, std::memory_order_release);
            producer_sig_processed_.store(true, std::memory_order_release);
            lock.unlock();
            consumer_cond_.notify_all();
          } else if (producer_sig_.load(std::memory_order_acquire) == kProduce) {
            produce_end_.store(true, std::memory_order_release);
            next_notify = nwait_consumer_ != 0;
            lock.unlock();
            if (next_notify)
              consumer_cond_.notify_all();
          }
        }
        return;
      }
    }
  };
  producer_thread_.reset(new ScopedThread{std::thread(producer_fun)});
}

template <typename DType>
inline bool ThreadedIter<DType>::Next(DType **out_dptr) {
  if (producer_sig_.load(std::memory_order_acquire) == kDestroy)
    return false;
  ThrowExceptionIfSet();
  std::unique_lock<std::mutex> lock(mutex_);
  CHECK(producer_sig_.load(std::memory_order_acquire) == kProduce)
      << "Make sure you call BeforeFirst not inconcurrent with Next!";
  ++nwait_consumer_;
  consumer_cond_.wait(lock,
                      [this]() { return queue_.size() != 0
                                 || produce_end_.load(std::memory_order_acquire); });
  --nwait_consumer_;
  if (queue_.size() != 0) {
    *out_dptr = queue_.front();
    queue_.pop();
    bool notify = nwait_producer_ != 0
                  && !produce_end_.load(std::memory_order_acquire);
    lock.unlock();
    if (notify)
      producer_cond_.notify_one();

    ThrowExceptionIfSet();
    return true;
  } else {
    CHECK(produce_end_.load(std::memory_order_acquire));
    lock.unlock();

    ThrowExceptionIfSet();
    return false;
  }
}

template <typename DType>
inline void ThreadedIter<DType>::Recycle(DType **inout_dptr) {
  bool notify;
  ThrowExceptionIfSet();
  {
    std::lock_guard<std::mutex> lock(mutex_);
    free_cells_.push(*inout_dptr);
    *inout_dptr = NULL;
    notify = nwait_producer_ != 0 && !produce_end_.load(std::memory_order_acquire);
  }
  if (notify)
    producer_cond_.notify_one();
  ThrowExceptionIfSet();
}

template <typename DType> inline void ThreadedIter<DType>::ThrowExceptionIfSet(void) {
  std::exception_ptr tmp_exception{nullptr};
  {
    std::lock_guard<std::mutex> lock(mutex_exception_);
    if (iter_exception_) {
      tmp_exception = iter_exception_;
    }
  }
  if (tmp_exception) {
    try {
      std::rethrow_exception(tmp_exception);
    } catch (std::exception& exc) {
      LOG(FATAL) << exc.what();
    }
  }
}

template <typename DType> inline void ThreadedIter<DType>::ClearException(void) {
  std::lock_guard<std::mutex> lock(mutex_exception_);
  iter_exception_ = nullptr;
}

}  // namespace dmlc
#endif  // DMLC_USE_CXX11
#endif  // DMLC_THREADEDITER_H_
