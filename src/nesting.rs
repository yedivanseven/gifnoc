use std::collections::HashMap;
use serde_json::{Map, Value};

pub fn flatten(value: Value) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    if let Value::Object(map) = value {
        flatten_object(map, String::new(), &mut result);
    }
    result
}

fn flatten_object(map: Map<String, Value>, prefix: String, result: &mut HashMap<String, Value>) {
    for (key, value) in map {
        let path = if prefix.is_empty() {
            key
        } else {
            format!("{}.{}", prefix, key)
        };
        match value {
            Value::Object(inner) => flatten_object(inner, path, result),
            leaf => {
                result.insert(path, leaf);
            }
        }
    }
}

pub fn nest(flat: HashMap<String, Value>) -> Value {
    let mut root = Map::new();
    for (path, value) in flat {
        insert_nested(&mut root, &path, value);
    }
    Value::Object(root)
}

fn insert_nested(map: &mut Map<String, Value>, path: &str, value: Value) {
    match path.split_once('.') {
        None => {
            map.insert(path.to_string(), value);
        }
        Some((head, rest)) => {
            let entry = map
                .entry(head.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if !matches!(entry, Value::Object(_)) {
                *entry = Value::Object(Map::new());
            }
            if let Value::Object(inner) = entry {
                insert_nested(inner, rest, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flatten_empty_object() {
        assert!(flatten(json!({})).is_empty());
    }

    #[test]
    fn flatten_non_object_returns_empty() {
        assert!(flatten(json!("string")).is_empty());
        assert!(flatten(json!(42)).is_empty());
        assert!(flatten(json!(null)).is_empty());
    }

    #[test]
    fn flatten_flat_object() {
        let result = flatten(json!({"a": 1, "b": "two"}));
        assert_eq!(result["a"], json!(1));
        assert_eq!(result["b"], json!("two"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn flatten_nested_object() {
        let result = flatten(json!({"a": {"b": 1}}));
        assert_eq!(result["a.b"], json!(1));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn flatten_deeply_nested() {
        let result = flatten(json!({"a": {"b": {"c": true}}}));
        assert_eq!(result["a.b.c"], json!(true));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn flatten_mixed() {
        let result = flatten(json!({"x": 1, "y": {"z": 2}}));
        assert_eq!(result["x"], json!(1));
        assert_eq!(result["y.z"], json!(2));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn nest_empty_map() {
        assert_eq!(nest(HashMap::new()), json!({}));
    }

    #[test]
    fn nest_flat_keys() {
        let map = HashMap::from([
            ("a".to_string(), json!(1)),
            ("b".to_string(), json!("two")),
        ]);
        let result = nest(map);
        assert_eq!(result["a"], json!(1));
        assert_eq!(result["b"], json!("two"));
    }

    #[test]
    fn nest_dotted_key() {
        let map = HashMap::from([("a.b".to_string(), json!(1))]);
        let result = nest(map);
        assert_eq!(result["a"]["b"], json!(1));
    }

    #[test]
    fn nest_deeply_dotted_key() {
        let map = HashMap::from([("a.b.c".to_string(), json!(true))]);
        let result = nest(map);
        assert_eq!(result["a"]["b"]["c"], json!(true));
    }

    #[test]
    fn nest_shared_prefix() {
        let map = HashMap::from([
            ("a.b".to_string(), json!(1)),
            ("a.c".to_string(), json!(2)),
        ]);
        let result = nest(map);
        assert_eq!(result["a"]["b"], json!(1));
        assert_eq!(result["a"]["c"], json!(2));
    }

    #[test]
    fn roundtrip() {
        let original = json!({"server": {"host": "localhost", "port": 8080}, "debug": false});
        assert_eq!(nest(flatten(original.clone())), original);
    }
}
