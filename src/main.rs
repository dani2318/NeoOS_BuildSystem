use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::exit;

mod directory_manager;
mod file_organizer;

struct subdir {
    sources: Vec<file_organizer::SourceFile>,
    linker_script: file_organizer::SourceFile,
}

fn read_subdir(
    subdir: &str,
    file_list: &HashMap<String, directory_manager::DirectoryFile>,
    proj_src_root_str: &str,
) -> Option<subdir> {
    let mut all_sources: Vec<file_organizer::SourceFile> = Vec::new();
    let mut linker_script: Option<file_organizer::SourceFile> = None;

    let target_path = format!("{}{}", proj_src_root_str, subdir);

    // Check if the target directory exists
    if !directory_exists(&target_path, file_list) {
        println!("Directory not found: {}", target_path);
        return None;
    }

    // Check if the target path has subdirectories
    if has_subdirectories(&target_path, file_list) {
        println!("Found subdirectories in: {}", target_path);
        // Recursively collect files from this directory and all subdirectories
        collect_files_recursive(
            &target_path,
            file_list,
            &mut all_sources,
            &mut linker_script,
        );
    } else {
        println!(
            "No subdirectories found in: {}, processing only main directory",
            target_path
        );
        // Only process the exact directory if no subdirectories exist
        collect_files_from_directory(
            &target_path,
            file_list,
            &mut all_sources,
            &mut linker_script,
        );
    }

    // If no linker script found, create a placeholder
    if linker_script.is_none() {
        linker_script = Some(file_organizer::SourceFile {
            path: "<not present>".to_string(),
            extension: "<not present>".to_string(),
            file_type: file_organizer::FileType::Unknown,
        });
    }

    Some(subdir {
        sources: all_sources,
        linker_script: linker_script.unwrap(),
    })
}

fn directory_exists(
    target_path: &str,
    file_list: &HashMap<String, directory_manager::DirectoryFile>,
) -> bool {
    file_list.contains_key(target_path)
}

fn has_subdirectories(
    target_path: &str,
    file_list: &HashMap<String, directory_manager::DirectoryFile>,
) -> bool {
    println!("  Checking for subdirectories in: {}", target_path);
    println!("  Available directories:");
    for folder_path in file_list.keys() {
        println!("    - {}", folder_path);
    }

    // Try both forward slash and backslash separators
    let forward_slash_pattern = format!("{}/", target_path);
    let backslash_pattern = format!("{}\\", target_path);

    for folder_path in file_list.keys() {
        // Check if any folder path starts with target_path/ or target_path\ (indicating a subdirectory)
        if folder_path.starts_with(&forward_slash_pattern)
            || folder_path.starts_with(&backslash_pattern)
        {
            println!("  Found subdirectory: {}", folder_path);
            return true;
        }
    }
    println!("  No subdirectories found for: {}", target_path);
    false
}

fn collect_files_from_directory(
    exact_path: &str,
    file_list: &HashMap<String, directory_manager::DirectoryFile>,
    all_sources: &mut Vec<file_organizer::SourceFile>,
    linker_script: &mut Option<file_organizer::SourceFile>,
) {
    if let Some(directory_file) = file_list.get(exact_path) {
        for source in &directory_file.files {
            all_sources.push(source.clone());

            // Check if this is a linker script and we haven't found one yet
            if linker_script.is_none() && source.file_type == file_organizer::FileType::Linker {
                *linker_script = Some(source.clone());
            }
        }
    }
}

fn collect_files_recursive(
    current_path: &str,
    file_list: &HashMap<String, directory_manager::DirectoryFile>,
    all_sources: &mut Vec<file_organizer::SourceFile>,
    linker_script: &mut Option<file_organizer::SourceFile>,
) {
    // Try both forward slash and backslash separators for subdirectory matching
    let forward_slash_pattern = format!("{}/", current_path);
    let backslash_pattern = format!("{}\\", current_path);

    for (folder_path, directory_file) in file_list {
        // Check if this folder is the current path or a subdirectory of it
        if folder_path == current_path
            || folder_path.starts_with(&forward_slash_pattern)
            || folder_path.starts_with(&backslash_pattern)
        {
            println!("  Processing directory: {}", folder_path);

            // Add all files from this directory
            for source in &directory_file.files {
                all_sources.push(source.clone());

                // Check if this is a linker script and we haven't found one yet
                if linker_script.is_none() && source.file_type == file_organizer::FileType::Linker {
                    *linker_script = Some(source.clone());
                }
            }
        }
    }
}

