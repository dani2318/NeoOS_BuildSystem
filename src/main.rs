use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::exit;

mod file_organizer;
mod directory_manager;

fn main() {
    let args: Vec<String> = env::args().collect();
    let configuration_file_path: &str = if args.len() > 1 && args[1].eq("--config-file") {
        &args[2]
    } else {
        println!("USAGE: --config-file [filepath]");
        exit(-1);
    };

    let config_file_path = Path::new(configuration_file_path);
    if config_file_path.exists() {
        match directory_manager::read_config_file(config_file_path) {
            Ok(config) => {
                println!("ProjectSrcRoot: {}", config.project_src_root);
                println!("C++ Compiler: {}", config.languages.cpp.cxx);
                println!("C Compiler: {}", config.languages.c.cc);
                println!("Assembler: {}", config.languages.asm.assembler);
                println!("Full config: {:#?}", config);
                println!("Looking for directory: {}", config.project_src_root);
                println!(
                    "Full path: {:?}",
                    std::path::Path::new(&config.project_src_root)
                        .canonicalize()
                        .unwrap()
                );

                let proj_src_root = std::path::Path::new(&config.project_src_root)
                    .canonicalize()
                    .unwrap();
                let proj_src_root_str = proj_src_root
                    .to_str()
                    .unwrap()
                    .strip_prefix(r"\\?\")
                    .unwrap_or_else(|| proj_src_root.to_str().unwrap());
                match directory_manager::read_directory(&proj_src_root_str) {
                    Ok(organized_files) => {
                        let mut file_list: HashMap<String, directory_manager::DirectoryFile> = HashMap::new();

                        for (folder, files) in &organized_files.by_folder {
                            let mut folder_sources_list: directory_manager::DirectoryFile =
                                directory_manager::DirectoryFile { files: Vec::new() };
                            let mut i = 0;
                            for file in files {
                                folder_sources_list.files.insert(i, file.clone().to_owned());
                                i += 1;
                            }
                            file_list.insert(folder.to_owned(), folder_sources_list);
                        }

                        println!("\nSubdivided in {} keys:", file_list.len());
                        println!("\nKeys:");
                        for key in file_list.keys() {
                            let relative = Path::new(key)
                                .strip_prefix(&proj_src_root_str)
                                .unwrap_or_else(|_| Path::new(key)); // fallback to full path if no match

                            println!("  -   {}", relative.to_str().unwrap())
                        }
                        println!(
                            "Seaching for Stage1 at {}",
                            format!("{}", &format!("{:?}\\boot\\stage1", proj_src_root_str))
                        );

                        let mut linker_script_stage1: Option<file_organizer::SourceFile> = None;
                        let mut linker_script_stage2: Option<file_organizer::SourceFile> = None;
                        let mut sources_stage2: Option<Vec<file_organizer::SourceFile>> = None;
                        let mut sources_stage1: Option<Vec<file_organizer::SourceFile>> = None;

                        for (folder, files) in file_list {
                            if folder.eq(&format!("{}\\boot\\stage1", proj_src_root_str)) {
                                let mut sources_internal: Vec<file_organizer::SourceFile> = Vec::new();
                                let i = 0;
                                for source in files.files {
                                    sources_internal.insert(i, source);
                                }
                                sources_stage1 = Some(sources_internal.clone());
                                for source in sources_internal {
                                    if source.file_type == file_organizer::FileType::Linker {
                                        linker_script_stage1 = Some(source.clone());
                                    }
                                }
                            } else if folder.eq(&format!("{}\\boot\\stage2", proj_src_root_str)) {
                                println!(
                                    "Seaching for Stage2 at {}",
                                    format!(
                                        "{}",
                                        &format!("{:?}\\boot\\stage2", proj_src_root_str)
                                    )
                                );
                                let mut sources_internal: Vec<file_organizer::SourceFile> = Vec::new();
                                let i = 0;
                                for source in files.files {
                                    sources_internal.insert(i, source);
                                }

                                sources_stage2 = Some(sources_internal.clone());

                                for source in sources_internal {
                                    if source.file_type == file_organizer::FileType::Linker {
                                        linker_script_stage2 = Some(source.clone());
                                    }
                                }
                            }
                        }

                        if let Some(unwrap_lnkscript1) = linker_script_stage1 {
                            println!("Linker script stage 1: {:?}", unwrap_lnkscript1.path);
                        } else {
                            eprintln!("Linker script for stage 1 not found!");
                            exit(1);
                        }

                        if let Some(unwrap_lnkscript2) = linker_script_stage2 {
                            println!("Linker script stage 2: {:?}", unwrap_lnkscript2.path);
                        } else {
                            eprintln!("Linker script for stage 2 not found!");
                            exit(1);
                        }

                        println!("Found {} sources for stage1", sources_stage1.unwrap().len());
                        println!("Found {} sources for stage2", sources_stage2.unwrap().len());
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
