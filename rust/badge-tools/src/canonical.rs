use serde_json::Value;
use std::fmt;
use std::fmt::Write as _;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NonInteger,
    TooDeep,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NonInteger => f.write_str("only integer numbers are allowed in signed JSON"),
            Error::TooDeep => f.write_str("JSON nesting is too deep"),
        }
    }
}

impl std::error::Error for Error {}

pub fn canonical_bytes(value: &Value) -> Result<Vec<u8>, Error> {
    let mut out = String::new();
    write_value(value, &mut out, 0)?;
    Ok(out.into_bytes())
}

fn write_value(value: &Value, out: &mut String, depth: usize) -> Result<(), Error> {
    if depth > 128 {
        return Err(Error::TooDeep);
    }
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                let _ = write!(out, "{integer}");
            } else if let Some(integer) = number.as_u64() {
                let _ = write!(out, "{integer}");
            } else {
                return Err(Error::NonInteger);
            }
        }
        Value::String(string) => write_string(string, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(item, out, depth + 1)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                let child = map.get(*key).expect("key came from the map");
                write_value(child, out, depth + 1)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn write_string(string: &str, out: &mut String) {
    out.push('"');
    for ch in string.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn canonical(value: &Value) -> String {
        String::from_utf8(canonical_bytes(value).unwrap()).unwrap()
    }

    #[test]
    fn keys_are_sorted_and_whitespace_removed() {
        let a = json!({"b": 1, "a": {"y": 2, "x": [1, 2]}});
        let b = json!({"a": {"x": [1, 2], "y": 2}, "b": 1});
        assert_eq!(canonical(&a), canonical(&b));
        assert_eq!(canonical(&a), "{\"a\":{\"x\":[1,2],\"y\":2},\"b\":1}");
        assert!(!canonical(&a).contains(' '));
    }

    #[test]
    fn keys_sort_by_code_point() {
        assert_eq!(
            canonical(&json!({"ё": 1, "a": 2, "Я": 3, "é": 4})),
            "{\"a\":2,\"é\":4,\"Я\":3,\"ё\":1}"
        );
    }

    #[test]
    fn non_ascii_is_literal() {
        assert_eq!(canonical(&json!({"badge": "grapegram", "who": "Иван"})),
                   "{\"badge\":\"grapegram\",\"who\":\"Иван\"}");
    }

    #[test]
    fn control_characters_match_python_json_dumps() {
        assert_eq!(canonical(&json!({"s": "\u{0001}\n\t\u{001f}\"\\"})),
                   "{\"s\":\"\\u0001\\n\\t\\u001f\\\"\\\\\"}");
    }

    #[test]
    fn floats_are_rejected() {
        assert_eq!(canonical_bytes(&json!({"v": 1.5})), Err(Error::NonInteger));
        assert_eq!(canonical_bytes(&json!({"v": 1e2})), Err(Error::NonInteger));
        assert!(canonical_bytes(&json!({"v": 7303})).is_ok());
        assert!(canonical_bytes(&json!({"v": 18446744073709551615u64})).is_ok());
    }

    #[test]
    fn duplicate_keys_keep_last_value_like_python() {
        let value: Value = serde_json::from_str("{\"a\":1,\"a\":2}").unwrap();
        assert_eq!(canonical(&value), "{\"a\":2}");
    }
}
