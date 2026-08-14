fn run_find_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "ods find [path] [--tag <name> ...] [--key <expr> ...] [<query>]\n\n\
             Find documents by tag, schema/custom keys, and/or id/path/stem query.\n\n\
             Flags:\n\
               --tag <name>           Filter by tag (repeatable)\n\
               --tag-match any|all    Tag intersection mode (default: any)\n\
               --key <expr>           Filter by key/val expression (comma values, AND/OR logic)\n\
               --key-match and|or     Key matching mode across multiple --key flags (default: and)\n\
               --status <status>      Shortcut for --key status=<status>\n\
               --profile <profile>    Shortcut for --key profile=<profile>\n\
               --owner <owner>        Shortcut for --key owner=<owner>\n\
               --format text|json     Output format (default: text)\n\n\
             Examples:\n\
               ods find --tag caching\n\
               ods find --key status=draft,stable\n\
               ods find --key \"status=draft AND owner=alice\"\n\
               ods find --key \"team=infra,frontend\" --format json\n"
        );
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let mut tags = Vec::new();
    let mut tag_match_all = false;
    let mut keys = Vec::new();
    let mut key_match_or = false;
    let mut query: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--tag" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--tag", "`ods find --tag api`")))?;
                tags.push(v.clone());
                i += 2;
            }
            "--tag-match" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--tag-match", "`--tag-match any|all`")))?;
                match v.as_str() {
                    "all" => tag_match_all = true,
                    "any" => tag_match_all = false,
                    other => {
                        return Err(usage_msg(ods_core::invalid_choice("--tag-match", other, "any|all")));
                    }
                }
                i += 2;
            }
            "--key" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--key", "`ods find --key status=draft`")))?;
                keys.push(v.clone());
                i += 2;
            }
            "--key-match" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--key-match", "`--key-match and|or`")))?;
                match v.as_str() {
                    "or" | "any" => key_match_or = true,
                    "and" | "all" => key_match_or = false,
                    other => {
                        return Err(usage_msg(ods_core::invalid_choice("--key-match", other, "and|or")));
                    }
                }
                i += 2;
            }
            "--status" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--status", "`ods find --status draft`")))?;
                keys.push(format!("status={v}"));
                i += 2;
            }
            "--profile" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--profile", "`ods find --profile rfc`")))?;
                keys.push(format!("profile={v}"));
                i += 2;
            }
            "--owner" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--owner", "`ods find --owner alice`")))?;
                keys.push(format!("owner={v}"));
                i += 2;
            }
            "--format" | "--root" => i += 2,
            "--all" | "--write" | "--check" | "--force" | "--help" | "-h" => i += 1,
            other if other.starts_with('-') => {
                return Err(usage_msg(ods_core::unknown_flag(other, "ods find --help")));
            }
            other => {
                let p = PathBuf::from(other);
                let is_root_positional = p.is_dir()
                    && (resolve_root_path(p.clone()) == root
                        || p.canonicalize().ok().as_ref() == Some(&root));
                if !is_root_positional && query.is_none() {
                    query = Some(other.to_string());
                }
                i += 1;
            }
        }
    }

    if tags.is_empty() && keys.is_empty() && query.is_none() {
        return Err(usage_msg(ods_core::missing_required_arg(
            "query, --tag, or --key",
            "ods find [path] [--tag <name>] [--key <expr>] [<query>]",
        )));
    }

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;

    let mut ids: Vec<String> = if tags.is_empty() {
        let mut all: Vec<String> = workspace.by_id.keys().cloned().collect();
        all.sort();
        all
    } else if tag_match_all {
        docs_with_all_tags(&workspace, &tags)
    } else {
        docs_with_any_tag(&workspace, &tags)
    };

    if !keys.is_empty() {
        let key_matched = ods_core::filter_documents_by_keys(&workspace, &keys, key_match_or);
        ids.retain(|id| key_matched.contains(id));
    }

    if let Some(q) = query.as_deref() {
        let q_lc = q.trim().to_lowercase();
        let q_path = Path::new(q);
        ids.retain(|id| {
            if id == &q_lc || id.ends_with(&q_lc) || id.contains(&q_lc) {
                return true;
            }
            if let Some(doc) = workspace.document_by_id(id) {
                let lossy = doc.path.to_string_lossy().replace('\\', "/").to_lowercase();
                if lossy.ends_with(&q_lc)
                    || lossy.ends_with(&format!("{q_lc}.md"))
                    || doc.path.ends_with(q_path)
                    || doc
                        .path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case(&q_lc))
                {
                    return true;
                }
            }
            false
        });
        if ids.is_empty() {
            if let Some(start) = ods_core::resolve_context_start(&workspace, q) {
                if let Some(doc) = workspace.document_by_path(&start) {
                    let fm = match &doc.frontmatter {
                        FrontmatterState::Parsed(fm) => Some(fm),
                        _ => None,
                    };
                    ids.push(ods_core::document_id(&workspace.root, &doc.path, fm));
                }
            }
        }
    }

    match format {
        OutputFormat::Text => {
            for id in &ids {
                println!("{id}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = ids.iter().map(|id| json_escape(id)).collect();
            let tag_items: Vec<_> = tags.iter().map(|t| json_escape(t)).collect();
            let key_items: Vec<_> = keys.iter().map(|k| json_escape(k)).collect();
            println!(
                r#"{{"tags":[{}],"keys":[{}],"query":{},"ids":[{}],"count":{}}}"#,
                tag_items.join(","),
                key_items.join(","),
                json_escape(query.as_deref().unwrap_or("")),
                items.join(","),
                ids.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_find_command {
    use super::*;

    #[test]
    fn test_run_find_command_errors() {
        let err1 = run_find_command(&["ods".into(), "find".into(), "--unknown".into()]);
        assert!(err1.is_err());

        let err2 = run_find_command(&["ods".into(), "find".into(), "--tag".into()]);
        assert!(err2.is_err());

        let err3 = run_find_command(&["ods".into(), "find".into()]);
        assert!(err3.is_err());

        let err4 = run_find_command(&[
            "ods".into(),
            "find".into(),
            "--tag-match".into(),
            "nope".into(),
        ]);
        assert!(err4.is_err());

        let err5 = run_find_command(&[
            "ods".into(),
            "find".into(),
            "--key-match".into(),
            "nope".into(),
        ]);
        assert!(err5.is_err());

        let err6 = run_find_command(&["ods".into(), "find".into(), "--key".into()]);
        assert!(err6.is_err());

        let err7 = run_find_command(&["ods".into(), "find".into(), "--status".into()]);
        assert!(err7.is_err());

        let err8 = run_find_command(&["ods".into(), "find".into(), "--profile".into()]);
        assert!(err8.is_err());

        let err9 = run_find_command(&["ods".into(), "find".into(), "--owner".into()]);
        assert!(err9.is_err());

        let help = run_find_command(&["ods".into(), "find".into(), "--help".into()]);
        assert!(help.is_ok());

        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        std::fs::write(
            td.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("a.md"),
            "---\nprofile: note\nstatus: draft\nowner: alice\nteam: infra\ntags:\n  - auth\n  - billing\n---\n\n# A\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("b.md"),
            "---\nprofile: note\nstatus: stable\nowner: bob\ntags:\n  - auth\n---\n\n# B\n",
        )
        .unwrap();
        let root = td.path().to_string_lossy().to_string();

        let res = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--tag".into(),
            "auth".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok(), "{res:?}");

        let res_key = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--key".into(),
            "status=draft,stable".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_key.is_ok(), "{res_key:?}");

        let res_and = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--status".into(),
            "draft".into(),
            "--profile".into(),
            "note".into(),
            "--key-match".into(),
            "and".into(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_and.is_ok(), "{res_and:?}");

        let res_or = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--key".into(),
            "status=draft".into(),
            "--key".into(),
            "status=stable".into(),
            "--key-match".into(),
            "or".into(),
        ]);
        assert!(res_or.is_ok(), "{res_or:?}");

        let res_tag_all = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--tag".into(),
            "auth".into(),
            "--tag".into(),
            "billing".into(),
            "--tag-match".into(),
            "all".into(),
        ]);
        assert!(res_tag_all.is_ok(), "{res_tag_all:?}");

        let res_owner = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--owner".into(),
            "nobody".into(),
        ]);
        assert!(res_owner.is_ok(), "{res_owner:?}");

        let res_custom = run_find_command(&[
            "ods".into(),
            "find".into(),
            root.clone(),
            "--key".into(),
            "team=infra".into(),
        ]);
        assert!(res_custom.is_ok(), "{res_custom:?}");

        let res_query = run_find_command(&[
            "ods".into(),
            "find".into(),
            root,
            "a".into(),
        ]);
        assert!(res_query.is_ok(), "{res_query:?}");
    }
}
