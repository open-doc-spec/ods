use ods_core::{ReadOptions, read_document_content};

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn parse_read_positionals(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip_next = false;
    let flags_with_vals = ["--section", "--max-tokens", "--format", "--root"];
    for arg in args.iter().skip(2) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if flags_with_vals.contains(&arg.as_str()) {
            skip_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        out.push(arg.clone());
    }
    out
}

pub(crate) fn run_read_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("read");
        return Ok(ExitCode::from(0));
    }

    let positionals = parse_read_positionals(args);
    let (root_path, target) = match positionals.as_slice() {
        [ws, target] => (PathBuf::from(ws), target.clone()),
        [target] => (
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            target.clone(),
        ),
        _ => {
            return Err(usage_msg(ods_core::missing_required_arg(
                "target",
                "ods read [root] <id-or-path> [--section ...] [--summary] [--max-tokens N]",
            )));
        }
    };

    let root = resolve_root_path(root_path);
    let workspace = ods_core::load_workspace_with_options(&root, ods_core::load_options_graph())
        .map_err(|e| fail_load(&root, e))?;

    let section = get_flag(args, "--section");
    let summary_only = args.iter().any(|a| a == "--summary");
    let max_tokens = get_flag(args, "--max-tokens").and_then(|v| v.parse::<usize>().ok());
    let format = get_flag(args, "--format").unwrap_or_else(|| "text".to_string());

    let options = ReadOptions {
        section,
        summary_only,
        max_tokens,
    };

    let result = read_document_content(&workspace, &target, &options)
        .map_err(CliError::Failure)?;

    if format == "json" {
        let json_output = serde_json::to_string_pretty(&result)
            .map_err(|e| CliError::Failure(format!("JSON serialization error: {e}")))?;
        println!("{json_output}");
    } else {
        println!("{}", result.content);
        if result.truncated {
            eprintln!("\nNotice: Output was truncated to fit token cap.");
        }
    }

    Ok(ExitCode::from(0))
}
