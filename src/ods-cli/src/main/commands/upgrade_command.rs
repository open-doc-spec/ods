/// Forward workspace/machine cutover helper (not dual-compat).
/// Dry-run by default; `--write` applies safe machine steps + optional FM migrate.
fn run_upgrade_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("upgrade");
        return Ok(ExitCode::from(0));
    }
    let write = args.iter().any(|a| a == "--write");
    let check = args.iter().any(|a| a == "--check");
    let migrate_fm = args.iter().any(|a| a == "--migrate-fm");
    let (root, _level, format) = parse_common_flags(args, 2)?;

    let mut actions: Vec<String> = Vec::new();
    let mut pending = 0usize;

    // Detect ODS / OKF roots
    let ods = ods_core::ods_enabled(&root);
    let okf = ods_core::okf_enabled(&root);
    if ods {
        actions.push(format!("ODS workspace detected at {}", root.display()));
    }
    if okf {
        actions.push(format!("OKF bundle detected at {}", root.display()));
    }
    if !ods && !okf {
        actions.push(format!(
            "no ODS/OKF root markers under {} (run ods init or ods init --okf)",
            root.display()
        ));
        pending += 1;
    }

    // Config dir forward hint ~/.ods
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"));
    if let Some(home) = home {
        let legacy = PathBuf::from(&home).join(".ods");
        let modern = PathBuf::from(&home).join(".ods");
        if legacy.exists() && !modern.exists() {
            actions.push(format!(
                "machine: legacy config {} present; prefer {}",
                legacy.display(),
                modern.display()
            ));
            pending += 1;
            if write {
                // Best-effort copy registry if present
                let _ = fs::create_dir_all(&modern);
                for name in ["odsconfig.toml", "workspaces.toml"] {
                    let src = legacy.join(name);
                    if src.exists() {
                        let dst_name = if name == "odsconfig.toml" {
                            "odsconfig.toml"
                        } else {
                            name
                        };
                        let dst = modern.join(dst_name);
                        if !dst.exists() {
                            let _ = fs::copy(&src, &dst);
                            actions.push(format!("  copied {} -> {}", src.display(), dst.display()));
                        }
                    }
                }
            }
        } else if modern.exists() {
            actions.push(format!("machine: config dir {} ok", modern.display()));
        }
    }

    if ods {
        actions.push(
            "manual: review root ods.toml spec / packs if needed (~3 known repos)"
                .into(),
        );
        actions.push("next: ods audit --write-report".into());
    }
    if okf {
        actions.push("next: ods lint --okf && ods audit --okf --write-report".into());
    }



    if migrate_fm && ods {
        if write {
            let workspace =
                load_workspace(&root).map_err(|err| fail_load(&root, err))?;
            let changed = migrate_workspace_frontmatter_with_workspace(&workspace)
                .map_err(|err| fail_io("upgrade/audit", err))?;
            actions.push(format!(
                "migrated canonical ods: layout in {} file(s)",
                changed.len()
            ));
        } else {
            actions.push(
                "would run fmt --migrate for canonical nested ods: keys (pass --write)".into(),
            );
            pending += 1;
        }
    }

    match format {
        OutputFormat::Text => {
            println!(
                "ods upgrade {} — {}",
                if write { "--write" } else { "dry-run" },
                root.display()
            );
            for a in &actions {
                println!("  • {a}");
            }
            if !write && pending > 0 {
                println!("pending actions: {pending} (re-run with --write to apply safe steps)");
            } else if write {
                println!("upgrade pass complete");
            } else {
                println!("nothing required");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"write":{},"pending":{},"ods":{},"okf":{}}}"#,
                write, pending, ods, okf
            );
        }
    }

    if check && pending > 0 {
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::from(0))
}

