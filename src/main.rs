use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CppLanguage{
    cxx: String,
    ld: String
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CLanguage{
    cc: String,
    ld: String
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AsmLanguage{
    assembler: String
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Languages{
    cpp: CppLanguage,
    c: CLanguage,
    asm: AsmLanguage
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StageFiles{
    asm: String
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CppFlags{
    stage1: StageFiles
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PathSpecificFlags{
    cpp: CppFlags
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    #[serde(rename= "ProjectSrcRoot")]
    project_src_root: String,
    #[serde(rename= "Languages")]
    languages: Languages,
    #[serde(rename= "PathSpecificFlags")]
    path_specific_flags: PathSpecificFlags,
}

#[derive(Debug)]
struct SourceFile {
    path: String,
    extension: String,
    file_type: FileType,
}

#[derive(Debug)]
enum FileType {
    CPlusPlus,
    C,
    Assembly,
    Header,
    Unknown,
}


fn read_config_file(config_path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    println!("Config file exists: {}", config_path.exists());
    
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    
    let config: Config = serde_json::from_reader(reader)?;
    
    Ok(config)
}

fn read_directory(path: &str) -> Result<Vec<SourceFile>, Box<dyn std::error::Error>> {
    println!("Reading directory: {}", path);
    
    let mut source_files = Vec::new();
    
    // Recursively walk through directory
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                let file_type = match extension {
                    "cpp" | "cxx" | "cc" => FileType::CPlusPlus,
                    "c" => FileType::C,
                    "asm" | "s" => FileType::Assembly,
                    "h" | "hpp" => FileType::Header,
                    _ => FileType::Unknown,
                };
                
                // Only include source files we care about
                if !matches!(file_type, FileType::Unknown) {
                    let source_file = SourceFile {
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
            let mut subdir_files = read_directory(&subdir_path)?;
            source_files.append(&mut subdir_files);
        }
    }
    
    Ok(source_files)
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let configuration_file_path:&str = if args.len() > 1 && args[1].eq("--config-file"){
        &args[2]
    }else {
        println!("USAGE: --config-file [filepath]");
        exit(-1);
    };

    let config_file_path = Path::new(configuration_file_path);
    if config_file_path.exists() {
        match read_config_file(config_file_path){
            Ok(config) => {
                println!("ProjectSrcRoot: {}", config.project_src_root);
                println!("C++ Compiler: {}", config.languages.cpp.cxx);
                println!("C Compiler: {}", config.languages.c.cc);
                println!("Assembler: {}", config.languages.asm.assembler);
                println!("Full config: {:#?}", config);
                println!("Looking for directory: {}", config.project_src_root);
                println!("Full path: {:?}", std::path::Path::new(&config.project_src_root).canonicalize().unwrap());

                let proj_src_root = std::path::Path::new(&config.project_src_root).canonicalize().unwrap();

                match read_directory(&proj_src_root.to_str().unwrap()) {
                    Ok(files) => {
                        println!("\nFound {} source files:", files.len());
                        for file in files {
                            println!("  Path: {}", file.path);
                            println!("    Type: {:?}", file.file_type);
                            println!("    Extension: {}", file.extension);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading source directory: {}", e);
                        exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error Reading config file: {}", e);
                exit(1);
            }
        }
    } else {
        eprintln!("Config file does not exist: {}", configuration_file_path);
        exit(1);
    }

}
