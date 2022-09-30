---
author: Lev Kokotov
description: Keeping backwards compatibility is one of the most important tenants of any system.
---


# Backwrards Compatible or Bust

<p class="author">
  <img width="54px" height="54px" src="/images/team/lev.jpg" alt="Author" />
  Lev Kokotov<br/>
  October 3, 2022
</p>


Some of you may remember the day Python 3 was released. The changes were pretty sublte, but they were enough to create shellshock: most projects and tools out there written in Python 2 won't work in Python 3. The next decade was spent migrating mission-critical infrastructure from `print` to `print()` and from `str` to `bytes`. Some just gave up and stayed on Python 2. Breaking backwards compatibility to make progress or correct mistakes of the past could be good but Python's move was risky; it endured mostly because we loved it more than we hated that change.

Most projects don't have that luxury, especially if you're just starting out. For us at PostgresML, backwards compatibility is as important as progress. PostgresML v2.0 is coming out in a few weeks and we're rewritten everything in Rust for a [35x performance increase](http://localhost:8001/blog/postgresml-is-moving-to-rust-for-our-2.0-release/). Version 1.0 was written in Python because Scikit is, and we couldn't do ML without it. So what were we to do to preserve backwards compatibility and our early adopters trust?

PyO3 to the rescue.

## Py in the Sky

PyO3 was originally written to write Python extensions in Rust. Native extensions are much faster than Python modules so when speed matters, most things were written in Cython or C. If you've ever tried that, you know that the experience isn't very user-friendly or forgiving. Rust on the other hand is fast and safe, with compiler hints getting awefully specific (my co-founder thinks it may be a singularity).

PyO3 also comes with a very important feature: it allows to execute Python code from inside a Rust program.

Sounds too good to be true? Not really, since PL/Python has been doing that for years inside Postgres, that's how we wrote PostgresML v1.0 after all. The path to running Scikit inside Rust was clear.


## The Recipe

#### Step 1: Bake

First thing we needed to write was a good Python wrapper around Scikit. The whole thing ended up with just 200 lines of code and most of it was just a map of algorithm names to Scikit classes. The remainer were functions that built the class instances according to hyperparameters and number of features we received from `pgml.train` and `pgml.predict`. The module is then read and parsed once on startup, so we didn't have to recompile it every time we answered a query.


#### Step 2: Consume

Now that Rust models and Python models lived side by side, we needed to make sure that Python models were deserialzied with Pickle and not Serde. So all existing models were automatically assigned a new flag: `runtime = 'python'` and every time we queried predictions with `pgml.predict`, we fetched the pickled model from the table, passed it to our Python wrapper for deserialization, and saved the resulting reference in a `PyObject` Rust reference in process memory. This way, we only deserialize once.

#### Step 3: Consume
