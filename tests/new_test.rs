//! Integration tests for the `new` command.
//!
//! These tests exercise template rendering and file creation without opening
//! a real editor (we pass `echo` as the file-reader).

use assert_cmd::prelude::OutputAssertExt;
use std::fs;
use tempfile::TempDir;

fn zettel() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("zettel-cli")
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Write a minimal MiniJinja template to a temp file and return the dir + path.
fn write_template(dir: &TempDir, content: &str) -> std::path::PathBuf {
    let path = dir.path().join("template.md");
    fs::write(&path, content).unwrap();
    path
}

// ── file creation ─────────────────────────────────────────────────────────────

#[test]
fn new_creates_note_in_target_directory() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "My Note",
        ])
        .assert()
        .success();

    let note = target_dir.path().join("my-note.md");
    assert!(note.exists(), "expected note file to be created at {note:?}");
}

#[test]
fn new_creates_target_directory_if_absent() {
    let tmpl_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");
    let target = tmpl_dir.path().join("new-subdir");

    assert!(!target.exists());

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target.to_str().unwrap(),
            "My Note",
        ])
        .assert()
        .success();

    assert!(target.exists(), "target directory should have been created");
}

#[test]
fn new_renders_title_in_template() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "Hello World",
        ])
        .assert()
        .success();

    // Filename is slugified; {{ title }} in the template stays un-slugified.
    let content = fs::read_to_string(target_dir.path().join("hello-world.md")).unwrap();
    assert_eq!(content.trim(), "# Hello World");
}

#[test]
fn new_renders_date_fields_in_template() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "date={{ date }} year={{ year }}");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "dated-note",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(target_dir.path().join("dated-note.md")).unwrap();
    // date should match YYYY-MM-DD
    assert!(
        content.contains("date=20"),
        "expected ISO date in output, got: {content}"
    );
    assert!(
        content.contains("year=20"),
        "expected year in output, got: {content}"
    );
}

#[test]
fn new_slug_filter_converts_title() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "{{ title | slug }}");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "Hello World",
        ])
        .assert()
        .success();

    // Both the filename and {{ title | slug }} in the template are slugified.
    let content = fs::read_to_string(target_dir.path().join("hello-world.md")).unwrap();
    assert_eq!(content.trim(), "hello-world");
}

#[test]
fn new_errors_when_template_file_is_missing() {
    let target_dir = TempDir::new().unwrap();

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", "/tmp/zettel-no-such-template.md",
            "--target-path", target_dir.path().to_str().unwrap(),
            "My Note",
        ])
        .assert()
        .failure();
}

#[test]
fn new_errors_on_malformed_minijinja_template() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    // Unclosed block is invalid MiniJinja syntax
    let tmpl = write_template(&tmpl_dir, "{% if %}broken{%");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "My Note",
        ])
        .assert()
        .failure();
}

// ── --dry-run ─────────────────────────────────────────────────────────────────

#[test]
fn new_dry_run_prints_rendered_content_to_stdout() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    let output = zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "My Note",
            "--dry-run",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# My Note"), "expected rendered content in stdout, got: {stdout}");
}

#[test]
fn new_dry_run_does_not_write_file() {
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", target_dir.path().to_str().unwrap(),
            "dry-note",
            "--dry-run",
        ])
        .assert()
        .success();

    // File must NOT be created
    assert!(!target_dir.path().join("dry-note.md").exists());
}

#[test]
fn new_dry_run_does_not_create_target_directory() {
    let tmpl_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "hello");
    let non_existent = tmpl_dir.path().join("should-not-be-created");

    zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", non_existent.to_str().unwrap(),
            "note",
            "--dry-run",
        ])
        .assert()
        .success();

    assert!(!non_existent.exists(), "directory should not have been created in dry-run mode");
}

// ── expand_path ───────────────────────────────────────────────────────────────

#[test]
fn new_expands_tilde_in_target_path() {
    // We can't actually write to ~/... in tests, but we CAN verify that a
    // tilde path is accepted syntactically (expand_path succeeds).
    // Use --dry-run so no file is written, just check the command succeeds.
    let tmpl_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    // Use a real expanded path via $HOME substitution: ~/tmp is safe for dry-run
    let tilde_target = "~/tmp";

    let result = zettel()
        .args([
            "new",
            "--file-reader", "echo",
            "--template-path", tmpl.to_str().unwrap(),
            "--target-path", tilde_target,
            "tilde-test",
            "--dry-run",
        ])
        .output()
        .unwrap();

    // Should succeed — expand_path handles ~
    assert!(result.status.success(), "expected success with tilde in target path");
    // And the expanded path should appear nowhere in stderr as an error
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("invalid"), "unexpected error: {stderr}");
}

// ── config-driven ─────────────────────────────────────────────────────────────

#[test]
fn new_uses_config_defaults_when_no_flags_given() {
    use std::io::Write;
    let tmpl_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let tmpl = write_template(&tmpl_dir, "# {{ title }}");

    let mut cfg_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(cfg_file, "[general]").unwrap();
    writeln!(cfg_file, "default_template_path = \"{}\"", tmpl.display()).unwrap();
    writeln!(cfg_file, "default_target_path = \"{}\"", target_dir.path().display()).unwrap();
    writeln!(cfg_file, "file_reader = \"true\"").unwrap();

    zettel()
        .args([
            "--config", cfg_file.path().to_str().unwrap(),
            "new", "My Config Note",
        ])
        .assert()
        .success();

    assert!(target_dir.path().join("my-config-note.md").exists());
}

// ── completions ───────────────────────────────────────────────────────────────

#[test]
fn completions_bash_outputs_to_stdout() {
    let output = zettel()
        .args(["completions", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "expected bash completion script on stdout");
    // Bash completions always contain the binary name
    assert!(stdout.contains("zettel"), "expected 'zettel' in completion script");
}

#[test]
fn completions_zsh_outputs_to_stdout() {
    let output = zettel()
        .args(["completions", "zsh"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "expected zsh completion script on stdout");
}

#[test]
fn completions_fish_outputs_to_stdout() {
    let output = zettel()
        .args(["completions", "fish"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "expected fish completion script on stdout");
}
