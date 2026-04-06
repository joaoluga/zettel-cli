use regex::Regex;
use std::sync::OnceLock;

pub struct TagMatch {
    pub tag: String,
    pub line: usize,
}

pub struct LinkMatch {
    pub target: String,
    pub line: usize,
}

fn inline_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:^|[^&/\w])#([\w][\w/-]*)").unwrap())
}

fn wikilink_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[\[([^\]|#\n]+?)(?:[|#][^\]]*)?\]\]").unwrap())
}

/// Extract tags from a markdown document.
///
/// Handles two conventions:
/// - YAML frontmatter `tags:` key (list or inline array)
/// - Inline `#tag-name` anywhere in the body (must not be a heading)
pub fn extract_tags(content: &str) -> Vec<TagMatch> {
    let mut tags = Vec::new();

    let (frontmatter, body_start) = split_frontmatter(content);

    if let Some(ref fm) = frontmatter {
        tags.extend(parse_frontmatter_tags(fm));
    }

    let re = inline_tag_re();
    for (idx, line) in content.lines().enumerate().skip(body_start) {
        // Skip YAML frontmatter delimiter lines and markdown headings
        if line.starts_with('#') {
            continue;
        }
        for cap in re.captures_iter(line) {
            tags.push(TagMatch {
                tag: cap[1].to_string(),
                line: idx + 1,
            });
        }
    }

    tags
}

/// Extract all `[[wikilink]]` targets from a markdown document.
pub fn extract_links(content: &str) -> Vec<LinkMatch> {
    let re = wikilink_re();
    content
        .lines()
        .enumerate()
        .flat_map(|(idx, line)| {
            re.captures_iter(line)
                .map(|cap| LinkMatch {
                    target: cap[1].trim().to_string(),
                    line: idx + 1,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Returns `(Option<frontmatter_string>, body_start_line_index)`.
/// If the document begins with `---`, everything up to the closing `---` is
/// considered frontmatter.
fn split_frontmatter(content: &str) -> (Option<String>, usize) {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return (None, 0);
    }
    let mut end_idx = None;
    for (i, line) in content.lines().enumerate().skip(1) {
        if line == "---" {
            end_idx = Some(i);
            break;
        }
    }
    match end_idx {
        None => (None, 0),
        Some(end) => {
            let fm = content.lines().skip(1).take(end - 1).collect::<Vec<_>>().join("\n");
            (Some(fm), end + 1)
        }
    }
}

fn parse_frontmatter_tags(fm: &str) -> Vec<TagMatch> {
    let mut tags = Vec::new();
    let mut in_tags_block = false;

    for (idx, line) in fm.lines().enumerate() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("tags:") {
            in_tags_block = true;
            let rest = rest.trim();
            // Inline array: tags: [rust, productivity]
            if rest.starts_with('[') {
                let inner = rest.trim_start_matches('[').trim_end_matches(']');
                for tag in inner.split(',') {
                    let t = tag.trim().trim_matches('"').trim_matches('\'');
                    if !t.is_empty() {
                        tags.push(TagMatch { tag: t.to_string(), line: idx + 1 });
                    }
                }
                in_tags_block = false;
            } else if !rest.is_empty() {
                // Inline scalar: tags: rust
                tags.push(TagMatch { tag: rest.to_string(), line: idx + 1 });
                in_tags_block = false;
            }
            continue;
        }

        if in_tags_block {
            if let Some(tag) = trimmed.strip_prefix("- ") {
                let t = tag.trim().trim_matches('"').trim_matches('\'');
                if !t.is_empty() {
                    tags.push(TagMatch { tag: t.to_string(), line: idx + 1 });
                }
            } else if !trimmed.is_empty() {
                // Another key started — end of tags block
                in_tags_block = false;
            }
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    // ── extract_links ─────────────────────────────────────────────────────────

    #[test]
    fn extracts_simple_wikilink() {
        let links = extract_links("See [[other-note]] for details.");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "other-note");
        assert_eq!(links[0].line, 1);
    }

    #[test]
    fn extracts_multiple_links_from_same_line() {
        let links = extract_links("Links: [[a]] and [[b]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "a");
        assert_eq!(links[1].target, "b");
    }

    #[test]
    fn extracts_link_with_display_text() {
        // [[target|display]] — only target is captured
        let links = extract_links("[[my-note|My Note]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "my-note");
    }

    #[test]
    fn extracts_link_with_anchor() {
        // [[target#heading]] — only target is captured
        let links = extract_links("[[my-note#section]]");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "my-note");
    }

    #[test]
    fn returns_empty_when_no_links() {
        let links = extract_links("No links here. Just text.");
        assert!(links.is_empty());
    }

    #[test]
    fn link_line_numbers_are_correct() {
        let content = "line one\n[[note-a]]\nline three\n[[note-b]]";
        let links = extract_links(content);
        assert_eq!(links[0].line, 2);
        assert_eq!(links[1].line, 4);
    }

    // ── extract_tags — inline ─────────────────────────────────────────────────

    #[test]
    fn extracts_inline_tag() {
        let tags = extract_tags("Some text with #rust in it.");
        assert!(tags.iter().any(|t| t.tag == "rust"));
    }

    #[test]
    fn does_not_extract_markdown_heading_as_tag() {
        let tags = extract_tags("# My Heading\nsome text");
        assert!(tags.is_empty());
    }

    #[test]
    fn does_not_extract_heading_level_two_as_tag() {
        let tags = extract_tags("## Section\nsome text");
        assert!(tags.is_empty());
    }

    #[test]
    fn extracts_multiple_inline_tags() {
        let tags = extract_tags("Note tagged #rust and #productivity today.");
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"rust"));
        assert!(names.contains(&"productivity"));
    }

    // ── extract_tags — frontmatter ────────────────────────────────────────────

    #[test]
    fn extracts_frontmatter_tags_inline_array() {
        let content = "---\ntags: [rust, productivity]\n---\n\nbody";
        let tags = extract_tags(content);
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"rust"));
        assert!(names.contains(&"productivity"));
    }

    #[test]
    fn extracts_frontmatter_tags_block_list() {
        let content = "---\ntags:\n  - rust\n  - reading\n---\n\nbody";
        let tags = extract_tags(content);
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"rust"));
        assert!(names.contains(&"reading"));
    }

    #[test]
    fn extracts_frontmatter_tags_with_slash_hierarchy() {
        let content = "---\ntags:\n  - type/hub\n  - status/processed\n---\n\nbody";
        let tags = extract_tags(content);
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"type/hub"));
        assert!(names.contains(&"status/processed"));
    }

    #[test]
    fn extracts_frontmatter_tags_when_other_keys_precede_tags() {
        let content = indoc!(
            "---
            title: \"2026\"
            created: 2026-01-22
            slug: \"2026\"
            tags:
              - type/hub
              - status/processed
              - source/periodic-notes
              - category/yearly
            ---

            body text"
        );
        let tags = extract_tags(content);
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"type/hub"), "should extract type/hub");
        assert!(names.contains(&"status/processed"), "should extract status/processed");
        assert!(names.contains(&"source/periodic-notes"), "should extract source/periodic-notes");
        assert!(names.contains(&"category/yearly"), "should extract category/yearly");
        // Other frontmatter keys must not be extracted as tags
        assert!(!names.contains(&"2026"), "title value should not be a tag");
    }

    #[test]
    fn extracts_frontmatter_tags_when_other_keys_follow_tags() {
        let content = "---\ntags:\n  - type/hub\n  - status/processed\ndate: 2026-01-22\n---\n\nbody";
        let tags = extract_tags(content);
        let names: Vec<&str> = tags.iter().map(|t| t.tag.as_str()).collect();
        assert!(names.contains(&"type/hub"));
        assert!(names.contains(&"status/processed"));
        assert_eq!(names.len(), 2, "date key should not be treated as a tag");
    }

    #[test]
    fn extracts_inline_tag_with_slash_hierarchy() {
        let tags = extract_tags("Some text with #type/hub in it.");
        assert!(tags.iter().any(|t| t.tag == "type/hub"));
    }

    #[test]
    fn returns_empty_for_empty_content() {
        assert!(extract_tags("").is_empty());
        assert!(extract_links("").is_empty());
    }
}
