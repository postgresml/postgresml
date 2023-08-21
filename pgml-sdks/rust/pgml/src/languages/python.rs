use pyo3::conversion::IntoPy;
use pyo3::types::{PyDict, PyFloat, PyInt, PyList, PyString};
use pyo3::{prelude::*, types::PyBool};
use std::collections::HashMap;

use crate::{pipeline::PipelineSyncData, types::Json};

////////////////////////////////////////////////////////////////////////////////
// Rust to PY //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl ToPyObject for Json {
    fn to_object(&self, py: Python) -> PyObject {
        match &self.0 {
            serde_json::Value::Bool(x) => x.to_object(py),
            serde_json::Value::Number(x) => {
                if x.is_f64() {
                    x.as_f64()
                        .expect("Error converting to f64 in impl ToPyObject for Json")
                        .to_object(py)
                } else {
                    x.as_i64()
                        .expect("Error converting to i64 in impl ToPyObject for Json")
                        .to_object(py)
                }
            }
            serde_json::Value::String(x) => x.to_object(py),
            serde_json::Value::Array(x) => {
                let list = PyList::empty(py);
                for v in x.iter() {
                    list.append(Json(v.clone()).to_object(py)).unwrap();
                }
                list.to_object(py)
            }
            serde_json::Value::Object(x) => {
                let dict = PyDict::new(py);
                for (k, v) in x.iter() {
                    dict.set_item(k, Json(v.clone()).to_object(py)).unwrap();
                }
                dict.to_object(py)
            }
            _ => panic!("Unsupported type for JSON conversion"),
        }
    }
}

impl IntoPy<PyObject> for Json {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

impl ToPyObject for PipelineSyncData {
    fn to_object(&self, py: Python) -> PyObject {
        Json::from(self.clone()).to_object(py)
    }
}

impl IntoPy<PyObject> for PipelineSyncData {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

////////////////////////////////////////////////////////////////////////////////
// PY to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl FromPyObject<'_> for Json {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        if ob.is_instance_of::<PyDict>()? {
            let dict: &PyDict = ob.downcast()?;
            let mut json = serde_json::Map::new();
            for (key, value) in dict.iter() {
                let value = Json::extract(value)?;
                json.insert(String::extract(key)?, value.0);
            }
            Ok(Self(serde_json::Value::Object(json)))
        } else if ob.is_instance_of::<PyBool>()? {
            let value = bool::extract(ob)?;
            Ok(Self(serde_json::Value::Bool(value)))
        } else if ob.is_instance_of::<PyInt>()? {
            let value = i64::extract(ob)?;
            Ok(Self(serde_json::Value::Number(value.into())))
        } else if ob.is_instance_of::<PyFloat>()? {
            let value = f64::extract(ob)?;
            let value = serde_json::value::Number::from_f64(value)
                .expect("Could not convert f64 to serde_json::Number");
            Ok(Self(serde_json::Value::Number(value)))
        } else if ob.is_instance_of::<PyString>()? {
            let value = String::extract(ob)?;
            Ok(Self(serde_json::Value::String(value)))
        } else if ob.is_instance_of::<PyList>()? {
            let value = ob.downcast::<PyList>()?;
            let mut json_values = Vec::new();
            for v in value {
                let v = v.extract::<Json>()?;
                json_values.push(v.0);
            }
            Ok(Self(serde_json::Value::Array(json_values)))
        } else {
            panic!("Unsupported type for JSON conversion");
        }
    }
}

impl FromPyObject<'_> for PipelineSyncData {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let json = Json::extract(ob)?;
        Ok(json.into())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Rust to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

macro_rules! gen_custom_into {
    ($t1:ty) => {
        impl CustomInto<$t1> for $t1 {
            fn custom_into(self) -> $t1 {
                self
            }
        }
    };
    (($($T1:ident),+), ($($T2:ident),+), ($($C:tt),+)) => {
        impl<$($T1, $T2: CustomInto<$T1>),+> CustomInto<($($T1),+,)> for ($($T2),+,) {
            fn custom_into(self) -> ($($T1),+,) {
                ($(self.$C.custom_into()),+,)
            }
        }
    }
}

/// Very similar to the `Into` trait, but we can implement it on foreign types
pub trait CustomInto<T> {
    fn custom_into(self) -> T;
}

impl CustomInto<Json> for PipelineSyncData {
    fn custom_into(self) -> Json {
        Json::from(self)
    }
}

impl<T1, T2: CustomInto<T1>> CustomInto<Option<T1>> for Option<T2> {
    fn custom_into(self) -> Option<T1> {
        self.map(|s| s.custom_into())
    }
}

impl<T1, T2: CustomInto<T1>> CustomInto<Vec<T1>> for Vec<T2> {
    fn custom_into(self) -> Vec<T1> {
        self.into_iter().map(|x| x.custom_into()).collect()
    }
}

impl<K1: std::cmp::Eq + std::hash::Hash, T1, K2: CustomInto<K1>, T2: CustomInto<T1>>
    CustomInto<HashMap<K1, T1>> for HashMap<K2, T2>
{
    fn custom_into(self) -> HashMap<K1, T1> {
        self.into_iter()
            .map(|(k, v)| (k.custom_into(), v.custom_into()))
            .collect()
    }
}

impl CustomInto<&'static str> for &str {
    fn custom_into(self) -> &'static str {
        // This is how we get around the liftime checker
        unsafe {
            let ptr = self as *const str;
            let ptr = ptr as *mut str;
            let boxed = Box::from_raw(ptr);
            Box::leak(boxed)
        }
    }
}

gen_custom_into!((T1), (TT2), (0));
gen_custom_into!((T1, T2), (TT1, TT2), (0, 1));
gen_custom_into!((T1, T2, T3), (TT1, TT2, TT3), (0, 1, 2));
gen_custom_into!((T1, T2, T3, T4), (TT1, TT2, TT3, TT4), (0, 1, 2, 3));
gen_custom_into!(
    (T1, T2, T3, T4, T5),
    (TT1, TT2, TT3, TT4, TT5),
    (0, 1, 2, 3, 4)
);
gen_custom_into!(
    (T1, T2, T3, T4, T5, T6),
    (TT1, TT2, TT3, TT4, TT5, TT6),
    (0, 1, 2, 3, 4, 5)
);

// There are some restrictions I cannot figure out around conflicting trait
// implimentations so this is my solution for now
gen_custom_into!(String);

gen_custom_into!(());

gen_custom_into!(bool);

gen_custom_into!(i8);
gen_custom_into!(i16);
gen_custom_into!(i32);
gen_custom_into!(i64);

gen_custom_into!(u8);
gen_custom_into!(u16);
gen_custom_into!(u32);
gen_custom_into!(u64);

gen_custom_into!(f32);
gen_custom_into!(f64);
