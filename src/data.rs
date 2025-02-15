use csv::Reader;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Mapping {
    pub source_name: String,
    pub target_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct MappingError {
    pub path: PathBuf,
    pub error: String,
}

impl fmt::Display for MappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in {:?}: {}", self.path, self.error)
    }
}

impl Error for MappingError {}

static MAPPINGS: OnceLock<HashMap<String, Mapping>> = OnceLock::new();


pub fn load_all_mappings(app_dir: &Path, user_dir: Option<&Path>) -> Result<(), Box<dyn Error>> {
    let mut all_mappings = HashMap::new();

    // First load app mappings
    info!("Loading app mappings from: {:?}", app_dir);
    load_directory_mappings(app_dir, &mut all_mappings)?;

    // Then load user mappings (will override any duplicates)
    if let Some(user_dir) = user_dir {
        info!("Loading user mappings from: {:?}", user_dir);
        load_directory_mappings(&user_dir, &mut all_mappings)?;
    }

    info!("Loaded {} total mappings", all_mappings.len());
    MAPPINGS.set(all_mappings).unwrap();
    Ok(())
}

// In data.rs, update where we create the Mapping struct:

fn load_directory_mappings(
    dir: &Path,
    mappings: &mut HashMap<String, Mapping>,
) -> Result<(), Box<dyn Error>> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            info!("Loading CSV file: {:?}", path);

            let mut reader = Reader::from_path(&path).map_err(|e| MappingError {
                path: path.clone(),
                error: format!("Failed to open CSV: {}", e),
            })?;

            let headers = reader
                .headers()
                .map_err(|e| MappingError {
                    path: path.clone(),
                    error: format!("Failed to read headers: {}", e),
                })?
                .clone();

            if headers.len() < 2 {
                return Err(Box::new(MappingError {
                    path: path.clone(),
                    error: "CSV must have at least 2 columns".to_string(),
                }));
            }

            let source_name = headers[0].trim().to_string();
            let target_name = headers[1].trim().to_string();

            info!(
                "Processing mappings from '{}' to '{}'",
                source_name, target_name
            );

            for (line_number, result) in reader.records().enumerate() {
                let record = result.map_err(|e| MappingError {
                    path: path.clone(),
                    error: format!("Error on line {}: {}", line_number + 2, e),
                })?;

                if record.len() < 2 {
                    return Err(Box::new(MappingError {
                        path: path.clone(),
                        error: format!("Line {} has fewer than 2 columns", line_number + 2),
                    }));
                }

                let source = record[0].trim();
                let target = record[1].trim();

                // Skip empty mappings
                if source.is_empty() || target.is_empty() {
                    info!("Skipping empty mapping at line {}", line_number + 2);
                    continue;
                }

                mappings.insert(
                    source.to_string(),
                    Mapping {
                        source_name: source_name.clone(),
                        target_name: target_name.clone(),
                        value: target.to_string(),
                    },
                );
            }
        }
    }

    Ok(())
}

pub fn find_target(source: &str) -> Option<&Mapping> {
    MAPPINGS.get().and_then(|mappings| mappings.get(source))
}
