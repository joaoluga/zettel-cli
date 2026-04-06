use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use zettel_cli::config::OutputFormat;
use zettel_cli::utils::fs::collect_md_files;
use zettel_cli::utils::parse::{extract_links, extract_tags};

// ── Output record types (used for JSON serialisation) ────────────────────────

#[derive(Serialize)]
struct FileRecord {
    file: String,
}

#[derive(Serialize)]
struct TagRecord {
    tag: String,
    file: String,
    line: usize,
}

#[derive(Serialize)]
struct LinkRecord {
    target: String,
    file: String,
}

#[derive(Serialize)]
struct BacklinkRecord {
    file: String,
    line: usize,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub enum SearchMode {
    ByFilename { filter: Option<String> },
    ByTag { filter: Option<String> },
    ByLink { note: String },
    ByBacklink { note: String },
}

pub fn search(root: &Path, mode: SearchMode, format: OutputFormat) -> Result<()> {
    match mode {
        SearchMode::ByFilename { filter } => by_filename(root, filter.as_deref(), format),
        SearchMode::ByTag { filter } => by_tag(root, filter.as_deref(), format),
        SearchMode::ByLink { note } => by_link(root, &note, format),
        SearchMode::ByBacklink { note } => by_backlink(root, &note, format),
    }
}

// ── by-filename ───────────────────────────────────────────────────────────────

fn by_filename(root: &Path, filter: Option<&str>, format: OutputFormat) -> Result<()> {
    let files = collect_md_files(root);
    let records: Vec<FileRecord> = files
        .into_iter()
        .filter_map(|p| {
            let full = p.to_string_lossy().to_string();
            if passes_filter(&full, filter) {
                Some(FileRecord { file: full })
            } else {
                None
            }
        })
        .collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&records)?),
        OutputFormat::Plain => records.iter().for_each(|r| println!("{}", r.file)),
    }
    Ok(())
}

// ── by-tag ────────────────────────────────────────────────────────────────────

fn by_tag(root: &Path, filter: Option<&str>, format: OutputFormat) -> Result<()> {
    let files = collect_md_files(root);
    let mut records: Vec<TagRecord> = Vec::new();

    for path in files {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let full = path.to_string_lossy().to_string();
        for m in extract_tags(&content) {
            if passes_filter(&m.tag, filter) {
                records.push(TagRecord { tag: m.tag, file: full.clone(), line: m.line });
            }
        }
    }

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&records)?),
        OutputFormat::Plain => records.iter().for_each(|r| println!("#{}\t{}", r.tag, r.file)),
    }
    Ok(())
}

// ── by-link ───────────────────────────────────────────────────────────────────

fn by_link(root: &Path, note: &str, format: OutputFormat) -> Result<()> {
    let note_path = resolve_note_path(root, note)?;
    let content = std::fs::read_to_string(&note_path)
        .with_context(|| format!("failed to read {}", note_path.display()))?;

    // Build a stem → path index so we can resolve [[target]] to a real file.
    let index: HashMap<String, PathBuf> = collect_md_files(root)
        .into_iter()
        .filter_map(|p| {
            let stem = p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_lowercase())?;
            Some((stem, p))
        })
        .collect();

    let records: Vec<LinkRecord> = extract_links(&content)
        .into_iter()
        .filter_map(|m| {
            let stem = m.target.to_lowercase();
            index.get(&stem).map(|p| LinkRecord {
                target: m.target,
                file: p.to_string_lossy().to_string(),
            })
        })
        .collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&records)?),
        OutputFormat::Plain => records.iter().for_each(|r| println!("{}", r.file)),
    }
    Ok(())
}

// ── by-backlink ───────────────────────────────────────────────────────────────

fn by_backlink(root: &Path, note: &str, format: OutputFormat) -> Result<()> {
    // Normalise the note name: strip extension and path separators so
    // [[my-note]] matches whether the user passed "my-note" or "my-note.md".
    let target_stem = Path::new(note)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(note)
        .to_lowercase();

    let files = collect_md_files(root);
    let mut records: Vec<BacklinkRecord> = Vec::new();

    for path in files {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let full = path.to_string_lossy().to_string();
        for m in extract_links(&content) {
            if m.target.to_lowercase() == target_stem {
                records.push(BacklinkRecord { file: full.clone(), line: m.line });
                break; // one entry per file is enough
            }
        }
    }

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&records)?),
        OutputFormat::Plain => records.iter().for_each(|r| println!("{}", r.file)),
    }
    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Case-insensitive substring filter. `None` means no filter (everything passes).
fn passes_filter(haystack: &str, filter: Option<&str>) -> bool {
    match filter {
        None | Some("") => true,
        Some(f) => haystack.to_lowercase().contains(&f.to_lowercase()),
    }
}

/// Resolve a note name/path. Accepts:
/// - an absolute path (`/home/user/notes/inbox/my-note.md`)
/// - a path relative to root (`inbox/my-note` or `inbox/my-note.md`)
/// - a bare stem (`my-note`) searched directly under root
fn resolve_note_path(root: &Path, note: &str) -> Result<PathBuf> {
    // 1. Try the value as-is (handles absolute paths and paths with .md)
    let direct = PathBuf::from(note);
    if direct.exists() {
        return Ok(direct);
    }
    // 2. Append .md if missing and try again as-is
    let direct_md = PathBuf::from(format!("{note}.md"));
    if direct_md.exists() {
        return Ok(direct_md);
    }
    // 3. Join with root (handles relative paths and bare stems)
    let via_root = if note.ends_with(".md") {
        root.join(note)
    } else {
        root.join(format!("{note}.md"))
    };
    if via_root.exists() {
        return Ok(via_root);
    }
    anyhow::bail!("note '{}' not found (tried absolute and relative to {})", note, root.display())
}
