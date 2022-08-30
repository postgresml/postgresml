#include <chrono>
#include <dmlc/io.h>
#include <dmlc/logging.h>
#include <dmlc/threadediter.h>
#include <gtest/gtest.h>

enum ExcType {
  kDMLCException,
  kStdException,
};

using namespace dmlc;
namespace producer_test {
inline void delay(int sleep) {
  if (sleep < 0) {
    int d = rand() % (-sleep);
    std::this_thread::sleep_for(std::chrono::milliseconds(d));
  } else {
    std::this_thread::sleep_for(std::chrono::milliseconds(sleep));
  }
}

// int was only used as example, in real life
// use big data blob
struct IntProducerNextExc : public ThreadedIter<int>::Producer {
  int counter;
  int maxcap;
  int sleep;
  ExcType exc_type;

  IntProducerNextExc(int maxcap, int sleep, ExcType exc_type = ExcType::kDMLCException)
      : counter(0), maxcap(maxcap), sleep(sleep), exc_type(exc_type) {}
  virtual ~IntProducerNextExc() = default;
  virtual void BeforeFirst(void) { counter = 0; }
  virtual bool Next(int **inout_dptr) {
    if (counter == maxcap)
      return false;
    if (counter == (maxcap - 1)) {
      counter++;
      if (exc_type == kDMLCException) {
        LOG(FATAL) << "Test Throw exception";
      } else {
        LOG(WARNING) << "Throw std::exception";
        throw std::exception();
      }
    }
    // allocate space if not exist
    if (*inout_dptr == NULL) {
      *inout_dptr = new int();
    }
    delay(sleep);
    **inout_dptr = counter++;
    return true;
  }
};

struct IntProducerBeforeFirst : public ThreadedIter<int>::Producer {
  ExcType exc_type;
  IntProducerBeforeFirst(ExcType exc_type = ExcType::kDMLCException)
      : exc_type(exc_type) {}
  virtual ~IntProducerBeforeFirst() = default;
  virtual void BeforeFirst(void) {
    if (exc_type == ExcType::kDMLCException) {
      LOG(FATAL) << "Throw exception in before first";
    } else {
      throw std::exception();
    }
  }
  virtual bool Next(int **inout_dptr) { return true; }
};
}

TEST(ThreadedIter, dmlc_exception) {
  using namespace producer_test;
  int* value = nullptr;
  ThreadedIter<int> iter2;
  iter2.set_max_capacity(7);
  auto prod = std::make_shared<IntProducerNextExc>(5, 100);
  bool caught = false;
  iter2.Init(prod);  // t1 is created in here, not passing ownership
  iter2.BeforeFirst();
  try {
    delay(1000);
    iter2.Recycle(&value);
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "recycle exception caught";
  }
  CHECK(caught);
  iter2.Init(prod);
  caught = false;
  iter2.BeforeFirst();
  try {
    while (iter2.Next(&value)) {
      iter2.Recycle(&value);
    }
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "next exception caught";
  }
  CHECK(caught);
  LOG(INFO) << "finish";
  ThreadedIter<int> iter3;
  iter3.set_max_capacity(1);
  auto prod2 = std::make_shared<IntProducerBeforeFirst>();
  iter3.Init(prod2);
  caught = false;
  try {
    iter3.BeforeFirst();
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "beforefirst exception caught";
  }
  caught = false;
  try {
  iter3.BeforeFirst();
  } catch (dmlc::Error &e) {
    LOG(INFO) << "beforefirst exception thrown/caught";
    caught = true;
  }
  CHECK(caught);
  delete(value);
}

TEST(ThreadedIter, std_exception) {
  using namespace producer_test;
  int *value = nullptr;
  ThreadedIter<int> iter2;
  iter2.set_max_capacity(7);
  auto prod =std::make_shared<IntProducerNextExc>(5, 100, ExcType::kStdException);
  bool caught = false;
  iter2.Init(prod);
  iter2.BeforeFirst();
  try {
    delay(1000);
    iter2.Recycle(&value);
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "recycle exception caught";
  }
  CHECK(caught);
  iter2.Init(prod);
  caught = false;
  iter2.BeforeFirst();
  try {
    while (iter2.Next(&value)) {
      iter2.Recycle(&value);
    }
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "next exception caught";
  }
  CHECK(caught);
  LOG(INFO) << "finish";
  ThreadedIter<int> iter3;
  iter3.set_max_capacity(1);
  auto prod2 = std::make_shared<IntProducerBeforeFirst>(ExcType::kStdException);
  iter3.Init(prod2);
  caught = false;
  try {
    iter3.BeforeFirst();
  } catch (dmlc::Error &e) {
    caught = true;
    LOG(INFO) << "beforefirst exception caught";
  }
  caught = false;
  try {
  iter3.BeforeFirst();
  } catch (dmlc::Error &e) {
    LOG(INFO) << "beforefirst exception thrown/caught";
    caught = true;
  }
  CHECK(caught);
  delete(value);
}
