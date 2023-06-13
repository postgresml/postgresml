# Open Source Alternative for Building End-to-End Vector Search Applications without OpenAI & Pinecone
# How to use this crate

Here is a brief outline of how to use this crate and specifically add new Python classes.

There are three main macros to know about:
- `custom_derive`
- `custom_methods`
- `custom_into_py`

## custom_derive 
`custom_derive` is used when defining a new struct that you want to be available as a Python class. This macro automatically creates a wrapper for the struct postfixing the name with `Python`. For example, the following code:
```
#[derive(custom_derive, Debug, Clone)]
pub struct TestStruct {
    pub name: String
}
```

Creates another struct:

```
pub struct TestStructPython {
    pub wrapped: TestStruct
}
```

You must currently implement `Debug` and `Clone` on the structs you use `custom_derive` on.

## custom_methods 
`custom_methods` is used on the impl block for a struct you want to be available as a Python class. This macro automatically creates methods that work seamlessly with pyO3. For example, the following code:
```
#[custom_methods(new, get_name)]
impl TestStruct {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
```

Produces similar code to the following:
```
impl TestStruct {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl TestStructPython {
    pub fn new<'a>(name: String, py: Python<'a>) -> PyResult<Self> {
        let x = TestStruct::new(name);
        Ok(TestStructPython::from(x))
    }
    pub fn get_name<'a>(&self, py: Python<'a>) -> PyResult<String> {
        let x = self.wrapped.get_name();
        Ok(x)
    }
} 
```

Note that the macro only works on methods marked with `pub`;

## custom_into_py 
`custom_into_py` is used when we want to seamlessly return Rust structs as Python dictionaries. For example, let's say we have the following code:
```
#[derive(custom_into_py, FromRow, Debug, Clone)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub parameters: Json<HashMap<String, String>>,
}

pub async fn get_text_splitters(&self) -> anyhow::Result<Vec<Splitter>> {
    Ok(sqlx::query_as(&query_builder!(
        "SELECT * from %s",
        self.splitters_table_name
    ))
    .fetch_all(self.pool.borrow())
    .await?)
}

```

The `custom_into_py` macro automatically generates the following code for us:
```
impl IntoPy<PyObject> for Splitter {
        fn into_py(self, py: Python<'_>) -> PyObject {
            let dict = PyDict::new(py);
            dict.set_item("id", self.id)
                .expect("Error setting python value in custom_into_py proc_macro");
            dict.set_item("created_at", self.created_at.timestamp())
                .expect("Error setting python value in custom_into_py proc_macro");
            dict.set_item("name", self.name)
                .expect("Error setting python value in custom_into_py proc_macro");
            dict.set_item("parameters", self.parameters.0)
                .expect("Error setting python value in custom_into_py proc_macro");
            dict.into()
        }
    }
```

Implementing `IntoPy` allows pyo3 to seamlessly convert between Rust and python. Note that Python users calling `get_text_splitters` will receive a list of dictionaries. 

## Other Noteworthy Things

Be aware that the only pyo3 specific code in this crate is the `pymodule` invocation in `lib.rs`. Everything else is handled by `pgml-macros`. If you want to expose a Python Class directly on the Python module you have to add it in the `pymodule` invocation. For example, if you wanted to expose `TestStruct` so Python module users could access it directly on `pgml`, you could do the following:
```
#[pymodule]
fn pgml(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TestStructPython>()?;
    Ok(())
}
```

Now Python users can access it like so:
```
import pgml

t = pgml.TestStruct("test")
print(t.get_name())

```

For local development, install [maturin](https://github.com/PyO3/maturin) and run:
```
maturin develop
```

You can now run the tests in `python/test.py`.
