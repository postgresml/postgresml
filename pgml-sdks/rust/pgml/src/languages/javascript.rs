use neon::prelude::*;
use std::cell::RefCell;

use crate::types::{DateTime, Json};

////////////////////////////////////////////////////////////////////////////////
// Rust to JS //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
macro_rules! gen_into {
    ($t1:ty, $t2:ty) => {
        impl IntoJsResult for $t1 {
            type Output = $t2;
            fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
                self,
                cx: &mut C,
            ) -> JsResult<'b, Self::Output> {
                Ok(<$t2>::new(cx, self))
            }
        }
    };
    ($t1:ty, $t2:ty, $t3:ty) => {
        impl IntoJsResult for $t1 {
            type Output = $t2;
            fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
                self,
                cx: &mut C,
            ) -> JsResult<'b, Self::Output> {
                Ok(<$t2>::new(cx, <$t3>::new(self)))
            }
        }
    };
    (($($T:ident),+); ($($C:tt),+), $len:literal) => {
        impl<$($T: IntoJsResult),+> IntoJsResult for ($($T),+,) {
            type Output = JsArray;
            fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
                self,
                cx: &mut C,
            ) -> JsResult<'b, Self::Output> {
                let js_array = JsArray::new(cx, $len as u32);
                $(
                    let js_value = self.$C.into_js_result(cx)?;
                    js_array.set(cx, $C, js_value)?;
                )+
                Ok(js_array)
            }
        }
    }
}

pub trait IntoJsResult {
    type Output: neon::handle::Managed + neon::types::Value;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output>;
}

impl IntoJsResult for () {
    type Output = JsUndefined;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        Ok(JsUndefined::new(cx))
    }
}

gen_into!(String, JsString); // String
gen_into!(bool, JsBoolean); // bool

gen_into!(i8, JsNumber); // i8
gen_into!(i16, JsNumber); // i16
gen_into!(i32, JsNumber); // i32
gen_into!(u8, JsNumber); // u8
gen_into!(u16, JsNumber); // u16
gen_into!(u32, JsNumber); // u32
gen_into!(f32, JsNumber); // f32
gen_into!(f64, JsNumber); // f64

// Tuples of size up to 6
gen_into!((T1); (0), 1);
gen_into!((T1, T2); (0, 1), 2);
gen_into!((T1, T2, T3); (0, 1, 2), 3);
gen_into!((T1, T2, T3, T4); (0, 1, 2, 3), 4);
gen_into!((T1, T2, T3, T4, T5); (0, 1, 2, 3, 4), 5);
gen_into!((T1, T2, T3, T4, T5, T6); (0, 1, 2, 3, 4, 5), 6);

impl IntoJsResult for i64 {
    type Output = JsNumber;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        Ok(JsNumber::new(cx, self as f64))
    }
}

impl IntoJsResult for DateTime {
    type Output = neon::types::JsDate;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        let date = neon::types::JsDate::new(cx, self.0.timestamp_millis() as f64)
            .expect("Error converting to JS Date");
        Ok(date)
    }
}

// impl IntoJsResult for JsonHashMap {
//     type Output = JsObject;
//     fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
//         self,
//         cx: &mut C,
//     ) -> JsResult<'b, Self::Output> {
//         self.0 .0.into_js_result(cx)
//     }
// }

impl IntoJsResult for Json {
    type Output = JsObject;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        let js_object = JsObject::new(cx);
        for (k, v) in self
            .0
            .as_object()
            .expect("We currently only support json objects")
            .iter()
        {
            let js_key = cx.string(k);
            match v {
                // TODO: Support more types like nested objects
                serde_json::Value::Number(x) => {
                    let js_value = x
                        .as_f64()
                        .expect("Error converting to f64 in impl IntoJsResult for Json");
                    let js_value = JsNumber::new(cx, js_value);
                    js_object.set(cx, js_key, js_value)?;
                }
                serde_json::Value::Bool(x) => {
                    let js_value = JsBoolean::new(cx, *x);
                    js_object.set(cx, js_key, js_value)?;
                }
                serde_json::Value::String(x) => {
                    let js_value = cx.string(x);
                    js_object.set(cx, js_key, js_value)?;
                }
                _ => {}
            }
        }
        Ok(js_object)
    }
}

impl<K: IntoJsResult, V: IntoJsResult> IntoJsResult for std::collections::HashMap<K, V> {
    type Output = JsObject;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        let js_object = JsObject::new(cx);
        for (key, value) in self.into_iter() {
            let js_key = key.into_js_result(cx)?;
            let js_value = value.into_js_result(cx)?;
            js_object.set(cx, js_key, js_value)?;
        }
        Ok(js_object)
    }
}

impl<T: IntoJsResult> IntoJsResult for Vec<T> {
    type Output = JsArray;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(self, cx: &mut C) -> JsResult<'b, JsArray> {
        let js_array = JsArray::new(cx, self.len() as u32);
        for (i, item) in self.into_iter().enumerate() {
            let js_item = item.into_js_result(cx)?;
            js_array.set(cx, i as u32, js_item)?;
        }
        Ok(js_array)
    }
}

// Our own types
gen_into!(
    crate::database::Database,
    JsBox<RefCell<crate::database::Database>>,
    RefCell<crate::database::Database>
);
impl Finalize for crate::database::Database {}
gen_into!(
    crate::collection::Collection,
    JsBox<RefCell<crate::collection::Collection>>,
    RefCell<crate::collection::Collection>
);
impl Finalize for crate::collection::Collection {}

