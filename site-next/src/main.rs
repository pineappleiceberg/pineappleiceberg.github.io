use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const ORIGIN: &str = "https://amrishhallberg.com";

#[derive(Debug, Deserialize)]
struct Page {
    title: String,
    description: String,
    permalink: String,
    date: Option<String>,
    script: Option<String>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn parse_page(input: &str) -> Result<(Page, &str), Box<dyn Error>> {
    let rest = input
        .strip_prefix("+++\n")
        .ok_or_else(|| invalid("page must begin with TOML front matter: +++"))?;
    let (header, body) = rest
        .split_once("\n+++\n")
        .ok_or_else(|| invalid("missing closing TOML front matter: +++"))?;
    let page: Page = toml::from_str(header)?;
    page_path(&page.permalink)?;
    if page.title.trim().is_empty() || page.description.trim().is_empty() {
        return Err(invalid("title and description are mandatory").into());
    }
    if let Some(script) = &page.script {
        if !script.starts_with("/assets/") || !script.ends_with(".js")
            || script.contains("..") || script.contains('\\') || script.contains('"')
        {
            return Err(invalid("script must be a local /assets/*.js file").into());
        }
    }
    Ok((page, body))
}

fn page_path(permalink: &str) -> Result<PathBuf, io::Error> {
    if !permalink.starts_with('/') || permalink.contains("..")
        || permalink.contains('\\') || permalink.contains('?') || permalink.contains('#')
    {
        return Err(invalid("permalink must be a safe absolute URL path"));
    }
    let clean = permalink.trim_matches('/');
    if clean.is_empty() {
        return Ok(PathBuf::from("index.html"));
    }
    for segment in clean.split('/') {
        if segment.is_empty()
            || !segment.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
            || segment.starts_with('.')
        {
            return Err(invalid("invalid path segment in permalink"));
        }
    }
    if permalink.ends_with('/') {
        Ok(PathBuf::from(clean).join("index.html"))
    } else if permalink.ends_with(".html") {
        Ok(PathBuf::from(clean))
    } else {
        Err(invalid("permalink must end with / or .html"))
    }
}

fn escape_html(value: &str) -> String {
    let mut s = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            '\'' => s.push_str("&#39;"),
            _ => s.push(ch),
        }
    }
    s
}

fn render(page: &Page, markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, opts);
    let mut article = String::new();
    html::push_html(&mut article, parser);

    let title = escape_html(&page.title);
    let description = escape_html(&page.description);
    let canonical = escape_html(&format!("{ORIGIN}{}", page.permalink));
    let meta_date = page.date.as_deref().map(|d| {
        format!("<meta property=\"article:published_time\" content=\"{}\">\n", escape_html(d))
    }).unwrap_or_default();
    let script = page.script.as_ref().map(|s| {
        format!("<script type=\"module\" src=\"{}\"></script>\n", escape_html(s))
    }).unwrap_or_default();

    format!(r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · Amrish Hallberg</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{canonical}">
<meta property="og:type" content="website">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{canonical}">
<meta name="twitter:card" content="summary">
{meta_date}<link rel="stylesheet" href="/assets/site.css">
</head>
<body>
<a class="skip-link" href="#main">Skip to content</a>
<header class="site-header"><div class="wrap"><a class="brand" href="/">Amrish Hallberg</a>
<nav aria-label="Primary"><a href="/projects/">Projects</a><a href="/experiments/bitset-overlap/">Experiments</a><a href="https://github.com/pineappleiceberg">GitHub</a></nav></div></header>
<main id="main" class="wrap prose">{article}</main>
<footer class="site-footer"><div class="wrap">Built as static HTML; optional C/WebAssembly experiments.</div></footer>
{script}</body></html>
"##)
}

fn visit_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_markdown(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else if src.is_file() {
            fs::copy(src, dst)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dist = root.join("dist");
    // Deliberately fixed output directory: never delete an arbitrary CLI path.
    if dist.exists() {
        fs::remove_dir_all(&dist)?;
    }
    fs::create_dir_all(&dist)?;
    let mut markdown_files = Vec::new();
    visit_markdown(&root.join("content"), &mut markdown_files)?;
    markdown_files.sort();
    if markdown_files.is_empty() {
        return Err(invalid("no Markdown pages found").into());
    }
    let mut seen = HashSet::new();
    let mut count = 0usize;
    for input in markdown_files {
        let raw = fs::read_to_string(&input)?;
        let (page, markdown) = parse_page(&raw)?;
        let relative = page_path(&page.permalink)?;
        if !seen.insert(relative.clone()) {
            return Err(invalid(format!("duplicate permalink: {}", page.permalink)).into());
        }
        let destination = dist.join(relative);
        fs::create_dir_all(destination.parent().ok_or_else(|| invalid("invalid output path"))?)?;
        fs::write(destination, render(&page, markdown))?;
        count += 1;
    }
    copy_tree(&root.join("assets"), &dist.join("assets"))?;
    println!("Built {count} static pages into {}", dist.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_paths() {
        assert!(page_path("/../index.html").is_err());
        assert!(page_path("/foo/../../bar/").is_err());
        assert!(page_path("/foo?x=1/").is_err());
        assert!(page_path("foo/").is_err());
    }

    #[test]
    fn valid_paths_keep_expected_urls() {
        assert_eq!(page_path("/").unwrap(), PathBuf::from("index.html"));
        assert_eq!(page_path("/projects/").unwrap(), PathBuf::from("projects/index.html"));
        assert_eq!(page_path("/2026/11/06/example.html").unwrap(), PathBuf::from("2026/11/06/example.html"));
    }

    #[test]
    fn metadata_is_escaped() {
        assert_eq!(escape_html("<a & \"b\">"), "&lt;a &amp; &quot;b&quot;&gt;");
    }

    #[test]
    fn normal_pages_have_no_js() {
        let page = Page {
            title: "Test".into(),
            description: "Description".into(),
            permalink: "/test/".into(),
            date: None,
            script: None,
        };
        let html = render(&page, "# Heading\n\nHello.");
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(!html.contains("<script"));
        assert!(html.contains("https://amrishhallberg.com/test/"));
    }
}
