//! Parallel parse stage: paths → Document values.

use crate::model::Document;
use crate::parse::parse_document_text;
use rayon::prelude::*;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Resolve parallel job count from `ODS_JOBS` (or legacy `ODC_JOBS`, positive integer) or rayon default.
pub fn parse_pool_jobs() -> Option<usize> {
    if std::env::var("ODS_LOW_MEMORY").as_deref() == Ok("1") {
        return Some(1);
    }
    std::env::var("ODS_JOBS")
        .or_else(|_| std::env::var("ODC_JOBS"))
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
}

fn is_heading_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("### ")
        || trimmed.starts_with("## ")
        || (trimmed.starts_with("# ") && !trimmed.starts_with("## "))
}

/// Stream-read a markdown file for graph mode: frontmatter + heading lines only (no full body).
fn read_graph_parse_text(path: &Path) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut out = String::new();

    let mut first = String::new();
    reader.read_line(&mut first)?;
    let first_trim = first.trim_end_matches('\r').trim_end();

    if first_trim != "---" {
        if is_heading_line(first_trim) {
            out.push_str(first_trim);
            out.push('\n');
        }
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim_end_matches('\r');
            if is_heading_line(trimmed) {
                out.push_str(trimmed);
                out.push('\n');
            }
        }
        return Ok(out);
    }

    out.push_str(&first);
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        out.push_str(&line);
        if line.trim_end_matches('\r').trim_end() == "---" && out.matches("---").count() >= 2 {
            break;
        }
    }

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim_end_matches('\r');
        if is_heading_line(trimmed) {
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    Ok(out)
}

/// Read and parse a single Markdown path.
///
/// Graph mode (`include_body == false`) streams frontmatter and `#`/`##`/`###` lines only —
/// never loads full note bodies into RAM.
pub fn parse_path(root: &Path, path: PathBuf, include_body: bool) -> io::Result<Document> {
    if include_body {
        let text = fs::read_to_string(&path)?;
        return Ok(parse_document_text(root, path, &text, true));
    }
    let text = read_graph_parse_text(&path)?;
    Ok(parse_document_text(root, path, &text, false))
}

/// Parse many paths in parallel (order-preserving). Honors `ODS_JOBS` / `ODS_LOW_MEMORY`.
pub fn parse_paths_parallel(
    root: &Path,
    paths: &[PathBuf],
    include_body: bool,
) -> io::Result<Vec<Document>> {
    let root = root.to_path_buf();
    let run = || {
        paths
            .par_iter()
            .map(|path| parse_path(&root, path.clone(), include_body))
            .collect::<Result<Vec<_>, _>>()
    };

    match parse_pool_jobs() {
        Some(n) => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .map_err(|e| io::Error::other(e.to_string()))?;
            pool.install(run)
        }
        None => run(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn graph_mode_skips_large_body_allocation() {
        let td = tempdir().unwrap();
        let path = td.path().join("big.md");
        let mut body = String::from("---\nprofile: note\nstatus: draft\n---\n\n## Section\n\n");
        body.push_str(&"x".repeat(512 * 1024));
        fs::write(&path, &body).unwrap();
        let doc = parse_path(td.path(), path.clone(), false).unwrap();
        assert!(doc.body.is_empty());
        assert!(doc.headings.iter().any(|h| h.contains("Section")));
    }
}
