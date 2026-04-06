use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Walk `dir` recursively and collect all `.md` files.
/// Returns an empty vec if none are found.
pub fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_markdown(p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── is_markdown ──────────────────────────────────────────────────────────

    #[test]
    fn is_markdown_returns_true_for_md_extension() {
        assert!(is_markdown(Path::new("note.md")));
    }

    #[test]
    fn is_markdown_returns_true_for_uppercase_md_extension() {
        assert!(is_markdown(Path::new("note.MD")));
    }

    #[test]
    fn is_markdown_returns_false_for_txt_extension() {
        assert!(!is_markdown(Path::new("note.txt")));
    }

    #[test]
    fn is_markdown_returns_false_for_rs_extension() {
        assert!(!is_markdown(Path::new("main.rs")));
    }

    #[test]
    fn is_markdown_returns_false_for_no_extension() {
        assert!(!is_markdown(Path::new("README")));
    }

    // ── collect_md_files ─────────────────────────────────────────────────────

    #[test]
    fn collect_md_files_returns_all_md_files_in_tree() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create a subdirectory with markdown files at different depths
        let subdir = root.join("sub");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(root.join("a.md"), "# A").unwrap();
        fs::write(root.join("b.md"), "# B").unwrap();
        fs::write(subdir.join("c.md"), "# C").unwrap();

        let mut found = collect_md_files(root);
        found.sort();

        assert_eq!(found.len(), 3);
        assert!(found.iter().any(|p| p.ends_with("a.md")));
        assert!(found.iter().any(|p| p.ends_with("b.md")));
        assert!(found.iter().any(|p| p.ends_with("c.md")));
    }

    #[test]
    fn collect_md_files_returns_empty_vec_for_empty_dir() {
        let dir = TempDir::new().unwrap();
        let found = collect_md_files(dir.path());
        assert!(found.is_empty());
    }

    #[test]
    fn collect_md_files_ignores_non_markdown_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        fs::write(root.join("note.md"), "# Note").unwrap();
        fs::write(root.join("note.txt"), "plain text").unwrap();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("config.toml"), "[general]").unwrap();

        let found = collect_md_files(root);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("note.md"));
    }
}
