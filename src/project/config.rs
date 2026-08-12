use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Lib,
    Bin,
    Both,
}
impl Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Lib => f.write_str("library"),
            ProjectType::Bin => f.write_str("program"),
            ProjectType::Both => f.write_str("library & program"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CrispToml {
    pub project: ProjectMetadata,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub r#type: ProjectType,
    pub version: String,
    pub authors: Vec<String>,
}
