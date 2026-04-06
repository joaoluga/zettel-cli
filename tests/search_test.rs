use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn zettel() -> Command {
    cargo_bin_cmd!("zettel-cli")
}

fn setup_notes() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // notes/inbox/my-note.md — inline tag + wikilinks
    let inbox = root.join("inbox");
    fs::create_dir_all(&inbox).unwrap();
    fs::write(
        inbox.join("my-note.md"),
        "# My Note\n\nTagged #rust and #productivity.\n\nSee [[other-note]] and [[foo]].\n",
    )
    .unwrap();

    // notes/projects/foo.md — frontmatter tags + backlink to my-note
    let projects = root.join("projects");
    fs::create_dir_all(&projects).unwrap();
    fs::write(
        projects.join("foo.md"),
        "---\ntags:\n  - reading\n  - rust\n---\n\nReference to [[my-note]].\n",
    )
    .unwrap();

    // notes/daily/2026-03-07.md — frontmatter inline array + no links
    let daily = root.join("daily");
    fs::create_dir_all(&daily).unwrap();
    fs::write(
        daily.join("2026-03-07.md"),
        "---\ntags: [daily, productivity]\n---\n\nJust a daily note.\n",
    )
    .unwrap();

    dir
}

// ── --by-filename ─────────────────────────────────────────────────────────────

#[test]
fn by_filename_lists_all_md_files() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-filename", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("my-note.md"));
    assert!(stdout.contains("foo.md"));
    assert!(stdout.contains("2026-03-07.md"));
}

#[test]
fn by_filename_filters_by_substring() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-filename", "daily", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("2026-03-07.md"), "should match the daily note");
    assert!(!stdout.contains("foo.md"), "should not include non-daily notes");
}

#[test]
fn by_filename_json_format() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-filename", "--format", "json", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json.is_array());
    let files: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["file"].as_str().unwrap())
        .collect();
    assert!(files.iter().any(|f| f.contains("my-note.md")));
}

// ── --by-tag ──────────────────────────────────────────────────────────────────

#[test]
fn by_tag_finds_inline_tags() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-tag", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("#rust"), "should find inline #rust tag");
    assert!(stdout.contains("#productivity"), "should find inline #productivity tag");
}

#[test]
fn by_tag_finds_frontmatter_block_list() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-tag", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("#reading"), "should find frontmatter block-list tag");
}

#[test]
fn by_tag_finds_frontmatter_inline_array() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-tag", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("#daily"), "should find frontmatter inline-array tag");
}

#[test]
fn by_tag_filters_by_tag_name() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-tag", "reading", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("#reading"));
    assert!(!stdout.contains("#daily"), "filter should exclude non-matching tags");
}

#[test]
fn by_tag_json_format() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-tag", "--format", "json", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json.is_array());
    let tags: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["tag"].as_str().unwrap())
        .collect();
    assert!(tags.contains(&"rust"));
}

// ── --by-link ─────────────────────────────────────────────────────────────────

#[test]
fn by_link_lists_outgoing_wikilinks() {
    let dir = setup_notes();
    let note = dir.path().join("inbox").join("my-note.md");
    let out = zettel()
        .args([
            "search",
            "--by-link",
            note.to_str().unwrap(),
            "--path",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    // [[foo]] resolves to projects/foo.md which exists in the fixture
    assert!(stdout.contains("foo.md"), "should resolve [[foo]] to its file path");
    // [[other-note]] has no matching file in the fixture — correctly absent
    assert!(!stdout.contains("other-note"), "unresolved links should be omitted");
}

#[test]
fn by_link_json_format() {
    let dir = setup_notes();
    let note = dir.path().join("inbox").join("my-note.md");
    let out = zettel()
        .args([
            "search",
            "--by-link",
            note.to_str().unwrap(),
            "--format",
            "json",
            "--path",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json.is_array());
    let files: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["file"].as_str().unwrap())
        .collect();
    assert!(files.iter().any(|f| f.contains("foo.md")), "should resolve [[foo]] to foo.md path");
}

// ── --by-backlink ─────────────────────────────────────────────────────────────

#[test]
fn by_backlink_finds_files_linking_to_note() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-backlink", "my-note", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("foo.md"), "foo.md links to [[my-note]]");
    assert!(!stdout.contains("my-note.md"), "source note should not appear as its own backlink");
}

#[test]
fn by_backlink_json_format() {
    let dir = setup_notes();
    let out = zettel()
        .args([
            "search",
            "--by-backlink",
            "my-note",
            "--format",
            "json",
            "--path",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json.is_array());
    let files: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["file"].as_str().unwrap())
        .collect();
    assert!(files.iter().any(|f| f.contains("foo.md")));
}

#[test]
fn by_backlink_returns_empty_for_note_with_no_backlinks() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--by-backlink", "other-note", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    // other-note is linked by my-note.md
    assert!(stdout.contains("my-note.md"));
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn search_without_any_by_flag_exits_with_error() {
    let dir = setup_notes();
    let out = zettel()
        .args(["search", "--path", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!out.status.success());
}

#[test]
fn search_missing_path_exits_with_error() {
    let out = zettel()
        .args([
            "--config", "/tmp/zettel-cli-no-such-config-for-test.toml",
            "search", "--by-filename",
        ])
        .output()
        .unwrap();

    assert!(!out.status.success());
}
