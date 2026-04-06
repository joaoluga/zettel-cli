mod commands;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use commands::lucky::lucky;
use commands::new::new_note;
use commands::preset::preset;
use commands::search::{search, SearchMode};
use std::path::PathBuf;
use zettel_cli::config::load_config;
use zettel_cli::config::resolver::{resolve_general, resolve_new, resolve_search};

#[derive(Parser)]
struct Cli {
    /// Optional config file path (defaults to ~/.config/zettel-cli/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lucky {
        #[arg(short, long)]
        path: Option<String>,

        #[arg(short, long)]
        file_reader: Option<String>,
    },

    New {
        /// Title / file name for the new note
        file_name: String,

        /// Override the file reader (defaults to [general].file_reader or nvim)
        #[arg(short, long)]
        file_reader: Option<String>,

        /// Path to the MiniJinja template (defaults to [general].default_template_path)
        #[arg(short, long)]
        template_path: Option<String>,

        /// Directory where the new note is created (defaults to [general].default_target_path)
        #[arg(short = 'T', long)]
        target_path: Option<String>,

        #[arg(long)]
        dry_run: bool,
    },

    Preset {
        /// Name of the preset defined in config under [preset.<name>]
        preset_name: String,

        /// Title / file name (optional if the preset defines default_title)
        #[arg(short, long)]
        title: Option<String>,

        /// Override the file reader from config
        #[arg(short, long)]
        file_reader: Option<String>,
    },

    /// Search notes by filename, tag, or wikilink
    Search {
        /// List .md files; optional substring filter on the path
        #[arg(long, value_name = "FILTER", num_args = 0..=1, default_missing_value = "")]
        by_filename: Option<String>,

        /// List tag-file pairs; optional substring filter on the tag name
        #[arg(long, value_name = "FILTER", num_args = 0..=1, default_missing_value = "")]
        by_tag: Option<String>,

        /// List outgoing wikilinks from NOTE
        #[arg(long, value_name = "NOTE")]
        by_link: Option<String>,

        /// List files that contain a [[NOTE]] wikilink (backlinks)
        #[arg(long, value_name = "NOTE")]
        by_backlink: Option<String>,

        /// Output format: plain (default) or json
        #[arg(short, long)]
        format: Option<String>,

        /// Notes root directory (defaults to [general].notes_path)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = load_config(cli.config)?;

    match cli.command {
        Commands::Lucky { path, file_reader } => {
            let resolved = resolve_general(&cfg, path, file_reader)?;
            lucky(resolved.path, resolved.file_reader)
        }
        Commands::New {
            file_name,
            file_reader,
            template_path,
            target_path,
            dry_run,
        } => {
            let resolved = resolve_new(&cfg, file_reader, template_path, target_path)?;
            new_note(
                resolved.file_reader,
                resolved.target_path,
                file_name,
                resolved.template_path,
                dry_run,
                resolved.date_format,
            )
        }
        Commands::Preset {
            preset_name,
            title,
            file_reader,
        } => {
            let resolved_reader = file_reader
                .or_else(|| cfg.general.file_reader.clone())
                .unwrap_or_else(|| "nvim".to_string());
            preset(&cfg, &preset_name, resolved_reader, title)
        }
        Commands::Search {
            by_filename,
            by_tag,
            by_link,
            by_backlink,
            format,
            path,
        } => {
            let mode_count = [
                by_filename.is_some(),
                by_tag.is_some(),
                by_link.is_some(),
                by_backlink.is_some(),
            ]
            .iter()
            .filter(|&&v| v)
            .count();

            if mode_count == 0 {
                bail!("provide one of: --by-filename, --by-tag, --by-link, --by-backlink");
            }
            if mode_count > 1 {
                bail!("--by-filename, --by-tag, --by-link and --by-backlink are mutually exclusive");
            }

            let resolved = resolve_search(&cfg, path, format)?;

            let mode = if let Some(f) = by_filename {
                SearchMode::ByFilename { filter: if f.is_empty() { None } else { Some(f) } }
            } else if let Some(f) = by_tag {
                SearchMode::ByTag { filter: if f.is_empty() { None } else { Some(f) } }
            } else if let Some(note) = by_link {
                SearchMode::ByLink { note }
            } else {
                SearchMode::ByBacklink { note: by_backlink.unwrap() }
            };

            search(&resolved.path, mode, resolved.format)
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "zettel-cli", &mut std::io::stdout());
            Ok(())
        }
    }
}
