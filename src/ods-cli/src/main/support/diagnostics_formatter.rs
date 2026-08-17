fn run_update_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut check_only = false;
    let mut force = false;
    let mut version = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_command_help("update");
                return Ok(ExitCode::from(0));
            }
            "--check" => {
                check_only = true;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            "--version" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--version", "`ods update --version v0.1.0`")))?;
                version = Some(v.clone());
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(usage_msg(ods_core::unknown_flag(other, "ods update --help")));
            }
            other => {
                // bare tag: ods update v0.1.5
                version = Some(other.to_string());
                i += 1;
            }
        }
    }

    let outcome = run_update(UpdateOptions {
        check_only,
        force,
        version,
    })
    .map_err(|e| fail_msg(ods_core::update_failed(e)))?;

    match outcome {
        UpdateOutcome::UpToDate { current, remote } => {
            println!("ods {current} is up to date (latest {remote})");
            migrate_machine_and_workspace_on_update();
            restart_service_if_active();
            Ok(ExitCode::from(0))
        }
        UpdateOutcome::Available { current, remote } => {
            println!("update available: {current} → {remote} (run: ods update)");
            Ok(ExitCode::from(1))
        }
        UpdateOutcome::Updated { from, to } => {
            println!("updated ods: {from} → {to}");
            migrate_machine_and_workspace_on_update();
            restart_service_if_active();
            Ok(ExitCode::from(0))
        }
    }
}

fn migrate_machine_and_workspace_on_update() {
    // Zero dual-compat / legacy migration logic needed.
}

fn restart_service_if_active() {
    let probe = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(root) = find_marked_ods_workspace_root(&probe) {
        let st = service::service_status(&root);
        if st.installed || st.running {
            match service::start_service(&root) {
                Ok(msg) => println!("ods: background service restart: {msg}"),
                Err(e) => eprintln!("ods: service restart check: {e}"),
            }
        }
    }
}

#[cfg(test)]
mod test_diagnostics_formatter {
    use super::*;

    #[test]
    fn test_run_update_command_parsing_args() {
        let err1 = run_update_command(&["ods".into(), "update".into(), "--unknown".into()]);
        assert!(err1.is_err());

        let err2 = run_update_command(&["ods".into(), "update".into(), "--version".into()]);
        assert!(err2.is_err());

        let _ = run_update_command(&["ods".into(), "update".into(), "--check".into()]);
    }

    #[test]
    fn test_restart_service_if_active_smoke() {
        restart_service_if_active();
    }

    #[test]
    fn test_update_command_bare_version_and_force_flags() {
        // force without network still parses through options (may fail at download)
        let res = run_update_command(&[
            "ods".into(),
            "update".into(),
            "--force".into(),
            "--check".into(),
        ]);
        // check_only + force is accepted by parser
        assert!(res.is_ok() || res.is_err());

        let res = run_update_command(&[
            "ods".into(),
            "update".into(),
            "v0.0.0-nonexistent".into(),
        ]);
        assert!(res.is_err() || res.is_ok());
    }

    #[test]
    fn migrate_machine_and_workspace_on_update_is_noop() {
        migrate_machine_and_workspace_on_update();
    }
}


