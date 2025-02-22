use std::env::home_dir;
use std::path::PathBuf;

pub fn get_user_config_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".gofer2"))
}
