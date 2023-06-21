---
author: Silas Marvin 
description: A story and example of simultaneously writing multi-language native libraries using Rust
---

# Simultaneously Writing Multi-Language Native Libraries

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/silas.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
  	<p class="m-0">Silas Marvin</p>
  	<p class="m-0">June 21, 2023</p>
  </div>
</div>


Many of the tools we have created at PostgresML can be overwhelming for new users. They require domain specific knowledge with postgres, vector embeddings and other topics related to natural language processing. Like many companies before us, we want our users to have the benefits of our tools without the drawbacks of reading through expansive documentation, so we built an SDK.

We are huge fans of Rust, but we are also advocates of other languages that are either more amenable to new programmers or better suited for specific use cases. Our goal with our SDK was to build it once in Rust, and make it a pleasurable experience to use in any number of different languages. This is typically done using FFIs ([foreign function interfaces](https://en.wikipedia.org/wiki/Foreign_function_interface)) but the method we use creates native modules in the specific language we target. For the rest of this post we will be looking at creating both a Rust and Python native library simultaneously, but rest assured we used this same technique to generate our Javascript native library.

## What is Wrong With FFIs 

FFIs are the standard way for programs written in one language to call functions written in another. In terms of our SDK, we could utilize FFIs by writing the core logic of our SDK in Rust, and calling our Rust functions through FFIs from Python. This unfortunately does not provide the utility we desire. Take for example the following Python code:
```
class Database:
	def __init__(self, connection_string: str):
		# Create some connection

	async def vector_search(self, query: str, model_id: int, splitter_id: int) -> str:
		# Do some async search here
		return result

async def main():
	db = Database(CONNECTION_STRING)
	result = await db.vector_search("What is the best way to do machine learning", 1, 1)
	if result != "PostgresML":
		print("The model still needs more training")
	else:
		print("The model is ready to go!")
```

As mentioned before, we only want to write our SDK once in Rust, and then use it in any language we target. Specifically, the `class Database` and its methods should be written in the Rust SDK. How can we translate the `Database` class and utilize it in Python through FFIs? Simply put, we cannot. There are two limitations we cannot surpass in the above code:
- FFI's have no concept of Python classes
- FFI's have no concept of Python async

We could remove the asynchronous nature of the `vector_search` function, and instead of having it in a class, make it a standalone function, but both of these would make using our SDK in Python a not so pleasurable experience. 

## Writing a Native Python Module in Rust

What we really want is to write a native Python module in Rust that understands async and can return classes. Luckily for us, there is an awesome crate called [pyo3](https://github.com/PyO3/pyo3) built exactly for this purpose.

First, let's backtrack a little bit. Remember our goal is to write an SDK once in Rust, and use it for any target language we choose. One of those target languages is Rust, so before getting into [pyo3](https://github.com/PyO3/pyo3) lets create a Rust implementation of the above `Database` class. 
```
struct Database{
	connection_string: String
}

impl Database {
	pub fn new(connection_string: String) -> Self {
		// The actual connection process has been removed 
		Self {
			connection_string
		}
	}

	pub async fn vector_search(&self, query: String, model_id: i64, splitter_id: i64) -> String {
		// Do some async vector search
		result
	}
}
```

Rust is not an object oriented programming language, so the idea of relating structs to classes is inherently flawed, but for the purposes of this simplified post, this relation works fine.

Getting this to work in [pyo3](https://github.com/PyO3/pyo3) and available as a native python module is actually very simple. To follow along, first run the following (enter yes to `pyo3 bindings`):
```
mkdir tmp
cd tmp
python3 -m venv venv
source venv/bin/activate
pip install maturin
maturin new pgml
cd pgml
```

All of the above creates a new Rust project with [maturin](https://github.com/PyO3/maturin), a utility written by [pyo3](https://github.com/PyO3/pyo3), preconfigured. Let's edit the `src/lib.rs` file to look like the following:
```
use pyo3::prelude::*;

#[pyclass]
struct Database{
	connection_string: String
}

#[pymethods]
impl Database {
	#[new]
	pub fn new(connection_string: String) -> Self {
		// The actual connection process has been removed
		Self {
			connection_string
		}
	}

	pub fn vector_search<'a>(&self, py: Python<'a>, query: String, model_id: i64, splitter_id: i64) -> PyResult<&'a PyAny> {
		pyo3_asyncio::tokio::future_into_py(py, async move {
			// Do some async vector search
			let result = "PostgresML".to_string(); // Filler so it compiles
			Ok(result)
		})
	}
}

/// A Python module implemented in Rust.
#[pymodule]
fn pgml(_py: Python, m: &PyModule) -> PyResult<()> {
	m.add_class::<Database>()?;
	Ok(())
}
```

NOTE: your dependencies should look like the following: 
```
pyo3 = "0.18.0"
pyo3-asyncio = { version = "0.18", features = ["attributes", "tokio-runtime"] }
```

After running `maturin develop` we can now run a python file such as the following:
```
import asyncio
import pgml

CONNECTION_STRING = "some filler string"

async def test():
	db = pgml.Database(CONNECTION_STRING)
	result = await db.vector_search("What is the best way to do machine learning?", 1, 1)
	if result != "PostgresML":
		print("The model still needs more training")
	else:
		print("The model is ready to go!")

asyncio.run(test())
```

And it will print out `The model is ready to go!`.

## Automatically Writing Native Libraries with Proc Macros

Together we have successfully written a native Python module in Rust. However, our goal is far from complete. Our desire is to write our SDK once in Rust, and make it available in any language we target. While the above made it available in Python, it is both no longer a valid Rust library, and would involve much rewriting to make available in Javascript. 

Really what we want is to write our Rust library without worrying about any translation, and apply some macros that auto convert into what [pyo3](https://github.com/PyO3/pyo3) needs to create a Python module. This sounds like a perfect use for [procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html). If you are unfamiliar with macros I really recommend reading the [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/book/README.html) it is free, a quick read, and provides an awesome introduction to macros.

Unfortunately, from this point forward this post will no longer provide easy copy and paste code snippets to follow along. It involves some extreme hand waving to simplify things, and mostly covers the idea behind our implementation, not the exact code. Our codebase is open source, and anyone interested is welcome to see our full production implementation: [github](https://github.com/postgresml/postgresml).

Lets slightly edit the struct we defined previously:
```
#[custom_derive_class]
struct Database{
	connection_string: String
}

#[custom_derive_methods]
impl Database {
	pub fn new(connection_string: String) -> Self {
		// The actual connection process has been removed 
		Self {
			connection_string
		}
	}

	pub async fn vector_search(&self, query: String, model_id: i64, splitter_id: i64) -> String {
		// Do some async vector search
		result
	}
}
```

Notice that there are two new macros we have not seen before: `custom_derive_class` and `custom_derive_methods`. Both of these are macros we have written.

`custom_derive_class` creates a wrapper for our `Database` struct. Let's show the expanded code our `custom_derive_class` generates:
```
#[pyclass]
struct DatabasePython {
	wrapped: Database
}
```

Notice it automatically creates a new struct `DatabasePython` and applies the `pyclass` macro just as we did originally to the `Database` struct above. How does it do this? Creating a macro like this is extraordinarily simple. The `custom_derive_class` macro is done entirely in the following block:
```
#[proc_macro_derive(custom_derive)]
pub fn custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let parsed = parse_macro_input!(input as DeriveInput);
	let name_ident = format_ident!("{}Python", parsed.ident);
	let wrapped_type_ident = parsed.ident;
	let expanded = quote! {
		#[pyclass]
		pub struct #name_ident {
			wrapped: #wrapped_type_ident
		}
	};
	proc_macro::TokenStream::from(expanded)
}

```

Let's look at the expanded code our `custom_derive_methods` macro produces when used on the `Database` struct:
```
#[pymethods]
impl DatabasePython {
	#[new]
	pub fn new(connection_string: String) -> Self {
		// The actual connection process has been removed
		Self::from(Database::new(connection_string))
	}

	pub fn vector_search<'a>(&self, py: Python<'a>, query: String, model_id: i64, splitter_id: i64) -> PyResult<&'a PyAny> {
		let wrapped = self.wrapped.clone();
		pyo3_asyncio::tokio::future_into_py(py, async move {
			// Do some async vector search
			let x = wrapped.vector_search(query, model_id, splitter_id).await;
			Ok(x)
		})
	}
}
```

You will notice that this is very similar to code we wrote above when we created our Python module except we simply treat `DatabasePython` as a wrapper, and call through to the `Database` struct for all actual work. This creates a very definite devision between our Python compatible code, and pure Rust code.

How does the macro actually work? We can break the `custom_derive_methods` macro code generation into three distinct phases:
- Method destruction 
- Signature translation 
- Method reconstruction

### Method Destruction
Utilizing the [syn crate](https://crates.io/crates/syn) we parse the `impl` block of the `Database` struct and iterate over the individual methods parsing them into our own type:
```
pub struct GetImplMethod {
	pub exists: bool,
	pub method_ident: Ident,
	pub is_async: bool,
	pub method_arguments: Vec<(String, SupportedType)>,
	pub receiver: Option<proc_macro2::TokenStream>,
	pub output_type: OutputType,
}
```

Here `SupportType` and `OutputType` are our custom enums of types we support, looking something like:
```
pub enum SupportedType {
	Reference(Box<SupportedType>),
	str,
	String,
	Vec(Box<SupportedType>),
	HashMap((Box<SupportedType>, Box<SupportedType>)),
	Option(Box<SupportedType>),
	Tuple(Vec<SupportedType>),
	S, // Self
	i64,
	i32,
	f64,
	// Other omitted types
}

pub enum OutputType {
	Result(SupportedType),
	Default,
	Other(SupportedType),
}
```

### Signature Translation
We must translate the signature into the Rust code [pyo3](https://github.com/PyO3/pyo3) expects. This means adjusting the arguments, async declaration, and output type. This is actually extraordinarily simple now that we have destructed the method. For instance, here is a simple example of translating the output type:
```
fn convert_output_type(
	ty: &SupportedType,
	method: &GetImplMethod,
) -> (
	Option<proc_macro2::TokenStream>
) {
	if method.is_async {
		Some(quote! {PyResult<&'a PyAny>})
	} else {
		let ty = t
			.to_type()
			.unwrap();
		Some(quote! {PyResult<#ty>})
    }
}
```

### Method Reconstruction
Now we have all the information we need to reconstruct the methods in the format [pyo3](https://github.com/PyO3/pyo3) needs to create a native Python module. 

The actual reconstruction is quite boring, mostly filled with a bunch of `if else` statements writing and combining token streams using the [quote crate](https://crates.io/crates/quote) so we will omit it for brevity's sake. For the curious, here is a link to our actual implementation: [github](https://github.com/postgresml/postgresml/blob/545ccb613413eab4751bf03ea4c020c09b20af3c/pgml-sdks/rust/pgml-macros/src/python.rs#L152C1-L238).

The entirety of the above three phases can be summed up with this extraordinarily abstract function:
```
fn do_custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let parsed_methods = parse_methods(input);
	let mut methods = Vec::new();
	for method in parsed_methods {
		// Destructure Method
		let destructured = destructure(method);
		// Translate Signature
		let signature = convert_signature(&destructured);
		// Restructure Method 
		let method = create_method(&destructured, &signature);
		methods.push(method);
	}
	// This is the actual Rust impl block we are generating
	proc_macro::TokenStream::from(quote! {
        #[pymethods]
        impl DatabasePython {
		#(#methods)*
        }
    })
}
```

## Closing and Future Endeavors
All of the above to show how we are simultaneously creating a native Rust, Python, and (while not discussed) Javascript library. There are quirks to the above methods, but we are still actively developing and improving on our designs.

While our macros are currently specialized for the specific use cases we have, we are exploring the idea of generalizing and pushing them out as their own crate to help everyone write native libraries in Rust and the languages of their choosing.

Thanks for reading!
