// Copyright by Contributors
#define DMLC_LOG_FATAL_THROW 1

#include <dmlc/logging.h>
#include <gtest/gtest.h>

TEST(Logging, basics) {
  LOG(INFO) << "hello";
  LOG(ERROR) << "error";

  int x = 1, y = 1;
  CHECK_EQ(x, y);
  CHECK_GE(x, y);

  int *z = &x;
  CHECK_EQ(*CHECK_NOTNULL(z), x);

  EXPECT_THROW(CHECK_NE(x, y), dmlc::Error);
}

TEST(Logging, signed_compare) {
  int32_t x = 1;
  uint32_t y = 2;
  CHECK_GT(y, x);

  EXPECT_THROW(CHECK_EQ(x, y), dmlc::Error);
}

TEST(Logging, expression_in_check) {
  uint32_t y = 64;
  CHECK_EQ(y & (y - 1), 0);
}

TEST(Logging, extra_message) {
  uint32_t y = 64;
  CHECK_EQ(y & (y - 1), 0) << y << " has to be power of 2";
}

TEST(Logging, single_evaluation) {
  uint32_t y = 1;
  try {
    CHECK_EQ(y++, 2);
    FAIL() << "y = 1; CHECK_EQ(y++, 2) must throw an exception";
  } catch (std::runtime_error& exception) {
    // if everything is correct, y++ is evaluated only once, and '1' would be
    // mentioned in error message. This relies on specific format of error message,
    // if it changes, this unit test will have to be changed as well.
    EXPECT_NE(std::string(exception.what()).find("(1 vs"), std::string::npos);
  } catch (...) {
    FAIL() << "unexpected exception in CHECK_EQ(y++, 2)"; 
  }
}

TEST(Logging, throw_fatal) {
  EXPECT_THROW({
    LOG(FATAL) << "message";
  }, dmlc::Error);
}
