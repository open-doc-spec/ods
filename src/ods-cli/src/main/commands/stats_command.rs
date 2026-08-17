fn run_stats_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("stats");
        return Ok(ExitCode::from(0));
    }
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;

    let total_docs = workspace.documents.len();

    struct StatsAcc {
        profiles_count: std::collections::BTreeMap<String, usize>,
        tags_count: std::collections::BTreeMap<String, usize>,
        total_depends: usize,
        total_related: usize,
        unindexed_docs: usize,
        compliant_count: usize,
    }

    let acc = workspace.documents.iter().fold(
        StatsAcc {
            profiles_count: std::collections::BTreeMap::new(),
            tags_count: std::collections::BTreeMap::new(),
            total_depends: 0,
            total_related: 0,
            unindexed_docs: 0,
            compliant_count: 0,
        },
        |mut acc, doc| {
            let diags = ods_core::lint_document_in_workspace(&workspace, &doc.path, level);
            let is_parsed = matches!(doc.frontmatter, FrontmatterState::Parsed(_));
            if is_parsed && diags.is_empty() {
                acc.compliant_count += 1;
            }

            match &doc.frontmatter {
                FrontmatterState::Parsed(fm) => {
                    let prof = fm.profile.as_deref().unwrap_or("note");
                    *acc.profiles_count.entry(prof.to_string()).or_insert(0) += 1;
                    for tag in &fm.tags {
                        *acc.tags_count.entry(tag.clone()).or_insert(0) += 1;
                    }
                    acc.total_depends += fm.depends.len();
                    acc.total_related += fm.related.len();
                }
                _ => {
                    acc.unindexed_docs += 1;
                }
            }
            acc
        },
    );

    let health_pct = if total_docs == 0 {
        100.0
    } else {
        (acc.compliant_count as f64 / total_docs as f64) * 100.0
    };

    match format {
        OutputFormat::Text => {
            println!("ODS Workspace Statistics: {}", root.display());
            println!("--------------------------------------------------");
            println!("  Total Documents:       {}", total_docs);
            println!("  Health Score:          {:.1}% ({}/{} compliant)", health_pct, acc.compliant_count, total_docs);
            println!("  Graph Dependencies:    {} depends, {} related", acc.total_depends, acc.total_related);
            println!("  Unparsed/Plain Docs:   {}", acc.unindexed_docs);
            println!("\nProfiles Distribution:");
            for (prof, count) in &acc.profiles_count {
                println!("  - {}: {}", prof, count);
            }
            if !acc.tags_count.is_empty() {
                println!("\nTop Tags:");
                for (tag, count) in acc.tags_count.iter().take(10) {
                    println!("  - #{}: {}", tag, count);
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let prof_items: Vec<_> = acc.profiles_count
                .iter()
                .map(|(k, v)| format!(r#""{}":{}"#, k, v))
                .collect();
            let tag_items: Vec<_> = acc.tags_count
                .iter()
                .map(|(k, v)| format!(r#""{}":{}"#, k, v))
                .collect();
            println!(
                r#"{{"root":"{}","total_documents":{},"health_pct":{:.1},"compliant":{},"depends_count":{},"related_count":{},"unindexed":{},"profiles":{{{}}},"tags":{{{}}}}}"#,
                root.display(),
                total_docs,
                health_pct,
                acc.compliant_count,
                acc.total_depends,
                acc.total_related,
                acc.unindexed_docs,
                prof_items.join(","),
                tag_items.join(",")
            );
        }
    }

    Ok(ExitCode::from(0))
}
