use anyhow::{bail, Context, Result};
use rand::seq::SliceRandom;
use std::path::PathBuf;
use std::process::Command;
use zettel_cli::utils::fs::collect_md_files;

pub fn lucky(dir: PathBuf, file_reader: String) -> Result<()> {
    if !dir.is_dir() {
        bail!("path is not a directory: {}", dir.display());
    }

    let md_files = collect_md_files(&dir);

    if md_files.is_empty() {
        bail!("no .md files found under {}", dir.display());
    }

    let chosen = md_files
        .choose(&mut rand::thread_rng())
        .expect("non-empty vec")
        .clone();

    let status = Command::new(&file_reader)
        .arg(&chosen)
        .status()
        .with_context(|| format!("failed to start {} for {}", file_reader, chosen.display()))?;

    if !status.success() {
        bail!("{} exited with status: {}", file_reader, status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn lucky_errors_for_nonexistent_path() {
        let result = lucky(
            PathBuf::from("/tmp/zettel-lucky-no-such-dir-xyz"),
            "echo".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn lucky_errors_when_path_is_a_file_not_a_directory() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("not-a-dir.md");
        fs::write(&file, "content").unwrap();
        let result = lucky(file, "echo".into());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not a directory"), "expected 'not a directory' in: {msg}");
    }

    #[test]
    fn lucky_errors_for_empty_directory() {
        let dir = TempDir::new().unwrap();
        let result = lucky(dir.path().to_path_buf(), "echo".into());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no .md files"), "expected 'no .md files' in: {msg}");
    }

    #[test]
    fn lucky_errors_when_directory_has_no_md_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.txt"), "not markdown").unwrap();
        fs::write(dir.path().join("config.toml"), "[general]").unwrap();
        let result = lucky(dir.path().to_path_buf(), "echo".into());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no .md files"), "expected 'no .md files' in: {msg}");
    }

    #[test]
    fn lucky_succeeds_with_single_md_file_using_echo() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("note.md"), "# Hello").unwrap();
        let result = lucky(dir.path().to_path_buf(), "echo".into());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    }

    #[test]
    fn lucky_succeeds_with_multiple_md_files_using_echo() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("b.md"), "# B").unwrap();
        fs::write(dir.path().join("c.md"), "# C").unwrap();
        let result = lucky(dir.path().to_path_buf(), "echo".into());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    }

    #[test]
    fn lucky_succeeds_with_md_file_in_subdirectory_using_echo() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("deep.md"), "# Deep").unwrap();
        let result = lucky(dir.path().to_path_buf(), "echo".into());
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    }

    #[test]
    fn lucky_errors_when_file_reader_does_not_exist() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("note.md"), "# Hello").unwrap();
        let result = lucky(
            dir.path().to_path_buf(),
            "zettel-cli-no-such-reader-xyz".into(),
        );
        assert!(result.is_err());
    }
}
