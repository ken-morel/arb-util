use std::{collections::BTreeMap, path::PathBuf};

use crate::utils::stringe;

#[derive(Debug, Clone)]
pub struct ArbFile {
    pub path: PathBuf,
}

impl ArbFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn read(&self) -> Result<BTreeMap<String, serde_json::Value>, String> {
        let content = stringe(
            format!("could not read arb file content at {0:?}", self.path).as_str(),
            std::fs::read_to_string(&self.path),
        )?;
        stringe(
            format!(
                "could not decode main arb file json: {0:?} of {1:?} ",
                content, self.path
            )
            .as_str(),
            serde_json::from_str(content.as_str()),
        )
    }
    pub fn write(&self, json: &BTreeMap<String, serde_json::Value>) -> Result<(), String> {
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
        let mut arb = self.read()?;
        arb.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        self.write(&arb)
    }
}
