fn run_tag_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "ods tag <subcommand> [flags]\n\n\
             Subcommands:\n\
               ods tag list [path] [--format text|json]          List observed tags in workspace\n\
               ods tag show [path] <tag> [--format text|json]    Show documents with a tag\n\
               ods tag rename [path] <old> <new> [--write]      Rename tag across workspace\n"
        );
        return Ok(ExitCode::from(0));
    }
    let sub = args
        .get(2)
        .map(String::as_str)
        .ok_or_else(|| usage_msg(ods_core::missing_required_arg("subcommand", "ods tag list|show|rename")))?;

    match sub {
        "list" => {
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
            let usage = ods_core::tag_usage(&workspace);

            match format {
                OutputFormat::Text => {
                    if usage.is_empty() {
                        println!("no tags found in workspace");
                    } else {
                        for (tag, count) in &usage {
                            println!("{tag:<20} ({count} doc(s))");
                        }
                    }
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    let items: Vec<_> = usage
                        .iter()
                        .map(|(t, count)| {
                            let docs = ods_core::docs_with_tag(&workspace, t);
                            let doc_items: Vec<_> = docs.iter().map(|d| json_escape(d)).collect();
                            format!(
                                r#"{{"tag":{},"count":{},"docs":[{}]}}"#,
                                json_escape(t),
                                count,
                                doc_items.join(",")
                            )
                        })
                        .collect();
                    println!("[{}]", items.join(","));
                }
            }
            Ok(ExitCode::from(0))
        }
        "show" => {
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let positionals = positional_args(args, 3);
            // Prefer last non-directory positional so `tag show <path> <tag>` works.
            let tag_name = positionals
                .iter()
                .rev()
                .find(|p| {
                    let pb = PathBuf::from(p);
                    !(pb.is_dir()
                        && (resolve_root_path(pb.clone()) == root
                            || pb.canonicalize().ok().as_ref() == Some(&root)))
                })
                .ok_or_else(|| {
                    usage_msg(ods_core::missing_required_arg(
                        "tag name",
                        "ods tag show <tag-name>",
                    ))
                })?;

            let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
            let docs = ods_core::docs_with_tag(&workspace, tag_name);

            match format {
                OutputFormat::Text => {
                    for id in &docs {
                        println!("{id}");
                    }
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    let items: Vec<_> = docs.iter().map(|d| json_escape(d)).collect();
                    println!(
                        r#"{{"tag":{},"count":{},"ids":[{}]}}"#,
                        json_escape(tag_name),
                        docs.len(),
                        items.join(",")
                    );
                }
            }
            Ok(ExitCode::from(0))
        }
        "rename" => {
            let write = args.iter().any(|a| a == "--write");
            let mut format = OutputFormat::Text;
            let mut i = 3;
            let mut bare = Vec::new();
            while i < args.len() {
                match args[i].as_str() {
                    "--write" | "--all" => i += 1,
                    "--format" => {
                        let value = args
                            .get(i + 1)
                            .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--format", "`--format text|json|sarif`")))?;
                        format = match value.as_str() {
                            "text" => OutputFormat::Text,
                            "json" => OutputFormat::Json,
                            other => {
                                return Err(usage_msg(ods_core::invalid_choice(
                                    "--format",
                                    other,
                                    "text|json",
                                )));
                            }
                        };
                        i += 2;
                    }
                    flag if flag.starts_with('-') => {
                        return Err(usage_msg(ods_core::unknown_flag(flag, "ods help")));
                    }
                    other => {
                        bare.push(other.to_string());
                        i += 1;
                    }
                }
            }
            let (root, from, to) = match bare.as_slice() {
                [from, to] => (
                    resolve_root_path(
                        env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    ),
                    from.clone(),
                    to.clone(),
                ),
                [maybe_root, from, to] if PathBuf::from(maybe_root).is_dir() => (
                    resolve_root_path(PathBuf::from(maybe_root)),
                    from.clone(),
                    to.clone(),
                ),
                _ => {
                    return Err(usage_msg(ods_core::missing_required_arg(
                        "old/new tags",
                        "ods tag rename [path] <old> <new> [--write]",
                    )));
                }
            };
            let workspace =
                load_workspace(&root).map_err(|err| fail_load(&root, err))?;
            let report = rename_tag_in_workspace(&workspace, &from, &to, write)
                .map_err(|err| fail_io("tag rename", err))?;
            match format {
                OutputFormat::Text => {
                    let mode = if report.dry_run { "dry-run" } else { "wrote" };
                    println!(
                        "tag rename {} → {} ({mode}; {} doc(s), {} file(s))",
                        report.from,
                        report.to,
                        report.matched_docs,
                        report.rewritten_files.len()
                    );
                    for path in &report.rewritten_files {
                        if let Ok(rel) = path.strip_prefix(&root) {
                            println!("  {}", rel.display());
                        } else {
                            println!("  {}", path.display());
                        }
                    }
                    if report.dry_run && !report.rewritten_files.is_empty() {
                        println!("re-run with --write to apply");
                    }
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    let files: Vec<_> = report
                        .rewritten_files
                        .iter()
                        .map(|p| json_escape(&p.display().to_string()))
                        .collect();
                    println!(
                        r#"{{"from":{},"to":{},"dry_run":{},"matched_docs":{},"files":[{}]}}"#,
                        json_escape(&report.from),
                        json_escape(&report.to),
                        if report.dry_run { "true" } else { "false" },
                        report.matched_docs,
                        files.join(",")
                    );
                }
            }
            Ok(ExitCode::from(0))
        }
        other => Err(usage_msg(ods_core::unknown_subcommand(
            "tag",
            other,
            "ods tag list|show|rename",
        ))),
    }
}

