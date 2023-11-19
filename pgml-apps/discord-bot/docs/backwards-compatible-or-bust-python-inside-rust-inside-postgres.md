---
author: Lev Kokotov
description: A story about including Scikit-learn into our Rust extension and preserving backwards compatibility in the process
---

# Backwards Compatible or Bust: Python Inside Rust Inside Postgres

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/lev.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
  	<p class="m-0">Lev Kokotov</p>
  	<p class="m-0">October 3, 2022</p>
  </div>
</div>



Some of you may remember the day Python 3 was released. The changes seemed sublte, but they were enough to create chaos: most projects and tools out there written in Python 2 would no longer work under Python 3. The next decade was spent migrating mission-critical infrastructure from `print` to `print()` and from `str` to `bytes`. Some just gave up and stayed on Python 2. Breaking backwards compatibility to make progress could be good but Python's move was risky. It endured because we loved it more than we disagreed with that change.

Most projects won't have that luxury, especially if you're just starting out. For us at PostgresML, backwards compatibility is as important as progress.

PostgresML 2.0 is coming out soon and we're rewritten everything in Rust for a [35x performance improvement](/blog/postgresml-is-moving-to-rust-for-our-2.0-release/). The previous version was written in Python, the de facto machine learning environment with the most libraries. Now that we were using Linfa and SmartCore, we could have theoretically went ahead without Python, but we weren't quite ready to let go of all the functionality provided by the Python ecosystem, and I'm sure many of our users weren't either. So what could we do to preserve features, backwards compatibility, and our users' trust?

PyO3 to the rescue.

## Python in Rust

PyO3 was written to build Python extensions in Rust. Native extensions are much faster than Python modules so, when speed matters, most things were written in Cython or C. If you've ever tried that, you know the experience isn't very user-friendly or forgiving. Rust, on the other hand, is fast and memory-safe, with compiler hints getting awfully specific (my co-founder thinks it may be becoming a singularity).

PyO3 comes with another very important feature: it allows running Python code from inside a Rust program.

Sounds too good to be true? We didn't think so at the time. PL/Python has been doing that for years and that's what we used initially to write PostgresML. The path to running Scikit inside Rust seemed clear.


## The Roadmap

Making a massive Python library work under a completely different environment isn't an obvious thing to do. If you dive into Scikit's source code, you would find Python, Cython, C extensions and SciPy. We were going to add that into a shared library which linked into Postgres and implemented its own machine learning algorithms.

In order to get this done, we split the work into two distinct steps:

1. Train a model in Rust using Scikit
2. Test for regressions using our 1.0 test suite

### Hello Python, I am Rust

First thing we needed to do was to make sure Scikit can even run under PyO3. So we wrote a small wrapper around all the algorithms we implemented in 1.0 and called it from inside our Rust source code. The wrapper was just 200 lines of code most of which was mapping algorithm names to Scikit's Python classes.

Using the wrapper was surprisingly easy:

```rust
use pyo3::prelude::*;
use pyo3::types::PyTuple;

pub fn sklearn_train() {
	// Copy the code into the Rust library at build time.
	let module = include_str!(concat!(
	    env!("CARGO_MANIFEST_DIR"),
	    "/src/bindings/sklearn.py"
	));

	let estimator = Python::with_gil(|py| -> Py<PyAny> {
		// Compile Python
		let module = PyModule::from_code(py, module, "", "").unwrap();

        // ... train the model
	});
}
```

Our Python code was compiled and ready to go. We trained a model with data coming from Rust arrays, passed into Python using PyO3 automatic conversions, and got back a trained Scikit model. It felt magical.

### Did it Work?

Since we have dozens of ML algorithms in 1.0, we had a pretty decent test suite to make sure all of them worked. My local dev is an Ubuntu 22.04 gaming rig (I still dual-boot though), so I had no issues running the test suite, training all Scikit algorithms on the toy datasets, and getting predictions back in a good amount of time. Drunk on my success, I called the job done, merged the PR, and moved on.

Then Montana decided to try my work on his slightly older gaming rig, but instead of getting a trained model, he got this:

```
server closed the connection unexpectedly
        This probably means the server terminated abnormally
        before or while processing the request.
```

and after checking the logs, he found an even scarier message:

```
LOG:  server process (PID 11352) was terminated by signal 11:

Segmentation fault
```

A segmentation fault in Rust? That's supposed to be impossible, but here it was.

A segmentation fault happens when a program attempts to read parts of memory that don't exist, either because they were freed, or were never allocated in the first place. That doesn't happen in Rust under normal conditions, but we knew our project was far from normal. More confusingly, the error was coming from inside Scikit. It would have made sense if it was XGBoost or LightGBM, which we wrapped with a bunch of Rust `unsafe` blocks, but the error was coming from a universally used Python library.

### Debugging Ten Layers Down

Debugging segmentation faults inside compiled executables is hard. Debugging segmentation faults inside shared libraries inside FFI wrappers inside a machine learning library running inside a database... is harder. We've had very few clues: it worked on my Ubuntu 22.04 but didn't on Montana's Ubuntu 20.04. I dual-booted 20.04 to check it out and, surprise, it segfaulted for me too.

At this point I was convinced something was terribly wrong and called the "universal debugger" to the rescue: I littered Scikit's code with  `raise Exception("I'm here")` to see where it was going and, more importantly, where it couldn't make it because of the segfault. After a few hours, I was inside SciPy, over 10 function calls deep from our wrapper.

SciPy implements many useful scientific computing subroutines and one of them happens to solve linear regressions, a very popular machine learning algorithm. SciPy doesn't do it alone but calls out to a BLAS subroutine written to crunch numbers as fast as possible, and that's where I found the segfault.

It clicked. Scikit uses SciPy, SciPy uses C-BLAS and we used OpenBLAS for `ndarray` and our own vector functions, and everything is dynamically linked together at compile time. So which BLAS is SciPy using? It couldn't find the BLAS function it needed and crashed.

### Static Link or Bust

The fix was surprisingly simple: statically link OpenBLAS using the Cargo build script:

_build.rs_
```rust
fn main() {
    println!("cargo:rustc-link-lib=static=openblas");
}
```

The linker included the code for OpenBLAS into our extension, SciPy was able to find the function it was looking for, and PostgresML 2.0 was working again.


## Recap

In the end, we got what we wanted:

- Rust machine learning in Postgres was on track
- Scikit-learn was coming along into PostgresML 2.0
- Backwards compatibility with PostgresML 1.0 was preserved

and we had a lot of fun working with PyO3 and pushing the limits of what we thought was possible.

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. You can show your support by [starring us on our GitHub](https://github.com/postgresml/postgresml).
