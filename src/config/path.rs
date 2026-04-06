use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn expand_path(input: &str) -> Result<PathBuf> {
    let expanded = shellexpand::full(input)
        .with_context(|| format!("failed to expand path: {input}"))?;
    Ok(PathBuf::from(expanded.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_absolute_passthrough() {
        let result = expand_path("/tmp/notes").unwrap();
        assert_eq!(result, PathBuf::from("/tmp/notes"));
    }

    #[test]
    fn expand_path_tilde_expands_to_home() {
        let home = std::env::var("HOME").unwrap();
        let result = expand_path("~/notes").unwrap();
        assert_eq!(result, PathBuf::from(format!("{home}/notes")));
    }

    #[test]
    fn expand_path_env_var_expands() {
        // SAFETY: test-only, single-threaded context.
        unsafe { std::env::set_var("ZETTEL_TEST_DIR", "/tmp/zettel-test") };
        let result = expand_path("$ZETTEL_TEST_DIR/notes").unwrap();
        assert_eq!(result, PathBuf::from("/tmp/zettel-test/notes"));
    }
}
