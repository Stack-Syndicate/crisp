use anyhow::Error;
use indicatif::{MultiProgress, ProgressBar};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::logging::spinner_style;

pub struct ModuleTree {
    modules: Vec<Module>,
}
impl ModuleTree {
    pub fn new(path: &Path, progress: MultiProgress) -> Result<Self, Error> {
        let spinner = progress.add(
            ProgressBar::new_spinner()
                .with_style(spinner_style())
                .with_message("Modules discovered:"),
        );
        let mut modules = Vec::new();
        let mut root_module = Module::new(path.join("src/mod.crisp"));
        root_module.name = "root".to_string();
        root_module.source = fs::read_to_string(path.join("src/mod.crisp")).unwrap();
        modules.push(root_module);
        spinner.inc(1);
        struct WorkItem {
            parent_index: Option<usize>,
            path: PathBuf,
        }
        let mut workstack = vec![WorkItem {
            path: path.to_path_buf().join("src/mod.crisp"),
            parent_index: None,
        }];
        while let Some(workitem) = workstack.pop() {
            let WorkItem {
                parent_index,
                path: current_path,
            } = workitem;
            if current_path.exists()
                && current_path.is_file()
                && let Some(file_name) = current_path.file_name()
                && file_name == "mod.crisp"
            {
                Self::scan_children_for_leaves(current_path.parent().unwrap())
                    .iter()
                    .for_each(|candidate| {
                        let new_parent_index = modules.len();
                        let (module_name, path) = if candidate.is_dir() {
                            (
                                candidate.file_name().unwrap().to_str().unwrap().to_string(),
                                candidate.join("mod.crisp"),
                            )
                        } else {
                            (
                                candidate.file_stem().unwrap().to_str().unwrap().to_string(),
                                candidate.to_path_buf(),
                            )
                        };
                        workstack.push(WorkItem {
                            parent_index: Some(new_parent_index),
                            path: path.to_path_buf(),
                        });
                        let mut new_module = Module::new(path.to_path_buf());
                        new_module.name = module_name.to_string();
                        new_module.source = fs::read_to_string(path).unwrap();
                        modules.push(new_module);
                        spinner.inc(1);
                        if let Some(parent_index) = parent_index {
                            modules[parent_index].children.push(new_parent_index);
                        }
                    });
            }
        }
        spinner.finish();
        Ok(Self { modules })
    }
    fn scan_children_for_leaves(path: &Path) -> Vec<PathBuf> {
        let mut children = Vec::new();
        for file in path.read_dir().unwrap() {
            let file = file.unwrap();
            let path = file.path();
            if (path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "crisp")
                && path.file_name() != Some("mod.crisp".as_ref()))
                || (path.is_dir()
                    && path
                        .read_dir()
                        .unwrap()
                        .any(|file| file.unwrap().file_name() == "mod.crisp"))
            {
                children.push(path);
            }
        }
        children
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
