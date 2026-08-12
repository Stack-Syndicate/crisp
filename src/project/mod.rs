pub mod config;

use crate::{
    logging::progress_bar_style,
    project::config::{CrispToml, ProjectType},
};
use anyhow::Error;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar};
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

pub fn new_project(path: &Path) -> Result<ProjectStructure, Error> {
    if let Ok(exists) = fs::exists(path) {
        if !exists {
            fs::create_dir(path).expect("Could not create project root directory");
        }
    } else {
        panic!("Path may or may not exist: {:?}", path)
    }
    Ok(init_project(path))
}

pub fn init_project(path: &Path) -> ProjectStructure {
    let mut main_file_path = PathBuf::new();
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
                    main_file_path = path.clone();
                }
                fs::write(path, contents).expect("Could not create file");
            }
        }
    }
    ProjectStructure {
        main_file: main_file_path.clone().to_path_buf(),
    }
}

pub fn check_project(path: &Path, progress: MultiProgress) -> Result<ProjectStructure, Error> {
    let paths = ProjectStructure::paths(path.to_path_buf());
    let toml: CrispToml = match toml::from_str(&fs::read_to_string(paths[0].0.inner())?) {
        Err(error) => {
            error!("Invalid Crisp.toml.");
            return Err(error.into());
        }
        Ok(toml) => {
            info!("{} {}", "Crisp.toml", "parsed successfully.".green());
            toml
        }
    };
    let required_paths = match toml.project.r#type {
        ProjectType::Bin | ProjectType::Lib => 3,
        ProjectType::Both => 4,
    };
    let progress_bar = progress.add(ProgressBar::new(required_paths));
    progress_bar.set_style(progress_bar_style());
    progress_bar.set_message("Checking project structure.");
    progress_bar.inc(1);
    info!("{:<5} {}", "Name:", toml.project.name.blue());
    info!("{} {}", "Type:", toml.project.r#type.to_string().blue());
    info!("{} {}", "Version:", toml.project.version.to_string().blue());
    info!(
        "{} {}",
        "Authors:",
        format!("{:?}", toml.project.authors).blue()
    );
    for (path, file_name) in paths.iter().skip(1).clone() {
        if !path.exists() {
            if (file_name == "src/lib.crisp" && toml.project.r#type == ProjectType::Bin)
                || (file_name == "src/main.crisp" && toml.project.r#type == ProjectType::Lib)
            {
                warn!("{:<15} {}", file_name, "not found".yellow())
            } else {
                error!("{:<15} {}", file_name, "not found".red());
                progress_bar.set_message("project check failed".red().to_string());
            }
        } else {
            trace!("{:<15} {}", file_name, "found".green());
            progress_bar.inc(1);
        }
    }
    if progress_bar.is_finished() {
        progress_bar.finish_and_clear();
        info!("Project structure valid.");
    }
    progress.remove(&progress_bar);
    Ok(ProjectStructure {
        main_file: paths[3].0.inner(),
    })
}
