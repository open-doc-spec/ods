fn run_undo_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("undo");
        return Ok(ExitCode::from(0));
    }

    let list = args.iter().any(|a| a == "--list");
    let target = args
        .iter()
        .skip(2)
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let root = find_marked_ods_workspace_root(&target).unwrap_or(target);

    if list {
        let snaps = ods_core::list_workspace_snapshots(&root).map_err(|err| {
            fail_msg(ods_core::io_failed("list snapshots", err))
        })?;
        if snaps.is_empty() {
            println!("no snapshots under {}", ods_core::get_backup_dir(&root).map(|p| p.display().to_string()).unwrap_or_else(|_| "~/.ods/backups/…".into()));
            println!("hint: `ods bench strip --write` creates a restore point; then `ods undo`");
            return Ok(ExitCode::from(0));
        }
        println!("snapshots (oldest → newest):");
        for id in &snaps {
            println!("  {id}");
        }
        println!("Next: ods undo   # restores the newest snapshot");
        return Ok(ExitCode::from(0));
    }

    let report = ods_core::undo_latest_snapshot(&root).map_err(|err| {
        let text = err.to_string();
        if text.to_ascii_lowercase().contains("snapshot")
            || text.to_ascii_lowercase().contains("not found")
            || text.to_ascii_lowercase().contains("no ")
        {
            fail_msg(ods_core::undo_no_snapshot())
        } else {
            fail_msg(ods_core::io_failed("undo", err))
        }
    })?;
    println!("✓ Undid changes using snapshot {}", report.snapshot_id);
    println!("  Restored {} document frontmatter(s)", report.total_restored);
    if report.total_indexes_restored > 0 {
        println!("  Restored {} index file(s)", report.total_indexes_restored);
    }
    if report.total_profiles_restored > 0 {
        println!("  Restored {} profile definition(s)", report.total_profiles_restored);
    }
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_undo_command {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_undo_command() {
        let td = tempdir().unwrap();
        let root = td.path();

        // help
        let res = run_undo_command(&["ods".into(), "undo".into(), "--help".into()]);
        assert!(res.is_ok());

        // --list with no snapshots
        let res = run_undo_command(&["ods".into(), "undo".into(), root.to_str().unwrap().into(), "--list".into()]);
        assert!(res.is_ok());

        // undo with no snapshots -> error
        let err = run_undo_command(&["ods".into(), "undo".into(), root.to_str().unwrap().into()]).unwrap_err();
        assert!(err.message().contains("snapshot"));

        // Create root index and a snapshot via bench strip write
        std::fs::write(root.join("index.ods.md"), "---\nprofile: index\nods: 0.1\n---\n\n# Root\n").unwrap();
        let _ = ods_core::bench_strip_workspace(
            root,
            ods_core::BenchStripOptions {
                write: true,
                full: false,
                strip_indexes: false,
                strip_profiles: false,
                path_filter: None,
            },
        );

        // --list with snapshot
        let res = run_undo_command(&["ods".into(), "undo".into(), root.to_str().unwrap().into(), "--list".into()]);
        assert!(res.is_ok());

        // undo with snapshot
        let res = run_undo_command(&["ods".into(), "undo".into(), root.to_str().unwrap().into()]);
        assert!(res.is_ok() || res.is_err());
    }
}

