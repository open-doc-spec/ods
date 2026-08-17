#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServeMode {
    Auto,
    Watch,
    Poll,
}

#[derive(Clone, Debug)]
struct ServeOptions {
    root: PathBuf,
    mode: ServeMode,
    memory_report: bool,
    poll_secs: u64,
}

fn serve_options_from_args(args: &[String]) -> Result<ServeOptions, CliError> {
    let mut root = None;
    let mut mode = env::var("ODS_SERVE_MODE")
        .ok()
        .map(|value| parse_serve_mode(&value))
        .transpose()?
        .unwrap_or(ServeMode::Auto);
    let mut memory_report = false;
    let mut poll_secs = env::var("ODS_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--root", "`ods serve --root .`")))?;
                root = Some(PathBuf::from(v));
                i += 2;
            }
            "--mode" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--mode", "`ods serve --mode auto|watch|poll`")))?;
                mode = parse_serve_mode(v)?;
                i += 2;
            }
            "--memory-report" => {
                memory_report = true;
                i += 1;
            }
            "--poll-secs" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--poll-secs", "`ods serve --poll-secs 10`")))?;
                poll_secs = v
                    .parse()
                    .map_err(|_| usage_msg(ods_core::missing_flag_value("--poll-secs", "`ods serve --poll-secs 10`")))?;
                i += 2;
            }
            other if !other.starts_with('-') => {
                root = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let path = root.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    Ok(ServeOptions {
        root: resolve_root_path(path),
        mode,
        memory_report,
        poll_secs: poll_secs.max(1),
    })
}

fn parse_serve_mode(value: &str) -> Result<ServeMode, CliError> {
    match value {
        "auto" => Ok(ServeMode::Auto),
        "watch" => Ok(ServeMode::Watch),
        "poll" => Ok(ServeMode::Poll),
        other => Err(usage_msg(ods_core::invalid_choice("--mode", other, "auto|watch|poll"))),
    }
}

fn resolved_serve_mode(mode: ServeMode) -> ServeMode {
    match mode {
        ServeMode::Auto if env::var("ODS_LOW_MEMORY").ok().as_deref() == Some("1") => {
            ServeMode::Poll
        }
        ServeMode::Auto => ServeMode::Watch,
        other => other,
    }
}

fn parse_export_args(args: &[String]) -> Result<(PathBuf, PathBuf, OutputFormat, String), CliError> {
    let mut out = None;
    let mut path = None;
    let mut format = OutputFormat::Text;
    let mut spec = "ods:0.1".to_string();

    let mut i = 2;
    // Skip optional "graph" subcommand token if present (e.g. ods export graph)
    if i < args.len() && (args[i] == "graph" || args[i] == "all") {
        i += 1;
    }

    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--out", "`ods export graph --out .ods/graph.md`")))?;
                out = Some(PathBuf::from(v));
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            "--format" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--format", "`--format text|json|md`")))?;
                format = match v.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "md" | "markdown" => OutputFormat::Text, // md triggers markdown file or text output
                    other => {
                        return Err(usage_msg(ods_core::invalid_choice(
                            "--format",
                            other,
                            "text|json|md",
                        )));
                    }
                };
                i += 2;
            }
            "--spec" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--spec", "`--spec ods` or `--spec okf`")))?;
                spec = match v.to_lowercase().as_str() {
                    "okf" | "okf:0.2" => "okf:0.2".to_string(),
                    _ => "ods:0.1".to_string(),
                };
                i += 2;
            }
            "--okf" => {
                spec = "okf:0.2".to_string();
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    // Default under `.ods/` so routine export does not pollute the workspace root
    // (agents reading root graph.md reintroduce full-workspace token burn).
    let out = out.unwrap_or_else(|| root.join(".ods").join("graph.md"));
    Ok((root, out, format, spec))
}

