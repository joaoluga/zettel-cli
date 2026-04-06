use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub preset: HashMap<String, PresetConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GeneralConfig {
    pub notes_path: Option<String>,
    pub file_reader: Option<String>,
    pub default_target_path: Option<String>,
    pub default_template_path: Option<String>,
    pub date_format: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SearchConfig {
    pub default_format: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PresetConfig {
    pub template_path: String,
    pub target_path: String,
    pub default_title: Option<String>,
    pub date_format: Option<String>,
}
