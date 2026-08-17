fn run_schema_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("schema");
        return Ok(ExitCode::from(0));
    }

    let is_keys_sub = args.get(2).map(String::as_str) == Some("keys");
    let mut write = false;
    let mut out_path = None;
    let mut dialect = "ods".to_string();
    let mut format = if is_keys_sub { OutputFormat::Text } else { OutputFormat::Json };

    let mut i = if is_keys_sub { 3 } else { 2 };
    while i < args.len() {
        match args[i].as_str() {
            "--write" | "-w" => {
                write = true;
                i += 1;
            }
            "--out" | "-o" => {
                let p = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--out", "`ods schema --out schema.json`")))?;
                out_path = Some(PathBuf::from(p));
                i += 2;
            }
            "--format" => {
                let val = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--format", "`--format text|json`")))?;
                format = match val.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    other => return Err(usage_msg(ods_core::invalid_choice("--format", other, "text|json"))),
                };
                i += 2;
            }
            "--okf" => {
                dialect = "okf".into();
                i += 1;
            }
            "--skills" => {
                dialect = "skills".into();
                i += 1;
            }
            "--spec" => {
                let p = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--spec", "`ods schema --spec ods`")))?;
                dialect = p.clone();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    if is_keys_sub {
        let registry = ods_core::SpecSchemaRegistry::with_defaults();
        let schema = registry.get(&dialect).ok_or_else(|| {
            usage_msg(ods_core::invalid_choice("--spec", &dialect, "ods|okf|skills"))
        })?;

        match format {
            OutputFormat::Text => {
                println!("Schema keys for {dialect} (v{}):", schema.version);
                let mut keys: Vec<_> = schema.keys.values().collect();
                keys.sort_by(|a, b| a.name.cmp(&b.name));
                for k in keys {
                    let req = if k.required { "[required]" } else { "[optional]" };
                    let placement_str = match k.placement {
                        ods_core::KeyPlacement::TopLevel => "top-level",
                        ods_core::KeyPlacement::NestedEngineMap => "nested (ods:)",
                        ods_core::KeyPlacement::RootIndexOnly
                        | ods_core::KeyPlacement::WorkspaceConfigOnly => "ods.toml only",
                    };
                    let alias_str = if k.aliases.is_empty() {
                        "".to_string()
                    } else {
                        format!(" (aliases: {})", k.aliases.join(", "))
                    };
                    println!("  {:<20} {:<18} {:<10} {}{}", k.name, placement_str, req, k.description, alias_str);
                }
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                let keys: Vec<_> = schema
                    .keys
                    .values()
                    .map(|k| {
                        let placement_str = match k.placement {
                            ods_core::KeyPlacement::TopLevel => "top-level",
                            ods_core::KeyPlacement::NestedEngineMap => "nested (ods:)",
                            ods_core::KeyPlacement::RootIndexOnly
                            | ods_core::KeyPlacement::WorkspaceConfigOnly => "ods.toml only",
                        };
                        serde_json::json!({
                            "name": k.name,
                            "placement": placement_str,
                            "key_type": format!("{:?}", k.key_type),
                            "required": k.required,
                            "description": k.description,
                            "aliases": k.aliases,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "dialect": dialect,
                        "version": schema.version,
                        "keys": keys,
                    }))
                    .unwrap_or_default()
                );
            }
        }
        return Ok(ExitCode::from(0));
    }

    let schema_json = match dialect.as_str() {
        "ods" => ods_core::generate_ods_json_schema(),
        other => {
            let registry = ods_core::SpecSchemaRegistry::with_defaults();
            let schema = registry
                .get(other)
                .ok_or_else(|| {
                    usage_msg(ods_core::invalid_choice("--spec", other, "ods|okf|skills"))
                })?;
            let keys: Vec<_> = schema
                .keys
                .values()
                .map(|k| {
                    serde_json::json!({
                        "name": k.name,
                        "placement": format!("{:?}", k.placement),
                        "required": k.required,
                        "description": k.description,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({
                "dialect": other,
                "version": schema.version,
                "keys": keys,
            }))
            .map_err(|e| fail_msg(ods_core::io_failed("serialize schema", e)))?
        }
    };

    if write || out_path.is_some() {
        let dest = out_path.unwrap_or_else(|| {
            if dialect == "ods" {
                PathBuf::from(".ods/ods.schema.json")
            } else {
                PathBuf::from(format!(".ods/{dialect}.schema.json"))
            }
        });
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&dest, &schema_json)
            .map_err(|e| fail_msg(ods_core::io_failed("write schema", e)))?;
        println!("wrote JSON Schema to {}", dest.display());
    } else {
        println!("{schema_json}");
    }

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_schema_command {
    use super::*;

    #[test]
    fn schema_help_and_keys_text_json() {
        assert!(run_schema_command(&["ods".into(), "schema".into(), "--help".into()]).is_ok());
        assert!(run_schema_command(&["ods".into(), "schema".into(), "keys".into()]).is_ok());
        assert!(run_schema_command(&[
            "ods".into(),
            "schema".into(),
            "keys".into(),
            "--format".into(),
            "json".into(),
        ])
        .is_ok());
        assert!(run_schema_command(&[
            "ods".into(),
            "schema".into(),
            "keys".into(),
            "--okf".into(),
            "--format".into(),
            "text".into(),
        ])
        .is_ok());
        assert!(run_schema_command(&[
            "ods".into(),
            "schema".into(),
            "keys".into(),
            "--skills".into(),
            "--format".into(),
            "json".into(),
        ])
        .is_ok());
        assert!(run_schema_command(&[
            "ods".into(),
            "schema".into(),
            "keys".into(),
            "--format".into(),
            "bad".into(),
        ])
        .is_err());
        // Bare schema still works.
        assert!(run_schema_command(&["ods".into(), "schema".into()]).is_ok());
        assert!(run_schema_command(&["ods".into(), "schema".into(), "--okf".into()]).is_ok());
    }

    #[test]
    fn schema_write_to_temp_out() {
        let td = tempfile::tempdir().unwrap();
        let out = td.path().join("ods.schema.json");
        let res = run_schema_command(&[
            "ods".into(),
            "schema".into(),
            "--out".into(),
            out.to_string_lossy().to_string(),
        ]);
        assert!(res.is_ok(), "{res:?}");
        assert!(out.exists());
    }
}
