fn run_doctor_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("doctor");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;

    let mut has_error = false;

    if engines.ods {
        let report = doctor_workspace(&root)?;
        match format {
            OutputFormat::Text => println!("{}", report.text),
            OutputFormat::Json | OutputFormat::Sarif => println!("{}", report.json),
        }
        has_error |= report.has_error;
    }

    if engines.okf {
        // OKF doctor (flag path: `ods doctor --okf`).
        return run_okf_doctor_command(args);
    }

    if engines.skills && !engines.ods {
        println!("skills: package detected; full doctor lands with skills engine (use `ods lint --skills`).");
    }

    Ok(ExitCode::from(if has_error { 1 } else { 0 }))
}

fn run_sync_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("sync");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let report = sync_git_renames(&root)?;
    print_path_change_report(&root, "git", "sync", &report, format, "synced");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}

fn run_watch_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("watch");
        return Ok(ExitCode::from(0));
    }
    maybe_auto_update_on_watch();
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_watch_command(args, false);
    }
    if !engines.ods {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "watch_requires_ods",
                ods_core::ErrorStage::Scope,
                "watch requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF watch"),
        ));
    }
    watch_workspace(&root, level, format, false)?;
    Ok(ExitCode::from(0))
}

fn run_serve_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("serve");
        return Ok(ExitCode::from(0));
    }
    // Headless loop for OS service (no interactive green spam).
    let options = serve_options_from_args(args)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&options.root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_watch_command(args, true);
    }
    if !engines.ods {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "serve_requires_ods",
                ods_core::ErrorStage::Scope,
                "serve requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF serve"),
        ));
    }
    serve_workspace(options)?;
    Ok(ExitCode::from(0))
}

fn run_export_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("export");
        return Ok(ExitCode::from(0));
    }
    let (root, out, format, spec) = parse_export_args(args)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let want_okf = extra.okf || spec.starts_with("okf");

    if want_okf {
        if !detected.okf {
            return Err(fail_msg(ods_core::not_okf_bundle()));
        }
        // JSON with --spec okf still uses ODS workspace renderer when ODS is present.
        if matches!(format, OutputFormat::Json) && detected.ods {
            // fall through to ODS JSON export with okf spec tag
        } else {
            return run_okf_export_command(args);
        }
    }

    if !detected.ods {
        if detected.okf {
            return run_okf_export_command(args);
        }
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "export_requires_ods",
                ods_core::ErrorStage::Scope,
                "export requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF export"),
        ));
    }
    let include_private = args.iter().any(|a| a == "--include-private");

    match format {
        OutputFormat::Json => {
            let workspace = ods_core::load_workspace(&root).map_err(|e| fail_load(&root, e))?;
            let json_str = ods_core::render_graph_json(&workspace, include_private, &spec);
            println!("{json_str}");
        }
        OutputFormat::Text | OutputFormat::Sarif => {
            let path = export_workspace_graph(&root, &out, include_private)
                .map_err(|e| fail_io("export", e))?;
            println!("wrote {}", path.display());
            if !include_private {
                println!("(documents marked share: private or share: org were omitted; pass --include-private to include them)");
            }
        }
    }
    Ok(ExitCode::from(0))
}

fn run_start_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("start");
        return Ok(ExitCode::from(0));
    }
    let status_only = args.iter().any(|a| a == "--status");
    let (root, _level, _format) = parse_common_flags(args, 2)?;
    let root = resolve_root_path(root);
    require_ods_workspace(&root)?;
    if status_only {
        let st = service::service_status(&root);
        println!(
            "installed={} running={} ({})",
            st.installed, st.running, st.detail
        );
        return Ok(ExitCode::from(if st.running { 0 } else { 1 }));
    }
    let msg = service::start_service(&root)
        .map_err(|e| fail_msg(ods_core::service_failed("start service", e)))?;
    println!("{msg}");
    Ok(ExitCode::from(0))
}

fn run_stop_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("stop");
        return Ok(ExitCode::from(0));
    }
    let unregister = args.iter().any(|a| a == "--unregister");
    let (root, _level, _format) = parse_common_flags(args, 2)?;
    let root = resolve_root_path(root);
    require_ods_workspace(&root)?;
    let msg = service::stop_service(&root, unregister)
        .map_err(|e| fail_msg(ods_core::service_failed("stop service", e)))?;
    println!("{msg}");
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_service_commands {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_doctor_export_start_stop_commands() {
        let td = tempdir().unwrap();
        let root = td.path();
        let path = root.to_str().unwrap().to_string();

        // 1. non-ODS workspace doctor error / skills hint
        let res = run_doctor_command(&["ods".into(), "doctor".into(), path.clone()]);
        assert!(res.is_err());

        // 2. non-ODS workspace export error
        let err = run_export_command(&["ods".into(), "export".into(), path.clone()]).unwrap_err();
        assert!(err.message().contains("ODS workspace") || err.message().contains("export"));

        // Setup ODS workspace
        std::fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        std::fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        // 3. doctor command text & json
        let res = run_doctor_command(&["ods".into(), "doctor".into(), path.clone()]);
        assert!(res.is_ok());

        let res = run_doctor_command(&[
            "ods".into(),
            "doctor".into(),
            path.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        // 4. export command text, json, include-private
        let out_graph = root.join("graph.md");
        let res = run_export_command(&[
            "ods".into(),
            "export".into(),
            path.clone(),
            "--out".into(),
            out_graph.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
        assert!(out_graph.exists());

        let res = run_export_command(&[
            "ods".into(),
            "export".into(),
            path.clone(),
            "--format".into(),
            "json".into(),
            "--include-private".into(),
        ]);
        assert!(res.is_ok());

        // 5. sync command (non-git repo returns Err)
        let _ = run_sync_command(&["ods".into(), "sync".into(), path.clone()]);
    }
}
