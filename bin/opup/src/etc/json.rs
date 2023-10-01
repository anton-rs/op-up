use eyre::Result;
use serde_json::{Map, Value};
use std::path::Path;

/// Read a JSON file and return a `serde_json::Value`.
pub fn read_json(file_path: &Path) -> Result<Value> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader)?;
    Ok(json_value)
}

/// Write a `serde_json::Value` to a JSON file.
pub fn write_json(file_path: &Path, json_value: &Value) -> Result<()> {
    let file = std::fs::File::create(file_path)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, json_value)?;
    Ok(())
}

/// Set a JSON property on a `serde_json::Value`.
pub fn set_json_property<P: Into<Value>>(json_value: &mut Value, key: &str, value: P) {
    if let Some(obj) = json_value.as_object_mut() {
        obj.insert(key.to_owned(), value.into());
    } else {
        let mut obj = Map::new();
        obj.insert(key.to_owned(), value.into());
        *json_value = Value::Object(obj);
    }
}
