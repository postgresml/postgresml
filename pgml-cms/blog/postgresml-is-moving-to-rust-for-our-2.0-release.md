---
description: >-
  In PostgresML 2.0, we'd like to address runtime speed, memory consumption and
  the overall reliability we've seen for machine learning deployments running at
  scale.
---

# PostgresML is Moving to Rust for our 2.0 Release

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

September 19, 2022

PostgresML is a fairly young project. We recently released v1.0 and now we're considering what we want to accomplish for v2.0. In addition to simplifying the workflow for building models, we'd like to address runtime speed, memory consumption and the overall reliability we've seen is needed for machine learning deployments running at scale.

Python is generally touted as fast enough for machine learning, and is the de facto industry standard with tons of popular libraries, implementing all the latest and greatest algorithms. Many of these algorithms (Torch, Tensorflow, XGboost, NumPy) have been optimized in C, but not all of them. For example, most of the [linear algorithms](https://github.com/scikit-learn/scikit-learn/tree/main/sklearn/linear\_model) in scikit-learn are written in pure Python, although they do use NumPy, which is a convenient optimization. It also uses Cython in a few performance critical places. This ecosystem has allowed PostgresML to offer a ton of functionality with minimal duplication of effort.

## Ambition Starts With a Simple Benchmark

<figure><img src=".gitbook/assets/image (46).png" alt=""><figcaption><p>Rust mascot image by opensource.com</p></figcaption></figure>

To illustrate our motivation, we'll create a test set of 10,000 random embeddings with 128 dimensions, and store them in a table. Our first benchmark will simulate semantic ranking, by computing the dot product against every member of the test set, sorting the results and returning the top match.

```postgresql
-- Generate 10,000 embeddings with 128 dimensions as FLOAT4[] type.
CREATE TABLE embeddings AS
SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
FROM generate_series(1, 1280000) i
GROUP BY i % 10000;
```

Spoiler alert: idiomatic Rust is about 10x faster than native SQL, embedded PL/pgSQL, and pure Python. Rust comes close to the hand-optimized assembly version of the Basic Linear Algebra Subroutines (BLAS) implementation. NumPy is supposed to provide optimizations in cases like this, but it's actually the worst performer. Data movement from Postgres to PL/Python is pretty good; it's even faster than the pure SQL equivalent, but adding the extra conversion from Python list to Numpy array takes almost as much time as everything else. Machine Learning systems that move relatively large quantities of data around can become dominated by these extraneous operations, rather than the ML algorithms that actually generate value.

{% tabs %}
{% tab title="SQL" %}
```postgresql
CREATE OR REPLACE FUNCTION dot_product_sql(a FLOAT4[], b FLOAT4[])
	RETURNS FLOAT4
	LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE AS
$$
	SELECT SUM(multiplied.values)
	FROM (SELECT UNNEST(a) * UNNEST(b) AS values) AS multiplied;
$$;
```

```postgresql
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT dot_product_sql(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}

{% tab title="PL/pgSQL" %}
```postgresql
CREATE OR REPLACE FUNCTION dot_product_plpgsql(a FLOAT4[], b FLOAT4[])
	RETURNS FLOAT4
	LANGUAGE plpgsql IMMUTABLE STRICT PARALLEL SAFE AS
$$
	BEGIN
		RETURN SUM(multiplied.values)
		FROM (SELECT UNNEST(a) * UNNEST(b) AS values) AS multiplied;
	END
$$;
```

```postgresql
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT dot_product_plpgsql(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}

{% tab title="Python" %}
```postgresql
CREATE OR REPLACE FUNCTION dot_product_python(a FLOAT4[], b FLOAT4[])
	RETURNS FLOAT4
	LANGUAGE plpython3u IMMUTABLE STRICT PARALLEL SAFE AS
$$
	return sum([a * b for a, b in zip(a, b)])
$$;
```

```postgresql
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT dot_product_python(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}

{% tab title="NumPy" %}
```postgresql
CREATE OR REPLACE FUNCTION dot_product_numpy(a FLOAT4[], b FLOAT4[])
	RETURNS FLOAT4
	LANGUAGE plpython3u IMMUTABLE STRICT PARALLEL SAFE AS
$$
	import numpy
	return numpy.dot(a, b)
$$;
```

```postgresql
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT dot_product_numpy(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}

{% tab title="Rust" %}
```rust
#[pg_extern(immutable, strict, parallel_safe)]
fn dot_product_rust(vector: Vec<f32>, other: Vec<f32>) -> f32 {
	vector
		.as_slice()
		.iter()
		.zip(other.as_slice().iter())
		.map(|(a, b)| (a * b))
		.sum()
}
```

```postgresql
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT pgml.dot_product_rust(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}

{% tab title="BLAS" %}

```rust
#[pg_extern(immutable, strict, parallel_safe)]
fn dot_product_blas(vector: Vec<f32>, other: Vec<f32>) -> f32 {
	unsafe {
		blas::sdot(
			vector.len().try_into().unwrap(),
			vector.as_slice(),
			1,
			other.as_slice(),
			1,
		)
	}
}

```

```
WITH test AS (
	SELECT ARRAY_AGG(random())::FLOAT4[] AS vector
	FROM generate_series(1, 128) i
)
SELECT pgml.dot_product_blas(embeddings.vector, test.vector) AS dot_product
FROM embeddings, test
ORDER BY 1
LIMIT 1;
```
{% endtab %}
{% endtabs %}

We're building with the Rust [pgrx](https://github.com/tcdi/pgrx/tree/master/pgrx) crate that makes our development cycle even nicer than the one we use to manage Python. It really streamlines creating an extension in Rust, so all we have to worry about is writing our functions. It took about an hour to port all of our vector operations to Rust with BLAS support, and another week to port all the "business logic" for maintaining model training and deployment. We've even gained some new capabilities for caching models across connections (independent processes), now that we have access to Postgres shared memory, without having to worry about Python's GIL and GC. This is the dream of Apache's Arrow project, realized for our applications, without having to change the world, just our implementations. 🤩 Single-copy end-to-end machine learning, with parallel processing and shared data access.

## What about XGBoost and friends?

ML isn't just about basic math and a little bit of business logic. It's about all those complicated algorithms beyond linear regression for gradient boosting and deep learning. The good news is that most of these libraries are implemented in C/C++, and just have Python bindings. There are also bindings for Rust ([lightgbm](https://github.com/vaaaaanquish/lightgbm-rs), [xgboost](https://github.com/davechallis/rust-xgboost), [tensorflow](https://github.com/tensorflow/rust), [torch](https://github.com/LaurentMazare/tch-rs)).

<figure><img src=".gitbook/assets/image (47).png" alt=""><figcaption><p>Layers of abstraction must remain a good value</p></figcaption></figure>

The results are somewhat staggering. We didn't spend any time intentionally optimizing Rust over Python. Most of the time spent was just trying to get things to compile. 😅 It's hard to believe the difference is this big, but those fringe operations outside of the core machine learning algorithms really do dominate, requiring up to 35x more time in Python during inference. The difference between classification and regression speeds here are related to the dataset size. The scikit learn handwritten image classification dataset effectively has 64 features (pixels) vs the diabetes regression dataset having only 10 features.

**The more data we're dealing with, the bigger the improvement we see in Rust**. We're even giving Python some leeway by warming up the runtime on the connection before the test, which typically takes a second or two to interpret all of PostgresML's dependencies. Since Rust is a compiled language, there is no longer a need to warmup the connection.

<figure><img src=".gitbook/assets/image (48).png" alt=""><figcaption><p>This language comparison uses in-process data access. Python based machine learning microservices that communicate with other services over HTTP with JSON or gRPC interfaces will look even worse in comparison, especially if they are stateless and rely on yet another database to provide their data over yet another wire.</p></figcaption></figure>

## Preserving Backward Compatibility

```postgresql
SELECT pgml.train(
  project_name => 'Handwritten Digit Classifier',
  task => 'classification',
  relation_name => 'pgml.digits',
  y_column_name => 'target',
  algorithm => 'xgboost'
);
```

```postgresql
SELECT pgml.predict('Handwritten Digit Classifier', image)
FROM pgml.digits;
```

The API is identical between v1.0 and v2.0. We take breaking changes seriously and we're not going to break existing deployments just because we're rewriting the whole project. The only reason we're bumping the major version is because we feel like this is a dramatic change, but we intend to preserve a full compatibility layer with models trained on v1.0 in Python. However, this does mean that to get the full performance benefits, you'll need to retrain models after upgrading.

## Ensuring High Quality Rust Implementations

Besides backwards compatibility, we're building a Python compatibility layer to guarantee we can preserve the full Python model training APIs, when Rust APIs are not at parity in terms of functionality, quality or performance. We started this journey thinking that the older vanilla Python algorithms in Scikit would be the best candidates for replacement in Rust, but that is only partly true. There are high quality efforts in [linfa](https://github.com/rust-ml/linfa) and [smartcore](https://github.com/smartcorelib/smartcore) that also show 10-30x speedup over Scikit, but they still lack some of the deeper functionality like joint regression, some of the more obscure algorithms and hyperparameters, and some of the error handling that has been hardened into Scikit with mass adoption.

<figure><img src=".gitbook/assets/image (49).png" alt=""><figcaption></figcaption></figure>

We see similar speed up in prediction time for the Rust implementations of classic algorithms.

<figure><img src=".gitbook/assets/image (50).png" alt=""><figcaption></figcaption></figure>

The Rust implementations also produce high quality predictions against test sets, although there is not perfect parity in the implementations where different optimizations have been chosen by default.

<figure><img src=".gitbook/assets/image (51).png" alt=""><figcaption></figcaption></figure>

Interestingly, the training times for some of the simplest algorithms are worse in the Rust implementation. Until we can guarantee each Rust algorithm is an upgrade in every way, we'll continue to use the Python compatibility layer on a case by case basis to avoid any unpleasant surprises.

We believe that [machine learning in Rust](https://www.arewelearningyet.com/) is mature enough to add significant value now. We'll be using the same underlying C/C++ libraries, and it's worth contributing to the Rust ML ecosystem to bring it up to full feature parity. Our v2.0 release will include a benchmark suite for the full API we support via all Python libraries, so that we can track our progress toward pure Rust implementations over time.

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. You can show your support by [starring us on our GitHub](https://github.com/postgresml/postgresml).

