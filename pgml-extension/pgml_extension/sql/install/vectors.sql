---
--- Elementwise vector addition with a constant
---
CREATE OR REPLACE FUNCTION pgml.add(a REAL[], b REAL) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(added.values)
		FROM (SELECT UNNEST(a) + b AS values) added;
	END
$$;


---
--- Elementwise vector subtraction with a constant
---
CREATE OR REPLACE FUNCTION pgml.subtract(minuend REAL[], subtrahend REAL) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(subtracted.values)
		FROM (SELECT UNNEST(minuend) - subtrahend AS values) subtracted;
	END
$$;


---
--- Elementwise vector multiplication with a constant
---
CREATE OR REPLACE FUNCTION pgml.multiply(multiplicand REAL[], multiplier REAL) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(multiplied.values)
		FROM (SELECT UNNEST(multiplicand) * multiplier AS values) multiplied;
	END
$$;


---
--- Elementwise vector division with a constant
---
CREATE OR REPLACE FUNCTION pgml.divide(dividend REAL[], divisor REAL) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(divided.values)
		FROM (SELECT UNNEST(dividend) / divisor AS values) divided;
	END
$$;


---
--- Pairwise vector addition 
---
CREATE OR REPLACE FUNCTION pgml.add(a REAL[], b REAL[]) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(added.values)
		FROM (SELECT UNNEST(a) + UNNEST(b) AS values) added;
	END
$$;


---
--- Pairwise vector subtraction
---
CREATE OR REPLACE FUNCTION pgml.subtract(minuend REAL[], subtrahend REAL[]) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(subtracted.values)
		FROM (SELECT UNNEST(minuend) - UNNEST(subtrahend) AS values) subtracted;
	END
$$;


---
--- Pairwise vector multiplication
---
CREATE OR REPLACE FUNCTION pgml.multiply(multiplicand REAL[], multiplier REAL[]) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(multiplied.values)
		FROM (SELECT UNNEST(multiplicand) * UNNEST(multiplier) AS values) multiplied;
	END
$$;


---
--- Pairwise vector division
---
CREATE OR REPLACE FUNCTION pgml.divide(dividend REAL[], divisor REAL[]) 
	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN ARRAY_AGG(divided.values)
		FROM (SELECT UNNEST(dividend) / UNNEST(divisor) AS values) divided;
	END
$$;


---
--- The number of non zero dimensions
---
CREATE OR REPLACE FUNCTION pgml.norm_l0(vector REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SUM((vector.values != 0)::INTEGER)
		FROM (SELECT UNNEST(vector) AS values) AS vector;
	END
$$;


---
--- Manhattan distance from the origin
---
CREATE OR REPLACE FUNCTION pgml.norm_l1(vector REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SUM(ABS(vector.values))
		FROM (SELECT UNNEST(vector) AS values) AS vector;
	END
$$;


---
--- Euclidean distance from the origin
---
CREATE OR REPLACE FUNCTION pgml.norm_l2(vector REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SQRT(SUM(squared.values))
		FROM (SELECT UNNEST(vector) * UNNEST(vector) AS values) AS squared;
	END
$$;


---
--- Furthest dimension from the origin
---
CREATE OR REPLACE FUNCTION pgml.norm_max(vector REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN MAX(ABS(unnested.values))
		FROM (SELECT UNNEST(vector) AS values) as unnested;
	END
$$;


---
--- Unit vector
--- 
CREATE OR REPLACE FUNCTION pgml.normalize_l1(vector REAL[]) 
  	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN pgml.divide(vector, pgml.norm_l1(vector));
	END
$$;


---
--- Squared unit vector
---
CREATE OR REPLACE FUNCTION pgml.normalize_l2(vector REAL[]) 
  	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN pgml.divide(vector, pgml.norm_l2(vector));
	END
$$;


---
--- Normalized values from -1 to 1
---
CREATE OR REPLACE FUNCTION pgml.normalize_max(vector REAL[]) 
  	RETURNS REAL[]
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN pgml.divide(vector, MAX(ABS(unnested.values)))
		FROM (SELECT UNNEST(vector) AS values) as unnested;
	END
$$;


---
--- Manhattan distance between 2 vectors
---
CREATE OR REPLACE FUNCTION pgml.distance_l1(a REAL[], b REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SUM(ABS(differences.values))
		FROM (SELECT UNNEST(a) - UNNEST(b) AS values) AS differences;
	END
$$;


---
--- Euclidean distance between 2 vectors
---
CREATE OR REPLACE FUNCTION pgml.distance_l2(a REAL[], b REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SQRT(SUM(differences.values * differences.values))
		FROM (SELECT UNNEST(a) - UNNEST(b) AS values) AS differences;
	END
$$;

---
--- A projection of `a` onto `b`
---
CREATE OR REPLACE FUNCTION pgml.dot_product(a REAL[], b REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN SUM(multiplied.values)
		FROM (SELECT UNNEST(a) * UNNEST(b) AS values) AS multiplied;
	END
$$;


---
--- The size of the angle between `a` and `b`
---
CREATE OR REPLACE FUNCTION pgml.cosine_similarity(a REAL[], b REAL[]) 
  	RETURNS REAL
	LANGUAGE plpgsql
	LEAKPROOF
	IMMUTABLE
	STRICT
	PARALLEL SAFE
AS $$
	BEGIN
		RETURN pgml.dot_product(a, b) / (pgml.norm_l2(a) * pgml.norm_l2(b));
	END
$$;
