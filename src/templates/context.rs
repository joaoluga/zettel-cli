use anyhow::{Context, Result};
use chrono::{Datelike, Duration, Local, Utc};
use minijinja::{Environment, context};

/// Render a MiniJinja template string with the standard zettel-cli context
/// variables (`title`, `date`, `year`, `month`, `day`, `yesterday`, `tomorrow`,
/// `weekday`, `tz_offset`, `now_iso`, `utc_iso`) and the `| slug` filter.
/// `date_format` is a strftime format string (e.g. `"%Y-%m-%d"`); defaults to
/// `"%Y-%m-%d"` when `None`.
pub fn render_template(template_src: &str, title: &str, date_format: Option<&str>) -> Result<String> {
    let fmt = date_format.unwrap_or("%Y-%m-%d");

    let now_local = Local::now();
    let now_utc = Utc::now();
    let today = now_local.date_naive();
    let yesterday = today - Duration::days(1);
    let tomorrow = today + Duration::days(1);

    let mut env = Environment::new();

    env.add_template("note", template_src)
        .context("failed to add template to minijinja environment")?;

    env.add_filter("slug", |s: String| -> Result<String, minijinja::Error> {
        Ok(slug::slugify(&s))
    });

    let tmpl = env
        .get_template("note")
        .context("failed to get template from minijinja environment")?;

    tmpl.render(context! {
        title     => title,
        date      => now_local.format(fmt).to_string(),
        year      => now_local.year(),
        month     => now_local.month(),
        day       => now_local.day(),
        yesterday => yesterday.format(fmt).to_string(),
        tomorrow  => tomorrow.format(fmt).to_string(),
        weekday   => now_local.format("%A").to_string(),
        tz_offset => now_local.format("%:z").to_string(),
        now_iso   => now_local.to_rfc3339(),
        utc_iso   => now_utc.to_rfc3339(),
    })
    .context("failed to render template")
}

/// Render a MiniJinja title template (e.g. `"{{ date }}"`) using the date
/// context variables. The `title` variable inside the template will be empty
/// because we are constructing the title itself.
pub fn render_title(title_template: &str, date_format: Option<&str>) -> Result<String> {
    render_template(title_template, "", date_format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_inserts_title() {
        let output = render_template("# {{ title }}", "My Note", None).unwrap();
        assert_eq!(output, "# My Note");
    }

    #[test]
    fn render_slug_filter() {
        let output = render_template("{{ title | slug }}", "Hello World", None).unwrap();
        assert_eq!(output, "hello-world");
    }

    #[test]
    fn render_contains_date_field() {
        let output = render_template("{{ date }}", "any title", None).unwrap();
        // YYYY-MM-DD has exactly 2 dashes and length 10
        assert_eq!(
            output.len(),
            10,
            "expected date of length 10 (YYYY-MM-DD), got: {output}"
        );
        assert_eq!(
            output.chars().filter(|c| *c == '-').count(),
            2,
            "expected exactly 2 dashes in date, got: {output}"
        );
    }

    #[test]
    fn render_contains_year_field() {
        let output = render_template("{{ year }}", "any title", None).unwrap();
        assert!(
            output.starts_with("20"),
            "expected year to start with '20' (current century), got: {output}"
        );
    }

    #[test]
    fn render_contains_yesterday_field() {
        let output = render_template("{{ yesterday }}", "any title", None).unwrap();
        // YYYY-MM-DD has exactly 2 dashes and length 10
        assert_eq!(
            output.len(),
            10,
            "expected yesterday of length 10 (YYYY-MM-DD), got: {output}"
        );
        assert_eq!(
            output.chars().filter(|c| *c == '-').count(),
            2,
            "expected exactly 2 dashes in yesterday, got: {output}"
        );
    }

    #[test]
    fn render_contains_tomorrow_field() {
        let output = render_template("{{ tomorrow }}", "any title", None).unwrap();
        // YYYY-MM-DD has exactly 2 dashes and length 10
        assert_eq!(
            output.len(),
            10,
            "expected tomorrow of length 10 (YYYY-MM-DD), got: {output}"
        );
        assert_eq!(
            output.chars().filter(|c| *c == '-').count(),
            2,
            "expected exactly 2 dashes in tomorrow, got: {output}"
        );
    }

    #[test]
    fn render_invalid_template_returns_error() {
        let result = render_template("{% if %}", "any title", None);
        assert!(result.is_err(), "expected Err for invalid template syntax");
    }

    #[test]
    fn render_empty_template() {
        let output = render_template("", "any title", None).unwrap();
        assert_eq!(output, "");
    }
}
