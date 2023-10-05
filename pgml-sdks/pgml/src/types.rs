use anyhow::Context;
use itertools::Itertools;
use rust_bridge::alias_manual;
use sea_query::Iden;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

/// A wrapper around serde_json::Value
// #[derive(sqlx::Type, sqlx::FromRow, Debug)]
#[derive(alias_manual, sqlx::Type, Debug, Clone)]
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

pub(crate) trait TryToNumeric {
    fn try_to_u64(&self) -> anyhow::Result<u64>;
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

/// A wrapper around sqlx::types::PrimitiveDateTime
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
