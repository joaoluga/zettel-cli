use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;
use zettel_cli::templates::context::render_template;
use slug::slugify;

pub fn new_note(
    file_reader: String,
    target_path: PathBuf,
    file_name: String,
    template_path: Option<PathBuf>,
    dry_run: bool,
    date_format: Option<String>,
) -> Result<()> {
    // Slugify for the filesystem; keep the original as {{ title }} in the template.
    let slug_name = slug::slugify(&file_name);
    let mut new_note_path = PathBuf::from(&target_path);
    new_note_path.push(&slug_name);
    new_note_path.set_extension("md");

    let rendered = match template_path.as_ref() {
        None => String::new(),
        Some(tp) => {
            let template_src = std::fs::read_to_string(tp)
                .with_context(|| format!("failed to read template: {}", tp.display()))?;
            render_template(&template_src, &file_name, date_format.as_deref())?
        }
    };

    if dry_run {
        println!("{rendered}");
        return Ok(());
    }

    if !target_path.exists() {
        std::fs::create_dir_all(&target_path)
            .with_context(|| format!("failed to create directory: {}", target_path.display()))?;
    }

    std::fs::write(&new_note_path, &rendered)
        .with_context(|| format!("failed to write note: {}", new_note_path.display()))?;

    let status = Command::new(&file_reader)
        .arg(&new_note_path)
        .status()
        .with_context(|| {
            format!(
                "failed to start {} for {}",
                file_reader,
                new_note_path.display()
            )
        })?;

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

    fn write_template(dir: &TempDir, content: &str) -> PathBuf {
        let p = dir.path().join("tmpl.md");
        fs::write(&p, content).unwrap();
        p
    }

    // ── dry_run = true ────────────────────────────────────────────────────────

    #[test]
    fn dry_run_returns_ok_without_creating_file() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "# {{ title }}");

        new_note(
            "echo".into(),
            target_dir.path().to_path_buf(),
            "my-note".into(),
            Some(tmpl),
            true,
            None,
        )
        .unwrap();

        assert!(!target_dir.path().join("my-note.md").exists());
    }

    #[test]
    fn dry_run_returns_ok_with_no_template() {
        let target_dir = TempDir::new().unwrap();

        let result = new_note(
            "echo".into(),
            target_dir.path().to_path_buf(),
            "untitled".into(),
            None,
            true,
            None,
        );
        assert!(result.is_ok());
        assert!(!target_dir.path().join("untitled.md").exists());
    }

    #[test]
    fn dry_run_does_not_create_missing_target_directory() {
        let tmpl_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "hello");
        let ghost_dir = tmpl_dir.path().join("ghost");

        new_note("echo".into(), ghost_dir.clone(), "note".into(), Some(tmpl), true, None).unwrap();

        assert!(!ghost_dir.exists(), "directory must not be created in dry-run mode");
    }

    #[test]
    fn dry_run_error_on_missing_template_file() {
        let target_dir = TempDir::new().unwrap();
        let result = new_note(
            "echo".into(),
            target_dir.path().to_path_buf(),
            "note".into(),
            Some(PathBuf::from("/tmp/zettel-no-such-template.md")),
            true,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn dry_run_error_on_invalid_jinja_template() {
        let tmpl_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "{% if %}broken");
        let target_dir = TempDir::new().unwrap();

        let result = new_note(
            "echo".into(),
            target_dir.path().to_path_buf(),
            "note".into(),
            Some(tmpl),
            true,
            None,
        );
        assert!(result.is_err());
    }

    // ── dry_run = false ───────────────────────────────────────────────────────

    #[test]
    fn writes_rendered_template_to_file() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "# {{ title }}");

        new_note(
            "true".into(), // mock editor — exits 0, ignores args
            target_dir.path().to_path_buf(),
            "My Note".into(),
            Some(tmpl),
            false,
            None,
        )
        .unwrap();

        let content = fs::read_to_string(target_dir.path().join("my-note.md")).unwrap();
        assert_eq!(content.trim(), "# My Note"); // {{ title }} stays un-slugified
    }

    #[test]
    fn creates_target_directory_when_absent() {
        let tmpl_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "content");
        let new_dir = tmpl_dir.path().join("new-subdir");

        assert!(!new_dir.exists());
        new_note("true".into(), new_dir.clone(), "note".into(), Some(tmpl), false, None).unwrap();
        assert!(new_dir.exists());
    }

    #[test]
    fn writes_empty_content_when_no_template() {
        let target_dir = TempDir::new().unwrap();

        new_note(
            "true".into(),
            target_dir.path().to_path_buf(),
            "empty".into(),
            None,
            false,
            None,
        )
        .unwrap();

        let content = fs::read_to_string(target_dir.path().join("empty.md")).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn note_path_uses_file_name_with_md_extension() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "hi");

        new_note(
            "true".into(),
            target_dir.path().to_path_buf(),
            "Hello World".into(),
            Some(tmpl),
            false,
            None,
        )
        .unwrap();

        assert!(target_dir.path().join("hello-world.md").exists());
    }

    #[test]
    fn slug_filter_works_in_template() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "{{ title | slug }}");

        new_note(
            "true".into(),
            target_dir.path().to_path_buf(),
            "Hello World".into(),
            Some(tmpl),
            false,
            None,
        )
        .unwrap();

        // File is at hello-world.md (slugified filename)
        // Content is "hello-world" ({{ title | slug }} applied in template)
        let content = fs::read_to_string(target_dir.path().join("hello-world.md")).unwrap();
        assert_eq!(content.trim(), "hello-world");
    }

    #[test]
    fn error_on_missing_template_file() {
        let target_dir = TempDir::new().unwrap();
        let result = new_note(
            "true".into(),
            target_dir.path().to_path_buf(),
            "note".into(),
            Some(PathBuf::from("/tmp/zettel-no-such-template.md")),
            false,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn error_on_nonexistent_file_reader() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl = write_template(&tmpl_dir, "hi");

        let result = new_note(
            "zettel-cli-no-such-editor-xyz".into(),
            target_dir.path().to_path_buf(),
            "note".into(),
            Some(tmpl),
            false,
            None,
        );
        assert!(result.is_err());
    }
}
