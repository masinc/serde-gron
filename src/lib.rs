mod ser;

pub use ser::{
    to_colored_string, to_colored_writer, to_string, to_string_with, to_writer, to_writer_with,
    FormatType,
};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_null() {
        assert_eq!(to_string(&json!(null)).unwrap(), "json = null;\n");
    }

    #[test]
    fn test_string() {
        assert_eq!(to_string(&json!("abc")).unwrap(), "json = \"abc\";\n");
    }

    #[test]
    fn test_number() {
        assert_eq!(to_string(&json!(1)).unwrap(), "json = 1;\n");
        assert_eq!(to_string(&json!(-1)).unwrap(), "json = -1;\n");
    }

    #[test]
    fn test_array() {
        assert_eq!(to_string(&json!([])).unwrap(), "json = [];\n");

        assert_eq!(
            to_string(&json!([1, 2, 3])).unwrap(),
            "json = [];
json[0] = 1;
json[1] = 2;
json[2] = 3;
"
        );

        assert_eq!(
            to_string(&json!([1, [2, 3], 4])).unwrap(),
            "json = [];
json[0] = 1;
json[1] = [];
json[1][0] = 2;
json[1][1] = 3;
json[2] = 4;
"
        );
    }

    #[test]
    fn test_object() {
        assert_eq!(to_string(&json!({})).unwrap(), "json = {};\n");
        assert_eq!(
            to_string(&json!({ "a": 1, "b": 2, "c": 3 })).unwrap(),
            "json = {};
json.a = 1;
json.b = 2;
json.c = 3;
"
        );

        assert_eq!(
            to_string(&json!({ "a": 1, "b": { "c": 2, "d": 3}, "e": 4 })).unwrap(),
            "json = {};
json.a = 1;
json.b = {};
json.b.c = 2;
json.b.d = 3;
json.e = 4;
"
        );

        assert_eq!(
            to_string(&json!({ "a-b-c": 1 })).unwrap(),
            "json = {};
json[\"a-b-c\"] = 1;
"
        );
    }
}
