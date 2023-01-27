use derive_more::From;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

pub struct ImageStorage {
    pub root: PathBuf,
}

impl ImageStorage {
    pub fn save(&self, file_bytes: &[u8]) -> Result<String, SaveImageError> {
        let file_name = Uuid::new_v4().to_string();
        let file_path = self.root.join(&file_name);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(file_bytes)?;
        Ok(file_name)
    }
}

#[derive(Debug, From)]
pub enum SaveImageError {
    WriteFileError(io::Error),
}
