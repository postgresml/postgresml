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
#include <benchmark/benchmark.h>
#include "../tests/test_utils.h"

using namespace gpu_treeshap;  // NOLINT

class Fixture : public benchmark::Fixture {
 public:
  void SetUp(const ::benchmark::State& state) override {
    num_groups = 5;
    num_rows = state.range(0);
    num_features = state.range(1);
    size_t max_depth = state.range(2);
    size_t num_paths = state.range(3);
    model = GenerateEnsembleModel(num_groups, max_depth, num_features,
                                  num_paths, 79);
    test_data.reset(new TestDataset(num_rows, num_features, 23));

    X = test_data->GetDeviceWrapper();

    phis.reset(new thrust::device_vector<float>(
        X.NumRows() * (X.NumCols() + 1) * num_groups));
  }
  void TearDown(const ::benchmark::State& state) {
    phis.reset();
    test_data.reset();
  }
  std::vector<PathElement<XgboostSplitCondition>> model;
  std::unique_ptr<TestDataset> test_data;
  DenseDatasetWrapper X;
  std::unique_ptr<thrust::device_vector<float>> phis;
  size_t num_groups;
  size_t num_rows;
  size_t num_features;
};

BENCHMARK_DEFINE_F(Fixture, GPUTreeShap)(benchmark::State& st) { // NOLINT
  for (auto _ : st) {
    GPUTreeShap(X, model.begin(), model.end(), num_groups, phis->begin(),
                phis->end());
  }
}
BENCHMARK_REGISTER_F(Fixture, GPUTreeShap)
    ->ArgNames({"n_rows", "n_feats", "max_depth", "n_leaves"})
    ->Args({1000, 10, 6, 1000})
    ->Args({10000, 50, 10, 1000})
    ->Args({100000, 500, 20, 10000});

BENCHMARK_DEFINE_F(Fixture, GPUTreeShapInterventional)
(benchmark::State& st) {  // NOLINT
  TestDataset R_test_data(1000, num_features, 1429);
  DenseDatasetWrapper R = R_test_data.GetDeviceWrapper();
  for (auto _ : st) {
    GPUTreeShapInterventional(X, R, model.begin(), model.end(), num_groups,
                              phis->begin(), phis->end());
  }
}
BENCHMARK_REGISTER_F(Fixture, GPUTreeShapInterventional)
    ->ArgNames({"n_rows", "n_feats", "max_depth", "n_leaves"})
    ->Args({1000, 10, 6, 1000})
    ->Args({10000, 50, 10, 1000});

BENCHMARK_DEFINE_F(Fixture, GPUTreeShapInteractions)(benchmark::State& st) {// NOLINT
  phis.reset(new thrust::device_vector<float>(X.NumRows() * (X.NumCols() + 1) *
                                              (X.NumCols() + 1) * num_groups));
  for (auto _ : st) {
    GPUTreeShapInteractions(X, model.begin(), model.end(), num_groups,
                            phis->begin(), phis->end());
  }
}

BENCHMARK_REGISTER_F(Fixture, GPUTreeShapInteractions)
    ->ArgNames({"n_rows", "n_feats", "max_depth", "n_leaves"})
    ->Args({1000, 10, 6, 1000})
    ->Args({1000, 50, 10, 1000})
    ->Args({1000, 250, 20, 10000});

BENCHMARK_DEFINE_F(Fixture, GPUTreeShapTaylorInteractions)
(benchmark::State& st) {// NOLINT
  phis.reset(new thrust::device_vector<float>(X.NumRows() * (X.NumCols() + 1) *
                                              (X.NumCols() + 1) * num_groups));
  for (auto _ : st) {
    GPUTreeShapTaylorInteractions(X, model.begin(), model.end(), num_groups,
                                  phis->begin(), phis->end());
  }
}

BENCHMARK_REGISTER_F(Fixture, GPUTreeShapTaylorInteractions)
    ->ArgNames({"n_rows", "n_feats", "max_depth", "n_leaves"})
    ->Args({1000, 10, 6, 1000})
    ->Args({1000, 50, 10, 1000})
    ->Args({1000, 250, 20, 10000});

std::vector<int> GenerateCounts(size_t n, size_t max_depth) {
  std::mt19937 gen(95);
  std::uniform_int_distribution<int> distrib(0, max_depth - 1);
  std::vector<int> out(n);
  for (auto& x : out) {
    x = distrib(gen);
  }
  return out;
}

static void BFDBinPacking(benchmark::State& state) {// NOLINT
  size_t n = state.range(0);
  size_t max_depth = state.range(1);
  thrust::device_vector<int> counts = GenerateCounts(n, max_depth);
  for (auto _ : state) {
    auto bin_packing = detail::BFDBinPacking(counts, max_depth);
  }
}
BENCHMARK(BFDBinPacking)
    ->ArgNames({"n", "max_depth"})
    ->Args({1000, 16})
    ->Args({100000, 32});

BENCHMARK_MAIN();
