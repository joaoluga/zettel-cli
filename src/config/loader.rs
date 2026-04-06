use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};
use crate::config::types::Config;

pub fn default_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "zettel", "zettel-cli")
        .context("could not resolve config directory")?;
    Ok(proj_dirs.config_dir().join("config.toml"))
}

pub fn load_config(path: Option<PathBuf>) -> Result<Config> {
    let path = path.unwrap_or(default_config_path()?);

    if !path.exists() {
        return Ok(Config::default());
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    let cfg: Config = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML: {}", path.display()))?;

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_config_path_ends_with_config_toml() {
        let path = default_config_path().unwrap();
        assert!(
            path.to_string_lossy().ends_with("config.toml"),
            "expected path to end with 'config.toml', got: {}",
            path.display()
        );
    }

    #[test]
    fn default_config_path_contains_zettel_cli() {
        let path = default_config_path().unwrap();
        let s = path.to_string_lossy();
        assert!(
            s.contains("zettel") || s.contains("zettel-cli"),
            "expected path to contain 'zettel', got: {s}"
        );
    }

    #[test]
    fn load_config_missing_file_returns_default() {
        let path = PathBuf::from("/tmp/zettel-cli-nonexistent-config-test.toml");
        let cfg = load_config(Some(path)).unwrap();
        assert!(cfg.general.notes_path.is_none());
        assert!(cfg.general.file_reader.is_none());
    }

    #[test]
    fn load_config_reads_general_section() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
[general]
notes_path = "/tmp/notes"
file_reader = "hx"
"#
        )
        .unwrap();

        let cfg = load_config(Some(f.path().to_path_buf())).unwrap();
        assert_eq!(cfg.general.notes_path.as_deref(), Some("/tmp/notes"));
        assert_eq!(cfg.general.file_reader.as_deref(), Some("hx"));
    }

    #[test]
    fn load_config_empty_file_returns_default() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let cfg = load_config(Some(f.path().to_path_buf())).unwrap();
        assert!(cfg.general.notes_path.is_none());
        assert!(cfg.general.file_reader.is_none());
    }

    #[test]
    fn load_config_malformed_toml_returns_error() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "this is not valid toml :::").unwrap();
        let result = load_config(Some(f.path().to_path_buf()));
        assert!(result.is_err(), "expected error for malformed TOML");
    }

    #[test]
    fn load_config_partial_general_section() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
[general]
file_reader = "micro"
"#
        )
        .unwrap();

        let cfg = load_config(Some(f.path().to_path_buf())).unwrap();
        assert!(cfg.general.notes_path.is_none());
        assert_eq!(cfg.general.file_reader.as_deref(), Some("micro"));
    }
}
