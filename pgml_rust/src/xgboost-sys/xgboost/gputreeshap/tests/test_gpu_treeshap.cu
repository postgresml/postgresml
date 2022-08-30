/*
 * Copyright (c) 2020, NVIDIA CORPORATION.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <GPUTreeShap/gpu_treeshap.h>
#include <cooperative_groups.h>
#include <limits>
#include <numeric>
#include <random>
#include <vector>
#include "gtest/gtest.h"
#include "tests/test_utils.h"
#include "../GPUTreeShap/gpu_treeshap.h"

using namespace gpu_treeshap;  // NOLINT

class ParameterisedModelTest
    : public ::testing::TestWithParam<
          std::tuple<size_t, size_t, size_t, size_t, size_t>> {
 protected:
  ParameterisedModelTest() {
    size_t max_depth, num_paths;
    std::tie(num_rows, num_features, num_groups, max_depth, num_paths) =
        GetParam();
    model = GenerateEnsembleModel(num_groups, max_depth, num_features,
                                  num_paths, 78);
    test_data = TestDataset(num_rows, num_features, 22);
    margin = Predict(model, test_data, num_groups);

    X = test_data.GetDeviceWrapper();

    phis.resize(X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1) *
                num_groups);
  }
  std::vector<PathElement<XgboostSplitCondition>> model;
  TestDataset test_data;
  DenseDatasetWrapper X;
  std::vector<float> margin;
  thrust::device_vector<float> phis;
  size_t num_groups;
  size_t num_rows;
  size_t num_features;
};

TEST_P(ParameterisedModelTest, ShapSum) {
  GPUTreeShap(X, model.begin(), model.end(), num_groups, phis.begin(),
              phis.end());
  thrust::host_vector<float> result(phis);
  std::vector<float> tmp(result.begin(), result.end());
  std::vector<float> sum(num_rows * num_groups);
  for (auto i = 0ull; i < num_rows; i++) {
    for (auto j = 0ull; j < num_features + 1; j++) {
      for (auto group = 0ull; group < num_groups; group++) {
        size_t result_index = IndexPhi(i, num_groups, group, num_features, j);
        sum[i * num_groups + group] += result[result_index];
      }
    }
  }
  for (auto i = 0ull; i < sum.size(); i++) {
    ASSERT_NEAR(sum[i], margin[i], 1e-3);
  }
}

TEST_P(ParameterisedModelTest, ShapInteractionsSum) {
  thrust::device_vector<float> phis_interactions(
      X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1) * num_groups);
  GPUTreeShap(X, model.begin(), model.end(), num_groups, phis.begin(),
              phis.end());
  GPUTreeShapInteractions(X, model.begin(), model.end(), num_groups,
                          phis_interactions.begin(), phis_interactions.end());
  thrust::host_vector<float> interactions_result(phis_interactions);
  std::vector<float> sum(phis.size());
  for (auto row_idx = 0ull; row_idx < num_rows; row_idx++) {
    for (auto group = 0ull; group < num_groups; group++) {
      for (auto i = 0ull; i < num_features + 1; i++) {
        for (auto j = 0ull; j < num_features + 1; j++) {
          size_t result_index = IndexPhiInteractions(row_idx, num_groups, group,
                                                     num_features, i, j);
          sum[IndexPhi(row_idx, num_groups, group, num_features, i)] +=
              interactions_result[result_index];
        }
      }
    }
  }

  thrust::host_vector<float> phis_host(phis);
  for (auto i = 0ull; i < sum.size(); i++) {
    ASSERT_NEAR(sum[i], phis_host[i], 1e-3);
  }
}

TEST_P(ParameterisedModelTest, ShapTaylorInteractionsSum) {
  GPUTreeShapTaylorInteractions(X, model.begin(), model.end(), num_groups,
                                phis.begin(), phis.end());
  thrust::host_vector<float> interactions_result(phis);
  std::vector<float> sum(margin.size());
  for (auto row_idx = 0ull; row_idx < num_rows; row_idx++) {
    for (auto group = 0ull; group < num_groups; group++) {
      for (auto i = 0ull; i < num_features + 1; i++) {
        for (auto j = 0ull; j < num_features + 1; j++) {
          size_t result_index = IndexPhiInteractions(row_idx, num_groups, group,
                                                     num_features, i, j);
          sum[row_idx * num_groups + group] +=
              interactions_result[result_index];
        }
      }
    }
  }

  for (auto i = 0ull; i < sum.size(); i++) {
    ASSERT_NEAR(sum[i], margin[i], 1e-3);
  }
}

TEST_P(ParameterisedModelTest, ShapSumInterventional) {
  auto r_test_data = TestDataset(400, num_features, 10);
  auto R = r_test_data.GetDeviceWrapper();
  GPUTreeShapInterventional(X, R, model.begin(), model.end(), num_groups,
                            phis.begin(), phis.end());
  thrust::host_vector<float> result(phis);
  std::vector<float> tmp(result.begin(), result.end());
  std::vector<float> sum(num_rows * num_groups);
  for (auto i = 0ull; i < num_rows; i++) {
    for (auto j = 0ull; j < num_features + 1; j++) {
      for (auto group = 0ull; group < num_groups; group++) {
        size_t result_index = IndexPhi(i, num_groups, group, num_features, j);
        sum[i * num_groups + group] += result[result_index];
      }
    }
  }
  for (auto i = 0ull; i < sum.size(); i++) {
    ASSERT_NEAR(sum[i], margin[i], 1e-3);
  }
}

std::string PrintTestName(
    const testing::TestParamInfo<ParameterisedModelTest::ParamType>& info) {
  std::string name = "nrow" + std::to_string(std::get<0>(info.param)) + "_";
  name += "nfeat" + std::to_string(std::get<1>(info.param)) + "_";
  name += "ngroup" + std::to_string(std::get<2>(info.param)) + "_";
  name += "mdepth" + std::to_string(std::get<3>(info.param)) + "_";
  name += "npaths" + std::to_string(std::get<4>(info.param));
  return name;
}

// Generate a bunch of random models and check the shap results sum up to the
// predictions
size_t test_num_rows[] = {1, 10, 100, 1000};
size_t test_num_features[] = {1, 5, 8, 31};
size_t test_num_groups[] = {1, 5};
size_t test_max_depth[] = {1, 8, 20};
size_t test_num_paths[] = {1, 10};
INSTANTIATE_TEST_CASE_P(ShapInstantiation, ParameterisedModelTest,
                        testing::Combine(testing::ValuesIn(test_num_rows),
                                         testing::ValuesIn(test_num_features),
                                         testing::ValuesIn(test_num_groups),
                                         testing::ValuesIn(test_max_depth),
                                         testing::ValuesIn(test_num_paths)),
                        PrintTestName);

#define EXPECT_THROW_CONTAINS_MESSAGE(stmt, etype, whatstring)             \
  EXPECT_THROW(try { stmt; } catch (const etype& ex) {                     \
    EXPECT_NE(std::string(ex.what()).find(whatstring), std::string::npos); \
    throw;                                                                 \
  },                                                                       \
               etype)

class APITest : public ::testing::Test {
 protected:
  APITest() {
    const float inf = std::numeric_limits<float>::infinity();
    model = {
        {0, -1, 0, {-inf, inf, false}, 1.0f, 2.0f},
        {0, 0, 0, {0.5f, inf, false}, 0.25f, 2.0f},
        {0, 1, 0, {0.5f, inf, false}, 0.5f, 2.0f},
        {0, 2, 0, {0.5f, inf, false}, 0.6f, 2.0f},
        {0, 3, 0, {0.5f, inf, false}, 1.0f, 2.0f},
    };
    data = std::vector<float>({1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f});
    X = DenseDatasetWrapper(data.data().get(), 2, 4);
    phis.resize((X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1)));
  }
  template <typename ExceptionT>
  void ExpectAPIThrow(std::string message) {
    EXPECT_THROW_CONTAINS_MESSAGE(
        GPUTreeShap(X, model.begin(), model.end(), 1, phis.begin(), phis.end()),
        ExceptionT, message);
    EXPECT_THROW_CONTAINS_MESSAGE(
        GPUTreeShapInteractions(X, model.begin(), model.end(), 1, phis.begin(),
                                phis.end()),
        ExceptionT, message);
    EXPECT_THROW_CONTAINS_MESSAGE(
        GPUTreeShapTaylorInteractions(X, model.begin(), model.end(), 1,
                                      phis.begin(), phis.end()),
        ExceptionT, message);
  }

  thrust::device_vector<float> data;
  std::vector<PathElement<XgboostSplitCondition>> model;
  DenseDatasetWrapper X;
  thrust::device_vector<float> phis;
};

TEST_F(APITest, PathTooLong) {
  model.resize(33);
  model[0] = {0, -1, 0, {0, 0, 0}, 0, 0};
  for (size_t i = 1; i < model.size(); i++) {
    model[i] = {0, static_cast<int64_t>(i), 0, {0, 0, 0}, 0, 0};
  }
  ExpectAPIThrow<std::invalid_argument>("Tree depth must be <= 32");
}

TEST_F(APITest, PathVIncorrect) {
  model = {{0, -1, 0, {0.0f, 0.0f, false}, 0.0, 1.0f},
           {0, 0, 0, {0.0f, 0.0f, false}, 0.0f, 0.5f}};

  ExpectAPIThrow<std::invalid_argument>(
      "Leaf value v should be the same across a single path");
}

TEST_F(APITest, PhisIncorrectLength) {
  phis.resize(1);
  ExpectAPIThrow<std::invalid_argument>("phis_out must be at least of size");
}

// Test a simple tree and compare output to xgb shap values
// 0:[f0<0.5] yes=1,no=2,missing=1,gain=1.63333321,cover=5
//  1:leaf=-1,cover=2
//  2:[f1<0.5] yes=3,no=4,missing=3,gain=2.04166675,cover=3
//    3:leaf=-1,cover=1
//    4:[f2<0.5] yes=5,no=6,missing=5,gain=0.125,cover=2
//      5:leaf=1,cover=1
//      6:leaf=0.5,cover=1
TEST(GPUTreeShap, BasicPaths) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, inf, false}, 1.0f, 0.5f},
      {0, 0, 0, {0.5f, inf, false}, 0.6f, 0.5f},
      {0, 1, 0, {0.5f, inf, false}, 2.0f / 3, 0.5f},
      {0, 2, 0, {0.5f, inf, false}, 0.5f, 0.5f},
      {1, -1, 0, {-inf, 0.0f, false}, 1.0f, 1.0f},
      {1, 0, 0, {0.5f, inf, false}, 0.6f, 1.0f},
      {1, 1, 0, {0.5f, inf, false}, 2.0f / 3, 1.0f},
      {1, 2, 0, {-inf, 0.5f, false}, 0.5f, 1.0f},
      {2, -1, 0, {-inf, 0.0f, false}, 1.0f, -1},
      {2, 0, 0, {0.5f, inf, false}, 0.6f, -1.0f},
      {2, 1, 0, {-inf, 0.5f, false}, 1.0f / 3, -1.0f},
      {3, -1, 0, {-inf, 0.0f, false}, 1.0f, -1.0f},
      {3, 0, 0, {-inf, 0.5f, false}, 0.4f, -1.0f}};
  thrust::device_vector<float> data =
      std::vector<float>({1.0f, 1.0f, 0.0f, 1.0f, 0.0f, 0.0f});
  DenseDatasetWrapper X(data.data().get(), 2, 3);
  size_t num_trees = 1;
  thrust::device_vector<float> phis(X.NumRows() * (X.NumCols() + 1));
  GPUTreeShap(X, path.begin(), path.end(), 1, phis.begin(), phis.end());
  thrust::host_vector<float> result(phis);
  // First instance
  EXPECT_NEAR(result[0], 0.6277778f * num_trees, 1e-5);
  EXPECT_NEAR(result[1], 0.5027776f * num_trees, 1e-5);
  EXPECT_NEAR(result[2], 0.1694444f * num_trees, 1e-5);
  EXPECT_NEAR(result[3], -0.3f * num_trees, 1e-5);
  // Second instance
  EXPECT_NEAR(result[4], 0.24444449f * num_trees, 1e-5);
  EXPECT_NEAR(result[5], -1.005555f * num_trees, 1e-5);
  EXPECT_NEAR(result[6], 0.0611111f * num_trees, 1e-5);
  EXPECT_NEAR(result[7], -0.3f * num_trees, 1e-5);
}

TEST(GPUTreeShap, BasicPathsInteractions) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, inf, false}, 1.0f, 0.5f},
      {0, 0, 0, {0.5f, inf, false}, 0.6f, 0.5f},
      {0, 1, 0, {0.5f, inf, false}, 2.0f / 3, 0.5f},
      {0, 2, 0, {0.5f, inf, false}, 0.5f, 0.5f},
      {1, -1, 0, {-inf, 0.0f, false}, 1.0f, 1.0f},
      {1, 0, 0, {0.5f, inf, false}, 0.6f, 1.0f},
      {1, 1, 0, {0.5f, inf, false}, 2.0f / 3, 1.0f},
      {1, 2, 0, {-inf, 0.5f, false}, 0.5f, 1.0f},
      {2, -1, 0, {-inf, 0.0f, false}, 1.0f, -1},
      {2, 0, 0, {0.5f, inf, false}, 0.6f, -1.0f},
      {2, 1, 0, {-inf, 0.5f, false}, 1.0f / 3, -1.0f},
      {3, -1, 0, {-inf, 0.0f, false}, 1.0f, -1.0f},
      {3, 0, 0, {-inf, 0.5f, false}, 0.4f, -1.0f}};
  thrust::device_vector<float> data =
      std::vector<float>({1.0f, 1.0f, 0.0f, 1.0f, 1.0f, 1.0f});
  DenseDatasetWrapper X(data.data().get(), 2, 3);
  thrust::device_vector<float> phis(X.NumRows() * (X.NumCols() + 1) *
                                    (X.NumCols() + 1));
  GPUTreeShapInteractions(X, path.begin(), path.end(), 1, phis.begin(),
                          phis.end());
  std::vector<float> result(phis.begin(), phis.end());
  std::vector<float> expected_result = {
      0.46111116,  0.125,       0.04166666,  0.,          0.125,
      0.34444442,  0.03333333,  0.,          0.04166666,  0.03333335,
      0.09444444,  0.,          0.,          0.,          0.,
      -0.3,        0.47222224,  0.1083333,   -0.04166666, 0.,
      0.10833332,  0.35555553,  -0.03333333, 0.,          -0.04166666,
      -0.03333332, -0.09444447, 0.,          0.,          0.,
      0.,          -0.3};
  for (auto i = 0ull; i < result.size(); i++) {
    EXPECT_NEAR(result[i], expected_result[i], 1e-5);
  }
}

// Test a tree with features occurring multiple times in a path
TEST(GPUTreeShap, BasicPathsWithDuplicates) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, 0.0f, false}, 1.0f, 3.0f},
      {0, 0, 0, {0.5f, inf, false}, 2.0f / 3, 3.0f},
      {0, 0, 0, {1.5f, inf, false}, 0.5f, 3.0f},
      {0, 0, 0, {2.5f, inf, false}, 0.5f, 3.0f},
      {1, -1, 0, {-inf, 0.0f, false}, 1.0f, 2.0f},
      {1, 0, 0, {0.5f, inf, false}, 2.0f / 3.0f, 2.0f},
      {1, 0, 0, {1.5f, inf, false}, 0.5f, 2.0f},
      {1, 0, 0, {-inf, 2.5f, false}, 0.5f, 2.0f},
      {2, -1, 0, {-inf, 0.0f, false}, 1.0f, 1.0f},
      {2, 0, 0, {0.5f, inf, false}, 2.0f / 3.0f, 1.0f},
      {2, 0, 0, {-inf, 1.5f, false}, 0.5f, 1.0f},
      {3, -1, 0, {-inf, 0.0f, false}, 1.0f, -1.0f},
      {3, 0, 0, {-inf, 0.5f, false}, 1.0f / 3, -1.0f}};
  thrust::device_vector<float> data = std::vector<float>({2.0f});
  DenseDatasetWrapper X(data.data().get(), 1, 1);
  size_t num_trees = 1;
  thrust::device_vector<float> phis(X.NumRows() * (X.NumCols() + 1));
  GPUTreeShap(X, path.begin(), path.end(), 1, phis.begin(), phis.end());
  thrust::host_vector<float> result(phis);
  // First instance
  EXPECT_FLOAT_EQ(result[0], 1.1666666f * num_trees);
  EXPECT_FLOAT_EQ(result[1], 0.83333337f * num_trees);
}

__device__ bool FloatApproximatelyEqual(float a, float b) {
  const float kEps = 1e-5;
  return fabs(a - b) < kEps;
}

// Expose pweight for testing
class TestGroupPath : public detail::GroupPath {
 public:
  __device__ TestGroupPath(const detail::ContiguousGroup& g,
                           float zero_fraction, float one_fraction)
      : detail::GroupPath(g, zero_fraction, one_fraction) {}
  using detail::GroupPath::pweight_;
  using detail::GroupPath::unique_depth_;
};

template <typename DatasetT, typename SplitConditionT>
__global__ void TestExtendKernel(
    DatasetT X, size_t num_path_elements,
    const PathElement<SplitConditionT>* path_elements) {
  cooperative_groups::thread_block block =
      cooperative_groups::this_thread_block();
  auto group =
      cooperative_groups::tiled_partition<32, cooperative_groups::thread_block>(
          block);
  bool thread_active = threadIdx.x < num_path_elements;
  uint32_t mask = __ballot_sync(FULL_MASK, thread_active);
  if (!thread_active) return;

  // Test first training instance
  cooperative_groups::coalesced_group active_group =
      cooperative_groups::coalesced_threads();
  PathElement<SplitConditionT> e = path_elements[active_group.thread_rank()];
  float one_fraction =
      e.split_condition.EvaluateSplit(X.GetElement(0, e.feature_idx));
  float zero_fraction = e.zero_fraction;
  auto labelled_group = detail::active_labeled_partition(mask, 0);
  TestGroupPath path(labelled_group, zero_fraction, one_fraction);
  path.Extend();
  assert(path.unique_depth_ == 1);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.3f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.5f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  path.Extend();
  assert(path.unique_depth_ == 2);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.133333f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.21111f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.33333f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  path.Extend();
  assert(path.unique_depth_ == 3);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.05f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.086111f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.147222f));
  } else if (active_group.thread_rank() == 3) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.25f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  float unwound_sum = path.UnwoundPathSum();

  if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.63888f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.61666f));
  } else if (active_group.thread_rank() == 3) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.67777f));
  } else if (active_group.thread_rank() > 3) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.0f));
  }

  // Test second training instance
  one_fraction =
      e.split_condition.EvaluateSplit(X.GetElement(1, e.feature_idx));
  TestGroupPath path2(labelled_group, zero_fraction, one_fraction);
  path2.Extend();
  assert(path2.unique_depth_ == 1);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.3f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.5f));
  } else {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.0f));
  }

  path2.Extend();
  assert(path2.unique_depth_ == 2);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.133333f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.11111f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.0f));
  } else {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.0f));
  }

  path2.Extend();
  assert(path2.unique_depth_ == 3);
  if (active_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.05f));
  } else if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.06111f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.05555f));
  } else if (active_group.thread_rank() == 3) {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.0f));
  } else {
    assert(FloatApproximatelyEqual(path2.pweight_, 0.0f));
  }

  unwound_sum = path2.UnwoundPathSum();

  if (active_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.22222f));
  } else if (active_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.61666f));
  } else if (active_group.thread_rank() == 3) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.244444f));
  } else if (active_group.thread_rank() > 3) {
    assert(FloatApproximatelyEqual(unwound_sum, 0.0f));
  }
}

TEST(GPUTreeShap, Extend) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, 0.0f, false}, 1.0f, 1.0f},
      {0, 0, 0, {0.5f, inf, false}, 3.0f / 5, 1.0f},
      {0, 1, 0, {0.5f, inf, false}, 2.0f / 3, 1.0f},
      {0, 2, 0, {-inf, 0.5f, false}, 1.0f / 2, 1.0f}};
  thrust::device_vector<PathElement<XgboostSplitCondition>> device_path(path);
  thrust::device_vector<float> data =
      std::vector<float>({1.0f, 1.0f, 0.0f, 1.0f, 0.0f, 0.0f});
  DenseDatasetWrapper X(data.data().get(), 2, 3);
  TestExtendKernel<<<1, 32>>>(X, 4, device_path.data().get());
}
template <typename DatasetT, typename SplitConditionT>
__global__ void TestExtendMultipleKernel(
    DatasetT X, size_t n_first, size_t n_second,
    const PathElement<SplitConditionT>* path_elements) {
  cooperative_groups::thread_block block =
      cooperative_groups::this_thread_block();
  auto warp =
      cooperative_groups::tiled_partition<32, cooperative_groups::thread_block>(
          block);
  bool thread_active = threadIdx.x < n_first + n_second;
  uint32_t mask = __ballot_sync(FULL_MASK, thread_active);
  if (!thread_active) return;
  cooperative_groups::coalesced_group active_group =
      cooperative_groups::coalesced_threads();
  int label = warp.thread_rank() >= n_first;
  auto labeled_group = detail::active_labeled_partition(mask, label);
  PathElement<SplitConditionT> e = path_elements[warp.thread_rank()];

  // Test first training instance
  float one_fraction =
      e.split_condition.EvaluateSplit(X.GetElement(0, e.feature_idx));
  float zero_fraction = e.zero_fraction;
  TestGroupPath path(labeled_group, zero_fraction, one_fraction);
  assert(path.unique_depth_ == 0);
  if (labeled_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 1.0f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  path.Extend();
  assert(path.unique_depth_ == 1);
  if (labeled_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.3f));
  } else if (labeled_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.5f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  path.Extend();
  assert(path.unique_depth_ == 2);
  if (labeled_group.thread_rank() == 0) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.133333f));
  } else if (labeled_group.thread_rank() == 1) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.21111f));
  } else if (labeled_group.thread_rank() == 2) {
    assert(FloatApproximatelyEqual(path.pweight_, 0.33333f));
  } else {
    assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
  }

  // Extend the first group only
  if (label == 0) {
    path.Extend();
    assert(path.unique_depth_ == 3);
    if (labeled_group.thread_rank() == 0) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.05f));
    } else if (labeled_group.thread_rank() == 1) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.086111f));
    } else if (labeled_group.thread_rank() == 2) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.147222f));
    } else if (labeled_group.thread_rank() == 3) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.25f));
    } else {
      assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
    }
  } else {
    assert(path.unique_depth_ == 2);
    if (labeled_group.thread_rank() == 0) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.133333f));
    } else if (labeled_group.thread_rank() == 1) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.21111f));
    } else if (labeled_group.thread_rank() == 2) {
      assert(FloatApproximatelyEqual(path.pweight_, 0.33333f));
    } else {
      assert(FloatApproximatelyEqual(path.pweight_, 0.0f));
    }
  }
  if (label == 0) {
    float unwound_sum = path.UnwoundPathSum();

    if (labeled_group.thread_rank() == 1) {
      assert(FloatApproximatelyEqual(unwound_sum, 0.63888f));
    } else if (labeled_group.thread_rank() == 2) {
      assert(FloatApproximatelyEqual(unwound_sum, 0.61666f));
    } else if (labeled_group.thread_rank() == 3) {
      assert(FloatApproximatelyEqual(unwound_sum, 0.67777f));
    } else if (labeled_group.thread_rank() > 3) {
      assert(FloatApproximatelyEqual(unwound_sum, 0.0f));
    }
  }
}

TEST(GPUTreeShap, ExtendMultiplePaths) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, 0.0f, false}, 1.0f, 1.0f},
      {0, 0, 0, {0.5f, inf, false}, 3.0f / 5, 1.0f},
      {0, 1, 0, {0.5f, inf, false}, 2.0f / 3, 1.0f},
      {0, 2, 0, {-inf, 0.5f, false}, 1.0f / 2, 1.0f}};
  // Add the first three elements again
  path.emplace_back(path[0]);
  path.emplace_back(path[1]);
  path.emplace_back(path[2]);

  thrust::device_vector<PathElement<XgboostSplitCondition>> device_path(path);
  thrust::device_vector<float> data =
      std::vector<float>({1.0f, 1.0f, 0.0f, 1.0f, 0.0f, 0.0f});
  DenseDatasetWrapper X(data.data().get(), 2, 3);
  TestExtendMultipleKernel<<<1, 32>>>(X, 4, 3, device_path.data().get());
}

__global__ void TestActiveLabeledPartition() {
  cooperative_groups::thread_block block =
      cooperative_groups::this_thread_block();
  auto warp =
      cooperative_groups::tiled_partition<32, cooperative_groups::thread_block>(
          block);
  int label = warp.thread_rank() < 5 ? 3 : 6;
  auto labelled_partition = detail::active_labeled_partition(FULL_MASK, label);

  if (label == 3) {
    assert(labelled_partition.size() == 5);
    assert(labelled_partition.thread_rank() == warp.thread_rank());
  } else if (label == 6) {
    assert(labelled_partition.size() == 32 - 5);
    assert(labelled_partition.thread_rank() == warp.thread_rank() - 5);
  }

  bool odd = warp.thread_rank() % 2 == 1;
  uint32_t odd_mask = __ballot_sync(FULL_MASK, odd);
  uint32_t even_mask = __ballot_sync(FULL_MASK, !odd);
  if (odd) {
    auto labelled_partition2 =
        detail::active_labeled_partition(odd_mask, label);
    if (label == 3) {
      assert(labelled_partition2.size() == 2);
      assert(labelled_partition2.thread_rank() == warp.thread_rank() / 2);
    } else if (label == 6) {
      assert(labelled_partition2.size() == 14);
      assert(labelled_partition2.thread_rank() == (warp.thread_rank() / 2) - 2);
    }
  } else {
    auto labelled_partition2 =
        detail::active_labeled_partition(even_mask, label);
    if (label == 3) {
      assert(labelled_partition2.size() == 3);
      assert(labelled_partition2.thread_rank() == warp.thread_rank() / 2);
    } else if (label == 6) {
      assert(labelled_partition2.size() == 13);
      assert(labelled_partition2.thread_rank() == (warp.thread_rank() / 2) - 3);
    }
  }
}

TEST(GPUTreeShap, ActiveLabeledPartition) {
  TestActiveLabeledPartition<<<1, 32>>>();
  EXPECT_EQ(cudaDeviceSynchronize(), 0);
}

TEST(GPUTreeShap, BFDBinPacking) {
  thrust::device_vector<int> counts(3);
  counts[0] = 2;
  counts[1] = 2;
  counts[2] = 1;
  auto bin_packing = detail::BFDBinPacking(counts, 3);
  EXPECT_EQ(bin_packing[0], 0u);
  EXPECT_EQ(bin_packing[1], 1u);
  EXPECT_EQ(bin_packing[2], 0u);

  counts.clear();
  counts.resize(12);
  counts[0] = 3;
  counts[1] = 3;
  counts[2] = 3;
  counts[3] = 3;
  counts[4] = 3;
  counts[5] = 3;
  counts[6] = 2;
  counts[7] = 2;
  counts[8] = 2;
  counts[9] = 2;
  counts[10] = 2;
  counts[11] = 2;
  bin_packing = detail::BFDBinPacking(counts, 10);
  EXPECT_EQ(bin_packing[0], 0u);
  EXPECT_EQ(bin_packing[1], 0u);
  EXPECT_EQ(bin_packing[2], 0u);
  EXPECT_EQ(bin_packing[3], 1u);
  EXPECT_EQ(bin_packing[4], 1u);
  EXPECT_EQ(bin_packing[5], 1u);
  EXPECT_EQ(bin_packing[6], 2u);
  EXPECT_EQ(bin_packing[7], 2u);
  EXPECT_EQ(bin_packing[8], 2u);
  EXPECT_EQ(bin_packing[9], 2u);
  EXPECT_EQ(bin_packing[10], 2u);
  EXPECT_EQ(bin_packing[11], 3u);
}

TEST(GPUTreeShap, NFBinPacking) {
  thrust::device_vector<int> counts(4);
  counts[0] = 3;
  counts[1] = 3;
  counts[2] = 1;
  counts[3] = 2;
  auto bin_packing = detail::NFBinPacking(counts, 5);
  EXPECT_EQ(bin_packing[0], 0u);
  EXPECT_EQ(bin_packing[1], 1u);
  EXPECT_EQ(bin_packing[2], 1u);
  EXPECT_EQ(bin_packing[3], 2u);
}

TEST(GPUTreeShap, FFDBinPacking) {
  thrust::device_vector<int> counts(5);
  counts[0] = 3;
  counts[1] = 2;
  counts[2] = 3;
  counts[3] = 4;
  counts[4] = 1;
  auto bin_packing = detail::FFDBinPacking(counts, 5);
  EXPECT_EQ(bin_packing[0], 1u);
  EXPECT_EQ(bin_packing[1], 1u);
  EXPECT_EQ(bin_packing[2], 2u);
  EXPECT_EQ(bin_packing[3], 0u);
  EXPECT_EQ(bin_packing[4], 0u);
}

__global__ void TestContiguousGroup() {
  int label = threadIdx.x > 2 && threadIdx.x < 6 ? 1 : threadIdx.x >= 6 ? 2 : 0;

  auto group = detail::active_labeled_partition(FULL_MASK, label);

  if (label == 1) {
    assert(group.size() == 3);
    assert(group.thread_rank() == threadIdx.x - 3);
    int up = group.shfl_up(threadIdx.x, 1);
    if (group.thread_rank() > 0) {
      assert(up == threadIdx.x - 1);
    }
    assert(group.shfl(threadIdx.x, 2) == 5);
  }
}

TEST(GPUTreeShap, ContiguousGroup) {
  TestContiguousGroup<<<1, 32>>>();
  EXPECT_EQ(cudaDeviceSynchronize(), 0);
}

class DeterminismTest : public ::testing::Test {
 protected:
  DeterminismTest() {
    size_t num_rows = 100;
    size_t num_features = 100;
    num_groups = 1;
    size_t max_depth = 10;
    size_t num_paths = 1000;
    samples = 100;
    model = GenerateEnsembleModel(num_groups, max_depth, num_features,
                                  num_paths, 78);
    test_data = TestDataset(num_rows, num_features, 22, 1e-15);

    X = test_data.GetDeviceWrapper();

    reference_phis.resize(X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1) *
                          num_groups);
  }

  std::vector<PathElement<XgboostSplitCondition>> model;
  TestDataset test_data;
  DenseDatasetWrapper X;
  size_t samples;
  size_t num_groups;
  thrust::device_vector<float> reference_phis;
};

TEST_F(DeterminismTest, GPUTreeShap) {
  GPUTreeShap(X, model.begin(), model.end(), num_groups, reference_phis.begin(),
              reference_phis.end());

  for (auto i = 0ull; i < samples; i++) {
    thrust::device_vector<float> phis(reference_phis.size());
    GPUTreeShap(X, model.begin(), model.end(), num_groups, phis.begin(),
                phis.end());
    ASSERT_TRUE(thrust::equal(reference_phis.begin(), reference_phis.end(),
                              phis.begin()));
  }
}

TEST_F(DeterminismTest, GPUTreeShapInteractions) {
  GPUTreeShapInteractions(X, model.begin(), model.end(), num_groups,
                          reference_phis.begin(), reference_phis.end());

  for (auto i = 0ull; i < samples; i++) {
    thrust::device_vector<float> phis(reference_phis.size());
    GPUTreeShapInteractions(X, model.begin(), model.end(), num_groups,
                            phis.begin(), phis.end());
    ASSERT_TRUE(thrust::equal(reference_phis.begin(), reference_phis.end(),
                              phis.begin()));
  }
}

TEST_F(DeterminismTest, GPUTreeShapTaylorInteractions) {
  GPUTreeShapTaylorInteractions(X, model.begin(), model.end(), num_groups,
                                reference_phis.begin(), reference_phis.end());

  for (auto i = 0ull; i < samples; i++) {
    thrust::device_vector<float> phis(reference_phis.size());
    GPUTreeShapTaylorInteractions(X, model.begin(), model.end(), num_groups,
                                  phis.begin(), phis.end());
    ASSERT_TRUE(thrust::equal(reference_phis.begin(), reference_phis.end(),
                              phis.begin()));
  }
}

// Example from page 10 section 4.1
// Dhamdhere, Kedar, Ashish Agarwal, and Mukund Sundararajan. "The Shapley
// Taylor Interaction Index." arXiv preprint arXiv:1902.05622 (2019).
TEST(GPUTreeShap, TaylorInteractionsPaperExample) {
  const float inf = std::numeric_limits<float>::infinity();
  float c = 3.0f;
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, inf, false}, 1.0f, 1.0f},
      {0, 0, 0, {0.5f, inf, false}, 0.0f, 1.0f},
      {1, -1, 0, {-inf, inf, false}, 1.0f, 1.0f},
      {1, 1, 0, {0.5f, inf, false}, 0.0f, 1.0f},
      {2, -1, 0, {-inf, inf, false}, 1.0f, 1.0f},
      {2, 2, 0, {0.5f, inf, false}, 0.0f, 1.0f},
      {3, -1, 0, {-inf, inf, false}, 1.0f, c},
      {3, 0, 0, {0.5f, inf, false}, 0.0f, c},
      {3, 1, 0, {0.5f, inf, false}, 0.0f, c},
      {3, 2, 0, {0.5f, inf, false}, 0.0f, c},
  };
  thrust::device_vector<float> data = std::vector<float>({1.0f, 1.0f, 1.0f});
  DenseDatasetWrapper X(data.data().get(), 1, 3);
  thrust::device_vector<float> interaction_phis(
      X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1));
  GPUTreeShapTaylorInteractions(X, path.begin(), path.end(), 1,
                                interaction_phis.begin(),
                                interaction_phis.end());

  std::vector<float> interactions_result(interaction_phis.begin(),
                                         interaction_phis.end());
  std::vector<float> expected_result = {1.0, 0.5, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0,
                                        0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0};

  ASSERT_EQ(interaction_phis, expected_result);
}

TEST(GPUTreeShap, TaylorInteractionsBasic) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, inf, false}, 1.0f, 2.0f},
      {0, 0, 0, {0.5f, inf, false}, 0.25f, 2.0f},
      {0, 1, 0, {0.5f, inf, false}, 0.5f, 2.0f},
      {0, 2, 0, {0.5f, inf, false}, 0.6f, 2.0f},
      {0, 3, 0, {0.5f, inf, false}, 1.0f, 2.0f},
  };
  thrust::device_vector<float> data =
      std::vector<float>({1.0f, 1.0f, 1.0f, 1.0f});
  DenseDatasetWrapper X(data.data().get(), 1, 4);
  thrust::device_vector<float> interaction_phis(
      X.NumRows() * (X.NumCols() + 1) * (X.NumCols() + 1));
  GPUTreeShapTaylorInteractions(X, path.begin(), path.end(), 1,
                                interaction_phis.begin(),
                                interaction_phis.end());

  thrust::host_vector<float> interactions_result(interaction_phis);
  float sum =
      std::accumulate(interaction_phis.begin(), interaction_phis.end(), 0.0f);

  ASSERT_FLOAT_EQ(sum, 2.0f);
}


TEST(GPUTreeShap, GetWCoefficients) {
  EXPECT_DOUBLE_EQ(detail::W(0, 1), 1.0);
  EXPECT_DOUBLE_EQ(detail::W(0, 2), 0.5);
  EXPECT_DOUBLE_EQ(detail::W(1, 2), 0.5);
  EXPECT_DOUBLE_EQ(detail::W(0, 3), 2.0 / 6);
  EXPECT_DOUBLE_EQ(detail::W(1, 3), 1.0 / 6);
  EXPECT_DOUBLE_EQ(detail::W(2, 3), 2.0 / 6);
  EXPECT_DOUBLE_EQ(detail::W(0, 4), 6.0 / 24);
  EXPECT_DOUBLE_EQ(detail::W(1, 4), 2.0 / 24);
  EXPECT_DOUBLE_EQ(detail::W(2, 4), 2.0 / 24);
  EXPECT_DOUBLE_EQ(detail::W(3, 4), 6.0 / 24);
}

TEST(GPUTreeShap, InterventionalBasic) {
  const float inf = std::numeric_limits<float>::infinity();
  std::vector<PathElement<XgboostSplitCondition>> path{
      {0, -1, 0, {-inf, inf, false}, 1.0f, 8.0f},
      {0, 0, 0, {5.0f, inf, false}, 0.0f, 8.0f},
      {0, 1, 0, {5.0f, inf, false}, 0.0f, 8.0f},
      {0, 0, 0, {5.0f, inf, false}, 0.0f, 8.0f},
      {1, -1, 0, {-inf, inf, false}, 1.0f, 6.0f},
      {1, 0, 0, {5.0f, inf, false}, 0.0f, 6.0f},
      {1, 1, 0, {-inf, 5.0f, false}, 0.0f, 6.0f},
      {1, 2, 0, {-5.0f, inf, false}, 0.0f, 6.0f},
      {2, -1, 0, {-inf, inf, false}, 1.0f, 5.0f},
      {2, 0, 0, {5.0f, inf, false}, 0.0f, 5.0f},
      {2, 1, 0, {-inf, 5.0f, false}, 0.0f, 5.0f},
      {2, 2, 0, {-inf, -5.0f, false}, 0.0f, 5.0f},
  };
  thrust::device_vector<float> X_data =
      std::vector<float>({10.0f, 0.0f, 10.0f});
  thrust::device_vector<float> R_data =
      std::vector<float>({10.0f, 10.0f, -10.0f, 10.0f, 10.0f, 10.0f});
  DenseDatasetWrapper X(X_data.data().get(), 1, 3);
  DenseDatasetWrapper R(R_data.data().get(), 2, 3);
  thrust::device_vector<float> phis(X.NumRows() * (X.NumCols() + 1));
  GPUTreeShapInterventional(X, R, path.begin(), path.end(), 1,
                            phis.begin(), phis.end());

  std::vector<float> result(phis.begin(), phis.end());
  ASSERT_FLOAT_EQ(result[0], 0.0f);
  ASSERT_FLOAT_EQ(result[1], -2.25f);
  ASSERT_FLOAT_EQ(result[2], 0.25f);
  ASSERT_FLOAT_EQ(result[3], 8.0f);
}
