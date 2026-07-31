//! Typed flag/config values and their declared types.
//!
//! A key is either a boolean feature flag or a typed config value (int, number,
//! string, small JSON). [`ValueType`] disambiguates `int` from `number` (both are
//! JSON numbers) so a stored override round-trips to the right variant.

use cymbra_platform::error::{AppError, Result};
use serde_json::Value as Json;

/// The declared type of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Bool,
    Int,
    Number,
    String,
    Json,
}

impl ValueType {
    /// Stable wire/DB string.
    pub fn as_str(self) -> &'static str {
        match self {
            ValueType::Bool => "bool",
            ValueType::Int => "int",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Json => "json",
        }
    }

    /// Parse from the wire/DB string.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "bool" => ValueType::Bool,
            "int" => ValueType::Int,
            "number" => ValueType::Number,
            "string" => ValueType::String,
            "json" => ValueType::Json,
            _ => return None,
        })
    }
}

/// A concrete typed value. Exactly one variant, matching a key's [`ValueType`].
#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    Bool(bool),
    Int(i64),
    Number(f64),
    String(String),
    Json(Json),
}

impl FlagValue {
    /// The type of this value.
    pub fn value_type(&self) -> ValueType {
        match self {
            FlagValue::Bool(_) => ValueType::Bool,
            FlagValue::Int(_) => ValueType::Int,
            FlagValue::Number(_) => ValueType::Number,
            FlagValue::String(_) => ValueType::String,
            FlagValue::Json(_) => ValueType::Json,
        }
    }

    /// Boolean payload, if this is a `Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FlagValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Integer payload, if this is an `Int`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FlagValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Numeric payload as `f64` — an `Int` widens to `f64`, a `Number` is itself.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FlagValue::Int(i) => Some(*i as f64),
            FlagValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// String payload, if this is a `String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FlagValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// JSON payload, if this is a `Json`.
    pub fn as_json(&self) -> Option<&Json> {
        match self {
            FlagValue::Json(j) => Some(j),
            _ => None,
        }
    }

    /// Encode to a JSON value for storage. `int`/`number` both become JSON numbers
    /// (the column's `value_type` records which); `json` passes through.
    pub fn to_json(&self) -> Json {
        match self {
            FlagValue::Bool(b) => Json::Bool(*b),
            FlagValue::Int(i) => Json::Number((*i).into()),
            FlagValue::Number(n) => serde_json::Number::from_f64(*n)
                .map(Json::Number)
                .unwrap_or(Json::Null),
            FlagValue::String(s) => Json::String(s.clone()),
            FlagValue::Json(j) => j.clone(),
        }
    }

    /// Decode a stored JSON value back to the variant its declared `vt` requires.
    /// A mismatch (e.g. a string stored where a bool is declared) is an
    /// `InvalidArgument`, so a bad override never silently coerces.
    pub fn from_json(vt: ValueType, j: &Json) -> Result<Self> {
        let bad = || AppError::InvalidArgument(format!("value is not a valid {}", vt.as_str()));
        Ok(match vt {
            ValueType::Bool => FlagValue::Bool(j.as_bool().ok_or_else(bad)?),
            ValueType::Int => FlagValue::Int(j.as_i64().ok_or_else(bad)?),
            ValueType::Number => FlagValue::Number(j.as_f64().ok_or_else(bad)?),
            ValueType::String => FlagValue::String(j.as_str().ok_or_else(bad)?.to_string()),
            ValueType::Json => {
                if j.is_object() || j.is_array() {
                    FlagValue::Json(j.clone())
                } else {
                    return Err(bad());
                }
            }
        })
    }

    /// A short human string for the change audit.
    pub fn display(&self) -> String {
        match self {
            FlagValue::Bool(b) => b.to_string(),
            FlagValue::Int(i) => i.to_string(),
            FlagValue::Number(n) => n.to_string(),
            FlagValue::String(s) => s.clone(),
            FlagValue::Json(j) => j.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn value_type_round_trips() {
        for vt in [
            ValueType::Bool,
            ValueType::Int,
            ValueType::Number,
            ValueType::String,
            ValueType::Json,
        ] {
            assert_eq!(ValueType::parse(vt.as_str()), Some(vt));
        }
        assert_eq!(ValueType::parse("nope"), None);
    }

    #[test]
    fn accessors_are_type_specific() {
        assert_eq!(FlagValue::Bool(true).as_bool(), Some(true));
        assert_eq!(FlagValue::Bool(true).as_i64(), None);
        assert_eq!(FlagValue::Int(5).as_i64(), Some(5));
        assert_eq!(FlagValue::Int(5).as_f64(), Some(5.0));
        assert_eq!(FlagValue::Number(2.5).as_f64(), Some(2.5));
        assert_eq!(FlagValue::String("x".into()).as_str(), Some("x"));
        assert_eq!(
            FlagValue::Json(json!({"a":1})).as_json(),
            Some(&json!({"a":1}))
        );
    }

    #[test]
    fn json_round_trip_by_declared_type() {
        let cases = [
            (ValueType::Bool, FlagValue::Bool(true)),
            (ValueType::Int, FlagValue::Int(-7)),
            (ValueType::Number, FlagValue::Number(2.0)),
            (ValueType::String, FlagValue::String("hi".into())),
            (
                ValueType::Json,
                FlagValue::Json(json!({"bands": [1, 2, 3]})),
            ),
        ];
        for (vt, v) in cases {
            let decoded = FlagValue::from_json(vt, &v.to_json()).unwrap();
            assert_eq!(decoded, v);
            assert_eq!(v.value_type(), vt);
        }
    }

    #[test]
    fn from_json_rejects_type_mismatch() {
        assert!(FlagValue::from_json(ValueType::Bool, &json!("nope")).is_err());
        assert!(FlagValue::from_json(ValueType::Int, &json!(1.5)).is_err());
        assert!(FlagValue::from_json(ValueType::Json, &json!(3)).is_err());
        assert!(FlagValue::from_json(ValueType::Number, &json!("x")).is_err());
    }

    #[test]
    fn display_is_human() {
        assert_eq!(FlagValue::Bool(false).display(), "false");
        assert_eq!(FlagValue::Int(20).display(), "20");
        assert_eq!(FlagValue::String("s".into()).display(), "s");
    }
}
