fn run_lint_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("lint");
        return Ok(ExitCode::from(0));
    }
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);

    let mut root_specs = ods_core::load_root_specs_config(&root);
    parse_key_suppression_flags(args, &mut root_specs);

    let engines = ods_core::resolve_engines_with_config(extra, detected, Some(&root_specs), true)
        .map_err(|e| failure(e.message()))?;

    // Pure OKF: dedicated runner (keeps formatting parity with OKF-only messages).
    if engines.okf && !engines.ods && !engines.skills {
        return run_okf_lint_command_with_config(args, &root_specs.okf);
    }

    let mut diagnostics = Vec::new();

    if engines.ods {
        let canonical_refs = args.iter().any(|arg| arg == "--canonical-refs");
        let workspace = load_workspace_with_options(&root, load_options_graph())
            .map_err(|err| fail_load(&root, err))?;
        let fix = args.iter().any(|arg| arg == "--fix");
        if fix && matches!(format, OutputFormat::Text) {
            println!(
                "Note: nested index generation was removed. --fix does not rewrite files; use `ods overview` / `ods find` / `ods tree` for discovery, and `ods fmt --migrate` for frontmatter shape."
            );
        }
        let ods_diags = if canonical_refs {
            lint_workspace_with_ref_style(&workspace, level, true)
        } else {
            lint_workspace_with_level(&workspace, level)
        };
        diagnostics.extend(ods_diags);
    }

    if engines.okf {
        let bundle = ods_core::load_okf_bundle(&root)
            .map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
        let _ = level;
        let okf_level = ods_core::OkfLintLevel::Level3;
        let mut okf_diags =
            ods_core::lint_okf_bundle_with_config(&bundle, okf_level, &root_specs.okf);
        for d in &mut okf_diags {
            if !d.message.starts_with("[okf]") {
                d.message = format!("[okf] {}", d.message);
            }
        }
        diagnostics.extend(okf_diags);
    }

    if engines.skills {
        let packages = ods_core::skill_package_roots(&root);
        if packages.is_empty() {
            diagnostics.push(ods_core::Diagnostic {
                path: root.clone(),
                severity: ods_core::Severity::Error,
                message: ods_core::error::skills_no_package(),
            });
        }
        for pkg_root in packages {
            match ods_core::parse_skill_package(&pkg_root) {
                Ok(pkg) => diagnostics
                    .extend(ods_core::lint_skill_package_with_config(&pkg, &root_specs.skills)),
                Err(e) => diagnostics.push(ods_core::Diagnostic {
                    path: pkg_root.join("SKILL.md"),
                    severity: ods_core::Severity::Error,
                    message: format!("[skills] failed to load package: {e}"),
                }),
            }
        }
    }

    print_diagnostics(&diagnostics, format);
    if engines.ods {
        write_or_clear_ods_error_report(&root, &diagnostics, format)?;
    }
    if diagnostics
        .iter()
        .all(|d| d.severity != ods_core::Severity::Error)
        && matches!(format, OutputFormat::Text)
    {
        if diagnostics.is_empty() {
            println!("Everything is fine — graph and links are consistent. No update required.");
        } else {
            println!("Lint finished with warnings only.");
        }
    }
    Ok(exit_code(&diagnostics))
}

fn run_tags_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("tags");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let include_all = args.iter().any(|a| a == "--all");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_io("index/lint io", err))?;
    print_tags(&workspace, include_all, format);
    Ok(ExitCode::from(0))
}

