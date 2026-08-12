pub mod config;
pub mod module;

use crate::project::{
    config::{CrispToml, ProjectType},
    module::ModuleTree,
};
use anyhow::Error;
use colored::Colorize;
use indicatif::MultiProgress;
use log::{error, info, trace, warn};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
enum PathItem {
    Dir(PathBuf),
    File(PathBuf),
}
impl PathItem {
    fn exists(&self) -> bool {
        match self {
            PathItem::Dir(path) | PathItem::File(path) => path.exists(),
        }
    }
    fn inner(&self) -> PathBuf {
        match self {
            PathItem::Dir(path) | PathItem::File(path) => path.to_path_buf(),
        }
    }
}

pub struct ProjectStructure {
    main_file: PathBuf,
    config: CrispToml,
    module_tree: ModuleTree,
}
impl ProjectStructure {
    fn paths(path: PathBuf) -> [(PathItem, String); 4] {
        [
            (
                PathItem::File(path.join("Crisp.toml")),
                "Crisp.toml".to_owned(),
            ),
            (PathItem::Dir(path.join("src")), "src".to_owned()),
            (
                PathItem::File(path.join("src/main.crisp")),
                "src/main.crisp".to_owned(),
            ),
            (
                PathItem::File(path.join("src/lib.crisp")),
                "src/lib.crisp".to_owned(),
            ),
        ]
    }
}

pub fn new_project(path: &Path, progress: MultiProgress) -> Result<ProjectStructure, Error> {
    if let Ok(exists) = fs::exists(path) {
        if !exists {
            fs::create_dir(path).expect("Could not create project root directory");
        }
    } else {
        panic!("Path may or may not exist: {:?}", path)
    }
    init_project(path, progress)
}

pub fn init_project(path: &Path, progress: MultiProgress) -> Result<ProjectStructure, Error> {
    let paths = ProjectStructure::paths(path.to_path_buf());
    for path in paths {
        if path.0.exists() {
            warn!("{} {}", path.0.inner().display(), "exists".yellow())
        }
        match path.0 {
            PathItem::Dir(path) => fs::create_dir(path).expect("Could not create directory"),
            PathItem::File(path) => {
                let mut contents = "";
                if path.file_name() == Some("main.crisp".as_ref()) {
                    contents = "(defn main[] -> void (\n    (return \"hello world!\")\n))";
                }
                fs::write(path, contents).expect("Could not create file");
            }
        }
    }
    check_project(path, progress)
}

pub fn check_project(path: &Path, progress: MultiProgress) -> Result<ProjectStructure, Error> {
    let paths = ProjectStructure::paths(path.to_path_buf());
    info!("{}", "Reading Crisp.toml".bold());
    let toml: CrispToml = match toml::from_str(&fs::read_to_string(paths[0].0.inner())?) {
        Err(error) => {
            error!("Invalid Crisp.toml.");
            return Err(error.into());
        }
        Ok(toml) => {
            info!("{} {}", "Crisp.toml", "parsed successfully".green().bold());
            toml
        }
    };
    println!(
        "----------------------\n{}\n\
    {:<8} {}\n\
    {:<8} {}\n\
    {:<8} {}\n\
    {:<8} {}\n----------------------",
        "Crisp Project Metadata".bold(),
        "Name:".bold().magenta(),
        toml.project.name.blue(),
        "Type:".bold().magenta(),
        toml.project.r#type.to_string().blue(),
        "Version:".bold().magenta(),
        toml.project.version.to_string().blue(),
        "Authors:".bold().magenta(),
        format!("{:?}", toml.project.authors).blue(),
    );
    info!("{}", "Checking project directory structure".bold());
    for (path, file_name) in paths.iter().skip(1) {
        if !path.exists() {
            if (file_name == "src/lib.crisp" && toml.project.r#type == ProjectType::Bin)
                || (file_name == "src/main.crisp" && toml.project.r#type == ProjectType::Lib)
            {
                warn!("{:<15} {}", file_name, "not found".yellow());
            } else {
                error!("{:<15} {}", file_name, "not found".red());
                return Err(Error::msg("Project structure invalid"));
            }
        } else {
            trace!("{:<15} {}", file_name, "found".green());
        }
    }
    info!(
        "{} {}",
        "Project structure",
        "validation complete".green().bold()
    );
    Ok(ProjectStructure {
        main_file: paths[3].0.inner(),
        config: toml,
        module_tree: ModuleTree::new(path, progress)?,
    })
}
