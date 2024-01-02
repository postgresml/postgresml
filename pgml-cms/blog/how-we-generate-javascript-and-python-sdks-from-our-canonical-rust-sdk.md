---
description: >-
  Our story of simultaneously writing multi-language native libraries using
  Rust.
---

# How We Generate JavaScript and Python SDKs From Our Canonical Rust SDK

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="128"><figcaption></figcaption></figure>

</div>

Silas Marvin

July 11, 2023

## Introduction

The tools we have created at PostgresML are powerful and flexible. There are almost an infinite number of ways our tools can be utilized to power vector search, model inference, and much more. Like many companies before us, we want our users to have the benefits of our tools without the drawbacks of reading through expansive documentation, so we built an SDK.

We are huge fans of Rust (almost our entire codebase is written in it), and we find that using it as our primary language allows us to write safer code and iterate through our development cycles faster. However, the majority of our users currently work in languages like Python and JavaScript. There would be no point making an SDK for Rust, when no one would use it. After much deliberation, we finalized the following requirements for our SDK:

1. It must be available natively in multiple languages
2. All languages must have identical behavior to the canonical Rust implementation
3. Adding new languages should only include minimal overhead

<figure><img src=".gitbook/assets/image (12) (1).png" alt=""><figcaption><p>TLDR we are building macros that convert vanilla Rust to compatible Pyo3 and Neon Rust, which is then further converted to native Python and JavaScript modules.</p></figcaption></figure>

## What is Wrong With FFIs

The first requirement of our SDK is that it is available natively in multiple languages, and the second is that it is written in Rust. At first glance, this seems like a contradiction, but there is a very well known system for writing functions in one language and using them in another known as FFIs (foreign function interfaces). In terms of our SDK, we could utilize FFIs by writing the core logic of our SDK in Rust, and calling our Rust functions through FFIs from the language of our choice. This unfortunately does not provide the utility we desire. Take for example the following Python code:

```python
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

One of the requirement of our SDK is that we write it in Rust. Specifically, in this instance, the `class Database` and its methods should be written in Rust and utilized in Python through FFIs. Unfortunately, doing this in Rust alone is not possible. There are two limitations we cannot surpass in the above code:

* FFI's have no concept of Python classes
* FFI's have no concept of Python async

We could write our own Python wrapper around our FFI, but that would go against requirement 3: Adding new languages should only include minimal overhead. Translating every update from our Rust SDK into a wrapper for each language we add is not minimal overhead.

## Enter pyO3 and Neon

[Pyo3](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/) are Rust crates that help with building native modules for Python and JavaScript. They provide systems that allow us to write Rust code that can seamlessly interact with async code and native classes in Python and JavaScript, bypassing the limitations that vanilla FFIs imposed.

Let's take a look at some Rust code that creates a Python class with [Pyo3](https://github.com/PyO3/pyo3) and a JavaScript class with [Neon](https://neon-bindings.com/). For ease of use, let's say we have the following struct in Rust:

```rust
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

Here is the code augmented to work with [Pyo3](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/):

{% tabs %}
{% tab title="Pyo3" %}
```rust
use pyo3::prelude::*;

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
{% endtab %}

{% tab title="Neon" %}
```rust
use neon::prelude::*;

struct Database{
     connection_string: String
}

