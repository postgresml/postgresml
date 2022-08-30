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
#pragma once
#include <GPUTreeShap/gpu_treeshap.h>
#include <limits>
#include <numeric>
#include <random>
#include <vector>

namespace gpu_treeshap {

class DenseDatasetWrapper {
  const float* data;
  int num_rows;
  int num_cols;

 public:
  DenseDatasetWrapper() = default;
  DenseDatasetWrapper(const float* data, int num_rows, int num_cols)
      : data(data), num_rows(num_rows), num_cols(num_cols) {}
  __device__ float GetElement(size_t row_idx, size_t col_idx) const {
    assert(col_idx >= 0);
    return data[row_idx * num_cols + col_idx];
  }
  __host__ __device__ size_t NumRows() const { return num_rows; }
  __host__ __device__ size_t NumCols() const { return num_cols; }
};

class TestDataset {
 public:
  std::vector<float> host_data;
  thrust::device_vector<float> device_data;
  size_t num_rows;
  size_t num_cols;
  TestDataset() = default;
  TestDataset(size_t num_rows, size_t num_cols, size_t seed,
              float missing_fraction = 0.25)
      : num_rows(num_rows), num_cols(num_cols) {
    std::mt19937 gen(seed);
    std::uniform_real_distribution<float> dis;
    std::bernoulli_distribution bern(missing_fraction);
    host_data.resize(num_rows * num_cols);
    for (auto& e : host_data) {
      e = bern(gen) ? std::numeric_limits<float>::quiet_NaN() : dis(gen);
    }
    device_data = host_data;
  }
  DenseDatasetWrapper GetDeviceWrapper() {
    return DenseDatasetWrapper(device_data.data().get(), num_rows, num_cols);
  }
};

template <typename SplitConditionT>
void GenerateModel(std::vector<PathElement<SplitConditionT>>* model, int group,
                   size_t max_depth, size_t num_features, size_t num_paths,
                   std::mt19937* gen, float max_v) {
  std::uniform_real_distribution<float> value_dis(-max_v, max_v);
  std::uniform_int_distribution<int64_t> feature_dis(0, num_features - 1);
  std::bernoulli_distribution bern_dis;
  const float inf = std::numeric_limits<float>::infinity();
  size_t base_path_idx = model->empty() ? 0 : model->back().path_idx + 1;
  float z = std::pow(0.5, 1.0 / max_depth);
  for (auto i = 0ull; i < num_paths; i++) {
    float v = value_dis(*gen);
    model->emplace_back(PathElement<SplitConditionT>{
        base_path_idx + i, -1, group, {-inf, inf, false}, 1.0, v});
    for (auto j = 0ull; j < max_depth; j++) {
      float lower_bound = -inf;
      float upper_bound = inf;
      // If the input feature value x_i is a uniform rv in [0,1)
      // We want a 50% chance of it reaching the end of this path
      // Each test should succeed with probability 0.5^(1/max_depth)
      std::uniform_real_distribution<float> bound_dis(0.0, 2.0 - 2 * z);
      if (bern_dis(*gen)) {
        lower_bound = bound_dis(*gen);
      } else {
        upper_bound = 1.0f - bound_dis(*gen);
      }
      // Don't make the zero fraction too small
      std::uniform_real_distribution<float> zero_fraction_dis(0.05, 1.0);
      model->emplace_back(PathElement<SplitConditionT>{
          base_path_idx + i,
          feature_dis(*gen),
          group,
          {lower_bound, upper_bound, bern_dis(*gen)},
          zero_fraction_dis(*gen),
          v});
    }
  }
}

std::vector<PathElement<gpu_treeshap::XgboostSplitCondition>>
GenerateEnsembleModel(size_t num_groups, size_t max_depth, size_t num_features,
                      size_t num_paths, size_t seed, float max_v = 1.0f) {
  std::mt19937 gen(seed);
  std::vector<PathElement<gpu_treeshap::XgboostSplitCondition>> model;
  for (auto group = 0llu; group < num_groups; group++) {
    GenerateModel(&model, group, max_depth, num_features, num_paths, &gen,
                  max_v);
  }
  return model;
}

std::vector<float> Predict(
    const std::vector<PathElement<gpu_treeshap::XgboostSplitCondition>>& model,
    const TestDataset& X, size_t num_groups) {
  std::vector<float> predictions(X.num_rows * num_groups);
  for (auto i = 0ull; i < X.num_rows; i++) {
    const float* row = X.host_data.data() + i * X.num_cols;
    float current_v = model.front().v;
    size_t current_path_idx = model.front().path_idx;
    int current_group = model.front().group;
    bool valid = true;
    for (const auto& e : model) {
      if (e.path_idx != current_path_idx) {
        if (valid) {
          predictions[i * num_groups + current_group] += current_v;
        }
        current_v = e.v;
        current_path_idx = e.path_idx;
        current_group = e.group;
        valid = true;
      }

      if (e.feature_idx != -1) {
        float fval = row[e.feature_idx];
        if (std::isnan(fval)) {
          valid = valid && e.split_condition.is_missing_branch;
        } else if (fval < e.split_condition.feature_lower_bound ||
                   fval >= e.split_condition.feature_upper_bound) {
          valid = false;
        }
      }
    }
    if (valid) {
      predictions[i * num_groups + current_group] += current_v;
    }
  }

  return predictions;
}
}  // namespace gpu_treeshap
