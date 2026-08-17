fn run_tree_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("tree");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let mut max_depth = 2usize;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--depth" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--depth", "`ods tree --depth 2`")))?;
                max_depth = v.parse().unwrap_or(2);
                i += 2;
            }
            _ => i += 1,
        }
    }

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;

    match format {
        OutputFormat::Text => {
            println!("ODS Workspace Tree: {}", root.display());
            println!("└── ods.toml (workspace marker)");

            let mut entries: Vec<(PathBuf, PathBuf)> = workspace
                .documents
                .iter()
                .filter_map(|doc| {
                    let rel = doc.path.strip_prefix(&root).ok()?.to_path_buf();
                    let depth = rel.components().count();
                    if depth == 0 || depth > max_depth {
                        return None;
                    }
                    Some((rel, doc.path.clone()))
                })
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));

            for (i, (rel, abs)) in entries.iter().enumerate() {
                let is_last = i + 1 == entries.len();
                let prefix = if is_last { "└── " } else { "├── " };
                let desc = workspace
                    .document_by_path(abs)
                    .and_then(|d| match &d.frontmatter {
                        ods_core::FrontmatterState::Parsed(fm) => fm.description.clone(),
                        _ => None,
                    });
                if let Some(d) = desc {
                    println!("{prefix}{} — {d}", rel.display());
                } else {
                    println!("{prefix}{}", rel.display());
                }
            }
            if entries.is_empty() {
                println!("(no documents within depth {max_depth})");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let docs: Vec<String> = workspace
                .documents
                .iter()
                .filter_map(|d| {
                    let rel = d.path.strip_prefix(&root).ok()?;
                    if rel.components().count() > max_depth {
                        return None;
                    }
                    Some(format!(r#""{}""#, rel.display()))
                })
                .collect();
            println!(
                r#"{{"root":"{}","depth":{},"tree":[{}]}}"#,
                root.display(),
                max_depth,
                docs.join(",")
            );
        }
    }

    Ok(ExitCode::from(0))
}