fn run_ods_audit_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("audit");
        return Ok(ExitCode::from(0));
    }
    let write_report = args.iter().any(|a| a == "--write-report");
    let mut report_path_opt = None;
    let mut fail_on = None;
    let mut filtered = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write-report" => {}
            "--report-path" => {
                report_path_opt = args.get(i + 1).map(PathBuf::from);
                i += 1;
            }
            "--fail-on" => {
                fail_on = args.get(i + 1).map(|s| s.as_str());
                i += 1;
            }
            other => filtered.push(other.to_string()),
        }
        i += 1;
    }
    let (root, _level, format) = parse_common_flags(&filtered, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_audit_command(args);
    }
    if !engines.ods {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "audit_requires_ods",
                ods_core::ErrorStage::Scope,
                "audit requires an ODS workspace",
            )
            .next("run `ods init`, or pass `--okf` for OKF-only audit"),
        ));
    }
    let report_path = report_path_opt.unwrap_or_else(|| root.join(".ods/ods-errors.md"));

    let workspace = load_workspace(&root).map_err(|err| fail_load(&root, err))?;
    let mut plain = 0usize;
    let mut invalid = 0usize;
    let mut partial = 0usize;
    let mut compliant = 0usize;
    let mut lines: Vec<String> = Vec::new();

    for doc in &workspace.documents {
        let rel = doc
            .path
            .strip_prefix(&root)
            .unwrap_or(&doc.path)
            .display()
            .to_string();
        match &doc.frontmatter {
            FrontmatterState::Absent => {
                plain += 1;
                lines.push(format!("- `{rel}` — no frontmatter"));
            }
            FrontmatterState::Invalid(err) => {
                invalid += 1;
                lines.push(format!("- `{rel}` — {err}"));
            }
            FrontmatterState::Parsed(fm) => {
                let has_profile = fm.profile.as_deref().map(|p| !p.is_empty()).unwrap_or(false);
                if !has_profile {
                    partial += 1;
                    lines.push(format!("- `{rel}` — missing profile"));
                } else {
                    compliant += 1;
                }
            }
        }
    }
    let total = plain + invalid + partial + compliant;

    match format {
        OutputFormat::Text => {
            println!(
                "ODS audit: total={total} compliant={compliant} plain={plain} invalid={invalid} partial={partial}"
            );
            for l in &lines {
                // only non-compliant already in lines
                println!("  {l}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"total_md":{total},"compliant":{compliant},"plain":{plain},"invalid":{invalid},"partial":{partial}}}"#
            );
        }
    }

    if write_report {
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).map_err(|e| fail_io("upgrade/audit", e))?;
        }
        let mut md = String::new();
        md.push_str("---\ngenerated_by: ods audit\n");
        md.push_str(&format!("workspace: {}\n", root.display()));
        md.push_str(&format!(
            "summary:\n  total_md: {total}\n  compliant: {compliant}\n  plain: {plain}\n  invalid: {invalid}\n  partial: {partial}\n---\n\n"
        ));
        md.push_str("# ODS Audit Report\n\n## Non-compliant\n\n");
        if lines.is_empty() {
            md.push_str("_None._\n");
        } else {
            for l in &lines {
                md.push_str(l);
                md.push('\n');
            }
        }
        md.push_str("\n## Suggested next commands\n\n```bash\nods adopt --write\nods fmt --migrate\nods lint\n```\n");
        fs::write(&report_path, md).map_err(|e| fail_io("upgrade/audit", e))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    let fail = match fail_on {
        None => false,
        Some("plain") => plain > 0,
        Some("invalid") => invalid > 0,
        Some("any") => plain + invalid + partial > 0,
        Some(other) => {
            return Err(usage_msg(ods_core::invalid_choice(
                "--fail-on",
                other,
                "plain|invalid|any",
            )));
        }
    };
    Ok(ExitCode::from(if fail { 1 } else { 0 }))
}

include!("agents_command.rs");