////////////////////////////////////////////////////////////////////////////////
// JS To Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
macro_rules! gen_from {
    ($t1:ty, $t2:ty) => {
        impl FromJsType for $t1 {
            type From = $t2;
            fn from_js_type<'a, C: Context<'a>>(
                cx: &mut C,
                arg: Handle<Self::From>,
            ) -> NeonResult<Self> {
                Ok(arg.value(cx))
            }
        }
    };
    ($t1:ty, $t2:ty, $t3:ty) => {
        impl FromJsType for $t1 {
            type From = $t2;
            fn from_js_type<'a, C: Context<'a>>(
                cx: &mut C,
                arg: Handle<Self::From>,
            ) -> NeonResult<Self> {
                Ok(arg.value(cx) as $t3)
            }
        }
    };
}

pub trait FromJsType: Sized {
    type From: neon::handle::Managed + neon::types::Value;
    fn from_js_type<'a, C: Context<'a>>(_cx: &mut C, _arg: Handle<Self::From>) -> NeonResult<Self> {
        panic!("Have not implimented from_js_type for type yet")
    }
    fn from_option_js_type<'a, C: Context<'a>>(
        _cx: &mut C,
        _arg: Option<Handle<Self::From>>,
    ) -> NeonResult<Self> {
        panic!("Have not implimented from_option_js_type for type yet")
    }
}

gen_from!(String, JsString); // String
gen_from!(bool, JsBoolean); // bool

gen_from!(i8, JsNumber, i8); // i8
gen_from!(i16, JsNumber, i16); // i16
gen_from!(i32, JsNumber, i32); // i32
gen_from!(i64, JsNumber, i64); // i32
gen_from!(u8, JsNumber, u8); // u8
gen_from!(u16, JsNumber, u16); // u16
gen_from!(u32, JsNumber, u32); // u32
gen_from!(f32, JsNumber, f32); // f32
gen_from!(f64, JsNumber); // f64

impl<T: FromJsType> FromJsType for Option<T> {
    type From = JsValue;
    fn from_option_js_type<'a, C: Context<'a>>(
        cx: &mut C,
        arg: Option<Handle<Self::From>>,
    ) -> NeonResult<Self> {
        Ok(match arg {
            Some(v) => {
                let arg: Handle<T::From> = v.downcast(cx).or_throw(cx)?;
                let arg = T::from_js_type(cx, arg)?;
                Some(arg)
            }
            None => None,
        })
    }
}

impl<T: FromJsType> FromJsType for Vec<T> {
    type From = JsArray;
    fn from_js_type<'a, C: Context<'a>>(cx: &mut C, arg: Handle<Self::From>) -> NeonResult<Self> {
        let arg = arg.to_vec(cx)?;
        let mut output = Vec::new();
        for item in arg {
            let item: Handle<T::From> = item.downcast(cx).or_throw(cx)?;
            let item = T::from_js_type(cx, item)?;
            output.push(item);
        }
        Ok(output)
    }
}

impl<K: FromJsType + std::hash::Hash + std::fmt::Display + std::cmp::Eq, V: FromJsType> FromJsType
    for std::collections::HashMap<K, V>
{
    type From = JsObject;
    fn from_js_type<'a, C: Context<'a>>(cx: &mut C, arg: Handle<Self::From>) -> NeonResult<Self> {
        let mut output = std::collections::HashMap::new();
        let keys = arg.get_own_property_names(cx)?.to_vec(cx)?;
        for key in keys {
            let key: Handle<K::From> = key.downcast(cx).or_throw(cx)?;
            let key: K = K::from_js_type(cx, key)?;
            let js_key = std::string::ToString::to_string(&key);
            let value: Handle<JsValue> = arg.get(cx, js_key.as_str())?;
            let value: Handle<V::From> = value.downcast(cx).or_throw(cx)?;
            let value = V::from_js_type(cx, value)?;
            output.insert(key, value);
        }
        Ok(output)
    }
}

impl FromJsType for Json {
    type From = JsObject;
    fn from_js_type<'a, C: Context<'a>>(cx: &mut C, arg: Handle<Self::From>) -> NeonResult<Self> {
        let mut json = serde_json::Map::new();
        let keys = arg.get_own_property_names(cx)?.to_vec(cx)?;
        for key in keys {
            let key: Handle<JsString> = key.downcast(cx).or_throw(cx)?;
            let key: String = String::from_js_type(cx, key)?;
            let value: Handle<JsValue> = arg.get(cx, key.as_str())?;
            // TODO: Support for more types
            if value.is_a::<JsString, _>(cx) {
                let value: Handle<JsString> = value.downcast(cx).or_throw(cx)?;
                let value: String = String::from_js_type(cx, value)?;
                let value = serde_json::Value::String(value);
                json.insert(key, value);
            } else if value.is_a::<JsNumber, _>(cx) {
                let value: Handle<JsNumber> = value.downcast(cx).or_throw(cx)?;
                let value: f64 = f64::from_js_type(cx, value)?;
                let value = serde_json::value::Number::from_f64(value)
                    .expect("Could not convert f64 to serde_json::Number");
                let value = serde_json::value::Value::Number(value);
                json.insert(key, value);
            } else if value.is_a::<JsBoolean, _>(cx) {
                let value: Handle<JsBoolean> = value.downcast(cx).or_throw(cx)?;
                let value: bool = bool::from_js_type(cx, value)?;
                let value = serde_json::Value::Bool(value);
                json.insert(key, value);
            } else {
                panic!("Unsupported type for json conversion");
            }
        }
        Ok(Self(serde_json::Value::Object(json)))
    }
}
