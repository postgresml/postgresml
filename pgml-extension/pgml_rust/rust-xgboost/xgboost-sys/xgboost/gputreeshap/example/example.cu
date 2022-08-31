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
#include <algorithm>
#include <iostream>
#include <limits>
#include <string>
#include <vector>
#include "../GPUTreeShap/gpu_treeshap.h"

// Minimal decision tree implementation that stores sample weights of training
// data at each node. The sample weight roughly corresponds to the "amount" of
// data that arrives in this node. Using this we can estimate the probability of
// an instance taking the left or right branch if its feature is unknown.
class DecisionTree {
 public:
  struct Node {
    int parent;
    int left_child;
    int right_child;
    int feature_idx;
    float split_condition;
    float leaf_value;
    float sample_weight;
    bool IsLeaf() const { return left_child == -1 && right_child == -1; }
    bool IsRoot() const { return parent == -1; }
  };
  std::vector<Node> nodes;
  explicit DecisionTree(float total_weight) {
    nodes.push_back({-1, -1, -1, -1, 0.0, 0.0, total_weight});
  }
  void AddSplit(int node_idx, int feature_idx, float split_condition,
                float left_sample_weight, float right_sample_weight,
                float left_leaf_value, float right_leaf_value) {
    nodes[node_idx].split_condition = split_condition;
    nodes[node_idx].feature_idx = feature_idx;
    int left_idx = nodes.size();
    nodes[node_idx].left_child = left_idx;
    nodes.push_back(
        {node_idx, -1, -1, -1, 0.0, left_leaf_value, left_sample_weight});
    int right_idx = nodes.size();
    nodes[node_idx].right_child = right_idx;
    nodes.push_back(
        {node_idx, -1, -1, -1, 0.0, right_leaf_value, right_sample_weight});
  }
};

void RecursivePrint(std::ostream& os, const DecisionTree& dt, int node_idx,
                    int depth) {
  if (node_idx == -1) return;
  DecisionTree::Node node = dt.nodes[node_idx];

  for (int i = 0; i < depth; i++) {
    os << "\t";
  }
  os << node_idx << ":";
  if (node.IsLeaf()) {
    os << "leaf=" << node.leaf_value;
  } else {
    os << "[f" << node.feature_idx << "<" << node.split_condition << "]";
  }
  os << ",cover=" << node.sample_weight;
  os << "\n";
  RecursivePrint(os, dt, node.left_child, depth + 1);
  RecursivePrint(os, dt, node.right_child, depth + 1);
}

std::ostream& operator<<(std::ostream& os, const DecisionTree& dt) {
  RecursivePrint(os, dt, 0, 0);
  return os;
}

// Define a custom split condition implementing EvaluateSplit and Merge
struct MySplitCondition {
  MySplitCondition() = default;
  MySplitCondition(float feature_lower_bound, float feature_upper_bound)
      : feature_lower_bound(feature_lower_bound),
        feature_upper_bound(feature_upper_bound) {
    assert(feature_lower_bound <= feature_upper_bound);
  }

  /*! Feature values >= lower and < upper flow down this path. */
  float feature_lower_bound;
  float feature_upper_bound;

  // Does this instance flow down this path?
  __host__ __device__ bool EvaluateSplit(float x) const {
    return x >= feature_lower_bound && x < feature_upper_bound;
  }

  // Combine two split conditions on the same feature
  __host__ __device__ void Merge(
      const MySplitCondition& other) {  // Combine duplicate features
    feature_lower_bound = max(feature_lower_bound, other.feature_lower_bound);
    feature_upper_bound = min(feature_upper_bound, other.feature_upper_bound);
  }
};

