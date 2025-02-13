// src/data.rs
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct Record {
    source: String,
    target: String,
}

static MAPPINGS: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn load_mappings(file_path: &Path, source_column: &str, target_column: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut mappings = HashMap::new();

    // Remap headers
    let headers = reader.headers()?.clone();
    let source_idx = headers.iter().position(|h| h == source_column)
        .ok_or("Source column not found")?;
    let target_idx = headers.iter().position(|h| h == target_column)
        .ok_or("Target column not found")?;

    for result in reader.records() {
        let record = result?;
        if let (Some(source), Some(target)) = (record.get(source_idx), record.get(target_idx)) {
            mappings.insert(source.to_string(), target.to_string());
        }
    }

    MAPPINGS.set(mappings).unwrap();
    Ok(())
}

pub fn find_target(source: &str) -> Option<String> {
    MAPPINGS.get().and_then(|mappings| mappings.get(source).cloned())
}
