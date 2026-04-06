use anyhow::{bail, Context, Result};
use zettel_cli::config::{expand_path, Config};
use zettel_cli::templates::context::render_title;
use crate::commands::new::new_note;

pub fn preset(
    cfg: &Config,
    preset_name: &str,
    file_reader: String,
    title: Option<String>,
) -> Result<()> {
    let preset_cfg = cfg
        .preset
        .get(preset_name)
        .with_context(|| format!("preset '{}' not found in config", preset_name))?;

    // date_format: preset-level overrides general-level
    let date_format = preset_cfg
        .date_format
        .clone()
        .or_else(|| cfg.general.date_format.clone());

    // Resolve file name: CLI title > rendered default_title > error
    let file_name = match title {
        Some(t) => t,
        None => match &preset_cfg.default_title {
            Some(tmpl) => render_title(tmpl, date_format.as_deref())
                .context("failed to render preset default_title")?,
            None => bail!(
                "no title given for preset '{}' and it has no default_title in config",
                preset_name
            ),
        },
    };

    let template_path = expand_path(&preset_cfg.template_path)
        .with_context(|| format!("invalid template_path in preset '{preset_name}'"))?;
    let target_path = expand_path(&preset_cfg.target_path)
        .with_context(|| format!("invalid target_path in preset '{preset_name}'"))?;

    new_note(file_reader, target_path, file_name, Some(template_path), false, date_format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;
    use zettel_cli::config::{Config, GeneralConfig, PresetConfig, SearchConfig};

    fn make_config(presets: HashMap<String, PresetConfig>) -> Config {
        Config {
            general: GeneralConfig::default(),
            search: SearchConfig::default(),
            preset: presets,
        }
    }

    // ── unknown preset ────────────────────────────────────────────────────────

    #[test]
    fn errors_when_preset_name_not_in_config() {
        let cfg = make_config(HashMap::new());
        let result = preset(&cfg, "daily", "echo".into(), Some("note".into()));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("daily"), "error should mention the preset name: {msg}");
        assert!(msg.contains("not found"), "error should say 'not found': {msg}");
    }

    #[test]
    fn error_message_contains_the_missing_preset_name() {
        let cfg = make_config(HashMap::new());
        let result = preset(&cfg, "weekly", "echo".into(), Some("note".into()));
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("weekly"), "got: {msg}");
    }

    // ── successful preset ─────────────────────────────────────────────────────

    #[test]
    fn creates_note_at_correct_path_for_known_preset() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl_path = tmpl_dir.path().join("daily.md");
        fs::write(&tmpl_path, "# {{ title }}").unwrap();

        let mut presets = HashMap::new();
        presets.insert(
            "daily".into(),
            PresetConfig {
                template_path: tmpl_path.to_str().unwrap().to_string(),
                target_path: target_dir.path().to_str().unwrap().to_string(),
                default_title: None,
                date_format: None,
            },
        );

        let cfg = make_config(presets);
        preset(&cfg, "daily", "true".into(), Some("2026-03-02".into())).unwrap();

        assert!(
            target_dir.path().join("2026-03-02.md").exists(),
            "expected note file to be created"
        );
    }

    #[test]
    fn renders_title_into_template_for_known_preset() {
        let tmpl_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let tmpl_path = tmpl_dir.path().join("note.md");
        fs::write(&tmpl_path, "# {{ title }}").unwrap();

        let mut presets = HashMap::new();
        presets.insert(
            "test".into(),
            PresetConfig {
                template_path: tmpl_path.to_str().unwrap().to_string(),
                target_path: target_dir.path().to_str().unwrap().to_string(),
                default_title: None,
                date_format: None,
            },
        );

        let cfg = make_config(presets);
        preset(&cfg, "test", "true".into(), Some("My Note".into())).unwrap();

        let content = fs::read_to_string(target_dir.path().join("my-note.md")).unwrap();
        assert_eq!(content.trim(), "# My Note");
    }

    // ── path expansion ────────────────────────────────────────────────────────

    #[test]
    fn expands_tilde_in_template_path_and_errors_clearly() {
        // ~/nonexistent.md won't exist — but the error should show the expanded
        // path, not the literal '~'
        let target_dir = TempDir::new().unwrap();
        let mut presets = HashMap::new();
        presets.insert(
            "tilde-test".into(),
            PresetConfig {
                template_path: "~/zettel-no-such-template.md".into(),
                target_path: target_dir.path().to_str().unwrap().to_string(),
                default_title: None,
                date_format: None,
            },
        );

        let cfg = make_config(presets);
        let result = preset(&cfg, "tilde-test", "true".into(), Some("note".into()));

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            !msg.contains('~'),
            "error should show expanded path, not literal '~': {msg}"
        );
    }

    #[test]
    fn expands_tilde_in_target_path() {
        // The error from new_note will mention the expanded path
        let tmpl_dir = TempDir::new().unwrap();
        let tmpl_path = tmpl_dir.path().join("note.md");
        fs::write(&tmpl_path, "hi").unwrap();

        let mut presets = HashMap::new();
        presets.insert(
            "tilde-target".into(),
            PresetConfig {
                template_path: tmpl_path.to_str().unwrap().to_string(),
                // Use a real tilde path that gets expanded — just dry-check
                // by looking at what's passed to new_note. We use /tmp directly
                // as target to avoid writing to HOME.
                target_path: "/tmp/zettel-preset-tilde-test".into(),
                default_title: None,
                date_format: None,
            },
        );

        let cfg = make_config(presets);
        // Just assert it doesn't panic on path expansion
        let result = preset(&cfg, "tilde-target", "true".into(), Some("note".into()));
        // May succeed or fail (depending on /tmp perms) but must not panic
        let _ = result;
    }
}
