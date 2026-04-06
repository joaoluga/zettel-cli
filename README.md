# zettel-cli

A Zettelkasten/Obsidian note manager for the terminal. Create notes from
[MiniJinja](https://docs.rs/minijinja) templates, open random notes for
review, search by filename, tag, or wikilink, and define reusable presets
for periodic notes (daily, weekly, etc.).

---

## Installation

```bash
cargo install --path .
```

---

## Commands

### `lucky` — open a random note

Walks a directory recursively, picks a random `.md` file, and opens it.

```
zettel-cli lucky [OPTIONS]

Options:
  -p, --path <PATH>              Directory to search (defaults to [general].notes_path)
  -f, --file-reader <PROGRAM>    Editor/viewer to open the note (defaults to nvim)
```

```bash
zettel-cli lucky --path ~/notes --file-reader nvim
```

---

### `new` — create a note from a template

Renders a MiniJinja template, writes it to a new `.md` file (filename is
automatically slugified), and opens it with your editor.

```
zettel-cli new [OPTIONS] <TITLE>

Arguments:
  <TITLE>    Title of the note (used as {{ title }} in the template)

Options:
  -f, --file-reader <PROGRAM>     Editor to open the note (defaults to [general].file_reader or nvim)
  -t, --template-path <FILE>      MiniJinja template file (defaults to [general].default_template_path)
  -T, --target-path <DIR>         Directory for the new note (defaults to [general].default_target_path)
      --dry-run                   Print rendered content to stdout without writing or opening
```

```bash
# With all flags
zettel-cli new --template-path ~/.config/zettel-cli/note.md \
               --target-path ~/notes/inbox \
               --file-reader nvim \
               "My Note Title"

# Using config defaults
zettel-cli new "My Note Title"

# Preview without writing
zettel-cli new --dry-run "My Note Title"
```

The output filename is the slugified title. `"My Note Title"` → `my-note-title.md`.
The `{{ title }}` variable in the template keeps the original, un-slugified value.

---

### `preset` — create a note using a saved preset

Uses the paths and settings defined under `[preset.<name>]` in the config
file. Designed for recurring note types such as daily journals or weekly
reviews.

```
zettel-cli preset [OPTIONS] <PRESET_NAME>

Arguments:
  <PRESET_NAME>    Name of the preset (matches [preset.<name>] in config)

Options:
  -t, --title <TITLE>             Note title (overrides preset default_title)
  -f, --file-reader <PROGRAM>     Override the editor
```

```bash
# Create today's daily note (title comes from preset's default_title)
zettel-cli preset daily

# Create with an explicit title
zettel-cli preset daily --title "2026-03-02"

# Create a weekly note
zettel-cli preset weekly
```

---

### `search` — find notes by filename, tag, or wikilink

A pure data command: walks the notes directory, collects candidates, and writes
results to stdout. No editor is opened. Pipe the output to `fzf`, telescope,
snacks, or any other tool.

Exactly one `--by-*` flag is required.

```
zettel-cli search [OPTIONS] <--by-filename [FILTER] | --by-tag [FILTER] | --by-link <NOTE> | --by-backlink <NOTE>>

Options:
      --by-filename [FILTER]    List .md files; optional substring filter on the path
      --by-tag [FILTER]         List tag-file pairs; optional substring filter on the tag name
      --by-link <NOTE>          List files that NOTE links to (outgoing wikilinks)
      --by-backlink <NOTE>      List files that contain a [[NOTE]] wikilink
  -f, --format <FORMAT>         Output format: plain (default) or json
  -p, --path <PATH>             Notes root directory (defaults to [general].notes_path)
```

Output is always **full file paths**, making it safe to pipe directly into
`xargs`, `nvim`, or fzf's `--preview`.

#### Tag conventions supported

- YAML frontmatter block list:
  ```yaml
  tags:
    - type/hub
    - status/processed
  ```
- YAML frontmatter inline array: `tags: [rust, productivity]`
- Inline hashtags anywhere in the body: `#rust`, `#type/hub`

Slash-separated hierarchical tags (`type/hub`) are preserved as-is, so
`--by-tag type` will match `type/hub`, `type/daily`, etc.

#### Output formats

**plain** (default) — one record per line, tab-separated where applicable:

```
# --by-filename / --by-backlink / --by-link
/home/user/notes/inbox/my-note.md

# --by-tag
#rust	/home/user/notes/inbox/my-note.md
```

**json** — array of objects with full path and metadata:

```json
[{ "tag": "rust", "file": "/home/user/notes/inbox/my-note.md", "line": 3 }]
```

#### Usage examples

```bash
# Browse all notes with fzf + bat preview
zettel-cli search --by-filename | fzf --preview 'bat --color=always {}'

# Browse all notes in the daily folder
zettel-cli search --by-filename daily | fzf --preview 'bat --color=always {}'

# Find notes by tag, show tag in list, preview file
zettel-cli search --by-tag \
  | fzf --delimiter '\t' --preview 'bat --color=always {2}'

# Filter to a specific tag prefix, open selected note
zettel-cli search --by-tag type \
  | fzf --delimiter '\t' --preview 'bat --color=always {2}' \
  | awk -F'\t' '{print $2}' \
  | xargs nvim

# Show all backlinks to a note
zettel-cli search --by-backlink my-note

# Show all outgoing links from a note
zettel-cli search --by-link my-note

# JSON output for use in Neovim telescope/snacks
zettel-cli search --by-tag --format json | jq -r '.[] | .file'
```

#### Boolean search with fzf

The optional FILTER in `--by-filename` and `--by-tag` is a simple
case-insensitive substring pre-filter applied before output. For boolean
logic, pipe to fzf and use its extended search syntax:

| fzf query              | Meaning |
| ---------------------- | ------- |
| `rust productivity`    | AND     |
| `rust \| productivity` | OR      |
| `!rust`                | NOT     |

```bash
zettel-cli search --by-tag | fzf --query "type !archived"
```

---

### `completions` — generate shell completion scripts

```bash
# Bash
zettel-cli completions bash >> ~/.bash_completion

# Zsh
zettel-cli completions zsh > "${fpath[1]}/_zettel-cli"

# Fish
zettel-cli completions fish > ~/.config/fish/completions/zettel-cli.fish
```

---

## Global flag

```
zettel-cli --config <FILE> <SUBCOMMAND>
```

Override the default config path (`~/.config/zettel-cli/config.toml`).

---

## Config file

Default location: `~/.config/zettel-cli/config.toml`

All fields are optional. If the file does not exist, built-in defaults are used.
Paths support `~` and environment variables (`$HOME`, `${XDG_DATA_HOME}`, etc.).

```toml
[general]
# Default notes directory for the `lucky` command
notes_path = "~/notes"

# Editor/viewer opened after creating or selecting a note (default: nvim)
file_reader = "nvim"

# Fallback target directory for the `new` command
default_target_path = "~/notes/inbox"

# Fallback template file for the `new` command
default_template_path = "~/.config/zettel-cli/templates/note.md"

# Default strftime date format used in templates (default: %Y-%m-%d)
date_format = "%Y-%m-%d"
```

### Presets

A preset bundles a template and a target directory under a single name.
Invoke it with `zettel-cli preset <name>`.

```toml
[preset.daily]
template_path  = "~/.config/zettel-cli/templates/daily.md"
target_path    = "~/notes/periodic-notes/daily"
# Optional: MiniJinja expression that generates the title when --title is omitted.
# All date context variables are available here.
default_title  = "{{ date }}"
# Optional: override the global date_format for this preset only
date_format    = "%Y-%m-%d"

[preset.weekly]
template_path  = "~/.config/zettel-cli/templates/weekly.md"
target_path    = "~/notes/periodic-notes/weekly"
default_title  = "{{ date }}"
date_format    = "%Y-W%V"    # e.g. "2026-W09"
```

### Search defaults

```toml
[search]
# Default output format for the `search` command (default: plain)
default_format = "plain"
```

**Priority for `default_format`:** `--format` flag → `[search].default_format` → `plain`

**Priority for `file_reader`:** CLI flag → `[general].file_reader` → `nvim`

**Priority for `date_format`:** `[preset.<name>].date_format` → `[general].date_format` → `%Y-%m-%d`

---

## Template variables

Templates are [MiniJinja](https://docs.rs/minijinja) (Jinja2-compatible)
files. The following variables are injected at render time:

| Variable    | Type   | Description                                                     |
| ----------- | ------ | --------------------------------------------------------------- |
| `title`     | string | The note title passed via `--title` or as positional argument   |
| `date`      | string | Today's date formatted with `date_format`                       |
| `yesterday` | string | Yesterday's date formatted with `date_format`                   |
| `tomorrow`  | string | Tomorrow's date formatted with `date_format`                    |
| `year`      | int    | Current year (e.g. `2026`)                                      |
| `month`     | int    | Current month (1–12)                                            |
| `day`       | int    | Current day of the month (1–31)                                 |
| `weekday`   | string | Full weekday name (e.g. `"Monday"`)                             |
| `tz_offset` | string | Local UTC offset (e.g. `"+01:00"`)                              |
| `now_iso`   | string | Local datetime as RFC 3339 (e.g. `"2026-03-02T09:00:00+01:00"`) |
| `utc_iso`   | string | UTC datetime as RFC 3339 (e.g. `"2026-03-02T08:00:00+00:00"`)   |

### `| slug` filter

Converts a string to a URL-safe, lowercase, hyphen-separated slug.

```
{{ title | slug }}        {# "Hello World" → "hello-world" #}
{{ "My Note" | slug }}    {# → "my-note" #}
```

### `date_format`

Controls the output of `date`, `yesterday`, and `tomorrow`. Uses
[strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
notation.

| Format string | Example output   |
| ------------- | ---------------- |
| `%Y-%m-%d`    | `2026-03-02`     |
| `%d/%m/%Y`    | `02/03/2026`     |
| `%Y-W%V`      | `2026-W09`       |
| `%B %d, %Y`   | `March 02, 2026` |

---

## Example templates

### Daily note (`~/.config/zettel-cli/templates/daily.md`)

```markdown
---
title: { { title } }
date: { { date } }
tags: [daily]
---

## {{ date }} — {{ weekday }}

### Tasks

- [ ]

### Notes
```

### Weekly review (`~/.config/zettel-cli/templates/weekly.md`)

```markdown
---
title: Week {{ date }}
date: { { date } }
tags: [weekly]
---

## Week {{ date }}

**Period:** {{ yesterday }} ← today

### Highlights

### Retrospective

### Goals for next week
```

### Note with slug and timestamps

```markdown
---
title: { { title } }
date: { { date } }
slug: { { title | slug } }
---

# {{ title }}

Created: {{ now_iso }}

## Content
```

---

## Project structure

```
src/
  main.rs               CLI definitions (clap) and dispatch
  lib.rs                Re-exports all library modules
  commands/
    lucky.rs            lucky command
    new.rs              new command
    preset.rs           preset command
    search.rs           search command
  config/
    mod.rs              Config structs, load_config, expand_path
    resolver.rs         CLI + config merging (resolve_general, resolve_new, resolve_search)
  templates/
    context.rs          render_template, render_title (MiniJinja)
  utils/
    fs.rs               is_markdown, collect_md_files
    parse.rs            extract_tags, extract_links
tests/
  new_test.rs           Integration tests for `new`
  preset_test.rs        Integration tests for `preset`
  search_test.rs        Integration tests for `search`
```

---

## Next Features

- [ ] Implement logging system and meaningful loggings for debugging purposes
- [ ] Implement command `stats` to gather analytics information regarding the notes and save as json.
  - E.g:
    - Notes per directory
    - 10 Last Created Notes
    - 10 Last Modified Notes
    - Total Notes on Inbox
    - Total Notes by Tag
- [ ] `stats --pretty-print` to display stats beautifully :)
- [ ] Implement command `review` to print out all notes in the inbox
- [ ] Add extra options to `lucky` like `--by-tag` or `--target-directory`

---
