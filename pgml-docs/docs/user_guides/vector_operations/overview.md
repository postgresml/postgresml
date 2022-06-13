# Vector Operations

PostgresML adds [native vector operations](https://github.com/postgresml/postgresml/tree/master/pgml-extension/sql/install/vectors.sql) that can be used in SQL queries. Vector operations are particularly useful for dealing with embeddings that have been generated from other machine learning algorithms and can provide functions like nearest neighbor calculations using the distance functions.

Emeddings can be a relatively efficient mechanism to leverage the power deep learning, without the runtime inference costs. These functions are relatively fast and the more expensive distance functions can compute ~100k per second for a memory resident dataset on modern hardware.

The PostgreSQL planner will also [automatically parallelize](https://www.postgresql.org/docs/current/parallel-query.html) evalualtion on larger datasets, as configured to take advantage of multiple CPU cores when available.

## Nearest neighbor example

If we had precalculated the embeddings for a set of user and product data, we could find the 100 best products for a user with a similarity search.

```sql linenums="1"
SELECT 
    products.id, 
    pgml.cosine_similarity(users.embedding, products.embedding) AS distance
FROM users
JOIN products
WHERE users.id = 123
ORDER BY distance ASC
LIMIT 100;
```

## Elementwise arithmetic w/ constants

#### Addition
```sql linenums="1"
pgml.add(a REAL[], b REAL) -> REAL[]
```

#### Subtraction
```sql linenums="1"
pgml.subtract(minuend REAL[], subtrahend REAL) -> REAL[]
```

#### Multiplication
```sql linenums="1"
pgml.multiply(multiplicand REAL[], multiplier REAL) -> REAL[]
```

#### Division
```sql linenums="1"
pgml.divide(dividend REAL[], divisor REAL) -> REAL[]
```

## Pairwise arithmetic w/ vectors

#### Addition
```sql linenums="1"
pgml.add(a REAL[], b REAL[]) -> REAL[]
```

#### Subtraction
```sql linenums="1"
pgml.subtract(minuend REAL[], subtrahend REAL[]) -> REAL[]
```

#### Multiplication
```sql linenums="1"
pgml.multiply(multiplicand REAL[], multiplier REAL[]) -> REAL[]
```

#### Division
```sql linenums="1"
pgml.divide(dividend REAL[], divisor REAL[]) -> REAL[]
```

## Norms

#### Dimensions not at origin
```sql linenums="1"
pgml.norm_l0(vector REAL[]) -> REAL
```

#### Manhattan distance from origin
```sql linenums="1"
pgml.norm_l1(vector REAL[]) -> REAL 
```

#### Euclidean distance from origin
```sql linenums="1"
pgml.norm_l2(vector REAL[]) -> REAL 
```

#### Absolute value of largest element
```sql linenums="1"
pgml.norm_max(vector REAL[]) -> REAL 
```

## Normalization

#### Unit Vector
```sql linenums="1"
pgml.normalize_l1(vector REAL[]) -> REAL[]
```

#### Squared Unit Vector
```sql linenums="1"
pgml.normalize_l2(vector REAL[]) -> REAL[]
```

#### -1:1 values
```sql linenums="1"
pgml.normalize_max(vector REAL[]) -> REAL[]
```

## Distances

#### Manhattan
```sql linenums="1"
pgml.distance_l1(a REAL[], b REAL[]) -> REAL
```

#### Euclidean
```sql linenums="1"
pgml.distance_l2(a REAL[], b REAL[]) -> REAL
```

#### Projection
```sql linenums="1"
pgml.dot_product(a REAL[], b REAL[]) -> REAL
```

#### Direction
```sql linenums="1"
pgml.cosine_similarity(a REAL[], b REAL[]) -> REAL
```
