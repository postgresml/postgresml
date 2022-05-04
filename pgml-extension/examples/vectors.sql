\set ON_ERROR_STOP true

-- Elementwise arithmetic w/ constants
SELECT pgml.add(ARRAY[1.0, 2.0, 3.0], 3);
SELECT pgml.subtract(ARRAY[1.0, 2.0, 3.0], 3);
SELECT pgml.multiply(ARRAY[1.0, 2.0, 3.0], 3);
SELECT pgml.divide(ARRAY[1.0, 2.0, 3.0], 100);

-- Pairwise arithmetic
SELECT pgml.add(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.subtract(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.multiply(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.divide(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);

-- Norms
SELECT pgml.norm_l0(ARRAY[1.0, 2.0, 3.0]);
SELECT pgml.norm_l1(ARRAY[1.0, 2.0, 3.0]);
SELECT pgml.norm_l2(ARRAY[1.0, 2.0, 3.0]);
SELECT pgml.norm_max(ARRAY[1.0, 2.0, 3.0]);

-- Normalization
SELECT pgml.normalize_l1(ARRAY[1.0, 2.0, 3.0]);
SELECT pgml.normalize_l2(ARRAY[1.0, 2.0, 3.0]);
SELECT pgml.normalize_max(ARRAY[1.0, 2.0, 3.0]);

-- Comparisons
SELECT pgml.distance_l1(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.distance_l2(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.dot_product(ARRAY[1.0, 2.0, 3.0], ARRAY[4.0, 5.0, 6.0]);
SELECT pgml.cosine_similarity(ARRAY[1.0, 2.0, 3.0], ARRAY[1.0, 2.0, 3.0]);
