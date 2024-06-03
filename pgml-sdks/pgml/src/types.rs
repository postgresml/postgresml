use anyhow::Context;
use futures::{stream::BoxStream, Stream, StreamExt};
use itertools::Itertools;
use sea_query::Iden;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "rust_bridge")]
use rust_bridge::alias_manual;

#[derive(Serialize, Deserialize)]
pub struct CustomU64Convertor(pub Value);

impl From<u64> for CustomU64Convertor {
    fn from(value: u64) -> Self {
        Self(json!(value))
    }
}

impl From<CustomU64Convertor> for u64 {
    fn from(value: CustomU64Convertor) -> Self {
        if value.0.is_f64() {
            value.0.as_f64().unwrap() as u64
        } else if value.0.is_i64() {
            value.0.as_i64().unwrap() as u64
        } else if value.0.is_u64() {
            value.0.as_u64().unwrap()
        } else {
            panic!("Cannot convert value into u64")
        }
    }
}

/// A wrapper around `serde_json::Value`
#[cfg_attr(feature = "rust_bridge", derive(alias_manual))]
#[derive(sqlx::Type, Debug, Clone, Deserialize, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct Json(pub serde_json::Value);

impl Default for Json {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

impl From<serde_json::Value> for Json {
    fn from(v: serde_json::Value) -> Self {
        Self(v)
    }
}

impl Deref for Json {
    type Target = serde_json::Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Json {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for Json {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::Value::serialize(&self.0, serializer)
    }
}

// This will cause some conflicting trait issue
// impl<T: Serialize> From<T> for Json {
//     fn from(v: T) -> Self {
//         Self(serde_json::to_value(v).unwrap())
//     }
// }

impl Json {
    pub fn from_serializable<T: Serialize>(v: T) -> Self {
        Self(serde_json::to_value(v).unwrap())
    }
}

pub(crate) trait TryToNumeric {
    fn try_to_u64(&self) -> anyhow::Result<u64>;

    #[allow(dead_code)]
    fn try_to_i64(&self) -> anyhow::Result<i64> {
        self.try_to_u64().map(|u| u as i64)
    }
}

impl TryToNumeric for serde_json::Value {
    fn try_to_u64(&self) -> anyhow::Result<u64> {
        match self {
            serde_json::Value::Number(n) => {
                if n.is_f64() {
                    Ok(n.as_f64().unwrap() as u64)
                } else if n.is_i64() {
                    Ok(n.as_i64().unwrap() as u64)
                } else {
                    n.as_u64().context("limit must be an integer")
                }
            }
            _ => Err(anyhow::anyhow!("Json value is not a number")),
        }
    }
}

/// A wrapper around `sqlx::types::PrimitiveDateTime`
#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
pub struct DateTime(pub sqlx::types::time::PrimitiveDateTime);

impl Serialize for DateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.assume_utc().unix_timestamp().serialize(serializer)
    }
}

#[derive(Clone)]
pub(crate) enum SIden<'a> {
    Str(&'a str),
    String(String),
}

impl Iden for SIden<'_> {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                SIden::Str(s) => s,
                SIden::String(s) => s.as_str(),
            }
        )
        .unwrap();
    }
}

pub(crate) trait IntoTableNameAndSchema {
    fn to_table_tuple<'b>(&self) -> (SIden<'b>, SIden<'b>);
}

impl IntoTableNameAndSchema for String {
    fn to_table_tuple<'b>(&self) -> (SIden<'b>, SIden<'b>) {
        self.split('.')
            .map(|s| SIden::String(s.to_string()))
            .collect_tuple()
            .expect("Malformed table name in IntoTableNameAndSchema")
    }
}

/// A wrapper around `BoxStream<'static, anyhow::Result<Json>>`
#[cfg_attr(feature = "rust_bridge", derive(alias_manual))]
pub struct GeneralJsonAsyncIterator(pub BoxStream<'static, anyhow::Result<Json>>);

impl Stream for GeneralJsonAsyncIterator {
    type Item = anyhow::Result<Json>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

/// A wrapper around `Box<dyn Iterator<Item = anyhow::Result<Json>> + Send>`
#[cfg_attr(feature = "rust_bridge", derive(alias_manual))]
pub struct GeneralJsonIterator(pub Box<dyn Iterator<Item = anyhow::Result<Json>> + Send>);

impl Iterator for GeneralJsonIterator {
    type Item = anyhow::Result<Json>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