fn run_coverage_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("coverage");
        return Ok(ExitCode::from(0));
    }
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let write_report = args.iter().any(|a| a == "--write-report");
    let summary_only = args.iter().any(|a| a == "--summary");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_io("index/lint io", err))?;

    let total = workspace.documents.len();
    let mut compliant = 0usize;
    let mut non_compliant = 0usize;
    struct NonCompliantFile {
        rel_path: String,
        reason: String,
    }
    let mut non_compliant_items: Vec<NonCompliantFile> = Vec::new();

    for doc in &workspace.documents {
        let is_parsed = matches!(doc.frontmatter, ods_core::FrontmatterState::Parsed(_));
        let diags = ods_core::lint_document_in_workspace(&workspace, &doc.path, level);
        let rel_path = doc
            .path
            .strip_prefix(&root)
            .unwrap_or(&doc.path)
            .display()
            .to_string();

        if is_parsed && diags.is_empty() {
            compliant += 1;
        } else {
            non_compliant += 1;
            let reason = if !is_parsed {
                "unparsed frontmatter or YAML syntax error".to_string()
            } else {
                diags
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            non_compliant_items.push(NonCompliantFile { rel_path, reason });
        }
    }

    let pct = if total == 0 {
        100.0
    } else {
        (compliant as f64 / total as f64) * 100.0
    };

    match format {
        OutputFormat::Text => {
            println!("Documentation Health: {:.1}% Compliant ({}/{} files)", pct, compliant, total);
            println!("  ✔ Compliant:     {} documents", compliant);
            println!("  ✖ Non-Compliant:  {} documents", non_compliant);
            if !non_compliant_items.is_empty() && !summary_only {
                println!("\nNon-Compliant Documents:");
                for item in &non_compliant_items {
                    println!("  • {} ({})", item.rel_path, item.reason);
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let json_files: Vec<_> = non_compliant_items
                .iter()
                .map(|item| {
                    format!(
                        r#"{{"path":{},"reason":{}}}"#,
                        json_escape(&item.rel_path),
                        json_escape(&item.reason)
                    )
                })
                .collect();
            println!(
                r#"{{"health_pct":{:.1},"compliant":{},"non_compliant":{},"total":{},"non_compliant_files":[{}]}}"#,
                pct,
                compliant,
                non_compliant,
                total,
                json_files.join(",")
            );
        }
    }

    if write_report {
        let mut report_content = format!(
            "# Documentation Health & Coverage Report\n\n- Score: {:.1}% Compliant\n- Compliant Documents: {}\n- Non-Compliant Documents: {}\n- Total Documents: {}\n\n",
            pct, compliant, non_compliant, total
        );
        if !non_compliant_items.is_empty() {
            report_content.push_str("## Non-Compliant Documents\n\n| File | Issue / Reason |\n| --- | --- |\n");
            for item in &non_compliant_items {
                report_content.push_str(&format!("| `{}` | {} |\n", item.rel_path, item.reason));
            }
            report_content.push('\n');
        }
        report_content.push_str("Note: this is separate from lint/audit diagnostics (`.ods/ods-errors.md`).\n");

        let ods_dir = root.join(".ods");
        let _ = std::fs::create_dir_all(&ods_dir);
        let report_path = ods_dir.join("coverage.md");
        std::fs::write(&report_path, report_content)
            .map_err(|e| fail_msg(ods_core::io_failed("write report", e)))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_lint_index_commands {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lint_fix_canonical_skills_and_index_check() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            root.join("x.md"),
            "---\nprofile: note\nstatus: draft\ndepends:\n  - missing\n---\n\n# X\n",
        )
        .unwrap();
        let skill = root.join("skills/demo");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: demo\ndescription: Lint skills hybrid package.\n---\n\n# D\n",
        )
        .unwrap();
        let path = root.to_str().unwrap().to_string();

        for args in [
            vec!["ods".into(), "lint".into(), path.clone(), "--fix".into()],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--canonical-refs".into(),
            ],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--skills".into(),
                "--format".into(),
                "json".into(),
            ],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--skills".into(),
                "--format".into(),
                "text".into(),
            ],
        ] {
            let res = run_lint_command(&args);
            assert!(res.is_ok());
        }

        let path = root.to_str().unwrap().to_string();
        let res = run_overview_command(&["ods".into(), "overview".into(), path.clone()]);
        assert!(res.is_ok());
        let res = run_find_command(&[
            "ods".into(),
            "find".into(),
            "--root".into(),
            path.clone(),
            "--key".into(),
            "status=draft".into(),
        ]);
        assert!(res.is_ok());
        let res = run_tree_command(&["ods".into(), "tree".into(), path.clone()]);
        assert!(res.is_ok());

        // fmt command
        let res = run_fmt_command(&["ods".into(), "fmt".into(), path.clone()]);
        assert!(res.is_ok());

        let res = run_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            path.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
    }
}

