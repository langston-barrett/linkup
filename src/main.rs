use std::collections::HashSet;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use pulldown_cmark::{Event, Options, Parser as MarkdownParser, Tag, TagEnd};

#[cfg(test)]
mod test;

/// Automatically add links to Markdown files
#[derive(Parser)]
#[command(name = "linkup")]
struct Args {
    /// Markdown files to process
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    for file in &args.files {
        process_file(file)?;
    }
    Ok(())
}

fn process_file(path: &Path) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let parent_dir = path
        .parent()
        .with_context(|| format!("File {} has no parent directory", path.display()))?;
    let modified = add_links(&content, parent_dir, |p: &Path| Ok(p.exists()))
        .with_context(|| format!("Failed to add links to {}", path.display()))?;
    if modified != content {
        fs::write(path, modified).with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(())
}

fn find_bracket_pair(text: &str, start: usize) -> Option<Range<usize>> {
    let bracket_pos = text[start..].find('[')?;
    let open = start + bracket_pos;
    text[open..].find(']').map(|close| open..open + close + 1)
}

fn in_code_block(before: &str) -> bool {
    let options = Options::empty();
    let parser = MarkdownParser::new_ext(before, options);
    let mut result = false;
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                result = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                result = false;
            }
            _ => {}
        }
    }
    result
}

fn collect_reference_link_labels(content: &str) -> HashSet<&str> {
    let mut labels = HashSet::new();
    for line in content.lines() {
        let line = line.trim_end();
        if let Some(bracket_start) = line.find('[')
            && let Some(bracket_end) = line[bracket_start + 1..].find(']')
        {
            let bracket_end = bracket_start + 1 + bracket_end;
            if Some(&b':') == line.as_bytes().get(bracket_end + 1) {
                let label = &line[bracket_start + 1..bracket_end];
                labels.insert(label);
            }
        }
    }
    labels
}

pub(crate) fn add_links(
    content: &str,
    base_dir: &Path,
    mut exists: impl FnMut(&Path) -> Result<bool>,
) -> Result<String> {
    let reference_labels = collect_reference_link_labels(content);
    let mut result = content.to_string();
    let mut search_start = 0;

    loop {
        let Some(range) = find_bracket_pair(&result, search_start) else {
            break;
        };
        if range.is_empty()
            || range.start.saturating_add(1) >= range.end
            || result.as_bytes().get(range.end) == Some(&b'(')
            || result.as_bytes().get(range.end) == Some(&b'[')
            || in_code_block(&result[..range.start])
        {
            search_start = range.end;
            continue;
        }

        let link_text = result[range.start + 1..range.end - 1].to_string();
        debug_assert!(!link_text.is_empty());

        if reference_labels.contains(link_text.as_str()) {
            search_start = range.end;
            continue;
        }

        let slug = link_text.replace(' ', "-").to_lowercase();
        let dest = base_dir.join(format!("{slug}.md"));
        if exists(&dest)? {
            let dest_name = dest
                .file_name()
                .and_then(|n| n.to_str())
                .with_context(|| "Invalid file name")?;
            result.replace_range(range.clone(), &format!("[{link_text}]({dest_name})"));
            search_start = range.start + link_text.len() + dest_name.len() + "[]()".len();
            continue;
        }
        search_start = range.end;
    }

    Ok(result)
}
