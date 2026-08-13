use crate::model::Workspace;
use crate::parse::{extract_headings, split_frontmatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdoptOptions {
    /// When true, write inferred frontmatter into documents that lack it.
    pub write: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdoptReport {
    pub scanned: usize,
    pub would_write: Vec<PathBuf>,
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

/// Scan a workspace and optionally draft minimal frontmatter for plain Markdown files.
pub fn adopt_workspace(workspace: &Workspace, options: AdoptOptions) -> io::Result<AdoptReport> {
    let mut report = AdoptReport {
        scanned: workspace.documents.len(),
        ..Default::default()
    };

    for document in &workspace.documents {
        let file_name = document
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if file_name == "index.md" {
            report.skipped.push(document.path.clone());
            continue;
        }

        if matches!(
            document.frontmatter,
            crate::model::FrontmatterState::Invalid(_)
        ) {
            report.skipped.push(document.path.clone());
            continue;
        }

        let text = match fs::read_to_string(&document.path) {
            Ok(t) => t,
            Err(_) => {
                report.skipped.push(document.path.clone());
                continue;
            }
        };

        let (existing_fm, _) = split_frontmatter(&text);
        let has_ods_key = existing_fm.is_some_and(|fm| {
            fm.lines().any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("ods:")
                    || trimmed
                        .split_once(':')
                        .is_some_and(|(k, _)| k.trim() == "ods")
            })
        });

        if has_ods_key {
            report.skipped.push(document.path.clone());
        } else {
            report.would_write.push(document.path.clone());
            if options.write {
                write_minimal_frontmatter(&document.path)?;
                report.written.push(document.path.clone());
            }
        }
    }

    Ok(report)
}

fn write_minimal_frontmatter(path: &Path) -> io::Result<()> {
    let text = fs::read_to_string(path)?;
    let (existing_fm, body) = split_frontmatter(&text);

    let profile = infer_profile(body);
    let drafted = if let Some(fm) = existing_fm {
        format!(
            "---\n{}\nods:\n  profile: {profile}\n  status: draft\n---\n\n{}",
            fm.trim(),
            body.trim_start()
        )
    } else {
        format!(
            "---\nods:\n  profile: {profile}\n  status: draft\n---\n\n{}",
            body.trim_start()
        )
    };

    fs::write(path, drafted)
}

fn infer_profile(body: &str) -> &'static str {
    let headings = extract_headings(body);
    let normalized: Vec<String> = headings
        .iter()
        .map(|h| {
            h.chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect()
        })
        .collect();

    let has = |candidates: &[&str]| candidates.iter().any(|c| normalized.iter().any(|h| h == c));

    if has(&[
        "task",
        "successcriteria",
        "failuremodes",
        "assumptions",
        "dependencies",
    ]) {
        "agent"
    } else if has(&[
        "goal",
        "objective",
        "scope",
        "requirements",
        "acceptancecriteria",
        "risks",
    ]) {
        "feature"
    } else if has(&["overview", "prerequisites", "steps", "troubleshooting"]) {
        "guide"
    } else if has(&["context", "decision", "alternatives", "consequences"]) {
        "decision"
    } else if has(&["rules", "exceptions"]) {
        "policy"
    } else if has(&["purpose", "validation", "rollback"]) {
        "sop"
    } else if has(&["request", "response", "errors", "examples", "endpoint"]) {
        "api"
    } else if has(&["attendees", "agenda", "actionitems"]) {
        "meeting"
    } else if has(&["qa", "questions", "answers", "faq"]) {
        "faq"
    } else {
        "note"
    }
}