impl Database {
    pub fn new<'a>(mut cx: FunctionContext<'a>) -> JsResult<'a, JsObject> {
        // The actual connection process has been removed
        let arg0 = cx.argument::<JsString>(0usize as i32)?;
        let arg0 = <String>::from_js_type(&mut cx, arg0)?;
        let x = Self {
            connection_string: arg0
        };
        x.into_js_result(&mut cx)
    }

    pub fn vector_search<'a>(mut cx: FunctionContext<'a>) -> JsResult<'a, JsPromise> {
        let this = cx.this();
        let s: neon::handle::Handle<
            neon::types::JsBox<std::cell::RefCell<DatabaseJavascript>>,
        > = this.get(&mut cx, "s")?;
        let wrapped = (*s).deref().borrow();
        let wrapped = wrapped.wrapped.clone();
        let arg0 = cx.argument::<neon::types::JsString>(0)?;
        let arg0 = <String>::from_js_type(&mut cx, arg0)?;
        let arg1 = cx.argument::<JsNumber>(1);
        let arg1 = <i64>::from_js_type(&mut cx, arg1);
        let arg2 = cx.argument::<JsNumber>(2);
        let arg2 = <i64>::from_js_type(&mut cx, arg2);
        let channel = cx.channel();
        let (deferred, promise) = cx.promise();
        deferred
            .try_settle_with(
                &channel,
                move |mut cx| {
                    // Do some async vector search
                    result.into_js_result(&mut cx)
                },
            )
            .expect("Error sending js");
        Ok(promise)
    }

    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(self, cx: &mut C) -> JsResult<'b, Self::Output> {
        let obj = cx.empty_object();
        let s = cx.boxed(std::cell::RefCell::new(self));
        obj.set(cx, "s", s)?;
        let f: Handle<JsFunction> = JsFunction::new(
            cx,
            Database::new,
        )?;
        obj.set(cx, "new", f)?;
        let f: Handle<JsFunction> = JsFunction::new(
            cx,
            Database::vector_search,
        )?;
        Ok(obj)
    }
}

impl neon::types::Finalize for Database {}

/// A JavaScript module implemented in Rust.
#[main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("newDatabase", Database::new)?;
    Ok(())
}
```
{% endtab %}
{% endtabs %}

## Automatically Converting Vanilla Rust to py03 and Neon compatible Rust

We have successfully written a native Python and JavaScript module in Rust. However, our goal is far from complete. Our desire is to write our SDK once in Rust, and make it available in any language we target. While the above made it available in Python and JavaScript, it is both no longer a valid Rust library, and required a bunch of manual edits to make available in both languages.

Really what we want is to write our Rust library without worrying about any translation, and apply some macros that auto convert into what [Pyo3](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/) need. This sounds like a perfect use for [procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html). If you are unfamiliar with macros I really recommend reading [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/book/README.html) it is free, a quick read, and provides an awesome introduction to macros.

We are creating a flow that looks like the following:

<figure><img src=".gitbook/assets/image (13).png" alt=""><figcaption></figcaption></figure>

Let's slightly edit the struct we defined previously:

```rust
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

`custom_derive_class` creates wrappers for our `Database` struct. Let's show the expanded code our `custom_derive_class` generates:

```rust
#[pyclass]
struct DatabasePython {
     wrapped: Database
}

impl From<Database> for DatabasePython {
    fn from(w: Database) -> Self {
        Self { wrapped: w }
    }
}

struct DatabaseJavascript {
    wrapped: Database
}

impl From<Database> for DatabaseJavascript {
    fn from(w: Database) -> Self {
        Self { wrapped: w }
    }
}

impl IntoJsResult for Database {
    type Output = neon::types::JsObject;
    fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(
        self,
        cx: &mut C,
    ) -> neon::result::JsResult<'b, Self::Output> {
        DatabaseJavascript::from(self).into_js_result(cx)
    }
}
```

There are a couple important things happening here:

