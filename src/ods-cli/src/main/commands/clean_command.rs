fn run_clean_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("clean");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let ods_dir = root.join(".ods");
    let cleaned_files: Vec<PathBuf> = if ods_dir.exists() {
        ["ods-errors.md", "coverage.md", "ods.schema.json"]
            .iter()
            .map(|name| ods_dir.join(name))
            .filter(|path| path.exists() && fs::remove_file(path).is_ok())
            .collect()
    } else {
        Vec::new()
    };

    match format {
        OutputFormat::Text => {
            if cleaned_files.is_empty() {
                println!("Workspace is already clean.");
            } else {
                for file in &cleaned_files {
                    println!("Removed {}", file.display());
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<String> = cleaned_files
                .iter()
                .map(|f| format!(r#""{}""#, f.display()))
                .collect();
            println!(r#"{{"cleaned":[{}]}}"#, items.join(","));
        }
    }

    Ok(ExitCode::from(0))
}
