
<!--
Our performance questions are centered around Machine Learning, which introduces factors beyond pure language runtime performance. Namely, where does the data from the computation come from and how does does it cross IO and process boundaries? Data movement for algorithms can be orders of magnitude more expensive than the other latency costs for ML applications, so we'd like our benchmarks to include the full picture. Inference is generally more latency sensitive than training, but training optimizations can change R&D iteration loops from days to minutes, or enable new data scales, so they can still be game changing.

In this post, we'll explore some potential architectures for Machine Learning, including a feature store, model store, and the performance characteristics of the inference layer. You can also compare the code snippets for "readability" and "maintainability" which is another important consideration.







 ![Machine Learning Infrastructure](/blog/benchmarks/Machine-Learning-Infrastructure-2.webp)

Warming up w/ some data generation
----------------------------

To get started, we'll generate some test data for benchmarking, and document the process for anyone that wants to reproduce the results on their own hardware. We'll create a test set of 10,000 random embeddings with 128 dimensions, and print them out to `/dev/null`. This test only involves serialization to stdout, not persistence, so we can get an initial idea of language runtime.

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vShmCVrYwmscys5TIo7c_C-1M3gE_GwENc4tTiU7A6_l3YamjJx7v5bZcafLIDcEIbFu-C2Buao4rQ6/pubchart?oid=278281764&amp;format=interactive"></iframe>
</center>

=== "SQL"
	`time psql -f embedding.sql > /dev/null`

	```sql linenums="1" title="embedding.sql"
	SELECT ARRAY_AGG(random()) AS vector
	FROM generate_series(1, 1280000) i
	GROUP BY i % 10000;
	```
=== "Python"
	`time python3 embedding.py > /dev/null`

	```python linenums="1" title="embedding.sql"
	import random
	embeddings = [
		[
			random.random() for _ in range(128)
		] for _ in range (10_000)
	]
	print(embeddings)
	```
=== "Numpy"
	`time python3 embedding_numpy.py > /dev/null`

	```python linenums="1" title="embedding_numpy.py" 
	import sys
	import numpy
	numpy.set_printoptions(threshold=sys.maxsize)

	embeddings = numpy.random.rand(10_000, 128)
	print(embeddings)
	```
=== "Rust"
	`time cargo run --release > /dev/null`

	```rust linenums="1" title="lib.rs" 
	fn main() {
		let mut embeddings = [[0_f32; 128]; 10_000];
		for i in 0..10_000 {
			for j in 0..128 {
				embeddings[i][j] = rand::random()
			}
		};
		println!("{:?}", embeddings);
	}
	```

#### _Well, that's unexpected_

Numpy is relatively slow, even though everyone says it's "fast". It's important to actually measure our workloads to find these order of magnitude aberations that differ from expectations. The reason numpy is so slow, is it spends a lot of time formatting output into neat rows and columns. These additional string manipulations eat up time. We can prove that by skipping the serialization time, and only look at the raw generation times.

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vShmCVrYwmscys5TIo7c_C-1M3gE_GwENc4tTiU7A6_l3YamjJx7v5bZcafLIDcEIbFu-C2Buao4rQ6/pubchart?oid=916756801&amp;format=interactive"></iframe>
</center>
=== "SQL"
	`time psql -f embedding.sql > /dev/null`

	```sql linenums="1" title="embedding.sql"
	SELECT NULL FROM (
		SELECT ARRAY_AGG(random()) AS vector
		FROM generate_series(1, 1280000) i
		GROUP BY i % 10000
	) temp;
	```
=== "Python"
	`time python3 embedding.py > /dev/null`

	```python linenums="1" title="embedding.sql"
	import random
	embeddings = [
		[
			random.random() for _ in range(128)
		] for _ in range (10_000)
	]
	```
=== "Numpy"
	`time python3 embedding_numpy.py > /dev/null`

	```python linenums="1" title="embedding_numpy.py" 
	import numpy
	embeddings = numpy.random.rand(10_000, 128)
	```
=== "Rust"
	`time cargo run --release > /dev/null`

	```rust linenums="1" title="lib.rs" 
	fn main() {
		let mut embeddings = [[0 as f32; 128]; 10_000];
		for i in 0..10_000 {
			for j in 0..128 {
				embeddings[i][j] = rand::random()
			}
		};
	}
	```

Numpy is the easiest implementation. It's a single function call that does exactly what we want, except when it doesn't. Then we have to search the docs or code or internet to figure out what's going on. Rust and Python are pretty close in terms of readability and maintainability for me, although Rust has extra type annotations. The SQL implementation is concise, but will probably be the most difficult to maintain. The GROUP BY modulo factor is not how I'd first thought to implement this, and it leaves a coupling between the two dimensions in the array. Most programmers are less used to thinking in a declarative language for this type of work.

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vShmCVrYwmscys5TIo7c_C-1M3gE_GwENc4tTiU7A6_l3YamjJx7v5bZcafLIDcEIbFu-C2Buao4rQ6/pubchart?oid=2007994359&amp;format=interactive"></iframe>
</center>

A look at memory usage for our simple benchmark reveals another unexpected bit. PSQL is actually not executing the program and consuming memory, the Postgres server process is, and PSQL is just a client. This means it's also had the burden of establishing connections and passing data across process boundaries that none of our other programs had, which we haven't accounted for. Rust is the only implementation that actually did what we set out to do in this very trivial exercise. We managed to introduce unnecessary and complexity in the other implementations. Benchmarks (like most software engineering) can be tricky business.

Overall, Rust shows enough promise in this microbenchmark at over twice the speed and memory efficiency of Numpy that it warrants digging deeper to see how far we can take things. -->
