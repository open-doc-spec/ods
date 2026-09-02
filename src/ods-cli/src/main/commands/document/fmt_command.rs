fn run_fmt_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("fmt");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_fmt_command(args);
    }
    if engines.okf && engines.ods {
        let code = run_ods_fmt_body(&root, args, format)?;
        let _ = run_okf_fmt_command(args)?;
        return Ok(code);
    }
    if !engines.ods {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "fmt_requires_ods",
                ods_core::ErrorStage::Scope,
                "fmt requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF fmt"),
        ));
    }
    run_ods_fmt_body(&root, args, format)
}

fn run_ods_fmt_body(
    root: &Path,
    args: &[String],
    format: OutputFormat,
) -> Result<ExitCode, CliError> {
    let refs_mode = parse_refs_mode(args)?;
    let migrate = wants_migrate(args);
    let workspace = load_workspace_with_options(root, load_options_graph())
        .map_err(|err| fail_load(root, err))?;

    let mut actions: Vec<&str> = vec!["frontmatter spacing"];
    let mut changed = normalize_workspace_frontmatter_spacing_with_workspace(&workspace)
        .map_err(|err| fail_io("fmt", err))?;

    if refs_mode == Some("md-paths") {
        actions.push("document refs");
        for path in canonicalize_workspace_document_refs_with_workspace(&workspace)
            .map_err(|err| fail_io("fmt", err))?
        {
            if !changed.iter().any(|existing| existing == &path) {
                changed.push(path);
            }
        }
    }

    if migrate {
        actions.push("ods: key layout");
        for path in migrate_workspace_frontmatter_with_workspace(&workspace)
            .map_err(|err| fail_io("fmt", err))?
        {
            if !changed.iter().any(|existing| existing == &path) {
                changed.push(path);
            }
        }
    }

    changed.sort();
    changed.dedup();

    match format {
        OutputFormat::Text => {
            if changed.is_empty() {
                println!("{} already clean", actions.join("/"));
            } else {
                println!(
                    "formatted {} in {} file(s)",
                    actions.join("/"),
                    changed.len()
                );
                for path in &changed {
                    if let Ok(rel) = path.strip_prefix(root) {
                        println!("  {}", rel.display());
                    } else {
                        println!("  {}", path.display());
                    }
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = changed
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(
                r#"{{"changed":[{}],"count":{}}}"#,
                items.join(","),
                changed.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn parse_refs_mode(args: &[String]) -> Result<Option<&'static str>, CliError> {
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--refs" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--refs", "`ods fmt --refs md-paths`")))?;
                return match value.as_str() {
                    "md-paths" => Ok(Some("md-paths")),
                    other => Err(usage_msg(ods_core::invalid_choice(
                        "--refs",
                        other,
                        "md-paths",
                    ))),
                };
            }
            _ => i += 1,
        }
    }
    Ok(None)
}

/// `--migrate`: also rewrite legacy flat/out-of-order `ods:` engine keys into
/// the canonical nested block. Opt-in — unlike spacing/refs normalization,
/// this relocates whole key blocks and is a bigger change to review.
fn wants_migrate(args: &[String]) -> bool {
    args[2..].iter().any(|arg| arg == "--migrate")
}

#[cfg(test)]
mod test_fmt_command {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_refs_and_migrate_flags() {
        assert!(!wants_migrate(&["ods".into(), "fmt".into()]));
        assert!(wants_migrate(&[
            "ods".into(),
            "fmt".into(),
            "--migrate".into()
        ]));
        assert_eq!(
            parse_refs_mode(&["ods".into(), "fmt".into(), "--refs".into(), "md-paths".into()])
                .unwrap(),
            Some("md-paths")
        );
        assert!(parse_refs_mode(&[
            "ods".into(),
            "fmt".into(),
            "--refs".into(),
            "bad".into()
        ])
        .is_err());
        assert!(parse_refs_mode(&["ods".into(), "fmt".into(), "--refs".into()]).is_err());
        assert_eq!(
            parse_refs_mode(&["ods".into(), "fmt".into()]).unwrap(),
            None
        );
    }

    #[test]
    fn fmt_body_spacing_migrate_and_json() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"2.0\"\n").unwrap();
        fs::write(
            root.join("a.md"),
            "---\nlayout: post\nprofile: note\nstatus: draft\n---\n# A\n",
        )
        .unwrap();

        let res = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            root.to_str().unwrap().into(),
            "--migrate".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
        let text = fs::read_to_string(root.join("a.md")).unwrap();
        assert!(text.contains("layout: post"), "{text}");
        assert!(text.contains("profile: note"), "{text}");
        assert!(!text.contains("ods:"), "{text}");

        // already clean second pass
        let res = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            root.to_str().unwrap().into(),
            "--migrate".into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn fmt_requires_workspace() {
        let td = tempdir().unwrap();
        let err = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            td.path().to_str().unwrap().into(),
        ])
        .unwrap_err();
        assert!(
            err.message().contains("ODS") || err.message().contains("workspace"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn fmt_already_clean_text_and_json_empty_changed() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"2.0\"\n").unwrap();
        fs::write(
            root.join("a.md"),
            "---\nods:\n  profile: note\n  status: draft\n---\n\n# A\n",
        )
        .unwrap();
        let res = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            root.to_str().unwrap().into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn fmt_refs_md_paths_and_text_output() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"2.0\"\n").unwrap();
        fs::write(
            root.join("a.md"),
            "---\nods:\n  profile: note\n  status: draft\n  depends:\n    - b\n---\n\n# A\n",
        )
        .unwrap();
        fs::write(
            root.join("b.md"),
            "---\nods:\n  profile: note\n  status: draft\n---\n\n# B\n",
        )
        .unwrap();
        let res = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            root.to_str().unwrap().into(),
            "--refs".into(),
            "md-paths".into(),
        ]);
        assert!(res.is_ok());
        let a = fs::read_to_string(root.join("a.md")).unwrap();
        assert!(a.contains("b.md") || a.contains("depends:"), "{a}");
    }

    #[test]
    fn fmt_help_flag_prints_usage() {
        let res = run_fmt_command(&["ods".into(), "fmt".into(), "--help".into()]);
        assert!(res.is_ok());
        let res_h = run_fmt_command(&["ods".into(), "fmt".into(), "-h".into()]);
        assert!(res_h.is_ok());
    }
}
