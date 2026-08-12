use anyhow::Error;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar};
use log::{info, warn};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::logging::spinner_style;

pub struct ModuleTree {
    modules: Vec<Module>,
}
impl ModuleTree {
    pub fn new(path: &Path, progress: MultiProgress) -> Result<Self, Error> {
        info!("{}", "Building module tree".bold());
        let mut modules = Vec::new();
        let spinner = progress.add(
            ProgressBar::new_spinner()
                .with_style(spinner_style())
                .with_message("Modules detected"),
        );
        if !path.join("Crisp.toml").exists() {
            warn!("Module tree build started outside of project root");
        }
        let src_dir = path.join("src");
        if !src_dir.exists() {
            return Err(Error::msg("Cannot build module tree without src directory"));
        }
        let main_path = path.join("src/main.crisp");
        let lib_path = path.join("src/lib.crisp");
        match (main_path.exists(), lib_path.exists()) {
            (true, true) => {
                let main_index = modules.len();
                let main_module = Module::new(main_path.clone());
                modules.push(main_module);
                spinner.inc(1);
                let lib_index = modules.len();
                let lib_module = Module::new(lib_path.clone());
                modules.push(lib_module);
                spinner.inc(1);
                modules[main_index].children.push(lib_index);
                let main_source = fs::read_to_string(&main_path)?;
                modules[main_index].source = main_source;
                modules[main_index].name = "main.crisp".to_string();
                let lib_source = fs::read_to_string(&lib_path)?;
                modules[lib_index].source = lib_source;
                modules[lib_index].name = "lib.crisp".to_string();
            }
            (true, false) => {
                let main_index = modules.len();
                let main_module = Module::new(main_path.clone());
                spinner.inc(1);
                modules.push(main_module);
                let main_source = fs::read_to_string(&main_path)?;
                modules[main_index].source = main_source;
                modules[main_index].name = "main.crisp".to_string();
            }
            (false, true) => {
                let lib_index = modules.len();
                let lib_module = Module::new(lib_path.clone());
                spinner.inc(1);
                modules.push(lib_module);
                let lib_source = fs::read_to_string(&lib_path)?;
                modules[lib_index].source = lib_source;
                modules[lib_index].name = "lib.crisp".to_string();
            }
            (false, false) => {
                return Err(Error::msg("Cannot build module tree without lib/main file"));
            }
        }
        let start_path = path.join("src");
        for entry in WalkDir::new(start_path).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file()
                || entry.path().extension().is_none_or(|ext| ext != "crisp")
                || entry.path() == main_path
                || entry.path() == lib_path
            {
                continue;
            }
            let index = modules.len();
            let mut module = Module::new(entry.path().to_path_buf());
            let module_source = fs::read_to_string(entry.path())?;
            module.source = module_source;
            module.name = entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let mut parent = entry.path().parent();
            while let Some(dir) = parent {
                let candidate = dir.with_extension("crisp");
                if let Some(parent_index) =
                    modules.iter().position(|module| module.path == candidate)
                {
                    modules[parent_index].children.push(index);
                    break;
                }
                parent = dir.parent();
            }
            modules.push(module);
            spinner.inc(1);
        }
        info!("{} {}", "Module tree", "build successful".green().bold());
        Ok(Self { modules })
    }
}

pub struct Module {
    pub children: Vec<usize>,
    pub path: PathBuf,
    pub source: String,
    pub name: String,
}
impl Module {
    pub fn new(path: PathBuf) -> Self {
        Self {
            children: Vec::new(),
            path,
            source: String::default(),
            name: String::default(),
        }
    }
}
