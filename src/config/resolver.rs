use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::{expand_path, Config};

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Plain,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Plain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedSearch {
    pub path: PathBuf,
    pub format: OutputFormat,
}

pub fn resolve_search(
    cfg: &Config,
    path: Option<String>,
    format: Option<String>,
) -> Result<ResolvedSearch> {
    let path_str = path
        .or_else(|| cfg.general.notes_path.clone())
        .context("missing path (provide --path or set [general].notes_path in config)")?;

    let path = expand_path(&path_str)
        .with_context(|| format!("invalid path: {path_str}"))?;

    let format_str = format
        .or_else(|| cfg.search.default_format.clone())
        .unwrap_or_else(|| "plain".to_string());

    let format = OutputFormat::from_str(&format_str);

    Ok(ResolvedSearch { path, format })
}

#[derive(Debug, Clone)]
pub struct ResolvedGeneral {
    pub path: PathBuf,
    pub file_reader: String,
}

pub fn resolve_general(
    cfg: &Config,
    path: Option<String>,
    file_reader: Option<String>,
) -> Result<ResolvedGeneral> {
    let path_str = path
        .or_else(|| cfg.general.notes_path.clone())
        .context("missing path (provide --path or set [general].path in config)")?;

    let path = expand_path(&path_str)
        .with_context(|| format!("invalid [general].path: {path_str}"))?;

    let file_reader = file_reader
        .or_else(|| cfg.general.file_reader.clone())
        .unwrap_or_else(|| "nvim".to_string());

    Ok(ResolvedGeneral { path, file_reader })
}

#[derive(Debug, Clone)]
pub struct ResolvedNew {
    pub file_reader: String,
    pub template_path: Option<PathBuf>,
    pub target_path: PathBuf,
    pub date_format: Option<String>,
}

