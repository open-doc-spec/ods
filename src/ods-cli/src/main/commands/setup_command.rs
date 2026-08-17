fn run_setup_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut path = None;
    let mut install_git_hooks = false;
    let mut editor: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_command_help("setup");
                return Ok(ExitCode::from(0));
            }
            "--git-hooks" => {
                install_git_hooks = true;
            }
            "--editor" => {
                let v = args.get(i + 1).ok_or_else(|| {
                    usage_msg(ods_core::missing_flag_value(
                        "--editor",
                        "`ods setup --editor zed|vscode|nvim|cursor`",
                    ))
                })?;
                editor = Some(v.to_lowercase());
                i += 1;
            }
            flag if flag.starts_with('-') => {
                return Err(usage_msg(ods_core::unknown_flag(flag, "ods setup --help")));
            }
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    if let Some(ed) = editor.as_deref() {
        let probe = path
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        write_editor_lsp_config(&probe, ed)?;
    }

    if install_git_hooks {
        let probe = path
            .clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let root = ods_core::find_workspace_root(&probe).unwrap_or(probe);
        let git_hooks_dir = root.join(".git").join("hooks");
        if git_hooks_dir.exists() {
            let hook_file = git_hooks_dir.join("pre-commit");
            let hook_script = "#!/bin/sh\n# ODS git pre-commit hook\nods lint . --format text\n";
            let _ = fs::write(&hook_file, hook_script);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&hook_file, fs::Permissions::from_mode(0o755));
            }
            println!("Installed ODS pre-commit hook to {}", hook_file.display());
        } else {
            println!(
                "No .git/hooks directory found at {}; skipped git hook setup.",
                root.display()
            );
        }
    }

    if setup_update_check_enabled() {
        println!("setup: checking for updates");
        match run_update(UpdateOptions {
            check_only: true,
            force: false,
            version: None,
        }) {
            Ok(UpdateOutcome::UpToDate { current, remote }) => {
                println!("setup: ods {current} is up to date (latest {remote})");
            }
            Ok(UpdateOutcome::Available { current, remote }) => {
                println!("setup: update available: {current} -> {remote}");
                println!("run: ods update");
                return Ok(ExitCode::from(1));
            }
            Ok(UpdateOutcome::Updated { .. }) => {}
            Err(err) => {
                println!("setup: update check skipped ({err})");
            }
        }
    } else {
        println!("setup: update check skipped (ODS_AUTO_UPDATE=0)");
    }

    let probe = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = match find_marked_ods_workspace_root(&probe) {
        Some(root) => root,
        None => {
            let target = if probe.is_dir() {
                probe.clone()
            } else {
                probe
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
            };
            println!(
                "setup: no ODS workspace found at or above {}",
                probe.display()
            );
            println!(
                "setup: run 'ods init {}' to make this folder ODS-compliant",
                target.display()
            );
            return Ok(ExitCode::from(0));
        }
    };

    let init = init_workspace(&root, ods_core::InitOptions { adopt: false })
        .map_err(|err| fail_io("setup", err))?;
    if init.initialized {
        println!(
            "setup: root index ensured with ods: {}",
            ods_core::current_ods_spec_version()
        );
    }

    println!("setup: workspace {}", root.display());
    let status = service::service_status(&root);
    println!(
        "setup: service installed={} running={} ({})",
        status.installed, status.running, status.detail
    );

    if !status.running {
        if env::var("ODS_SETUP_NO_START")
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "yes" || v == "on"
            })
            .unwrap_or(false)
        {
            println!("setup: service start skipped by ODS_SETUP_NO_START");
        } else {
            let msg = service::start_service(&root).map_err(|e| fail_io("setup", e))?;
            println!("setup: {msg}");
        }
    }

    println!("setup: doctor");
    let report = doctor_workspace(&root)?;
    println!("{}", report.text);
    Ok(ExitCode::from(if report.has_error { 1 } else { 0 }))
}

fn setup_update_check_enabled() -> bool {
    match env::var("ODS_AUTO_UPDATE") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "no" || v == "off")
        }
        Err(_) => true,
    }
}

fn find_marked_ods_workspace_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    loop {
        if ods_core::ods_toml_enabled(&current)
            || current.join("index.ods.md").is_file()
            || current.join("index.md").is_file()
        {
            return Some(current);
        }
        if current.join(".git").exists() {
            return None;
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Write editor config so the host launches `ods lsp` for Markdown.
fn write_editor_lsp_config(root: &Path, editor: &str) -> Result<(), CliError> {
    match editor {
        "zed" => {
            let dir = root.join(".zed");
            fs::create_dir_all(&dir).map_err(|e| fail_io("setup", e))?;
            let path = dir.join("settings.json");
            let body = r#"{
  "languages": {
    "Markdown": {
      "language_servers": ["ods-lsp"],
      "enable_language_server": true
    }
  },
  "lsp": {
    "ods-lsp": {
      "binary": {
        "path": "ods",
        "arguments": ["lsp"]
      }
    }
  }
}
"#;
            fs::write(&path, body).map_err(|e| fail_io("setup", e))?;
            println!("setup: wrote {}", path.display());
        }
        "vscode" | "cursor" => {
            let dir = root.join(".vscode");
            fs::create_dir_all(&dir).map_err(|e| fail_io("setup", e))?;
            let path = dir.join("settings.json");
            // Generic LSP client settings (extension may map these keys).
            let body = r#"{
  "ods.lsp.path": "ods",
  "ods.lsp.args": ["lsp"]
}
"#;
            fs::write(&path, body).map_err(|e| fail_io("setup", e))?;
            println!(
                "setup: wrote {} (configure your LSP client to use path/args)",
                path.display()
            );
        }
        "nvim" => {
            let dir = root.join(".nvim");
            fs::create_dir_all(&dir).map_err(|e| fail_io("setup", e))?;
            let path = dir.join("ods-lsp.lua");
            let body = r#"-- Open Document Spec LSP (ods lsp)
-- Add to your Neovim config, e.g. require from this file:
--   vim.lsp.start({ name = 'ods-lsp', cmd = { 'ods', 'lsp' }, root_dir = vim.fn.getcwd() })
vim.api.nvim_create_autocmd('FileType', {
  pattern = 'markdown',
  callback = function(args)
    vim.lsp.start({
      name = 'ods-lsp',
      cmd = { 'ods', 'lsp' },
      root_dir = vim.fs.root(args.buf, { 'index.md', '.git' }) or vim.fn.getcwd(),
    })
  end,
})
"#;
            fs::write(&path, body).map_err(|e| fail_io("setup", e))?;
            println!("setup: wrote {}", path.display());
        }
        other => {
            return Err(usage_msg(ods_core::invalid_choice(
                "--editor",
                other,
                "zed|vscode|nvim|cursor",
            )));
        }
    }
    Ok(())
}