/// Parsed `ods share` arguments: `(workspace root, scope, out, include_org, include_private)`.
///
/// `scope` defaults to the discovered workspace root when `[path]` is omitted;
/// when given, it limits which documents are published without changing
/// where the workspace is loaded from (ancestor `share` cascades above
/// `scope` still apply).
fn parse_share_args(
    args: &[String],
) -> Result<(PathBuf, PathBuf, PathBuf, bool, bool), CliError> {
    let mut out = None;
    let mut path = None;
    let mut include_org = false;
    let mut include_private = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--out", "`ods share --out ./public`")))?;
                out = Some(PathBuf::from(v));
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            "--include-org" => {
                include_org = true;
                i += 1;
            }
            "--include-private" => {
                include_private = true;
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    let scope = path.unwrap_or_else(|| root.clone());
    let out = out.ok_or_else(|| usage_msg(ods_core::missing_flag_value("--out", "`ods share --out ./public`")))?;
    Ok((root, scope, out, include_org, include_private))
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
    Sarif,
}

fn parse_common_flags(
    args: &[String],
    start: usize,
) -> Result<(PathBuf, LintLevel, OutputFormat), CliError> {
    let level = LintLevel::Full;
    let mut format = OutputFormat::Text;
    let mut path = None;

    let mut i = start;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--format", "`--format text|json|sarif`")))?;
                format = match value.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "sarif" => OutputFormat::Sarif,
                    other => {
                        return Err(usage_msg(ods_core::invalid_choice("--format", other, "text|json|sarif")));
                    }
                };
                i += 2;
            }
            "--ods" => {
                return Err(usage_msg(ods_core::forbidden_ods_flag()));
            }
            "--okf"
            | "--skills"
            | "--check"
            | "--write"
            | "--fix"
            | "--force"
            | "--write-report"
            | "--all"
            | "--adopt"
            | "--canonical-refs"
            | "--include-private"
            | "--keep-frontmatter"
            | "--remove-indexes"
            | "--remove-root-index"
            | "--full"
            | "--indexes"
            | "--strip-indexes"
            | "--profiles"
            | "--strip-profiles"
            | "--dry-run"
            | "--skip-frontmatter-keys"
            | "--skip-keys"
            | "--no-key-lint"
            | "--no-keys"
            | "--migrate"
            | "--migrate-fm" => {
                i += 1;
            }
            // Dual-use: `ods start --status` (boolean) vs `ods find --status draft` (value).
            "--status" => {
                if let Some(next) = args.get(i + 1) {
                    match next.as_str() {
                        "draft" | "stable" | "deprecated" | "archived" => i += 2,
                        _ => i += 1,
                    }
                } else {
                    i += 1;
                }
            }
            "--refs" | "--ignore-keys" | "--ignore-key" => {
                i += 2;
            }
            "--root" => {
                if let Some(val) = args.get(i + 1) {
                    path = Some(PathBuf::from(val));
                }
                i += 2;
            }
            "--max-tokens" => {
                i += 2;
            }
            "--include-code"
            | "--include-related"
            | "--explain"
            | "--print"
            | "--list"
            | "--no-register"
            | "--register"
            | "--help"
            | "-h" => {
                i += 1;
            }
            // Value flags for find/context/tag discovery (and similar).
            "--tag"
            | "--key"
            | "--key-match"
            | "--tag-match"
            | "--profile"
            | "--owner"
            | "--prompt"
            | "--llm"
            | "--agent"
            | "--snapshot"
            | "--path"
            | "--name" => {
                i += 2;
            }
            flag if flag.starts_with('-') => {
                return Err(usage_msg(ods_core::unknown_flag(flag, "ods help")));
            }
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                }
                i += 1;
            }
        }
    }

    Ok((
        resolve_root_path(
            path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        ),
        level,
        format,
    ))
}


fn parse_key_suppression_flags(args: &[String], config: &mut ods_core::WorkspaceSpecsConfig) {
    let skip_keys = args.iter().any(|a| {
        a == "--skip-frontmatter-keys"
            || a == "--skip-keys"
            || a == "--no-key-lint"
            || a == "--no-keys"
    });
    if skip_keys {
        config.okf.check_keys = false;
        config.skills.check_keys = false;
    }

    if let Some(pos) = args
        .iter()
        .position(|a| a == "--ignore-keys" || a == "--ignore-key")
    {
        if let Some(val) = args.get(pos + 1) {
            for k in val.split(',') {
                let trimmed = k.trim();
                if !trimmed.is_empty() {
                    config.okf.ignore_keys.insert(trimmed.to_string());
                    config.skills.ignore_keys.insert(trimmed.to_string());
                }
            }
        }
    }
}

