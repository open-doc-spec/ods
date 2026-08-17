fn run_adopt_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("adopt");
        return Ok(ExitCode::from(0));
    }
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_adopt_command(args);
    }
    if !engines.ods {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "adopt_requires_ods",
                ods_core::ErrorStage::Scope,
                "adopt requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF adopt"),
        ));
    }
    let write = args.iter().any(|a| a == "--write");
    let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
    let report = adopt_workspace(&workspace, AdoptOptions { write })
        .map_err(|err| fail_msg(ods_core::io_failed("adopt", err)))?;
    // Re-load after writes for accurate lint
    let workspace = if write {
        load_workspace(&root).map_err(|err| fail_load(&root, err))?
    } else {
        workspace
    };
    let diagnostics = lint_workspace_with_level(&workspace, level);
    print_diagnostics(&diagnostics, format);
    println!("profiles: {}", known_profiles(&workspace).join(", "));
    if write {
        println!("adopt wrote {} document(s)", report.written.len());
        for path in &report.written {
            println!("  wrote {}", path.display());
        }
    } else {
        println!(
            "adopt dry-run: {} document(s) would receive frontmatter (pass --write)",
            report.would_write.len()
        );
        for path in report.would_write.iter().take(20) {
            println!("  would write {}", path.display());
        }
        if report.would_write.len() > 20 {
            println!("  ... and {} more", report.would_write.len() - 20);
        }
    }
    Ok(exit_code(&diagnostics))
}

fn run_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("init");
        return Ok(ExitCode::from(0));
    }
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    if extra.okf {
        return run_okf_init_command(args);
    }
    if extra.skills {
        return run_skills_init_command(args);
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let adopt = args.iter().any(|a| a == "--adopt");
    let report = init_workspace(&root, InitOptions { adopt })
        .map_err(|err| fail_io("init/adopt", err))?;
    match format {
        OutputFormat::Text => {
            if report.already_initialized && !report.initialized {
                println!("ODS already initialized at {}", report.root.display());
            } else if report.initialized {
                println!("initialized ODS at {}", report.root.display());
            } else {
                println!("ODS workspace {}", report.root.display());
            }
            if !report.adopted.is_empty() {
                println!("adopted {} document(s)", report.adopted.len());
            }
            println!("workspace: ods.toml ready");
            println!("next: ods lint   # or: ods watch");
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"root":{},"initialized":{},"already_initialized":{},"adopted":{},"indexes":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.initialized,
                report.already_initialized,
                report.adopted.len(),
                0
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn run_skills_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let name = args
        .windows(2)
        .find(|w| w[0] == "--name")
        .map(|w| w[1].clone());
    let report = ods_core::init_skill_package(
        &root,
        ods_core::SkillsInitOptions { name },
    )
    .map_err(|e| fail_io("init/adopt", e))?;
    match format {
        OutputFormat::Text => {
            println!("initialized Agent Skills package at {}", report.root.display());
            for p in &report.created {
                println!("  created {}", p.display());
            }
            for p in &report.skipped {
                println!("  skipped {}", p.display());
            }
            println!("next: ods lint --skills");
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"root":{},"created":{},"skipped":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.created.len(),
                report.skipped.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_adopt_init {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn adopt_and_init_command_paths() {
        let td = tempdir().unwrap();
        let root = td.path();
        let path = root.to_str().unwrap().to_string();
        fs::write(root.join("plain.md"), "# plain\n").unwrap();

        // init
        let res = run_init_command(&[
            "ods".into(),
            "init".into(),
            path.clone(),
        ]);
        assert!(res.is_ok());

        let res = run_init_command(&[
            "ods".into(),
            "init".into(),
            path.clone(),
            "--adopt".into(),
        ]);
        assert!(res.is_ok());

        let res = run_adopt_command(&["ods".into(), "adopt".into(), path.clone()]);
        assert!(res.is_ok());

        let res = run_adopt_command(&[
            "ods".into(),
            "adopt".into(),
            path.clone(),
            "--write".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_adopt_command(&[
            "ods".into(),
            "adopt".into(),
            path.clone(),
            "--write".into(),
        ]);
        assert!(res.is_ok());

        let skill = root.join("skill-pkg");
        fs::create_dir_all(&skill).unwrap();
        let res = run_skills_init_command(&[
            "ods".into(),
            "init".into(),
            skill.to_str().unwrap().into(),
            "--skills".into(),
        ]);
        assert!(res.is_ok());

        // init --okf and init --force
        let okf_dir = root.join("okf-bundle");
        fs::create_dir_all(&okf_dir).unwrap();
        let res = run_init_command(&[
            "ods".into(),
            "init".into(),
            okf_dir.to_str().unwrap().into(),
            "--okf".into(),
        ]);
        assert!(res.is_ok());

        let res = run_init_command(&[
            "ods".into(),
            "init".into(),
            path.clone(),
        ]);
        assert!(res.is_ok());

        let res = run_adopt_command(&[
            "ods".into(),
            "adopt".into(),
            okf_dir.to_str().unwrap().into(),
            "--okf".into(),
        ]);
        assert!(res.is_ok());

        let res = run_skills_init_command(&[
            "ods".into(),
            "init".into(),
            skill.to_str().unwrap().into(),
            "--skills".into(),
            "--name".into(),
            "my-skill".into(),
        ]);
        assert!(res.is_ok());
        let _ = path;
    }
}

