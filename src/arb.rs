use std::path::PathBuf;

use crate::utils::stringe;

pub struct ArbFile {
    pub path: PathBuf,
}

impl ArbFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn read(&self) -> Result<serde_json::Value, String> {
        let content = stringe(
            "could not main arb file content",
            std::fs::read_to_string(&self.path),
        )?;
        stringe(
            "could not decode main arb file yaml",
            serde_json::from_str(content.as_str()),
        )
    }
    pub fn write(&self, json: &serde_json::Value) -> Result<(), String> {
        let new_data = stringe(
            "could not serialize modified arb file back to json ",
            serde_json::to_string_pretty(json),
        )?;
        if let Err(e) = stringe(
            "could not write json content to arb file",
            std::fs::write(&self.path, new_data),
        ) {
            Err(e)
        } else {
            Ok(())
        }
    }
    pub fn add_key(&self, key: &str, value: &str) -> Result<(), String> {
        let mut json: serde_json::Value = self.read()?;
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                key.to_string(),
                serde_json::Value::String(value.to_string()),
            );
            self.write(&json)
        } else {
            Err(String::from(
                "Invalid arb file, arb file should contain json object",
            ))
        }
    }
}