/// Positional args after `start`, skipping flags and their values.
fn positional_args(args: &[String], start: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = start;
    while i < args.len() {
        match args[i].as_str() {
            "--format"
            | "--version"
            | "--root"
            | "--refs"
            | "--max-tokens"
            | "--mode"
            | "--tag"
            | "--key"
            | "--key-match"
            | "--tag-match"
            | "--profile"
            | "--owner" => i += 2,
            "--status" => {
                if let Some(next) = args.get(i + 1) {
                    match next.as_str() {
                        "draft" | "stable" | "deprecated" | "archived" => i += 2,
                        _ => i += 1,
                    }
                } else {
                    i += 1;
                }
            }
            "--check"
            | "--write"
            | "--force"
            | "--canonical-refs"
            | "--include-private"
            | "--include-code"
            | "--print"
            | "--help"
            | "-h" => i += 1,
            flag if flag.starts_with('-') => i += 1,
            other => {
                out.push(other.to_string());
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod test_cli_arg_parser {
    use super::*;

    #[test]
    fn parse_serve_mode_variants() {
        assert!(matches!(parse_serve_mode("auto").unwrap(), ServeMode::Auto));
        assert!(matches!(parse_serve_mode("watch").unwrap(), ServeMode::Watch));
        assert!(matches!(parse_serve_mode("poll").unwrap(), ServeMode::Poll));
        assert!(parse_serve_mode("bogus").is_err());
    }

    #[test]
    fn resolved_serve_mode_non_auto() {
        assert!(matches!(
            resolved_serve_mode(ServeMode::Poll),
            ServeMode::Poll
        ));
        assert!(matches!(
            resolved_serve_mode(ServeMode::Watch),
            ServeMode::Watch
        ));
        // Auto depends on env; just ensure it returns a concrete mode.
        let mode = resolved_serve_mode(ServeMode::Auto);
        assert!(matches!(mode, ServeMode::Watch | ServeMode::Poll));
    }

    #[test]
    fn parse_export_args_matrix() {
        let args = vec![
            "ods".into(),
            "export".into(),
            "graph".into(),
            ".".into(),
            "--out".into(),
            "g.md".into(),
            "--format".into(),
            "json".into(),
            "--spec".into(),
            "okf".into(),
        ];
        assert!(parse_export_args(&args).is_ok());

        let args = vec![
            "ods".into(),
            "export".into(),
            "--out=out.json".into(),
            "--format".into(),
            "md".into(),
            "--okf".into(),
            "/tmp".into(),
        ];
        assert!(parse_export_args(&args).is_ok());

        let args = vec![
            "ods".into(),
            "export".into(),
            "--format".into(),
            "nope".into(),
        ];
        assert!(parse_export_args(&args).is_err());

        let args = vec!["ods".into(), "export".into(), "all".into(), ".".into()];
        let _ = parse_export_args(&args);
    }

    #[test]
    fn parse_serve_args_and_poll() {
        let args = vec![
            "ods".into(),
            "serve".into(),
            "--mode".into(),
            "poll".into(),
            "--poll-secs".into(),
            "5".into(),
            "--memory-report".into(),
            "/tmp".into(),
        ];
        let opts = serve_options_from_args(&args).unwrap();
        assert!(matches!(opts.mode, ServeMode::Poll));
        assert!(opts.memory_report);
        assert_eq!(opts.poll_secs, 5);

        assert!(serve_options_from_args(&[
            "ods".into(),
            "serve".into(),
            "--mode".into(),
            "nope".into(),
        ])
        .is_err());
        assert!(serve_options_from_args(&[
            "ods".into(),
            "serve".into(),
            "--poll-secs".into(),
            "x".into(),
        ])
        .is_err());
        assert!(serve_options_from_args(&[
            "ods".into(),
            "serve".into(),
            "--root".into(),
        ])
        .is_err());

        let opts = serve_options_from_args(&[
            "ods".into(),
            "serve".into(),
            "--root".into(),
            "/tmp".into(),
            "--mode".into(),
            "watch".into(),
        ])
        .unwrap();
        assert!(matches!(opts.mode, ServeMode::Watch));
    }

    #[test]
    fn parse_common_flags_format_and_rejection_of_legacy_level() {
        let args = vec![
            "ods".into(),
            "lint".into(),
            ".".into(),
            "--format".into(),
            "json".into(),
        ];
        assert!(parse_common_flags(&args, 2).is_ok());

        let legacy_args = vec![
            "ods".into(),
            "lint".into(),
            ".".into(),
            "--level".into(),
            "1".into(),
        ];
        assert!(parse_common_flags(&legacy_args, 2).is_err());
    }
}
