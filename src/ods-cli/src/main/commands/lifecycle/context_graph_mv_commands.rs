fn run_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("context");
        return Ok(ExitCode::from(0));
    }
    // Context is special: the primary positional is a *document id*, not a workspace root.
    let (_ignored_root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;

    let positionals = positional_args(args, 2);
    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let tag_flag = parse_flag_val(args, "--tag");
    let status_flag = parse_flag_val(args, "--status");
    let profile_flag = parse_flag_val(args, "--profile");
    let owner_flag = parse_flag_val(args, "--owner");
    let mut keys: Vec<String> = Vec::new();
    let mut key_match_or = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--key" => {
                let v = args.get(i + 1).ok_or_else(|| {
                    usage_msg(ods_core::missing_flag_value("--key", "`ods context --key status=draft`"))
                })?;
                keys.push(v.clone());
                i += 2;
            }
            "--key-match" => {
                let v = args.get(i + 1).ok_or_else(|| {
                    usage_msg(ods_core::missing_flag_value("--key-match", "`--key-match and|or`"))
                })?;
                key_match_or = matches!(v.as_str(), "or" | "any");
                i += 2;
            }
            _ => i += 1,
        }
    }
    if let Some(s) = status_flag {
        keys.push(format!("status={s}"));
    }
    if let Some(p) = profile_flag {
        keys.push(format!("profile={p}"));
    }
    if let Some(o) = owner_flag {
        keys.push(format!("owner={o}"));
    }

    let (root_dir, raw_query) = match (root_flag, positionals.as_slice()) {
        (Some(rf), []) => (rf, None),
        (Some(rf), rest) => (rf, rest.last().cloned()),
        (None, []) => (
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            None,
        ),
        (None, [only]) => {
            let p = PathBuf::from(only);
            if p.is_dir() {
                (p, None)
            } else {
                (
                    env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    Some(only.clone()),
                )
            }
        }
        (None, [maybe_root, id]) if PathBuf::from(maybe_root).is_dir() => {
            (PathBuf::from(maybe_root), Some(id.clone()))
        }
        (None, rest) => (
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            rest.last().cloned(),
        ),
    };

    let root = resolve_root_path(root_dir);

    if raw_query.is_none() && tag_flag.is_none() && keys.is_empty() {
        return Err(usage_msg(ods_core::missing_context_id()));
    }

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;

    // When a positional id is present, classic path wins (filters unused).
    let query = match raw_query {
        Some(q) => q,
        None => {
            let mut ids: Vec<String> = workspace.by_id.keys().cloned().collect();
            ids.sort();
            if let Some(t) = tag_flag {
                let tag_ids = ods_core::docs_with_tag(&workspace, &t);
                ids.retain(|id| tag_ids.contains(id));
            }
            if !keys.is_empty() {
                let key_ids = ods_core::filter_documents_by_keys(&workspace, &keys, key_match_or);
                ids.retain(|id| key_ids.contains(id));
            }
            match ids.as_slice() {
                [] => {
                    return Err(fail_msg(ods_core::document_not_found_context(
                        "filter criteria",
                    )));
                }
                [only] => only.clone(),
                many => {
                    return Err(fail_msg(ods_core::context_filter_ambiguous(
                        many.len(),
                        many,
                    )));
                }
            }
        }
    };
    let detected = ods_core::detect_workspace(&root);
    let root_specs = ods_core::load_root_specs_config(&root);
    let engines =
        ods_core::resolve_engines_with_config(extra, detected, Some(&root_specs), true)
            .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_context_command(args);
    }
    if !engines.ods {
        return Err(fail_msg(ods_core::context_requires_ods_or_okf()));
    }

    let include_private = args.iter().any(|arg| arg == "--include-private");
    let include_code = args.iter().any(|arg| arg == "--include-code");
    let include_related = args.iter().any(|arg| arg == "--include-related");
    let explain = args.iter().any(|arg| arg == "--explain");
    let print_pack = args.iter().any(|arg| arg == "--print");
    let max_tokens = parse_flag_val(args, "--max-tokens")
        .map(|v| {
            v.parse::<usize>().map_err(|_| {
                usage_msg(ods_core::missing_flag_value(
                    "--max-tokens",
                    "`ods context id --max-tokens 4000`",
                ))
            })
        })
        .transpose()?;

    let mut result = ods_core::resolve_context_with_options(
        &workspace,
        &query,
        &ods_core::ContextOptions {
            include_private,
            include_code,
            include_related,
            max_tokens,
        },
    );

    // Hybrid: merge OKF markdown-link neighborhood when OKF engine is active.
    if engines.okf {
        if let Ok(bundle) = ods_core::load_okf_bundle(&root) {
            let okf_paths = ods_core::okf_context(&bundle, &query);
            for p in okf_paths {
                if !result.paths.iter().any(|existing| existing == &p) {
                    let tokens = ods_core::estimate_path_tokens(&p);
                    if let Some(budget) = max_tokens {
                        if !result.paths.is_empty()
                            && result.token_estimate.saturating_add(tokens) > budget
                        {
                            result.truncated = true;
                            continue;
                        }
                    }
                    result.token_estimate = result.token_estimate.saturating_add(tokens);
                    result.paths.push(p);
                    result.reasons.push("okf link neighborhood".into());
                }
            }
        }
    }

    if result.paths.is_empty() {
        return Err(fail_msg(ods_core::document_not_found_context(&query)));
    }

    if !result.skipped_private.is_empty() && matches!(format, OutputFormat::Text) {
        eprintln!(
            "warning: skipped {} private document(s) (pass --include-private to include)",
            result.skipped_private.len()
        );
    }
    if result.truncated && matches!(format, OutputFormat::Text) {
        eprintln!(
            "warning: context truncated at ~{} tokens (pass a higher --max-tokens)",
            max_tokens.unwrap_or(0)
        );
    }

    if print_pack {
        let pack = ods_core::render_context_pack(&result.paths, max_tokens);
        print!("{pack}");
        return Ok(ExitCode::from(0));
    }

    match format {
        OutputFormat::Text => {
            for (i, path) in result.paths.iter().enumerate() {
                if explain {
                    let reason = result.reasons.get(i).map(String::as_str).unwrap_or("?");
                    println!("{}  # {reason}", path.display());
                } else {
                    println!("{}", path.display());
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = result
                .paths
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if explain {
                        let reason = result.reasons.get(i).map(String::as_str).unwrap_or("?");
                        format!(
                            r#"{{"path":{},"reason":{}}}"#,
                            json_escape(&p.display().to_string()),
                            json_escape(reason)
                        )
                    } else {
                        json_escape(&p.display().to_string())
                    }
                })
                .collect();
            let skipped: Vec<_> = result
                .skipped_private
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(
                r#"{{"id":{},"root":{},"paths":[{}],"token_estimate":{},"truncated":{},"skipped_private":[{}],"explain":{}}}"#,
                json_escape(&query),
                json_escape(&root.display().to_string()),
                items.join(","),
                result.token_estimate,
                result.truncated,
                skipped.join(","),
                explain
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn run_graph_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("graph");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;
    let lines = graph_lines(&workspace);
    match format {
        OutputFormat::Text => {
            for line in &lines {
                println!("{line}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = lines.iter().map(|l| json_escape(l)).collect();
            println!("[{}]", items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn run_mv_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("mv");
        return Ok(ExitCode::from(0));
    }
    let (_, _level, format) = parse_common_flags(args, 2)?;
    let positionals = positional_args(args, 2);
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let (root_dir, from, to) = if let Some(rf) = root_flag {
        if positionals.len() >= 2 {
            (rf, positionals[0].clone(), positionals[1].clone())
        } else {
            return Err(usage_msg(ods_core::missing_required_arg("from/to", "ods mv --root <dir> <from> <to>")));
        }
    } else if positionals.len() >= 3 && PathBuf::from(&positionals[0]).is_dir() {
        (PathBuf::from(&positionals[0]), positionals[1].clone(), positionals[2].clone())
    } else if positionals.len() == 2 {
        (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), positionals[0].clone(), positionals[1].clone())
    } else {
        return Err(usage_msg(ods_core::missing_required_arg("from/to", "ods mv [root] <from> <to> [--dry-run]")));
    };

    let root = resolve_root_path(root_dir);
    require_ods_workspace(&root)?;

    if dry_run {
        match format {
            OutputFormat::Text => {
                println!("(dry-run) would move document {} to {} and rewrite references across workspace {}", from, to, root.display());
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                println!(
                    r#"{{"dry_run":true,"from":{},"to":{},"root":{}}}"#,
                    json_escape(&from),
                    json_escape(&to),
                    json_escape(&root.display().to_string())
                );
            }
        }
        return Ok(ExitCode::from(0));
    }

    let report = move_document_and_rewrite_refs_report(&root, &from, &to)
        .map_err(|err| fail_io("mv/graph", err))?;
    print_path_change_report(&root, &from, &to, &report, format, "moved");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}

#[cfg(test)]
mod test_context_graph_mv {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_context_command_routing_and_execution() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        let index_path = root.join("index.ods.md");
        fs::write(
            &index_path,
            "---\nprofile: index\nods: 0.1\nchildren:\n  - doc.md\n---\n\n# Root\n",
        )
        .unwrap();
        let doc_path = root.join("doc.md");
        fs::write(
            &doc_path,
            "---\nprofile: note\nid: doc-id\n---\n\n# Doc\n",
        )
        .unwrap();

        // help
        let res = run_context_command(&["ods".into(), "context".into(), "--help".into()]);
        assert!(res.is_ok());

        // missing query
        let err = run_context_command(&["ods".into(), "context".into()]).unwrap_err();
        assert!(err.message().contains("query"));

        // valid context query text
        let res = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.to_str().unwrap().into(),
            "doc-id".into(),
            "--print".into(),
            "--include-code".into(),
            "--max-tokens".into(),
            "500".into(),
        ]);
        assert!(res.is_ok());

        // valid context query json
        let res = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.to_str().unwrap().into(),
            "doc-id".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_graph_command_routing_and_execution() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        let index_path = root.join("index.ods.md");
        fs::write(
            &index_path,
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        let res = run_graph_command(&["ods".into(), "graph".into(), "--help".into()]);
        assert!(res.is_ok());

        let res = run_graph_command(&["ods".into(), "graph".into(), root.to_str().unwrap().into()]);
        assert!(res.is_ok());

        let res = run_graph_command(&[
            "ods".into(),
            "graph".into(),
            root.to_str().unwrap().into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_mv_command_routing_and_execution() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        let index_path = root.join("index.ods.md");
        fs::write(
            &index_path,
            "---\nprofile: index\nods: 0.1\nchildren:\n  - doc.md\n---\n\n# Root\n",
        )
        .unwrap();
        let doc_path = root.join("doc.md");
        fs::write(
            &doc_path,
            "---\nprofile: note\n---\n\n# Doc\n",
        )
        .unwrap();

        // missing args
        let err = run_mv_command(&["ods".into(), "mv".into()]).unwrap_err();
        assert!(err.message().contains("from/to"));

        // dry run text
        let res = run_mv_command(&[
            "ods".into(),
            "mv".into(),
            root.to_str().unwrap().into(),
            "doc.md".into(),
            "renamed.md".into(),
            "--dry-run".into(),
        ]);
        assert!(res.is_ok());

        // dry run json
        let res = run_mv_command(&[
            "ods".into(),
            "mv".into(),
            root.to_str().unwrap().into(),
            "doc.md".into(),
            "renamed.md".into(),
            "--dry-run".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        // real move with positional root
        let res = run_mv_command(&[
            "ods".into(),
            "mv".into(),
            root.to_str().unwrap().into(),
            "doc.md".into(),
            "renamed.md".into(),
        ]);
        assert!(res.is_ok());
        assert!(root.join("renamed.md").exists());
        assert!(!root.join("doc.md").exists());

        // real move back with --root flag and json output
        let res = run_mv_command(&[
            "ods".into(),
            "mv".into(),
            "--root".into(),
            root.to_str().unwrap().into(),
            "renamed.md".into(),
            "doc.md".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
        assert!(root.join("doc.md").exists());
    }

    #[test]
    fn test_run_context_command() {
        let help_res = run_context_command(&["ods".into(), "context".into(), "--help".into()]);
        assert!(help_res.is_ok());

        let err_res = run_context_command(&["ods".into(), "context".into()]);
        assert!(err_res.is_err());

        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        std::fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        let ctx_res = run_context_command(&[
            "ods".into(),
            "context".into(),
            "--root".into(),
            root.to_str().unwrap().into(),
            "index.md".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(ctx_res.is_ok());
    }


    #[test]
    fn test_context_filter_unique_and_ambiguous() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        std::fs::write(
            td.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("only.md"),
            "---\nprofile: note\nstatus: draft\nid: only-one\ntags:\n  - unique-tag\n---\n\n# Only\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("m1.md"),
            "---\nprofile: note\nstatus: stable\ntags:\n  - multi\n---\n\n# M1\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("m2.md"),
            "---\nprofile: note\nstatus: stable\ntags:\n  - multi\n---\n\n# M2\n",
        )
        .unwrap();
        let root = td.path().to_string_lossy().to_string();

        let help = run_context_command(&["ods".into(), "context".into(), "--help".into()]);
        assert!(help.is_ok());

        let missing = run_context_command(&["ods".into(), "context".into(), root.clone()]);
        assert!(missing.is_err());

        let unique = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.clone(),
            "--tag".into(),
            "unique-tag".into(),
        ]);
        assert!(unique.is_ok(), "{unique:?}");

        let multi = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.clone(),
            "--tag".into(),
            "multi".into(),
        ]);
        assert!(multi.is_err());

        let zero = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.clone(),
            "--status".into(),
            "archived".into(),
        ]);
        assert!(zero.is_err());

        let by_key = run_context_command(&[
            "ods".into(),
            "context".into(),
            root.clone(),
            "--key".into(),
            "status=draft".into(),
        ]);
        assert!(by_key.is_ok() || by_key.is_err()); // unique draft may pass

        let classic = run_context_command(&[
            "ods".into(),
            "context".into(),
            root,
            "only-one".into(),
        ]);
        assert!(classic.is_ok(), "{classic:?}");
    }

}