fn main() {
    let mut linker_script_stage1: Option<file_organizer::SourceFile> = None;
    let mut linker_script_stage2: Option<file_organizer::SourceFile> = None;
    let mut linker_script_kernel: Option<file_organizer::SourceFile> = None;
    let mut sources_libcore: Option<Vec<file_organizer::SourceFile>> = None;
    let mut sources_kernel: Option<Vec<file_organizer::SourceFile>> = None;
    let mut sources_stage2: Option<Vec<file_organizer::SourceFile>> = None;
    let mut sources_stage1: Option<Vec<file_organizer::SourceFile>> = None;

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
                        let mut file_list: HashMap<String, directory_manager::DirectoryFile> =
                            HashMap::new();

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

                        // Check and read subdirectories with better error handling
                        println!("\n=== Checking subdirectories ===");

                        let subdir_s1 =
                            read_subdir("\\boot\\stage1", &file_list, proj_src_root_str);
                        let subdir_s2 =
                            read_subdir("\\boot\\stage2", &file_list, proj_src_root_str);
                        let subdir_kernel = read_subdir("\\kernel", &file_list, proj_src_root_str);
                        let subdir_libcore =
                            read_subdir("\\libs\\core", &file_list, proj_src_root_str);

                        // Handle stage1
                        match subdir_s1 {
                            Some(s1) => {
                                sources_stage1 = Some(s1.sources);
                                linker_script_stage1 = Some(s1.linker_script);
                            }
                            None => {
                                eprintln!("Failed to read stage1 directory");
                                exit(1);
                            }
                        }

                        // Handle stage2
                        match subdir_s2 {
                            Some(s2) => {
                                sources_stage2 = Some(s2.sources);
                                linker_script_stage2 = Some(s2.linker_script);
                            }
                            None => {
                                eprintln!("Failed to read stage2 directory");
                                exit(1);
                            }
                        }

                        // Handle kernel
                        match subdir_kernel {
                            Some(kernel) => {
                                sources_kernel = Some(kernel.sources);
                                linker_script_kernel = Some(kernel.linker_script);
                            }
                            None => {
                                eprintln!("Failed to read kernel directory");
                                exit(1);
                            }
                        }

                        // Handle libcore
                        match subdir_libcore {
                            Some(libcore) => {
                                sources_libcore = Some(libcore.sources);
                            }
                            None => {
                                eprintln!("Failed to read libcore directory");
                                exit(1);
                            }
                        }

                        println!("\n=== Linker Script Information ===");
                        if let Some(ref unwrap_lnkscript1) = linker_script_stage1 {
                            println!("Linker script stage 1: {:?}", unwrap_lnkscript1.path);
                        } else {
                            eprintln!("Linker script for stage 1 not found!");
                            exit(1);
                        }

                        if let Some(ref unwrap_lnkscript2) = linker_script_stage2 {
                            println!("Linker script stage 2: {:?}", unwrap_lnkscript2.path);
                        } else {
                            eprintln!("Linker script for stage 2 not found!");
                            exit(1);
                        }

                        if let Some(ref unwrap_lnkkernel) = linker_script_kernel {
                            println!("Linker script kernel: {:?}", unwrap_lnkkernel.path);
                        } else {
                            eprintln!("Linker script for kernel not found!");
                            exit(1);
                        }

                        println!("\n=== Source File Counts ===");
                        println!(
                            "Found {} sources for stage1",
                            sources_stage1.as_ref().unwrap().len()
                        );
                        println!(
                            "Found {} sources for stage2",
                            sources_stage2.as_ref().unwrap().len()
                        );
                        println!(
                            "Found {} sources for kernel",
                            sources_kernel.as_ref().unwrap().len()
                        );
                        println!(
                            "Found {} sources for libcore",
                            sources_libcore.as_ref().unwrap().len()
                        );
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