1. Our `custom_derive_class` macro creates a new struct for each language we target.
2. The derived Python struct automatically implements `pyclass`
3. Because [Neon](https://neon-bindings.com/) does not have a version of the `pyclass` macro, we implement our own trait `IntoJsResult` to do some conversions between vanilla Rust types and [Neon](https://neon-bindings.com/) Rust

Creating a macro like the above is actually incredibly simple. The code below shows how it is done for the Python variant.

```rust
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

```rust
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

impl DatabaseJavascript {
    pub fn new<'a>(
        mut cx: neon::context::FunctionContext<'a>,
    ) -> neon::result::JsResult<'a, JsObject> {
        let arg0 = cx.argument::<JsString>(0usize as i32)?;
        let arg0 = <String>::from_js_type(&mut cx, arg0)?;
        let x = Database::new(&arg0);
        let x = x.expect("Error in rust method");
        let x = Self::from(x);
        x.into_js_result(&mut cx)
    }

    pub fn vector_search<'a>(
        mut cx: neon::context::FunctionContext<'a>,
    ) -> neon::result::JsResult<'a, neon::types::JsPromise> {
        use neon::prelude::*;
        use core::ops::Deref;
        let this = cx.this();
        let s: neon::handle::Handle<
            neon::types::JsBox<std::cell::RefCell<DatabaseJavascript>>,
        > = this.get(&mut cx, "s")?;
        let wrapped = (*s).deref().borrow();
        let wrapped = wrapped.wrapped.clone();
        let arg0 = cx.argument::<neon::types::JsString>(0)?;
        let arg0 = <String>::from_js_type(&mut cx, arg0)?;
        let arg1 = cx.argument::<JsNumber>(1);
        let arg1 = <i64>::from_js_type(&mut cx, arg1);
        let arg2 = cx.argument::<JsNumber>(2);
        let arg2 = <i64>::from_js_type(&mut cx, arg2);
        let channel = cx.channel();
        let (deferred, promise) = cx.promise();
        deferred
            .try_settle_with(
                &channel,
                move |mut cx| {
                    let runtime = crate::get_or_set_runtime();
                    let x = runtime.block_on(wrapped.vector_search(&arg0, arg1, arg2));
                    let x = x.expect("Error in rust method");
                    x.into_js_result(&mut cx)
                },
            )
            .expect("Error sending js");
        Ok(promise)
    }
}

impl IntoJsResult for DatabaseJavascript {
    type Output = neon::types::JsObject;
    fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(
        self,
        cx: &mut C,
    ) -> neon::result::JsResult<'b, Self::Output> {
        use neon::object::Object;
        let obj = cx.empty_object();
        let s = cx.boxed(std::cell::RefCell::new(self));
        obj.set(cx, "s", s)?;
        let f: neon::handle::Handle<neon::types::JsFunction> = neon::types::JsFunction::new(
            cx,
            DatabaseJavascript::new,
        )?;
        obj.set(cx, "new", f)?;
        let f: neon::handle::Handle<neon::types::JsFunction> = neon::types::JsFunction::new(
            cx,
            DatabaseJavascript::vector_search,
        )?;
        obj.set(cx, "vector_search", f)?;
        Ok(obj)
    }
}

impl neon::types::Finalize for DatabaseJavascript {}
```

You will notice this is very similar to code we have showed already except the `DatabaseJavascript` and `DatabasePython` structs just call their respective methods on the `Database` struct.

How does the macro actually work? We can break the `custom_derive_methods` macro code generation into three distinct phases:

* Method destruction
* Signature translation
* Method reconstruction

### Method Destruction

Utilizing the [syn crate](https://crates.io/crates/syn) we parse the `impl` block of the `Database` struct and iterate over the individual methods parsing them into our own type:

```rust
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

```rust
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

We must translate the signature into the Rust code [Pyo3](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/) expects. This means adjusting the arguments, async declaration, and output type. This is actually extraordinarily simple now that we have destructed the method. For instance, here is a simple example of translating the output type for Python:

```rust
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

Now we have all the information we need to reconstruct the methods in the format [Pyo3](https://github.com/PyO3/pyo3) and [Neon](https://neon-bindings.com/) need to create native modules.

The actual reconstruction is quite boring, mostly filled with a bunch of `if else` statements writing and combining token streams using the [quote crate](https://crates.io/crates/quote), so we will omit it for brevity's sake. For the curious, here is a link to our actual implementation: [github](https://github.com/postgresml/postgresml/blob/545ccb613413eab4751bf03ea4c020c09b20af3c/pgml-sdks/rust/pgml-macros/src/python.rs#L152C1-L238).

The entirety of the above three phases can be summed up with this extraordinarily abstract function (Python specific though it is almost identical for JavaScript):

```rust
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

All of the above to show how we are simultaneously creating a native Rust, Python, and JavaScript library. There are quirks to the above methods, but we are still actively developing and improving on our designs.

While our macros are currently specialized for the specific use cases we have, we are exploring the idea of generalizing and pushing them out as their own crate to help everyone write native libraries in Rust and the languages of their choosing. We're also planning to add support for more languages, and we'd love to hear feedback on your language of choice.

Thanks for reading!
