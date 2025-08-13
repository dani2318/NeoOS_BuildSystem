use std::io::BufReader;
use std::fs::{self, File};
use std::path::Path;
use crate::file_organizer;

pub fn read_config_file(config_path: &Path) -> Result<file_organizer::Config, Box<dyn std::error::Error>> {
    println!("Config file exists: {}", config_path.exists());

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);

    let config: file_organizer::Config = serde_json::from_reader(reader)?;

    Ok(config)
}
#[derive(Debug)]
pub struct DirectoryFile {
    pub files: Vec<file_organizer::SourceFile>,
}

pub fn read_directory(path: &str) -> Result<file_organizer::OrganizedFiles, Box<dyn std::error::Error>> {
    println!("Reading directory: {}", path);

    let mut source_files = Vec::new();

    // Recursively walk through directory
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                let file_type = match extension {
                    "cpp" | "cxx" | "cc" => file_organizer::FileType::CPlusPlus,
                    "c" => file_organizer::FileType::C,
                    "asm" | "s" => file_organizer::FileType::Assembly,
                    "h" | "hpp" => file_organizer::FileType::Header,
                    "ld" => file_organizer::FileType::Linker,
                    _ => file_organizer::FileType::Unknown,
                };

                // Only include source files we care about
                if !matches!(file_type, file_organizer::FileType::Unknown) {
                    let source_file = file_organizer::SourceFile {
                        path: path.to_string_lossy().to_string(),
                        extension: extension.to_string(),
                        file_type,
                    };

                    println!("Found source file: {:?}", source_file);
                    source_files.push(source_file);
                }
            }
        } else if path.is_dir() {
            // Recursively read subdirectories
            let subdir_path = path.to_string_lossy();
            let mut subdir_organized = read_directory(&subdir_path)?;
            source_files.append(&mut subdir_organized.all_files);
        }
    }

    // Create organized structure from the files
    Ok(file_organizer::OrganizedFiles::from_files(source_files))
}
