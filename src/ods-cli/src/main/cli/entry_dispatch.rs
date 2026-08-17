

fn dispatch_platform_command(args: &[String]) -> Result<ExitCode, CliError> {
    let command = args.get(1).map(String::as_str).unwrap_or("");
    match command {
        "--version" | "-V" | "version" => {
            println!("ods {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::from(0))
        }
        "--help" | "-h" | "help" => {
            print_help();
            Ok(ExitCode::from(0))
        }
        "setup" => run_setup_command(args),
        "update" => run_update_command(args),
        "upgrade" => run_upgrade_command(args),
        "workspaces" => run_workspaces_command(args),
        "skill" => run_skill_command(args),
        "pack" => run_pack_command(args),
        "stats" => run_stats_command(args),
        "completion" => run_completion_command(args),
        "schema" => run_schema_command(args),
        "tree" => run_tree_command(args),
        "diff" => run_diff_command(args),
        "clean" => run_clean_command(args),
        "lsp" => run_lsp_command(args),
        other => Err(usage_msg(ods_core::unknown_platform_command(other))),
    }
}

fn dispatch_ods_command(args: &[String]) -> Result<ExitCode, CliError> {
    let command = args.get(1).map(String::as_str).unwrap_or("");
    match command {
        "--version" | "-V" | "version" => {
            println!("ods {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::from(0))
        }
        "--help" | "-h" | "help" => {
            print_ods_help();
            Ok(ExitCode::from(0))
        }
        "lint" => run_lint_command(args),
        "index" => {
            if args.iter().any(|a| a == "--okf") {
                run_okf_index_command(args)
            } else {
                eprintln!("error: ods index was removed for ODS (use overview/find/tree/context)");
                eprintln!("Next: ods overview  |  for OKF: ods index --okf");
                Ok(ExitCode::from(2))
            }
        }
        "profile" | "profiles" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            match sub {
                "init" => run_profile_init_command(args),
                "show" => run_profile_show_command(args),
                "list" | "" => run_profile_list_command(args),
                other if other.starts_with('-') => run_profile_list_command(args),
                _ => run_profile_list_command(args),
            }
        }
        "tags" => run_tags_command(args),
        "find" => run_find_command(args),
        "tag" => run_tag_command(args),
        "context" => run_context_command(args),
        "graph" => run_graph_command(args),
        "mv" => run_mv_command(args),
        "fmt" => run_fmt_command(args),
        "adopt" => run_adopt_command(args),
        "new" => run_new_command(args),
        "rm" | "remove" => run_rm_command(args),
        "status" => run_status_command(args),
        "archive" => run_archive_command(args),
        "init" | "enable" => run_init_command(args),
        "disable" | "revert" => run_disable_command(args),
        "doctor" => run_doctor_command(args),
        "sync" => run_sync_command(args),
        "watch" => run_watch_command(args),
        "logs" => run_logs_command(args),
        "serve" => run_serve_command(args),
        "export" => run_export_command(args),
        "start" => run_start_command(args),
        "stop" => run_stop_command(args),
        "share" => run_share_command(args),
        "bench" | "sandbox" => run_bench_command(args),
        "audit" => run_ods_audit_command(args),
        "coverage" => run_coverage_command(args),
        "stats" => run_stats_command(args),
        "overview" | "summary" => run_overview_command(args),
        "completion" => run_completion_command(args),
        "schema" => run_schema_command(args),
        "tree" => run_tree_command(args),
        "diff" => run_diff_command(args),
        "clean" => run_clean_command(args),
        "lsp" => run_lsp_command(args),
        "read" => run_read_command(args),
        "undo" => run_undo_command(args),
        "update" => run_update_command(args),
        "upgrade" => run_upgrade_command(args),
        other => {
            let suggestion = ods_core::suggest_command(other);
            Err(usage_msg(ods_core::unknown_ods_command(other, suggestion)))
        }
    }
}
