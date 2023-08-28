# A Tool for Automatically Translating to Py03 and Neon compatible Rust

This crate provides a number of macros that automatically handle translating mostly vanilla Rust to [Py03](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/) compatible Rust code.

It is designed to greatly improve the rate at which simultaneously developing libraries for multiple languages can be done. We use it internally for developing our [Rust SDK](https://github.com/postgresml/postgresml/tree/master/pgml-sdks/rust/pgml).

# Overview 

Here is a simple example of what this crate does.

Let's say we have some complex library we are writing in Rust, but want to give our users interfaces in Python and other high level languages.

What we really want is to take something like the following Rust code:

```rust
struct Test {
  x: i32
}

impl Test {
  fn new(x: i32) -> Self {
    Self {
      x
    }
  }

  fn get(&self) -> i32 {
    self.x
  }
}
```

and automatically produce the following Python code:

```python
class Test:
  def __new__(self, x):
    self.x = x

  def get(self):
    return self.x
```

This is not what our library does. We have not written a Rust to Python transpiler, instead we wrote a Rust to Rust transpiler that uses crates like [Py03](https://github.com/PyO3/pyo3) to automatically write the necessary [foreign function interfaces](https://en.wikipedia.org/wiki/Foreign_function_interface).

A real example of translating our `Test` struct would look like the following:

```rust
#[derive(alias, Debug, Clone)]
pub struct Test {
    x: i32,
}

#[alias_methods(new, get)]
impl Test {
    pub fn new(x: i32) -> Self {
        Self { x }
    }

    pub fn get(&self) -> i32{
        self.x
    }
}
```

These macros produce a lot of code, you can inspect it using [cargo-expand](https://github.com/dtolnay/cargo-expand) but we will just show the snippets necessary to understand what is happening:

```rust
pub struct Test {
    x: i32,
}

#[cfg(feature = "python")]
#[pyo3::pyclass]
pub struct TestPython {
    pub wrapped: std::boxed::Box<Test>,
}

impl Test {
    pub fn new(x: i32) -> Self {
        Self { x }
    }

    pub fn get(&self) -> i32 {
        self.x
    }
}

#[cfg(feature = "python")]
impl TestPython {
    pub fn new<'a>(x: i32, py: pyo3::Python<'a>) -> pyo3::PyResult<Self> {
        use rust_bridge::python::CustomInto;
        let x: i32 = x.custom_into();
        let x = Test::new(x);
        let x: Self = x.custom_into();
        Ok(x)
    }

    pub fn get<'a>(&mut self, py: pyo3::Python<'a>) -> pyo3::PyResult<i32> {
        use rust_bridge::python::CustomInto;
        let mut wrapped: &Test = self.custom_into();
        let x = wrapped.get();
        let x: i32 = x.custom_into();
        Ok(x)
    }
}

```

NOTE: this is not the exact code output, but very similar to what you would see with `cargo-expand`.

The struct `Test` is defined just like before, but notice there is another struct `TestPython`. This struct also has the `pyo3::pyclass` macro being applied to it. This macro is used by Py03 to generate a Python class.

`TestPython` can be thought of as a Py03 compatible wrapper around `Test`. It wraps `Test` drawing a clear line between our normal Rust code, and Py03 compatible Rust.

The impl block for the struct `Test` is there just like before, but notice the impl block for `TestPython` which is written in the fashion Py03 requires, but simply parses arguments and calls through to `Test`'s methods.

The key takeaway is that the macros used in this crate take vanilla Rust code and translate it to Rust that crates like Py03 use to generate native Python classes.

# Python

This crate translates Rust to [Py03](https://github.com/PyO3/pyo3) compatible Rust. See the [examples](examples) folder for how to use it.

# JavaScript

This crate translates Rust to [Neon](https://neon-bindings.com/) compatible Rust. See the [examples](examples) folder for how to use it.
