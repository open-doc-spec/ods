fn run_bench_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("stats");

    match sub {
        "help" | "--help" | "-h" => {
            print_command_help("bench");
        }
        "strip" => {
            let write = args.iter().any(|a| a == "--write");
            let full = args.iter().any(|a| a == "--full");
            let strip_indexes = args
                .iter()
                .any(|a| a == "--indexes" || a == "--strip-indexes");
            let strip_profiles = args
                .iter()
                .any(|a| a == "--profiles" || a == "--strip-profiles");
            let path_filter = parse_flag_val(args, "--path").map(PathBuf::from);
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let root = resolve_root_path(root);

            let options = ods_core::BenchStripOptions {
                write,
                path_filter,
                strip_indexes,
                strip_profiles,
                full,
            };
            let report = ods_core::bench_strip_workspace(&root, options)
                .map_err(|err| fail_io("bench", err))?;

            match format {
                OutputFormat::Text => {
                    let mode = if report.dry_run { "dry-run" } else { "wrote" };
                    println!(
                        "ods bench strip ({mode}) snapshot={} path={}\nProcessed {} files; stripped {} frontmatter blocks, deleted {} index lockfiles, removed {} profile files.",
                        report.snapshot_id,
                        report.snapshot_path.display(),
                        report.total_processed,
                        report.total_stripped,
                        report.total_indexes_deleted,
                        report.total_profiles_removed
                    );
                    if report.dry_run {
                        println!("Re-run with --write to apply frontmatter and workspace baseline stripping.");
                    }
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!(
                        r#"{{"snapshot_id":{},"snapshot_path":{},"total_processed":{},"total_stripped":{},"total_indexes_deleted":{},"total_profiles_removed":{},"dry_run":{}}}"#,
                        json_escape(&report.snapshot_id),
                        json_escape(&report.snapshot_path.display().to_string()),
                        report.total_processed,
                        report.total_stripped,
                        report.total_indexes_deleted,
                        report.total_profiles_removed,
                        report.dry_run
                    );
                }
            }
        }
        "restore" => {
            let snapshot_id = parse_flag_val(args, "--snapshot");
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let root = resolve_root_path(root);

            let report = ods_core::bench_restore_workspace(&root, snapshot_id.as_deref())
                .map_err(|err| fail_io("bench", err))?;

            match format {
                OutputFormat::Text => {
                    println!(
                        "ods bench restore (wrote) snapshot={}\nRestored frontmatter in {} files, {} index lockfiles, and {} profile files.",
                        report.snapshot_id,
                        report.total_restored,
                        report.total_indexes_restored,
                        report.total_profiles_restored
                    );
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!(
                        r#"{{"snapshot_id":{},"snapshot_path":{},"total_restored":{},"total_indexes_restored":{},"total_profiles_restored":{}}}"#,
                        json_escape(&report.snapshot_id),
                        json_escape(&report.snapshot_path.display().to_string()),
                        report.total_restored,
                        report.total_indexes_restored,
                        report.total_profiles_restored
                    );
                }
            }
        }
        "stats" | "roi" => {
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let root = resolve_root_path(root);

            let stats = ods_core::bench_calculate_stats(&root)
                .map_err(|err| fail_io("bench", err))?;

            match format {
                OutputFormat::Text => {
                    println!("=== ODS Value & Token Efficiency Report ===");
                    println!("Total Repository Markdown Files: {}", stats.total_files);
                    println!(
                        "Total Raw Repository Tokens:   ~{} tokens ({} bytes)",
                        stats.estimated_total_tokens, stats.total_raw_bytes
                    );
                    println!(
                        "Avg ODS Bounded Context:      ~{} tokens",
                        stats.avg_ods_context_tokens
                    );
                    println!(
                        "Token Context Savings:        {:.1}%",
                        stats.token_reduction_percentage
                    );
                    println!(
                        "Est. Dev Savings (100 queries/mo): ~${:.2} USD",
                        stats.est_monthly_cost_savings_usd
                    );
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!(
                        r#"{{"total_files":{},"total_raw_bytes":{},"estimated_total_tokens":{},"avg_ods_context_tokens":{},"token_reduction_percentage":{:.2},"est_monthly_cost_savings_usd":{:.2}}}"#,
                        stats.total_files,
                        stats.total_raw_bytes,
                        stats.estimated_total_tokens,
                        stats.avg_ods_context_tokens,
                        stats.token_reduction_percentage,
                        stats.est_monthly_cost_savings_usd
                    );
                }
            }
        }
        "run" | "agent" => {
            let prompt = parse_flag_val(args, "--prompt")
                .unwrap_or_else(|| "Refactor API endpoints".to_string());
            let agent_flag = parse_flag_val(args, "--agent");
            let is_agent_mode = sub == "agent" || agent_flag.is_some();
            let agent = agent_flag
                .or_else(|| parse_flag_val(args, "--llm"))
                .unwrap_or_else(|| "antigravity".to_string());
            let (root, _level, format) = parse_common_flags(args, 3)?;
            let root = resolve_root_path(root);

            let report = ods_core::bench_run_simulation(&root, &prompt, &agent)
                .map_err(|err| fail_io("bench", err))?;

            let fitness_score = (report.token_savings_pct * 0.9 + 10.0).min(99.9);

            match format {
                OutputFormat::Text => {
                    if is_agent_mode {
                        println!("=== ODS AI / Agent Benchmark Report ===");
                        println!("Agent Profile Target: {}", agent);
                        println!("Benchmark Prompt:     \"{}\"", prompt);
                        println!("Raw Repository Context: ~{} tokens", report.raw_context_tokens);
                        println!("ODS Bounded Context:    ~{} tokens", report.ods_context_tokens);
                        println!("Context Reduction:      {:.1}%", report.token_savings_pct);
                        println!("Agent Prompt Fitness:   {:.1}/100", fitness_score);
                        println!("Simulated USD Savings:  ${:.4} per query", report.est_raw_cost_usd - report.est_ods_cost_usd);
                    } else {
                        println!("{}", report.simulated_output);
                    }
                }
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!(
                        r#"{{"agent_profile":{},"prompt":{},"raw_context_tokens":{},"ods_context_tokens":{},"token_savings_pct":{:.2},"agent_fitness_score":{:.1},"est_raw_cost_usd":{:.4},"est_ods_cost_usd":{:.4}}}"#,
                        json_escape(&agent),
                        json_escape(&report.prompt),
                        report.raw_context_tokens,
                        report.ods_context_tokens,
                        report.token_savings_pct,
                        fitness_score,
                        report.est_raw_cost_usd,
                        report.est_ods_cost_usd
                    );
                }
            }
        }
        other => {
            return Err(usage_msg(ods_core::unknown_subcommand(
                "bench",
                other,
                "ods bench strip|restore|stats|run|agent",
            )));
        }
    }

    Ok(ExitCode::from(0))
}

fn parse_flag_val(args: &[String], flag: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            return args.get(i + 1).cloned();
        }
        if args[i].starts_with(&format!("{flag}=")) {
            return Some(args[i][flag.len() + 1..].to_string());
        }
        i += 1;
    }
    None
}
