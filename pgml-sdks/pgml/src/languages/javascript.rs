use neon::prelude::*;
use rust_bridge::javascript::{FromJsType, IntoJsResult};

use crate::{
    pipeline::PipelineSyncData,
    types::{DateTime, Json},
};

////////////////////////////////////////////////////////////////////////////////
// Rust to JS //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl IntoJsResult for DateTime {
    type Output = neon::types::JsDate;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        let date = neon::types::JsDate::new(cx, self.0.assume_utc().unix_timestamp() as f64 * 1000.0)
            .expect("Error converting to JS Date");
        Ok(date)
    }
}

impl IntoJsResult for Json {
    type Output = JsValue;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        match self.0 {
            serde_json::Value::Bool(x) => Ok(JsBoolean::new(cx, x).upcast()),
            serde_json::Value::Number(x) => Ok(JsNumber::new(
                cx,
                x.as_f64()
                    .expect("Error converting to f64 in impl IntoJsResult for Json"),
            )
            .upcast()),
            serde_json::Value::String(x) => Ok(JsString::new(cx, &x).upcast()),
            serde_json::Value::Array(x) => {
                let js_array = JsArray::new(cx, x.len() as u32);
                for (i, v) in x.into_iter().enumerate() {
                    let js_value = Json::into_js_result(Self(v), cx)?;
                    js_array.set(cx, i as u32, js_value)?;
                }
                Ok(js_array.upcast())
            }
            serde_json::Value::Object(x) => {
                let js_object = JsObject::new(cx);
                for (k, v) in x.into_iter() {
                    let js_key = cx.string(k);
                    let js_value = Json::into_js_result(Self(v), cx)?;
                    js_object.set(cx, js_key, js_value)?;
                }
                Ok(js_object.upcast())
            }
            serde_json::Value::Null => Ok(cx.null().upcast()),
        }
    }
}

impl IntoJsResult for PipelineSyncData {
    type Output = JsValue;
    fn into_js_result<'a, 'b, 'c: 'b, C: Context<'c>>(
        self,
        cx: &mut C,
    ) -> JsResult<'b, Self::Output> {
        Json::from(self).into_js_result(cx)
    }
}

////////////////////////////////////////////////////////////////////////////////
// JS To Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl FromJsType for Json {
    type From = JsValue;
    fn from_js_type<'a, C: Context<'a>>(cx: &mut C, arg: Handle<Self::From>) -> NeonResult<Self> {
        if arg.is_a::<JsArray, _>(cx) {
            let value: Handle<JsArray> = arg.downcast(cx).or_throw(cx)?;
            let mut json = Vec::new();
            for item in value.to_vec(cx)? {
                let item = Json::from_js_type(cx, item)?;
                json.push(item.0);
            }
            Ok(Self(serde_json::Value::Array(json)))
        } else if arg.is_a::<JsBoolean, _>(cx) {
            let value: Handle<JsBoolean> = arg.downcast(cx).or_throw(cx)?;
            let value = bool::from_js_type(cx, value)?;
            let value = serde_json::Value::Bool(value);
            Ok(Self(value))
        } else if arg.is_a::<JsString, _>(cx) {
            let value: Handle<JsString> = arg.downcast(cx).or_throw(cx)?;
            let value = String::from_js_type(cx, value)?;
            let value = serde_json::Value::String(value);
            Ok(Self(value))
        } else if arg.is_a::<JsNumber, _>(cx) {
            let value: Handle<JsNumber> = arg.downcast(cx).or_throw(cx)?;
            let value = f64::from_js_type(cx, value)?;
            let value = serde_json::value::Number::from_f64(value)
                .expect("Could not convert f64 to serde_json::Number");
            let value = serde_json::value::Value::Number(value);
            Ok(Self(value))
        } else if arg.is_a::<JsObject, _>(cx) {
            let value: Handle<JsObject> = arg.downcast(cx).or_throw(cx)?;
            let mut json = serde_json::Map::new();
            let keys = value.get_own_property_names(cx)?.to_vec(cx)?;
            for key in keys {
                let key: Handle<JsString> = key.downcast(cx).or_throw(cx)?;
                let key: String = String::from_js_type(cx, key)?;
                let json_value: Handle<JsValue> = value.get(cx, key.as_str())?;
                let json_value = Json::from_js_type(cx, json_value)?;
                json.insert(key, json_value.0);
            }
            Ok(Self(serde_json::Value::Object(json)))
        } else if arg.is_a::<JsNull, _>(cx) {
            Ok(Self(serde_json::Value::Null))
        } else {
            panic!("Unsupported type for Json conversion");
        }
    }
}
