fn run_share_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("share");
        return Ok(ExitCode::from(0));
    }
    let (root, scope, out, include_org, include_private) = parse_share_args(args)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;
    let report = ods_core::publish_workspace(
        &workspace,
        &scope,
        &out,
        ods_core::ShareOptions {
            include_org,
            include_private,
        },
    )
    .map_err(|e| fail_io("share", e))?;

    println!(
        "wrote {} document(s) to {}",
        report.written.len(),
        out.display()
    );
    if !report.excluded.is_empty() {
        println!(
            "({} document(s) excluded by share visibility; pass --include-org/--include-private to include more)",
            report.excluded.len()
        );
    }
    println!("(this only writes files; run git init/add/commit/push yourself to publish {})", out.display());
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_share_command {
    use super::*;

    #[test]
    fn share_command_not_an_ods_workspace_error() {
        let td = tempfile::tempdir().unwrap();
        let args = vec!["ods".into(), "share".into(), td.path().to_string_lossy().to_string(), "--out".into(), td.path().join("out").to_string_lossy().to_string()];
        let res = run_share_command(&args);
        assert!(res.is_err());
    }
}
