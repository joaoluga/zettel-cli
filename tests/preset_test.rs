//! Integration tests for the `preset` command.
//!
//! The `preset` command does not exist yet — it is listed as a task in
//! tasks.md.  These tests are written ahead of time to define the expected
//! behaviour and will be enabled once the command is implemented.
//!
//! Each test is marked `#[ignore]` so `cargo test` does not fail on the
//! unimplemented command.  Run them individually once the feature lands:
//!
//!   cargo test --test preset_test -- --ignored

use assert_cmd::prelude::OutputAssertExt;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

fn zettel() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("zettel-cli")
}

/// Build a config file with a `[preset.daily]` section and return the file.
fn config_with_daily_preset(
    template_path: &std::path::Path,
    target_path: &std::path::Path,
) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        f,
        r#"
[preset.daily]
template_path = "{}"
target_path   = "{}"
"#,
        template_path.display(),
        target_path.display()
    )
    .unwrap();
    f
}

// ── preset command ────────────────────────────────────────────────────────────

#[test]
fn preset_creates_note_using_config_paths() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("daily.md");
    fs::write(&tmpl_path, "# {{ title }}").unwrap();

    let cfg = config_with_daily_preset(&tmpl_path, target_dir.path());

    zettel()
        .args([
            "--config",
            cfg.path().to_str().unwrap(),
            "preset",
            "daily",
            "--file-reader",
            "echo",
            "--title",
            "2026-03-02",
        ])
        .assert()
        .success();

    let note = target_dir.path().join("2026-03-02.md");
    assert!(note.exists(), "expected note at {note:?}");
}

#[test]
fn preset_errors_on_unknown_preset_name() {
    let cfg_file = tempfile::NamedTempFile::new().unwrap();

    zettel()
        .args([
            "--config",
            cfg_file.path().to_str().unwrap(),
            "preset",
            "nonexistent",
            "--file-reader",
            "echo",
            "--title",
            "test",
        ])
        .assert()
        .failure();
}

#[test]
fn preset_expands_tilde_in_template_path() {
    // Ensure `~` in the config's preset paths is expanded before use.
    let cfg_content = format!(
        "[preset.test]\ntemplate_path = \"~/zettel-test-tmpl.md\"\ntarget_path = \"/tmp\"\n"
    );
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "{cfg_content}").unwrap();

    // This will fail because the template won't exist, but the error should
    // mention the expanded path, not the literal `~`.
    let output = zettel()
        .args([
            "--config",
            f.path().to_str().unwrap(),
            "preset",
            "test",
            "--file-reader",
            "echo",
            "--title",
            "test",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains('~'),
        "error output should show expanded path, not literal '~': {stderr}"
    );
}

// ── default_title ─────────────────────────────────────────────────────────────

#[test]
fn preset_uses_default_title_when_no_title_flag_given() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("daily.md");
    fs::write(&tmpl_path, "# {{ title }}").unwrap();

    // Build a config file with default_title = "my-default"
    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[preset.daily]").unwrap();
    writeln!(cfg_file, "template_path = \"{}\"", tmpl_path.display()).unwrap();
    writeln!(cfg_file, "target_path = \"{}\"", target_dir.path().display()).unwrap();
    writeln!(cfg_file, "default_title = \"my-default\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "preset", "daily",
            "--file-reader", "true",
            // no --title flag
        ])
        .assert()
        .success();

    assert!(target_dir.path().join("my-default.md").exists(),
        "expected note named after default_title");
}

#[test]
fn preset_cli_title_overrides_default_title() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("daily.md");
    fs::write(&tmpl_path, "# {{ title }}").unwrap();

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[preset.daily]").unwrap();
    writeln!(cfg_file, "template_path = \"{}\"", tmpl_path.display()).unwrap();
    writeln!(cfg_file, "target_path = \"{}\"", target_dir.path().display()).unwrap();
    writeln!(cfg_file, "default_title = \"should-not-be-used\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "preset", "daily",
            "--file-reader", "true",
            "--title", "cli-wins",
        ])
        .assert()
        .success();

    assert!(target_dir.path().join("cli-wins.md").exists());
    assert!(!target_dir.path().join("should-not-be-used.md").exists());
}

#[test]
fn preset_errors_when_no_title_and_no_default_title() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("daily.md");
    fs::write(&tmpl_path, "# {{ title }}").unwrap();

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[preset.daily]").unwrap();
    writeln!(cfg_file, "template_path = \"{}\"", tmpl_path.display()).unwrap();
    writeln!(cfg_file, "target_path = \"{}\"", target_dir.path().display()).unwrap();
    // no default_title

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "preset", "daily",
            "--file-reader", "true",
            // no --title
        ])
        .assert()
        .failure();
}

// ── date_format ───────────────────────────────────────────────────────────────

#[test]
fn preset_date_format_controls_date_variable_in_default_title() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("weekly.md");
    fs::write(&tmpl_path, "# {{ title }}").unwrap();

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[preset.weekly]").unwrap();
    writeln!(cfg_file, "template_path = \"{}\"", tmpl_path.display()).unwrap();
    writeln!(cfg_file, "target_path = \"{}\"", target_dir.path().display()).unwrap();
    writeln!(cfg_file, "default_title = \"{{{{ date }}}}\"").unwrap();
    writeln!(cfg_file, "date_format = \"%Y-W%V\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "preset", "weekly",
            "--file-reader", "true",
        ])
        .assert()
        .success();

    // File should be named like "2026-w09.md" — slugify lowercases the W from %Y-W%V.
    let files: Vec<_> = fs::read_dir(target_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1, "expected exactly one file");
    let name = files[0].file_name();
    let name_str = name.to_string_lossy();
    assert!(
        name_str.starts_with("20") && name_str.contains("-w"),
        "expected week-format filename like '2026-w09.md', got: {name_str}"
    );
}

#[test]
fn preset_date_format_also_applies_to_template_body() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("weekly.md");
    // Template uses {{ date }} in body
    fs::write(&tmpl_path, "week: {{ date }}").unwrap();

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[preset.weekly]").unwrap();
    writeln!(cfg_file, "template_path = \"{}\"", tmpl_path.display()).unwrap();
    writeln!(cfg_file, "target_path = \"{}\"", target_dir.path().display()).unwrap();
    writeln!(cfg_file, "date_format = \"%Y-W%V\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "preset", "weekly",
            "--file-reader", "true",
            "--title", "weekly-note",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(target_dir.path().join("weekly-note.md")).unwrap();
    assert!(
        content.contains("-W"),
        "expected week-format date in template body, got: {content}"
    );
}

#[test]
fn general_date_format_applies_to_new_command_template_body() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl_path = tmpl_dir.path().join("note.md");
    fs::write(&tmpl_path, "date: {{ date }}").unwrap();

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[general]").unwrap();
    writeln!(cfg_file, "date_format = \"%d/%m/%Y\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "new",
            "--file-reader", "true",
            "--template-path", tmpl_path.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "my-note",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(target_dir.path().join("my-note.md")).unwrap();
    // With format "%d/%m/%Y" the date should contain slashes
    assert!(
        content.contains('/'),
        "expected slash-separated date with custom format, got: {content}"
    );
}