pub fn resolve_new(
    cfg: &Config,
    file_reader: Option<String>,
    template_path: Option<String>,
    target_path: Option<String>,
) -> Result<ResolvedNew> {
    let file_reader = file_reader
        .or_else(|| cfg.general.file_reader.clone())
        .unwrap_or_else(|| "nvim".to_string());

    let template_path = template_path
        .or_else(|| cfg.general.default_template_path.clone())
        .map(|s| expand_path(&s))
        .transpose()
        .context("invalid template_path")?;

    let target_str = target_path
        .or_else(|| cfg.general.default_target_path.clone())
        .context("missing target_path (provide --target-path or set [general].default_target_path in config)")?;

    let target_path = expand_path(&target_str).context("invalid target_path")?;

    let date_format = cfg.general.date_format.clone();

    Ok(ResolvedNew {
        file_reader,
        template_path,
        target_path,
        date_format,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, GeneralConfig};

    fn cfg_with(path: Option<&str>, file_reader: Option<&str>) -> Config {
        Config {
            general: GeneralConfig {
                notes_path: path.map(str::to_string),
                file_reader: file_reader.map(str::to_string),
                default_target_path: None,
                default_template_path: None,
                date_format: None,
            },
            search: Default::default(),
            preset: std::collections::HashMap::new(),
        }
    }

    // ── path resolution ───────────────────────────────────────────────────────

    #[test]
    fn cli_path_takes_priority_over_config() {
        let cfg = cfg_with(Some("/config/notes"), None);
        let resolved = resolve_general(&cfg, Some("/cli/notes".into()), None).unwrap();
        assert_eq!(resolved.path, PathBuf::from("/cli/notes"));
    }

    #[test]
    fn config_path_used_when_cli_absent() {
        let cfg = cfg_with(Some("/config/notes"), None);
        let resolved = resolve_general(&cfg, None, None).unwrap();
        assert_eq!(resolved.path, PathBuf::from("/config/notes"));
    }

    #[test]
    fn missing_path_returns_error() {
        let cfg = cfg_with(None, None);
        let result = resolve_general(&cfg, None, None);
        assert!(result.is_err(), "expected error when path is absent from both CLI and config");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("path"), "error message should mention 'path': {msg}");
    }

    // ── file_reader resolution ────────────────────────────────────────────────

    #[test]
    fn cli_file_reader_takes_priority_over_config() {
        let cfg = cfg_with(Some("/tmp"), Some("vim"));
        let resolved = resolve_general(&cfg, Some("/tmp".into()), Some("hx".into())).unwrap();
        assert_eq!(resolved.file_reader, "hx");
    }

    #[test]
    fn config_file_reader_used_when_cli_absent() {
        let cfg = cfg_with(Some("/tmp"), Some("hx"));
        let resolved = resolve_general(&cfg, Some("/tmp".into()), None).unwrap();
        assert_eq!(resolved.file_reader, "hx");
    }

    #[test]
    fn file_reader_defaults_to_nvim() {
        let cfg = cfg_with(Some("/tmp"), None);
        let resolved = resolve_general(&cfg, Some("/tmp".into()), None).unwrap();
        assert_eq!(resolved.file_reader, "nvim");
    }

    #[test]
    fn both_cli_and_config_absent_file_reader_defaults_to_nvim() {
        let cfg = cfg_with(Some("/tmp"), None);
        let resolved = resolve_general(&cfg, Some("/tmp".into()), None).unwrap();
        assert_eq!(resolved.file_reader, "nvim");
    }

    // ── path expansion ────────────────────────────────────────────────────────

    #[test]
    fn tilde_in_config_path_is_expanded() {
        let cfg = cfg_with(Some("~/notes"), None);
        let resolved = resolve_general(&cfg, None, None).unwrap();
        let home = std::env::var("HOME").unwrap();
        assert_eq!(resolved.path, PathBuf::from(format!("{home}/notes")));
    }

    #[test]
    fn tilde_in_cli_path_is_expanded() {
        let cfg = cfg_with(None, None);
        let resolved = resolve_general(&cfg, Some("~/notes".into()), None).unwrap();
        let home = std::env::var("HOME").unwrap();
        assert_eq!(resolved.path, PathBuf::from(format!("{home}/notes")));
    }

    // ── resolve_new ───────────────────────────────────────────────────────────

    fn cfg_new(
        file_reader: Option<&str>,
        default_template_path: Option<&str>,
        default_target_path: Option<&str>,
    ) -> Config {
        Config {
            general: GeneralConfig {
                notes_path: None,
                file_reader: file_reader.map(str::to_string),
                default_template_path: default_template_path.map(str::to_string),
                default_target_path: default_target_path.map(str::to_string),
                date_format: None,
            },
            search: Default::default(),
            preset: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn resolve_new_cli_target_path_overrides_config() {
        let cfg = cfg_new(None, None, Some("/config/target"));
        let r = resolve_new(&cfg, None, None, Some("/cli/target".into())).unwrap();
        assert_eq!(r.target_path, PathBuf::from("/cli/target"));
    }

    #[test]
    fn resolve_new_config_target_path_used_when_cli_absent() {
        let cfg = cfg_new(None, None, Some("/config/target"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.target_path, PathBuf::from("/config/target"));
    }

    #[test]
    fn resolve_new_missing_target_path_returns_error() {
        let cfg = cfg_new(None, None, None);
        let err = resolve_new(&cfg, None, None, None).unwrap_err();
        assert!(err.to_string().contains("target_path"), "got: {err}");
    }

    #[test]
    fn resolve_new_cli_template_path_overrides_config() {
        let cfg = cfg_new(None, Some("/config/tmpl.md"), None);
        let r = resolve_new(&cfg, None, Some("/cli/tmpl.md".into()), Some("/tmp".into())).unwrap();
        assert_eq!(r.template_path, Some(PathBuf::from("/cli/tmpl.md")));
    }

    #[test]
    fn resolve_new_config_template_path_used_when_cli_absent() {
        let cfg = cfg_new(None, Some("/config/tmpl.md"), Some("/tmp"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.template_path, Some(PathBuf::from("/config/tmpl.md")));
    }

    #[test]
    fn resolve_new_template_path_is_none_when_absent_everywhere() {
        let cfg = cfg_new(None, None, Some("/tmp"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.template_path, None);
    }

    #[test]
    fn resolve_new_cli_file_reader_overrides_config() {
        let cfg = cfg_new(Some("vim"), None, Some("/tmp"));
        let r = resolve_new(&cfg, Some("hx".into()), None, None).unwrap();
        assert_eq!(r.file_reader, "hx");
    }

    #[test]
    fn resolve_new_config_file_reader_used_when_cli_absent() {
        let cfg = cfg_new(Some("hx"), None, Some("/tmp"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.file_reader, "hx");
    }

    #[test]
    fn resolve_new_file_reader_defaults_to_nvim() {
        let cfg = cfg_new(None, None, Some("/tmp"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.file_reader, "nvim");
    }

    #[test]
    fn resolve_new_tilde_in_target_path_is_expanded() {
        let home = std::env::var("HOME").unwrap();
        let cfg = cfg_new(None, None, Some("~/notes"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.target_path, PathBuf::from(format!("{home}/notes")));
    }

    #[test]
    fn resolve_new_tilde_in_template_path_is_expanded() {
        let home = std::env::var("HOME").unwrap();
        let cfg = cfg_new(None, Some("~/tmpl.md"), Some("/tmp"));
        let r = resolve_new(&cfg, None, None, None).unwrap();
        assert_eq!(r.template_path, Some(PathBuf::from(format!("{home}/tmpl.md"))));
    }
}
