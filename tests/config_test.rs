use std::io::Write;
use zettel_cli::config::{load_config, expand_path};
use std::path::PathBuf;

// ── expand_path ───────────────────────────────────────────────────────────────

#[test]
fn expand_path_absolute_is_unchanged() {
    let result = expand_path("/tmp/notes").unwrap();
    assert_eq!(result, PathBuf::from("/tmp/notes"));
}

#[test]
fn expand_path_tilde_resolves_to_home() {
    let home = std::env::var("HOME").unwrap();
    let result = expand_path("~/notes").unwrap();
    assert_eq!(result, PathBuf::from(format!("{home}/notes")));
}

#[test]
fn expand_path_env_var_is_substituted() {
    // SAFETY: test-only, single-threaded context.
    unsafe { std::env::set_var("ZETTEL_INTEG_TEST_DIR", "/tmp/zettel-integ") };
    let result = expand_path("$ZETTEL_INTEG_TEST_DIR/inbox").unwrap();
    assert_eq!(result, PathBuf::from("/tmp/zettel-integ/inbox"));
}

// ── load_config ───────────────────────────────────────────────────────────────

#[test]
fn load_config_nonexistent_path_returns_default() {
    let path = PathBuf::from("/tmp/zettel-cli-definitely-not-here-12345.toml");
    let cfg = load_config(Some(path)).unwrap();
    assert!(cfg.general.notes_path.is_none());
    assert!(cfg.general.file_reader.is_none());
}

#[test]
fn load_config_parses_general_path_and_file_reader() {
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
fn load_config_empty_file_returns_default_values() {
    let f = tempfile::NamedTempFile::new().unwrap();
    let cfg = load_config(Some(f.path().to_path_buf())).unwrap();
    assert!(cfg.general.notes_path.is_none());
    assert!(cfg.general.file_reader.is_none());
}

#[test]
fn load_config_malformed_toml_is_an_error() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "not valid toml === :::").unwrap();
    assert!(load_config(Some(f.path().to_path_buf())).is_err());
}

#[test]
fn load_config_only_file_reader_set() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "[general]\nfile_reader = \"micro\"").unwrap();

    let cfg = load_config(Some(f.path().to_path_buf())).unwrap();
    assert!(cfg.general.notes_path.is_none());
    assert_eq!(cfg.general.file_reader.as_deref(), Some("micro"));
}
