use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CppLanguage {
    pub cxx: String,
    pub ld: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CLanguage {
    pub cc: String,
    pub ld: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AsmLanguage {
    pub assembler: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Languages {
    pub cpp: CppLanguage,
    pub c: CLanguage,
    pub asm: AsmLanguage,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StageFiles {
    pub asm: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CppFlags {
    pub stage1: StageFiles,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PathSpecificFlags {
    pub cpp: CppFlags,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(rename = "ProjectSrcRoot")]
    pub project_src_root: String,
    #[serde(rename = "Languages")]
    pub languages: Languages,
    #[serde(rename = "PathSpecificFlags")]
    pub path_specific_flags: PathSpecificFlags,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub extension: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileType {
    CPlusPlus,
    C,
    Assembly,
    Header,
    Linker,
    Unknown,
}

// New pub structure to organize files
#[derive(Debug)]
pub struct OrganizedFiles {
    // folder_path -> file_type -> Vec<SourceFile>
    pub by_folder_and_type: HashMap<String, HashMap<FileType, Vec<SourceFile>>>,
    // file_type -> Vec<SourceFile>
    pub by_type: HashMap<FileType, Vec<SourceFile>>,
    // folder_path -> Vec<SourceFile>
    pub by_folder: HashMap<String, Vec<SourceFile>>,
    // Original flat list
    pub all_files: Vec<SourceFile>,
}

impl OrganizedFiles {
    pub fn from_files(files: Vec<SourceFile>) -> Self {
        let mut organized = OrganizedFiles {
            by_folder_and_type: HashMap::new(),
            by_type: HashMap::new(),
            by_folder: HashMap::new(),
            all_files: files.clone(),
        };

        for file in files {
            let folder_path = std::path::Path::new(&file.path)
                .parent()
                .unwrap_or(std::path::Path::new(""))
                .to_string_lossy()
                .to_string();

            // Organize by folder and type
            organized
                .by_folder_and_type
                .entry(folder_path.clone())
                .or_insert_with(HashMap::new)
                .entry(file.file_type.clone())
                .or_insert_with(Vec::new)
                .push(file.clone());

            // Organize by type
            organized
                .by_type
                .entry(file.file_type.clone())
                .or_insert_with(Vec::new)
                .push(file.clone());

            // Organize by folder
            organized
                .by_folder
                .entry(folder_path)
                .or_insert_with(Vec::new)
                .push(file);
        }

        organized
    }

    // Access methods
    pub fn get_files_by_type(&self, file_type: &FileType) -> Option<&Vec<SourceFile>> {
        self.by_type.get(file_type)
    }

    pub fn get_files_by_folder(&self, folder_path: &str) -> Option<&Vec<SourceFile>> {
        self.by_folder.get(folder_path)
    }

    pub fn get_files_by_folder_and_type(
        &self,
        folder_path: &str,
        file_type: &FileType,
    ) -> Option<&Vec<SourceFile>> {
        self.by_folder_and_type.get(folder_path)?.get(file_type)
    }

    pub fn get_all_files(&self) -> &Vec<SourceFile> {
        &self.all_files
    }
}
