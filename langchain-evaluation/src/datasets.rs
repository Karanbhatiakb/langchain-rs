//! Evaluation dataset types and loaders.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetExample {
    pub input: Value,
    pub output: Value,
    pub metadata: HashMap<String, Value>,
}

impl DatasetExample {
    pub fn new(input: Value, output: Value) -> Self {
        Self {
            input,
            output,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub description: Option<String>,
    pub examples: Vec<DatasetExample>,
}

impl Dataset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            examples: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_example(&mut self, example: DatasetExample) {
        self.examples.push(example);
    }

    pub fn len(&self) -> usize {
        self.examples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    pub fn load_csv(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error + Send>> {
        let mut reader = csv::Reader::from_path(path.as_ref())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        let mut examples = Vec::new();

        for result in reader.records() {
            let record = result
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            let input_str = record.get(0).unwrap_or("");
            let output_str = record.get(1).unwrap_or("");

            examples.push(DatasetExample {
                input: Value::String(input_str.to_string()),
                output: Value::String(output_str.to_string()),
                metadata: HashMap::new(),
            });
        }

        let name = path
            .as_ref()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "dataset".to_string());

        Ok(Self {
            name,
            description: None,
            examples,
        })
    }

    pub fn load_jsonl(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error + Send>> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        let mut examples = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let example: DatasetExample = serde_json::from_str(line)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            examples.push(example);
        }

        let name = path
            .as_ref()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "dataset".to_string());

        Ok(Self {
            name,
            description: None,
            examples,
        })
    }

    pub fn from_list(examples: Vec<DatasetExample>) -> Self {
        Self {
            name: "custom".to_string(),
            description: None,
            examples,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &DatasetExample> {
        self.examples.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = DatasetExample> {
        self.examples.into_iter()
    }
}