std::vector<gpu_treeshap::PathElement<MySplitCondition>> ExtractPaths(
    const DecisionTree& dt) {
  std::vector<gpu_treeshap::PathElement<MySplitCondition>> paths;
  size_t path_idx = 0;
  // Find leaf nodes
  // Work backwards from leaf to root, order does not matter
  // It's also possible to work from root to leaf
  for (int i = 0; i < static_cast<int>(dt.nodes.size()); i++) {
    if (dt.nodes[i].IsLeaf()) {
      auto child = dt.nodes[i];
      float v = child.leaf_value;
      int child_idx = i;
      const float inf = std::numeric_limits<float>::infinity();
      while (!child.IsRoot()) {
        auto parent = dt.nodes[child.parent];
        float zero_fraction = child.sample_weight / parent.sample_weight;
        // Encode the range of feature values that flow down this path
        bool is_left_path = parent.left_child == child_idx;
        float lower_bound = is_left_path ? -inf : parent.split_condition;
        float upper_bound = is_left_path ? parent.split_condition : inf;
        paths.push_back({path_idx,
                         parent.feature_idx,
                         0,
                         {lower_bound, upper_bound},
                         zero_fraction,
                         v});
        child_idx = child.parent;
        child = parent;
      }
      // Root node has feature -1
      paths.push_back({path_idx, -1, 0, {-inf, inf}, 1.0, v});
      path_idx++;
    }
  }
  return paths;
}

std::ostream& operator<<(
    std::ostream& os,
    const std::vector<gpu_treeshap::PathElement<MySplitCondition>>& paths) {
  std::vector<gpu_treeshap::PathElement<MySplitCondition>> tmp(paths);
  std::sort(tmp.begin(), tmp.end(),
            [&](const gpu_treeshap::PathElement<MySplitCondition>& a,
                const gpu_treeshap::PathElement<MySplitCondition>& b) {
              if (a.path_idx < b.path_idx) return true;
              if (b.path_idx < a.path_idx) return false;

              if (a.feature_idx < b.feature_idx) return true;
              if (b.feature_idx < a.feature_idx) return false;
              return false;
            });

  for (auto i = 0ull; i < tmp.size(); i++) {
    auto e = tmp[i];
    if (i == 0 || e.path_idx != tmp[i - 1].path_idx) {
      os << "path_idx:" << e.path_idx << ", leaf value:" << e.v;
      os << "\n";
    }
    os << " (feature:" << e.feature_idx << ", pz:" << e.zero_fraction << ", ["
       << e.split_condition.feature_lower_bound << "<=x<"
       << e.split_condition.feature_upper_bound << "])";
    os << "\n";
  }
  return os;
}

class DenseDatasetWrapper {
  const float* data;
  int num_rows;
  int num_cols;

 public:
  DenseDatasetWrapper() = default;
  DenseDatasetWrapper(const float* data, int num_rows, int num_cols)
      : data(data), num_rows(num_rows), num_cols(num_cols) {}
  __device__ float GetElement(size_t row_idx, size_t col_idx) const {
    return data[row_idx * num_cols + col_idx];
  }
  __host__ __device__ size_t NumRows() const { return num_rows; }
  __host__ __device__ size_t NumCols() const { return num_cols; }
};

int main() {
  // Create a very basic decision tree
  DecisionTree tree(5.0);
  tree.AddSplit(0, 0, 0.5, 2.0, 3.0, -1.0, 0.0);
  tree.AddSplit(2, 1, 0.5, 1.0, 2.0, -1.0, 0.0);

  tree.AddSplit(4, 2, 0.5, 1.0, 1.0, 1.0, 0.5);

  // Visualise it
  std::cout << "Decision tree:\n";
  std::cout << tree;

  auto paths = ExtractPaths(tree);

  // Visualise unique paths
  std::cout << "Extracted paths:\n";
  std::cout << paths;

  // Create a dataset with two rows in row major format
  thrust::device_vector<float> data(3 * 2);
  // First row
  data[0] = 1.0;
  data[1] = 1.0;
  data[2] = 0.0;
  // Second row
  data[3] = 1.0;
  data[4] = 1.0;
  data[5] = 1.0;
  DenseDatasetWrapper X(data.data().get(), 2, 3);
  thrust::device_vector<float> phis((X.NumCols() + 1) * X.NumRows());
  gpu_treeshap::GPUTreeShap(X, paths.begin(), paths.end(), 1, phis.begin(),
                            phis.end());

  // Print the resulting feature contributions
  std::cout << "\n";
  for (auto i = 0ull; i < X.NumRows(); i++) {
    std::cout << "Row " << i << " contributions:\n";
    for (auto j = 0ull; j < X.NumCols(); j++) {
      std::cout << "f" << j << ":" << phis[i * (X.NumCols() + 1) + j] << " ";
    }
    std::cout << "bias"
              << ":" << phis[i * (X.NumCols() + 1) + X.NumCols()];
    std::cout << "\n";
  }
}
