// src/data.rs
use csv::Reader;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Mapping {
    pub source_name: String,
    pub target_name: String,
    pub value: String,
}

static MAPPINGS: OnceLock<HashMap<String, Mapping>> = OnceLock::new();

pub fn load_all_mappings(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut all_mappings = HashMap::new();

    // Read all CSV files in the directory
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            info!("Loading CSV file: {:?}", path);

            let mut reader = Reader::from_path(&path)?;
            let headers = reader.headers()?.clone();

            // Use first two columns as source and target
            if headers.len() < 2 {
                info!("Skipping {:?} - needs at least 2 columns", path);
                continue;
            }

            let source_name = headers[0].to_string();
            let target_name = headers[1].to_string();

            info!(
                "Processing mappings from '{}' to '{}'",
                source_name, target_name
            );

            for result in reader.records() {
                let record = result?;
                if record.len() < 2 {
                    continue;
                }

                let source = record[0].trim().to_string();
                let target = record[1].trim().to_string();

                all_mappings.insert(
                    source.clone(),
                    Mapping {
                        source_name: source_name.clone(),
                        target_name: target_name.clone(),
                        value: target,
                    },
                );
            }
        }
    }

    info!("Loaded {} total mappings", all_mappings.len());
    MAPPINGS.set(all_mappings).unwrap();
    Ok(())
}

pub fn find_target(source: &str) -> Option<&Mapping> {
    MAPPINGS.get().and_then(|mappings| mappings.get(source))
}
