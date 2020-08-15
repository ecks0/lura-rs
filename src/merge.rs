// merge functions for json and toml

use {
  serde_json::Value as JsonValue,
  toml::Value as TomlValue,
};

pub fn json(a: &mut JsonValue, b: &JsonValue) {
  // merge json value `b` on to value `a`

  match (a, b) {
    (&mut JsonValue::Object(ref mut a), &JsonValue::Object(ref b)) => {
      for (k, v) in b {
        json(a.entry(k.clone()).or_insert(JsonValue::Null), v);
      }
    },
    (a, b) => {
      *a = b.clone();
    },
  }
}

pub fn toml(a: &mut TomlValue, b: &TomlValue) {
  // merge toml value `b` on to value `a`

  match (a, b) {
    (&mut TomlValue::Table(ref mut a), &TomlValue::Table(ref b)) => {
      for (k, v) in b {
        toml(a.entry(k.clone()).or_insert(TomlValue::Integer(0)), v);
      }
    },
    (a, b) => {
      *a = b.clone();
    },
  }
}