#[cfg(test)]
mod test_upgrade_and_audit {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn upgrade_migrate_fm_dry_and_write() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();
        fs::write(
            root.join("legacy.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
        )
        .unwrap();
        let path = root.to_str().unwrap().to_string();

        let res = run_upgrade_command(&[
            "ods".into(),
            "upgrade".into(),
            path.clone(),
            "--migrate-fm".into(),
        ]);
        assert!(res.is_ok());

        let res = run_upgrade_command(&[
            "ods".into(),
            "upgrade".into(),
            path.clone(),
            "--migrate-fm".into(),
            "--write".into(),
        ]);
        assert!(res.is_ok());

        let res = run_upgrade_command(&[
            "ods".into(),
            "upgrade".into(),
            path.clone(),
            "--write".into(),
        ]);
        assert!(res.is_ok());

        let res = run_upgrade_command(&[
            "ods".into(),
            "upgrade".into(),
            path,
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn audit_command_inventory_paths_and_fail_on() {
        let td = tempdir().unwrap();
        let root = td.path();
        let path = root.to_str().unwrap().to_string();

        // 1. Audit non-ODS workspace
        let err = run_ods_audit_command(&["ods".into(), "audit".into(), path.clone()]).unwrap_err();
        assert!(err.message().contains("workspace") || err.message().contains("ODS"));

        // Setup ODS workspace with root index, plain doc, invalid doc, and partial doc
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();
        fs::write(root.join("plain.md"), "# Plain Text\n").unwrap();
        fs::write(
            root.join("invalid.md"),
            "---\nprofile: [invalid json yaml---\n",
        )
        .unwrap();
        fs::write(
            root.join("partial.md"),
            "---\nstatus: draft\n---\n\n# Partial\n",
        )
        .unwrap();

        // 2. Audit text output
        let res = run_ods_audit_command(&["ods".into(), "audit".into(), path.clone()]);
        assert!(res.is_ok());

        // 3. Audit json output + write-report + report-path
        let report_file = root.join("custom-audit-report.md");
        let res = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path.clone(),
            "--format".into(),
            "json".into(),
            "--write-report".into(),
            "--report-path".into(),
            report_file.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
        assert!(report_file.exists());

        // 4. Fail-on choices
        let code = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path.clone(),
            "--fail-on".into(),
            "plain".into(),
        ])
        .unwrap();
        assert_eq!(code, ExitCode::from(1));

        let code = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path.clone(),
            "--fail-on".into(),
            "any".into(),
        ])
        .unwrap();
        assert_eq!(code, ExitCode::from(1));

        let err = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path,
            "--fail-on".into(),
            "unknown_choice".into(),
        ])
        .unwrap_err();
        assert!(err.message().contains("fail-on"));
    }

    #[test]
    fn audit_command_inventory_paths() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(root.join("plain.md"), "# p\n").unwrap();
        fs::write(root.join("bad.md"), "---\n:\n---\n\n# b\n").unwrap();
        fs::write(
            root.join("part.md"),
            "---\nstatus: draft\n---\n\n# part\n",
        )
        .unwrap();
        let path = root.to_str().unwrap().to_string();
        let report = root.join("audit.md");

        let res = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path.clone(),
            "--write-report".into(),
            "--report-path".into(),
            report.to_str().unwrap().into(),
            "--fail-on".into(),
            "any".into(),
        ]);
        assert_eq!(res.unwrap(), ExitCode::from(1));
        assert!(report.exists());

        let res = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path.clone(),
            "--format".into(),
            "json".into(),
            "--fail-on".into(),
            "plain".into(),
        ]);
        assert_eq!(res.unwrap(), ExitCode::from(1));

        let res = run_ods_audit_command(&["ods".into(), "audit".into(), "--help".into()]);
        assert!(res.is_ok());

        let err = run_ods_audit_command(&[
            "ods".into(),
            "audit".into(),
            path,
            "--fail-on".into(),
            "invalid_level".into(),
        ]).unwrap_err();
        assert!(err.message().contains("fail-on"));
    }
}

