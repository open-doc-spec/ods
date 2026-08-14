fn print_help() {
    println!(
        "ods — Open Document Spec CLI

  ods lint / init / doctor / status …   Manage ODS document graph and profiles
  ods init                              Initialize ODS workspace (writes ods.toml)
  ods init --okf                        OKF bundle (okf_version: \"0.2\")

Platform & Service:
  update                   Self-update binary from GitHub Releases
  setup [path]             Machine service + health check
  workspaces …             Global workspace registry
  skill install            Install skill into an AI agent
  version / help

Root markers: ods.toml (ODS) · okf_version: (OKF)

Commands:
  init [path]              Make folder/repo ODS-compliant (writes root ods.toml)
  disable [path]           Opt-out dry-run: strip ODS metadata (alias: revert)
  disable --write [path]   Apply disable / revert to plain Markdown
  lint [path]              Validate workspace (green message when clean)
  index --okf [path]       OKF only: generate OKF navigation indexes
  overview [path]          Compact workspace snapshot (AI cold-start; alias: summary)
  ls / tree [path]         Progressive discovery (no nested index files)
  profiles [path]          List loaded profiles
  profile init <name>      Scaffold custom profile (registers under custom_profiles in ods.toml)
  profile show <name>      Show profile layer, sections, expected keys
  aliases [path]           List workspace section-heading aliases
  alias add <Can> <Syn>    Add a section alias (ods.toml [aliases]; legacy root index ok)
  tags [path]              List root-level project tags (observed) with use counts
  tags --all [path]        Include unused default ODS tags
  tag list [path]          List observed workspace tags with document counts
  tag show [path] <tag>    Show documents matching a tag
  tag rename <old> <new>   Rewrite a root-level tag across frontmatter (dry-run; --write)
                           Nested tags under ods: are invalid — run: ods fmt --migrate
  find [path] [--tag t] [--key k] [q]  Find docs by tag, schema/custom keys, and/or query
  schema [keys]            Inspect schema keys or generate JSON schema (--write)
  read <id>                Read document sections / summary with token budget
  setup [path]             Set up machine service for workspace + check updates and workspace health
  context <id>             Bounded reading list (depends + context.load; --explain / --include-related)
  undo [path]              Restore latest frontmatter snapshot (`ods undo --list` to inspect)
  graph [path]             Print depends/related edges
  export [path]            Write graph under .ods/graph.md (optional --out PATH, --include-private)
  share [path] --out DIR   Publish a share-filtered copy of a workspace/subtree
  new <path>               Scaffold new document with inferred profile and valid frontmatter
  rm <path-or-id>          Atomically delete document and scrub graph references workspace-wide
  status <path> <value>    Set lifecycle status (draft|stable|deprecated|archived)
  archive <path-or-id>     Alias for status … archived
  mv [path] <from> <to>    Move file/folder and rewrite document refs
  fmt [path]               Normalize frontmatter/body blank lines
  fmt --migrate            Canonical ods: nesting; hoist misplaced tags; preserve non-ODS keys
  fmt --refs md-paths      Also rewrite Document refs to .md paths
  doctor [path]            Report workspace health and version skew
  audit [path]             Inventory plain/invalid/partial Markdown
  audit --write-report     Write .ods/ods-errors.md (shared with lint diagnostics)
  coverage [path]          Documentation health % (--write-report → .ods/coverage.md)
  sync [path]              Reconcile git-tracked renames and rewrite refs
  logs [-f]                Show background service logs (~/.ods/logs/ods-serve.log); -f follows
  watch [path]             Foreground live rename map + re-lint
  serve --root <path>      Headless watch loop (used by OS service)
  serve --mode poll        Low-memory polling loop (auto|watch|poll)
  start [path]             Register+start user service (background watch)
  start --status [path]    Service install/running status
  stop [path]              Stop user service
  stop --unregister [path] Stop and remove service registration
  adopt [path]             Report adoption status (dry-run)
  adopt --write [path]     Draft minimal frontmatter for plain Markdown
  bench stats [path]       Display token & cost efficiency ROI report

Extra specs (ODS is the default — there is no `--ods` flag):
  --okf                    Enable Google OKF v0.2 engine for this command
  --skills                 Enable Agent Skills package engine for this command

  Native in binary ≠ always on: OKF/Skills activate with flags or ods.toml [specs.*]
  --okf supported: init lint doctor audit adopt index context export fmt watch serve
  ODS-only (no --okf graph rewrite): mv tags status archive pack share graph new rm
  ods lint --okf           Pure OKF or hybrid ODS+OKF lint
  ods init --okf           Scaffold OKF v0.2 bundle
  ods lint --skills        Lint Agent Skills packages (parse/lint/init surface)

Also: `ods lsp` — JSON-RPC Language Server for editors (stdio; not the same as `ods serve`).

Flags:
  --version, -V            Print version and exit
  --format text|json       Output format for supported commands (default text)
  --okf                    Extra-spec: OKF v0.2
  --skills                 Extra-spec: Agent Skills
  --write                  With adopt / tag rename / disable: apply changes
  --adopt                  With init: also draft frontmatter on plain files
  --keep-frontmatter       With disable: only drop ods: / root policy keys
  --remove-indexes         With disable: delete leftover non-root index.ods.md files
  --all                    With tags: include unused default ODS tags
  --tag <name>             With find/context: filter by tag (repeatable)
  --tag-match any|all      With find: tag intersection mode (default: any)
  --key <expr>             With find/context: filter by key expression (comma values, AND/OR logic)
  --key-match and|or       With find: key matching mode across flags (default: and)
  --status <status>        Shortcut for --key status=<status>
  --profile <profile>      Shortcut for --key profile=<profile>
  --owner <owner>          Shortcut for --key owner=<owner>
  --check                  With OKF index / update: check only
  --canonical-refs         With lint: warn on extensionless Document refs
  --refs md-paths          With fmt: rewrite Document refs to .md paths
  --migrate                With fmt: nested ods: + hoist tags (preserve non-ODS keys)
  --write-report           With audit/coverage: write report file
  --fail-on plain|invalid|any  With audit: CI gate
  --force                  With update: reinstall even if current
  --version <tag>          With update: install exact release tag (e.g. v0.0.13)
  --mode auto|watch|poll   With serve: choose watcher strategy
  --max-tokens N           With context/read: cap estimated tokens
  --print                  With context: emit budgeted file contents
  --include-code           With context: expand code: edges
  --help / -h              Command usage (most subcommands)

Environment:
  ODS_AUTO_UPDATE=0        Disable auto-update (default: on)
  ODS_LOW_MEMORY=1         serve --mode auto → poll
  ODS_SERVE_MODE           Default serve mode
  ODS_POLL_SECS             Default poll interval
  GH_TOKEN / GITHUB_TOKEN  Optional token for rate limits
"
    );
}

fn print_ods_help() {
    print_help();
}
fn run_setup_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut path = None;
    let mut install_git_hooks = false;
    let mut editor: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "ods setup [path] [--git-hooks] [--editor zed|vscode|nvim|cursor]\n\n\
                     Checks release freshness, detects an ODS workspace, starts the user service when possible, and runs doctor.\n\
                     --git-hooks   Install .git/hooks/pre-commit lint runner\n\
                     --editor X    Write Language Server config for `ods lsp` (zed|vscode|nvim|cursor)"
                );
                return Ok(ExitCode::from(0));
            }
            "--git-hooks" => {
                install_git_hooks = true;
            }
            "--editor" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--editor", "`ods setup --editor zed|vscode|nvim|cursor`")))?;
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
        let probe = path.clone().unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
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
            println!("No .git/hooks directory found at {}; skipped git hook setup.", root.display());
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
            println!("setup: no ODS workspace found at or above {}", probe.display());
            println!("setup: run 'ods init {}' to make this folder ODS-compliant", target.display());
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
        if ods_core::ods_toml_enabled(&current) || current.join("index.ods.md").is_file() || current.join("index.md").is_file() {
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
            println!("setup: wrote {} (configure your LSP client to use path/args)", path.display());
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