#[cfg(test)]
mod test_tag_command {
    use super::*;

    #[test]
    fn test_run_tag_command_errors() {
        let err1 = run_tag_command(&["ods".into(), "tag".into()]);
        assert!(err1.is_err());

        let err2 = run_tag_command(&["ods".into(), "tag".into(), "invalid".into()]);
        assert!(err2.is_err());

        let err3 = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            "--unknown".into(),
        ]);
        assert!(err3.is_err());

        let err4 = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            "--format".into(),
            "invalid".into(),
        ]);
        assert!(err4.is_err());

        let err_show = run_tag_command(&["ods".into(), "tag".into(), "show".into()]);
        assert!(err_show.is_err());
    }

    #[test]
    fn test_tag_list_show_and_rename() {
        let help = run_tag_command(&["ods".into(), "tag".into(), "--help".into()]);
        assert!(help.is_ok());
        let help_h = run_tag_command(&["ods".into(), "tag".into(), "-h".into()]);
        assert!(help_h.is_ok());

        let sample = ["fixtures/ecommerce", "src/fixtures/ecommerce"]
            .into_iter()
            .map(std::path::Path::new)
            .find(|p| p.exists());
        if let Some(sample) = sample {
            let res_list = run_tag_command(&[
                "ods".into(),
                "tag".into(),
                "list".into(),
                sample.to_str().unwrap().into(),
                "--format".into(),
                "json".into(),
            ]);
            assert!(res_list.is_ok());

            let res_list_txt = run_tag_command(&[
                "ods".into(),
                "tag".into(),
                "list".into(),
                sample.to_str().unwrap().into(),
                "--format".into(),
                "text".into(),
            ]);
            assert!(res_list_txt.is_ok());

            let res_show = run_tag_command(&[
                "ods".into(),
                "tag".into(),
                "show".into(),
                sample.to_str().unwrap().into(),
                "auth".into(),
            ]);
            assert!(res_show.is_ok());

            let res_show_json = run_tag_command(&[
                "ods".into(),
                "tag".into(),
                "show".into(),
                sample.to_str().unwrap().into(),
                "auth".into(),
                "--format".into(),
                "json".into(),
            ]);
            assert!(res_show_json.is_ok());

            let res_show_missing = run_tag_command(&[
                "ods".into(),
                "tag".into(),
                "show".into(),
                sample.to_str().unwrap().into(),
                "no-such-tag-xyz".into(),
            ]);
            assert!(res_show_missing.is_ok());
        }

        // Empty-tag workspace: list prints friendly empty text / empty json array.
        let empty = tempfile::tempdir().unwrap();
        std::fs::write(
            empty.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            empty.path().join("plain.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# Plain\n",
        )
        .unwrap();
        let empty_root = empty.path().to_string_lossy().to_string();
        assert!(run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "list".into(),
            empty_root.clone(),
            "--format".into(),
            "text".into(),
        ])
        .is_ok());
        assert!(run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "list".into(),
            empty_root.clone(),
            "--format".into(),
            "json".into(),
        ])
        .is_ok());
        assert!(run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "show".into(),
            empty_root,
            "missing".into(),
            "--format".into(),
            "text".into(),
        ])
        .is_ok());

        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("doc.md"),
            "---\nprofile: note\ntags:\n  - oldtag\n---\n\n# D\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("doc2.md"),
            "---\nprofile: note\ntags:\n  - oldtag\n  - keep\n---\n\n# D2\n",
        )
        .unwrap();
        let root = td.path().to_string_lossy().to_string();

        let res_list = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "list".into(),
            root.clone(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_list.is_ok());
        let res_list_json = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "list".into(),
            root.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_list_json.is_ok());

        let res_show = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "show".into(),
            root.clone(),
            "oldtag".into(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_show.is_ok());

        // Dry-run text path (lists files + re-run hint).
        let res_txt = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            root.clone(),
            "oldtag".into(),
            "newtag".into(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_txt.is_ok(), "{res_txt:?}");

        // Missing rename args.
        assert!(run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            root.clone(),
        ])
        .is_err());
        assert!(run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            "onlyone".into(),
        ])
        .is_err());

        // Write + JSON report path.
        let res_json = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            root.clone(),
            "oldtag".into(),
            "newtag".into(),
            "--write".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_json.is_ok(), "{res_json:?}");
        let body = std::fs::read_to_string(td.path().join("doc.md")).unwrap();
        assert!(body.contains("newtag"), "{body}");
        assert!(!body.contains("oldtag"), "{body}");
        let body2 = std::fs::read_to_string(td.path().join("doc2.md")).unwrap();
        assert!(body2.contains("newtag") && body2.contains("keep"), "{body2}");

        // No-op rename after write (dry-run finds nothing).
        let res_noop = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            root,
            "oldtag".into(),
            "newtag".into(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_noop.is_ok());
    }

    #[test]
    fn test_tag_command_help_and_errors() {
        // help flag
        let res_help = run_tag_command(&["ods".into(), "tag".into(), "--help".into()]);
        assert!(res_help.is_ok());

        // missing subcommand
        let err_no_sub = run_tag_command(&["ods".into(), "tag".into()]).unwrap_err();
        assert!(err_no_sub.message().contains("subcommand"));

        // tag list json format
        let td = tempfile::tempdir().unwrap();
        let root = td.path().to_str().unwrap().to_string();
        std::fs::write(
            td.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\ntags:\n  - sample\n---\n# Root\n",
        )
        .unwrap();

        let res_list_json = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "list".into(),
            root.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_list_json.is_ok());

        // tag show json format
        let res_show_json = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "show".into(),
            root,
            "sample".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_show_json.is_ok());
    }
}
