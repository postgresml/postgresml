# Vector Operations

PostgresML adds [native vector operations](https://github.com/postgresml/postgresml/tree/master/pgml-extension/sql/install/vectors.sql) that can be called from SQL:

```sql linenums="1"
-- Elementwise arithmetic w/ constants
pgml.add(a REAL[], b REAL) -> REAL[]
pgml.subtract(minuend REAL[], subtrahend REAL) -> REAL[]
pgml.multiply(multiplicand REAL[], multiplier REAL) -> REAL[]
pgml.divide(dividend REAL[], divisor REAL) -> REAL[]

-- Pairwise arithmetic w/ vectors
pgml.add(a REAL[], b REAL[]) -> REAL[]
pgml.subtract(minuend REAL[], subtrahend REAL[]) -> REAL[]
pgml.multiply(multiplicand REAL[], multiplier REAL[]) -> REAL[]
pgml.divide(dividend REAL[], divisor REAL[]) -> REAL[]

-- Norms
pgml.norm_l0(vector REAL[]) -> REAL -- Dimensions not at the origin
pgml.norm_l1(vector REAL[]) -> REAL -- Manhattan distance from origin
pgml.norm_l2(vector REAL[]) -> REAL -- Euclidean distance from origin
pgml.norm_max(vector REAL[]) -> REAL -- Absolute value of largest element

-- Normalization
pgml.normalize_l1(vector REAL[]) -> REAL[] -- Unit Vector
pgml.normalize_l2(vector REAL[]) -> REAL[] -- Squared Unit Vector
pgml.normalize_max(vector REAL[]) -> REAL[] -- -1:1 values

-- Distances
pgml.distance_l1(a REAL[], b REAL[]) -> REAL -- Manhattan
pgml.distance_l2(a REAL[], b REAL[]) -> REAL -- Euclidean
pgml.dot_product(a REAL[], b REAL[]) -> REAL -- Projection
pgml.cosine_similarity(a REAL[], b REAL[]) -> REAL -- Direction
```

