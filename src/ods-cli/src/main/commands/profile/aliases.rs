fn run_aliases_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    match sub {
        "--help" | "-h" => {
            print_command_help("aliases");
            Ok(ExitCode::from(0))
        }
        "add" => run_alias_add_command(args),
        "list" => run_aliases_list_command(args, 3),
        _ => run_aliases_list_command(args, 2),
    }
}

fn run_aliases_list_command(args: &[String], flag_start: usize) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, flag_start)?;
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;
    let aliases = workspace_aliases(&workspace);

    match format {
        OutputFormat::Text => {
            println!("section aliases (workspace root):");
            if aliases.is_empty() {
                println!("  (none declared — standard profile pipe-alternatives still apply)");
                println!("hint: ods alias add Goal Objective");
            } else {
                for (canonical, values) in &aliases {
                    let mut v: Vec<_> = values.iter().cloned().collect();
                    v.sort();
                    println!("  {canonical}: {}", v.join(", "));
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let mut items = Vec::new();
            for (canonical, values) in &aliases {
                let mut v: Vec<_> = values.iter().cloned().collect();
                v.sort();
                let vals: Vec<_> = v.iter().map(|s| json_escape(s)).collect();
                items.push(format!(
                    r#"{{"canonical":{},"aliases":[{}]}}"#,
                    json_escape(canonical),
                    vals.join(",")
                ));
            }
            println!("[{}]", items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn insert_alias_into_ods_toml(text: &str, canonical: &str, synonym: &str) -> String {
    if text.contains("[aliases]") {
        let target = format!("{canonical} = [");
        if text.contains(&target) {
            if let Some(idx) = text.find(&target) {
                let rest = &text[idx..];
                if let Some(end) = rest.find(']') {
                    let abs_end = idx + end;
                    return format!("{}\"{synonym}\", {}", &text[..abs_end], &text[abs_end..]);
                }
            }
        }
        let insert = format!("{canonical} = [\"{synonym}\"]\n");
        return text.replace("[aliases]", &format!("[aliases]\n{insert}"));
    }
    format!("{}\n\n[aliases]\n{canonical} = [\"{synonym}\"]\n", text.trim_end())
}

fn run_alias_add_command(args: &[String]) -> Result<ExitCode, CliError> {
    // ods alias add <Canonical> <Synonym> [root]
    let positionals: Vec<&String> = args
        .iter()
        .skip(3)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let (canonical, synonym, root) = match positionals.as_slice() {
        [c, s] => (
            (*c).clone(),
            (*s).clone(),
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ),
        [c, s, r] => ((*c).clone(), (*s).clone(), PathBuf::from(*r)),
        _ => {
            return Err(usage_msg(ods_core::missing_required_arg(
                "Canonical Synonym",
                "ods alias add <Canonical> <Synonym>",
            )));
        }
    };
    let root = resolve_root_path(root);
    let toml_path = root.join("ods.toml");
    if !toml_path.is_file() {
        return Err(fail_msg(ods_core::root_index_missing()));
    }

    let text = fs::read_to_string(&toml_path).map_err(|e| fail_io("alias", e))?;
    let updated = insert_alias_into_ods_toml(&text, &canonical, &synonym);
    fs::write(&toml_path, updated).map_err(|e| fail_io("alias", e))?;
    println!("registered section alias: {canonical} -> {synonym}");
    Ok(ExitCode::from(0))
}
