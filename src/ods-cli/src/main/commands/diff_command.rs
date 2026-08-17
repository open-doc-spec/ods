fn run_diff_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("diff");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let target = args.get(2).map(String::as_str).unwrap_or("HEAD");

    let output = Command::new("git")
        .current_dir(&root)
        .args(["diff", target, "--name-status", "--", "*.md"])
        .output()
        .map_err(|e| fail_msg(ods_core::io_failed("git diff", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    match format {
        OutputFormat::Text => {
            if stdout.trim().is_empty() {
                println!("No Markdown document changes compared to {target}.");
            } else {
                println!("ODS Document Diff vs {target}:");
                println!("{}", stdout);
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let lines: Vec<String> = stdout
                .lines()
                .map(|line| format!(r#""{}""#, line.replace('"', "\\\"")))
                .collect();
            println!(r#"{{"target":"{}","changes":[{}]}}"#, target, lines.join(","));
        }
    }

    Ok(ExitCode::from(0))
}
