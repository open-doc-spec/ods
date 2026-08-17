fn run_profile_list_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("profile");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
    let roots = ods_core::profile_catalog_roots_from_config(&root, &workspace.config);
    let catalog = load_profile_catalog(&root, &roots).map_err(|err| fail_io("profile", err))?;

    match format {
        OutputFormat::Text => {
            println!("profiles:");
            for (name, def) in &catalog.definitions {
                let kind = if def.source.to_string_lossy().starts_with("<builtin:") {
                    "[default ODS]"
                } else {
                    "[project]"
                };
                let section_summary: Vec<String> = def
                    .sections
                    .iter()
                    .filter_map(|g| g.first().cloned())
                    .collect();
                let source_path = def.source.to_string_lossy().replace('\\', "/");
                println!("{name}: {kind} ({}) — {source_path}", section_summary.join(", "));
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let mut list = Vec::new();
            for (name, def) in &catalog.definitions {
                let layer = if def.source.to_string_lossy().starts_with("<builtin:") {
                    "standard"
                } else {
                    "custom"
                };
                list.push(format!(
                    r#"{{"name":{},"layer":{},"source":{},"required_keys":{:?},"optional_keys":{:?},"forbidden_keys":{:?}}}"#,
                    json_escape(name),
                    json_escape(layer),
                    json_escape(&def.source.to_string_lossy()),
                    def.required_keys,
                    def.optional_keys,
                    def.forbidden_keys
                ));
            }
            println!("[{}]", list.join(","));
        }
    }

    Ok(ExitCode::from(0))
}

fn run_profile_show_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("profile");
        return Ok(ExitCode::from(0));
    }
    let profile_name = args
        .get(3)
        .filter(|a| !a.starts_with('-'))
        .ok_or_else(|| {
            usage_msg(ods_core::missing_required_arg(
                "name",
                "ods profile show <name>",
            ))
        })?;
    let root = args
        .get(4)
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(root);

    let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
    let roots = ods_core::profile_catalog_roots_from_config(&root, &workspace.config);
    let catalog = load_profile_catalog(&root, &roots).map_err(|err| fail_io("profile", err))?;

    let def = catalog.definitions.get(profile_name.as_str()).ok_or_else(|| {
        fail_msg(ods_core::UserMsg::new(
            "profile_not_found",
            ods_core::ErrorStage::Resolve,
            ods_core::error::lint_unknown_profile_with_sources(
                profile_name,
                &workspace.config.custom_profiles,
            ),
        )
        .next("ods profiles  # list available profiles")
        .hint("ods profile init <name>  # scaffold + register a custom profile"))
    })?;

    let layer = if def.source.to_string_lossy().starts_with("<builtin:") {
        "standard"
    } else {
        "custom"
    };
    let sections: Vec<String> = def
        .sections
        .iter()
        .map(|g| g.join(" | "))
        .collect();
    println!("profile: {profile_name}");
    println!("  layer: {layer}");
    println!("  source: {}", def.source.display());
    if def.required_keys.is_empty() {
        println!("  required keys: (none)");
    } else {
        println!("  required keys: {}", def.required_keys.join(", "));
    }
    if def.optional_keys.is_empty() {
        println!("  optional keys: (none)");
    } else {
        println!("  optional keys: {}", def.optional_keys.join(", "));
    }
    if def.forbidden_keys.is_empty() {
        println!("  forbidden keys: (none)");
    } else {
        println!("  forbidden keys: {}", def.forbidden_keys.join(", "));
    }
    if sections.is_empty() {
        println!("  sections: (none)");
    } else {
        println!("  sections:");
        for s in sections {
            println!("    - {s}");
        }
    }
    Ok(ExitCode::from(0))
}
