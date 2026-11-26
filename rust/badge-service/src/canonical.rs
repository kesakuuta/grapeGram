use serde_json::Value;

use crate::error::{bad_request, ApiError};

struct DumpOptions {
    indent: Option<usize>,
    item_separator: &'static str,
    key_separator: &'static str,
    integers_only: bool,
}

fn canonical_options() -> DumpOptions {
    DumpOptions {
        indent: None,
        item_separator: ",",
        key_separator: ":",
        integers_only: true,
    }
}

fn line_options() -> DumpOptions {
    DumpOptions {
        indent: None,
        item_separator: ", ",
        key_separator: ": ",
        integers_only: false,
    }
}

fn pretty_options() -> DumpOptions {
    DumpOptions {
        indent: Some(2),
        item_separator: ",",
        key_separator: ": ",
        integers_only: false,
    }
}

pub fn canonical_bytes(value: &Value) -> Result<Vec<u8>, ApiError> {
    dump(value, &canonical_options())
}

pub fn json_line(value: &Value) -> Result<Vec<u8>, ApiError> {
    dump(value, &line_options())
}

pub fn pretty_bytes(value: &Value) -> Result<Vec<u8>, ApiError> {
    dump(value, &pretty_options())
}

fn dump(value: &Value, options: &DumpOptions) -> Result<Vec<u8>, ApiError> {
    let mut out = Vec::new();
    write_value(value, options, 0, &mut out)?;
    Ok(out)
}

fn newline(out: &mut Vec<u8>) {
    out.push(b'\n');
}

fn pad(out: &mut Vec<u8>, level: usize, options: &DumpOptions) {
    if let Some(width) = options.indent {
        for _ in 0..level * width {
            out.push(b' ');
        }
    }
}

fn write_value(
    value: &Value,
    options: &DumpOptions,
    level: usize,
    out: &mut Vec<u8>,
) -> Result<(), ApiError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.extend_from_slice(i.to_string().as_bytes());
            } else if let Some(u) = n.as_u64() {
                out.extend_from_slice(u.to_string().as_bytes());
            } else if options.integers_only {
                return Err(bad_request("only integers are allowed in signed documents"));
            } else {
                out.extend_from_slice(n.to_string().as_bytes());
            }
        }
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.extend_from_slice(b"[]");
                return Ok(());
            }
            out.push(b'[');
            if options.indent.is_some() {
                newline(out);
            }
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(options.item_separator.as_bytes());
                    if options.indent.is_some() {
                        newline(out);
                    }
                }
                pad(out, level + 1, options);
                write_value(item, options, level + 1, out)?;
            }
            if options.indent.is_some() {
                newline(out);
                pad(out, level, options);
            }
            out.push(b']');
        }
        Value::Object(map) => {
            if map.is_empty() {
                out.extend_from_slice(b"{}");
                return Ok(());
            }
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push(b'{');
            if options.indent.is_some() {
                newline(out);
            }
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(options.item_separator.as_bytes());
                    if options.indent.is_some() {
                        newline(out);
                    }
                }
                pad(out, level + 1, options);
                write_string(key, out);
                out.extend_from_slice(options.key_separator.as_bytes());
                if let Some(item) = map.get(*key) {
                    write_value(item, options, level + 1, out)?;
                }
            }
            if options.indent.is_some() {
                newline(out);
                pad(out, level, options);
            }
            out.push(b'}');
        }
    }
    Ok(())
}

fn write_string(text: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for ch in text.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{08}' => out.extend_from_slice(b"\\b"),
            '\u{0c}' => out.extend_from_slice(b"\\f"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => {
                let mut buffer = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    out.push(b'"');
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn matches_canonical_py() {
        let value = json!({
            "b": 2,
            "a": "привет",
            "nested": {"z": 1, "а": true, "esc": "a\n\"b\\c"},
            "list": [3, {"y": 2, "x": 1}]
        });
        let expected = concat!(
            "{\"a\":\"привет\",\"b\":2,",
            "\"list\":[3,{\"x\":1,\"y\":2}],",
            "\"nested\":{\"esc\":\"a\\n\\\"b\\\\c\",\"z\":1,\"а\":true}}"
        );
        assert_eq!(String::from_utf8(canonical_bytes(&value).unwrap()).unwrap(), expected);
    }

    #[test]
    fn canonical_rejects_non_integers() {
        assert!(canonical_bytes(&json!({"x": 1.5})).is_err());
        assert!(canonical_bytes(&json!({"x": 1})).is_ok());
    }

    #[test]
    fn json_line_matches_python_default() {
        let value = json!({"ok": true, "code": "AB12CD34", "expires_in": 300});
        assert_eq!(
            String::from_utf8(json_line(&value).unwrap()).unwrap(),
            "{\"code\": \"AB12CD34\", \"expires_in\": 300, \"ok\": true}"
        );
    }
}
