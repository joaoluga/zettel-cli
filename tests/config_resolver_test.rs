use std::path::PathBuf;
use zettel_cli::config::{Config, GeneralConfig};
use zettel_cli::config::resolver::resolve_general;

fn cfg(path: Option<&str>, file_reader: Option<&str>) -> Config {
    Config {
        general: GeneralConfig {
            notes_path: path.map(str::to_string),
            file_reader: file_reader.map(str::to_string),
            ..Default::default()
        },
        search: Default::default(),
        preset: std::collections::HashMap::new(),
    }
}

// ── path resolution ───────────────────────────────────────────────────────────

#[test]
fn cli_path_overrides_config_path() {
    let c = cfg(Some("/config/notes"), None);
    let r = resolve_general(&c, Some("/cli/notes".into()), None).unwrap();
    assert_eq!(r.path, PathBuf::from("/cli/notes"));
}

#[test]
fn config_path_used_when_cli_flag_absent() {
    let c = cfg(Some("/config/notes"), None);
    let r = resolve_general(&c, None, None).unwrap();
    assert_eq!(r.path, PathBuf::from("/config/notes"));
}

#[test]
fn missing_path_produces_an_error() {
    let c = cfg(None, None);
    let err = resolve_general(&c, None, None).unwrap_err();
    assert!(
        err.to_string().contains("path"),
        "error should mention 'path', got: {err}"
    );
}

// ── file_reader resolution ────────────────────────────────────────────────────

#[test]
fn cli_file_reader_overrides_config_value() {
    let c = cfg(Some("/tmp"), Some("vim"));
    let r = resolve_general(&c, Some("/tmp".into()), Some("hx".into())).unwrap();
    assert_eq!(r.file_reader, "hx");
}

#[test]
fn config_file_reader_used_when_cli_flag_absent() {
    let c = cfg(Some("/tmp"), Some("hx"));
    let r = resolve_general(&c, Some("/tmp".into()), None).unwrap();
    assert_eq!(r.file_reader, "hx");
}

#[test]
fn file_reader_defaults_to_nvim_when_unset_everywhere() {
    let c = cfg(Some("/tmp"), None);
    let r = resolve_general(&c, Some("/tmp".into()), None).unwrap();
    assert_eq!(r.file_reader, "nvim");
}

// ── tilde / env-var expansion ─────────────────────────────────────────────────

#[test]
fn tilde_in_cli_path_is_expanded() {
    let c = cfg(None, None);
    let r = resolve_general(&c, Some("~/notes".into()), None).unwrap();
    let home = std::env::var("HOME").unwrap();
    assert_eq!(r.path, PathBuf::from(format!("{home}/notes")));
}

#[test]
fn tilde_in_config_path_is_expanded() {
    let c = cfg(Some("~/notes"), None);
    let r = resolve_general(&c, None, None).unwrap();
    let home = std::env::var("HOME").unwrap();
    assert_eq!(r.path, PathBuf::from(format!("{home}/notes")));
}
