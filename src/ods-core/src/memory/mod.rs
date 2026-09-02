//! Process RSS sampling and soft memory budgets for graph/service paths.

use crate::model::Workspace;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;

/// Default soft RSS budget (MiB) for `ods serve` and memory regression tests.
pub const DEFAULT_MAX_RSS_MB: u64 = 10;

/// Current process RSS in kilobytes (platform-specific; `None` when unavailable).
#[must_use]
pub fn current_rss_kb() -> Option<u64> {
    current_rss_kb_impl()
}

#[cfg(target_os = "linux")]
fn current_rss_kb_impl() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("VmRSS:")?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

#[cfg(target_os = "windows")]
fn current_rss_kb_impl() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("INFO:") || text.is_empty() {
        return None;
    }
    let last_col = text.rsplit(',').next()?;
    let digits: String = last_col.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(target_os = "macos")]
fn current_rss_kb_impl() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn current_rss_kb_impl() -> Option<u64> {
    None
}

/// True when sampled RSS exceeds `max_rss_mb`.
#[must_use]
pub fn rss_over_budget(max_rss_mb: u64) -> bool {
    let Some(rss_kb) = current_rss_kb() else {
        return false;
    };
    rss_kb > max_rss_mb.saturating_mul(1024)
}

/// Drop retained markdown bodies from every loaded document (graph-mode hygiene).
pub fn strip_workspace_bodies(workspace: &mut Workspace) {
    for doc in &mut workspace.documents {
        doc.body.clear();
        doc.body.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_workspace_bodies_clears_retained_text() {
        use crate::model::{Document, FrontmatterState};
        use std::path::PathBuf;

        let mut ws = Workspace::empty(PathBuf::from("/ws"));
        ws.documents.push(Document {
            path: PathBuf::from("/ws/a.md"),
            directory: PathBuf::from("/ws"),
            body: "large body".into(),
            headings: vec![],
            frontmatter: FrontmatterState::Absent,
        });
        strip_workspace_bodies(&mut ws);
        assert!(ws.documents[0].body.is_empty());
    }
}
