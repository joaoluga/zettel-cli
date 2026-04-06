//! Integration tests for the `lucky` command.
//!
//! These tests cover the file-collection and selection logic that currently
//! lives in `main.rs`.  Once `lucky` is extracted to
//! `src/commands/lucky.rs` and made public, the `use` import below should be
//! updated to point at the new module.
//!
//! For now the CLI binary is exercised via `assert_cmd` so we can at least
//! verify the error paths without an interactive editor.

use assert_cmd::prelude::OutputAssertExt;
use std::fs;
use tempfile::TempDir;

fn zettel() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("zettel-cli")
}

// ── is_markdown (future unit-test placeholder) ────────────────────────────────
//
// Once `is_markdown` is extracted to `src/utils/fs.rs` and made `pub`, add:
//
//   use zettel_cli::utils::fs::is_markdown;
//
//   #[test]
//   fn is_markdown_accepts_md_extension() { assert!(is_markdown(Path::new("note.md"))); }
//   #[test]
//   fn is_markdown_accepts_uppercase_md() { assert!(is_markdown(Path::new("NOTE.MD"))); }
//   #[test]
//   fn is_markdown_rejects_txt()          { assert!(!is_markdown(Path::new("note.txt"))); }
//   #[test]
//   fn is_markdown_rejects_no_extension() { assert!(!is_markdown(Path::new("README"))); }

// ── lucky via CLI ─────────────────────────────────────────────────────────────

#[test]
fn lucky_errors_on_nonexistent_directory() {
    zettel()
        .args(["lucky", "--path", "/tmp/zettel-does-not-exist-xyz"])
        .assert()
        .failure();
}

#[test]
fn lucky_errors_when_no_md_files_in_directory() {
    let dir = TempDir::new().unwrap();
    // put a non-markdown file so the dir is not empty
    fs::write(dir.path().join("notes.txt"), "not markdown").unwrap();

    zettel()
        .args(["lucky", "--path", dir.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn lucky_errors_when_directory_is_empty() {
    let dir = TempDir::new().unwrap();

    zettel()
        .args(["lucky", "--path", dir.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn lucky_succeeds_with_md_file_and_echo_reader() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("note.md"), "# Hello").unwrap();

    // Use `echo` as the file-reader so no real editor is spawned.
    zettel()
        .args([
            "lucky",
            "--path",
            dir.path().to_str().unwrap(),
            "--file-reader",
            "echo",
        ])
        .assert()
        .success();
}

#[test]
fn lucky_recurses_into_subdirectories() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("deep.md"), "# Deep").unwrap();

    zettel()
        .args([
            "lucky",
            "--path",
            dir.path().to_str().unwrap(),
            "--file-reader",
            "echo",
        ])
        .assert()
        .success();
}
