// Self-update for `ods` from GitHub Release artifacts.
//
// Downloads multi-OS packages published by `.github/workflows/release.yml`,
// verifies SHA256, and replaces the `ods` binary next to the running executable.

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_BASE: &str = "https://api.github.com/repos/open-doc-spec/ods";
const API_LATEST: &str =
    "https://api.github.com/repos/open-doc-spec/ods/releases/latest";
const USER_AGENT: &str = concat!("ods/", env!("CARGO_PKG_VERSION"));
const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct UpdateOptions {
    pub check_only: bool,
    pub force: bool,
    /// Exact tag e.g. `v0.1.5`; None = latest stable.
    pub version: Option<String>,
}

#[derive(Debug)]
pub enum UpdateOutcome {
    UpToDate { current: String, remote: String },
    Available { current: String, remote: String },
    Updated { from: String, to: String },
}

pub fn run_update(opts: UpdateOptions) -> Result<UpdateOutcome, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let target = host_target()?;
    let remote_tag = match &opts.version {
        Some(v) => normalize_tag(v),
        None => fetch_latest_tag()?,
    };
    let remote_ver = strip_v(&remote_tag);

    if !opts.force && cmp_semver(&current, &remote_ver) != std::cmp::Ordering::Less {
        return Ok(UpdateOutcome::UpToDate {
            current,
            remote: remote_ver,
        });
    }

    if opts.check_only {
        return Ok(UpdateOutcome::Available {
            current,
            remote: remote_ver,
        });
    }

    let prefix = install_prefix()?;
    install_release(&remote_tag, &target, &prefix)?;
    write_state_after_update(&remote_tag);

    Ok(UpdateOutcome::Updated {
        from: current,
        to: remote_ver,
    })
}

/// Best-effort auto-update (24h throttle). Never returns Err to callers for soft path.
pub fn maybe_auto_update() {
    maybe_auto_update_inner(false);
}

/// Like [`maybe_auto_update`], but always checks once (used when starting `ods watch`).
/// Still respects `ODS_AUTO_UPDATE=0` and soft-fails on network errors.
pub fn maybe_auto_update_on_watch() {
    maybe_auto_update_inner(true);
}

fn maybe_auto_update_inner(force_check: bool) {
    if !auto_update_enabled() {
        return;
    }
    if !force_check && !should_check_now() {
        return;
    }
    // Mark check time first so flaky networks don't hammer the API every command.
    let _ = touch_check_time();

    if force_check {
        eprintln!("ods: checking for updates…");
    }

    match run_update(UpdateOptions {
        check_only: false,
        force: false,
        version: None,
    }) {
        Ok(UpdateOutcome::Updated { from, to }) => {
            eprintln!("ods: updated {from} → {to}");
            if force_check {
                eprintln!(
                    "ods: restart `ods watch` / `ods start` to use the new binary in long-running processes"
                );
            }
        }
        Ok(UpdateOutcome::UpToDate { current, remote }) => {
            if force_check {
                eprintln!("ods: up to date ({current}, latest {remote})");
            }
        }
        Ok(UpdateOutcome::Available { .. }) => {}
        Err(err) => {
            // Soft: never break lint/index/watch. Keep one line; full `ods update` uses Next:.
            eprintln!("ods: auto-update skipped — {err} (run `ods update` to retry)");
        }
    }
}

fn auto_update_enabled() -> bool {
    for key in ["ODS_AUTO_UPDATE", "ODC_AUTO_UPDATE"] {
        if let Ok(v) = env::var(key) {
            let v = v.trim().to_ascii_lowercase();
            return !(v == "0" || v == "false" || v == "no" || v == "off");
        }
    }
    true
}

/// Short release asset id: `{os}-{arch}` (e.g. `linux-arm64`, `macos-x86_64`).
/// Matches package names from `.github/workflows/release.yml`.
pub fn host_target() -> Result<String, String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => Ok("linux-x86_64".into()),
        ("linux", "aarch64") => Ok("linux-arm64".into()),
        ("macos", "aarch64") => Ok("macos-arm64".into()),
        ("macos", "x86_64") => Ok("macos-x86_64".into()),
        ("windows", "x86_64") => Ok("windows-x86_64".into()),
        ("windows", "aarch64") => Ok("windows-arm64".into()),
        _ => Err(ods_core::error::update_unsupported_platform(os, arch)),
    }
}

fn is_windows_target(target: &str) -> bool {
    target.starts_with("windows") || target.contains("windows")
}

fn github_token() -> Option<String> {
    env::var("GH_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

fn normalize_tag(v: &str) -> String {
    let v = v.trim();
    if v.starts_with('v') || v.starts_with('V') {
        format!("v{}", strip_v(v))
    } else {
        format!("v{v}")
    }
}

fn strip_v(v: &str) -> String {
    v.trim()
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

/// Compare dotted numeric semver (major.minor.patch only).
pub fn cmp_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_semver(a);
    let pb = parse_semver(b);
    pa.cmp(&pb)
}

fn parse_semver(v: &str) -> (u64, u64, u64) {
    let v = strip_v(v);
    let mut parts = v.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (major, minor, patch)
}

fn fetch_latest_tag() -> Result<String, String> {
    let body = http_get_string(API_LATEST)?;
    // Minimal JSON parse: "tag_name": "v0.1.5"
    let key = "\"tag_name\"";
    let Some(idx) = body.find(key) else {
        return Err("could not parse latest release (no tag_name)".into());
    };
    let rest = &body[idx + key.len()..];
    let Some(start) = rest.find('"') else {
        return Err("could not parse tag_name value".into());
    };
    let rest = &rest[start + 1..];
    let Some(end) = rest.find('"') else {
        return Err("could not parse tag_name value".into());
    };
    let tag = rest[..end].trim();
    if tag.is_empty() {
        return Err("empty tag_name from GitHub API".into());
    }
    Ok(normalize_tag(tag))
}

fn apply_auth(req: ureq::Request, accept: &str) -> ureq::Request {
    let req = req.set("User-Agent", USER_AGENT).set("Accept", accept);
    match github_token() {
        Some(token) => req.set("Authorization", &format!("Bearer {token}")),
        None => req,
    }
}
