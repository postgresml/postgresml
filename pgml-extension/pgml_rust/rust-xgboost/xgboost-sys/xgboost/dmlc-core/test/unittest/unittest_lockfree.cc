#include <dmlc/json.h>
#include <dmlc/io.h>
#include <dmlc/memory_io.h>
#include <dmlc/concurrentqueue.h>
#include <dmlc/blockingconcurrentqueue.h>
#include <dmlc/thread_group.h>
#include <gtest/gtest.h>

template<typename TQueue>
struct LFQThreadData {
  LFQThreadData() : count_(0) {}
  std::atomic<size_t> count_;
  std::shared_ptr<TQueue> q_ = std::make_shared<TQueue>();
  std::shared_ptr<dmlc::ManualEvent> ready_ = std::make_shared<dmlc::ManualEvent>();
  std::mutex cs_map_;
  std::set<int> thread_map_;
};

template<typename TQueue>
static int PushThread(const int id, std::shared_ptr<LFQThreadData<TQueue>> data) {
  ++data->count_;
  data->ready_->wait();
  data->q_->enqueue(id);
  std::unique_lock<std::mutex> lk(data->cs_map_);
  data->thread_map_.erase(id);
  return 0;
}

template<typename TQueue>
static int PullThread(const int id, std::shared_ptr<LFQThreadData<TQueue>> data) {
  ++data->count_;
  data->ready_->wait();
  int val;
  CHECK_EQ(data->q_->try_dequeue(val), true);
  std::unique_lock<std::mutex> lk(data->cs_map_);
  data->thread_map_.erase(id);
  return 0;
}

template<typename TQueue>
static int BlockingPullThread(const int id, std::shared_ptr<LFQThreadData<TQueue>> data) {
  ++data->count_;
  data->ready_->wait();
  int val;
  data->q_->wait_dequeue(val);
  std::unique_lock<std::mutex> lk(data->cs_map_);
  data->thread_map_.erase(id);
  return 0;
}

static inline std::string TName(const std::string& s, int x) { return s + "-" + std::to_string(x); }

TEST(Lockfree, ConcurrentQueue) {
  dmlc::ThreadGroup threads;
  const size_t ITEM_COUNT = 100;
  auto data = std::make_shared<LFQThreadData<dmlc::moodycamel::ConcurrentQueue<int>>>();
  for(size_t x = 0; x < ITEM_COUNT; ++x) {
    std::unique_lock<std::mutex> lk(data->cs_map_);
    data->thread_map_.insert(x);
    threads.create(TName("PushThread", x), true, PushThread<dmlc::moodycamel::ConcurrentQueue<int>>, x, data);
  }
  while(data->count_ < ITEM_COUNT) {
    std::this_thread::sleep_for(std::chrono::milliseconds(1));
  }
  data->ready_->signal();
  size_t remaining = ITEM_COUNT;
  do {
    std::this_thread::sleep_for(std::chrono::milliseconds(10));
    std::unique_lock<std::mutex> lk(data->cs_map_);
    remaining = data->thread_map_.size();
  } while (remaining);

  size_t count = data->q_->size_approx();
  GTEST_ASSERT_EQ(count, ITEM_COUNT);

  threads.join_all();
  GTEST_ASSERT_EQ(threads.size(), 0U);

  for(size_t x = 0; x < ITEM_COUNT; ++x) {
    std::unique_lock<std::mutex> lk(data->cs_map_);
    data->thread_map_.insert(x);
    // Just to mix things up, don't auto-remove
    threads.create(TName("PullThread", x), false, PullThread<dmlc::moodycamel::ConcurrentQueue<int>>, x, data);
  }
  data->ready_->signal();
  threads.join_all();
  GTEST_ASSERT_EQ(threads.size(), 0U);

  count = data->q_->size_approx();
  GTEST_ASSERT_EQ(count, 0UL);
}

TEST(Lockfree, BlockingConcurrentQueue) {
  using BlockingQueue = dmlc::moodycamel::BlockingConcurrentQueue<
    int, dmlc::moodycamel::ConcurrentQueueDefaultTraits>;

  using BlockingQueue = dmlc::moodycamel::BlockingConcurrentQueue<
    int, dmlc::moodycamel::ConcurrentQueueDefaultTraits>;

  dmlc::ThreadGroup threads;
  const size_t ITEM_COUNT = 100;
  auto data = std::make_shared<LFQThreadData<BlockingQueue>>();
  for(size_t x = 0; x < ITEM_COUNT; ++x) {
    std::unique_lock<std::mutex> lk(data->cs_map_);
    data->thread_map_.insert(x);
    // Just to mix things up, don't auto-remove
    threads.create(TName("PushThread", x), false, PushThread<BlockingQueue>, x, data);
  }
  while(data->count_ < ITEM_COUNT) {
    std::this_thread::sleep_for(std::chrono::milliseconds(1));
  }
  data->ready_->signal();
  size_t remaining = ITEM_COUNT;
  do {
    std::this_thread::sleep_for(std::chrono::milliseconds(10));
    std::unique_lock<std::mutex> lk(data->cs_map_);
    remaining = data->thread_map_.size();
  } while (remaining);

  size_t count = data->q_->size_approx();
  GTEST_ASSERT_EQ(count, ITEM_COUNT);

  threads.join_all();
  GTEST_ASSERT_EQ(threads.size(), 0U);

  for(size_t x = 0; x < ITEM_COUNT; ++x) {
    std::unique_lock<std::mutex> lk(data->cs_map_);
    data->thread_map_.insert(static_cast<int>(x));
    threads.create(TName("BlockingPullThread", x), true, BlockingPullThread<BlockingQueue>, x, data);
  }
  data->ready_->signal();
  threads.join_all();
  GTEST_ASSERT_EQ(threads.size(), 0U);

  count = data->q_->size_approx();
  GTEST_ASSERT_EQ(count, 0UL);
}

