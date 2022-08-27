mod ser;

use serde::{ser::SerializeMap, Deserialize, Serialize};

#[derive(Debug)]
pub struct Gron {
    pub root_name: String,
    pub value: serde_json::Value,
}

impl From<Gron> for serde_json::Value {
    fn from(val: Gron) -> Self {
        val.value
    }
}

impl From<serde_json::Value> for Gron {
    fn from(val: serde_json::Value) -> Self {
        Gron::new(val)
    }
}

impl Gron {
    pub fn new(value: serde_json::Value) -> Self {
        Self {
            root_name: "json".into(),
            value,
        }
    }

    pub fn new_with_root_name(value: serde_json::Value, root_name: impl Into<String>) -> Self {
        Self {
            root_name: root_name.into(),
            value,
        }
    }

    pub fn to_string(&self) -> Result<String, ser::Error> {
        ser::to_string_with(&self.value, &self.root_name, ser::FormatType::Regular)
    }

    pub fn to_colored_string(&self) -> Result<String, ser::Error> {
        ser::to_string_with(&self.value, &self.root_name, ser::FormatType::Color)
    }
}

impl Serialize for Gron {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Gron {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(serde_json::Value::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_null() {
        let gron = Gron::new(json!(null));
        assert_eq!(gron.to_string().unwrap(), "json = null;\n");
    }

    #[test]
    fn test_string() {
        let gron = Gron::new(json!("abc"));
        assert_eq!(gron.to_string().unwrap(), "json = \"abc\";\n");
    }

    #[test]
    fn test_number() {
        let gron = Gron::new(json!(1));
        assert_eq!(gron.to_string().unwrap(), "json = 1;\n");

        let gron = Gron::new(json!(-1));
        assert_eq!(gron.to_string().unwrap(), "json = -1;\n");
    }

    #[test]
    fn test_array() {
        let gron = Gron::new(json!([]));
        assert_eq!(gron.to_string().unwrap(), "json = [];\n");

        let gron = Gron::new(json!([1, 2, 3]));

        assert_eq!(
            gron.to_string().unwrap(),
            "json = [];
json[0] = 1;
json[1] = 2;
json[2] = 3;
"
        );

        let gron = Gron::new(json!([1, [2, 3], 4]));

        assert_eq!(
            gron.to_string().unwrap(),
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
        let gron = Gron::new(json!({}));
        assert_eq!(gron.to_string().unwrap(), "json = {};\n");

        let gron = Gron::new(json!({ "a": 1, "b": 2, "c": 3 }));
        assert_eq!(
            gron.to_string().unwrap(),
            "json = {};
json.a = 1;
json.b = 2;
json.c = 3;
"
        );

        let gron = Gron::new(json!({ "a": 1, "b": { "c": 2, "d": 3}, "e": 4 }));
        assert_eq!(
            gron.to_string().unwrap(),
            "json = {};
json.a = 1;
json.b = {};
json.b.c = 2;
json.b.d = 3;
json.e = 4;
"
        );

        let gron = Gron::new(json!({ "a-b-c": 1 }));

        assert_eq!(
            gron.to_string().unwrap(),
            "json = {};
json[\"a-b-c\"] = 1;
"
        );
    }
}
