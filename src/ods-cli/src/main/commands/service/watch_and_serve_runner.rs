/// Install a `SIGTERM`/`SIGINT`/Ctrl-C handler that flips a shared flag instead
/// of leaving the process to be hard-killed. Lets `ods serve`/`watch` exit via
/// a normal return from `main` (flushing coverage/profiling data, closing
/// files cleanly) instead of only ever dying to an external `SIGKILL`. Safe to
/// call once per process; a failed registration (e.g. handler already set) is
/// non-fatal — the process just falls back to old hard-kill-only behavior.
fn install_shutdown_flag() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&shutdown);
    let _ = ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    });
    shutdown
}

/// Sleep up to `total`, waking early (without sleeping the full remainder) once
/// `shutdown` flips, by polling in short increments.
fn sleep_checking_shutdown(total: Duration, shutdown: &AtomicBool) {
    const STEP: Duration = Duration::from_millis(200);
    let mut waited = Duration::ZERO;
    while waited < total {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }
        let this_step = STEP.min(total - waited);
        std::thread::sleep(this_step);
        waited += this_step;
    }
}

/// Threshold: if more paths change than this, fall back to a full parallel reload.
const WATCH_FULL_RELOAD_DIRTY: usize = 500;

fn watch_workspace(
    root: &Path,
    level: LintLevel,
    format: OutputFormat,
    headless: bool,
) -> Result<(), CliError> {
    use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
    use std::cell::RefCell;
    use std::rc::Rc;

    let shutdown = install_shutdown_flag();

    // Long-lived workspace: parallel graph load once at start.
    let workspace = Rc::new(RefCell::new(
        load_workspace_with_options(root, load_options_graph())
            .map_err(|err| fail_io("watch/serve", err))?,
    ));

    let tree = {
        let ws = workspace.borrow();
        Rc::new(RefCell::new(WatchTree::from_scan(
            scan_markdown_tree_with_code_paths(root, &ws.ignore, &ws.code_paths)
                .map_err(|err| fail_io("watch/serve", err))?,
        )))
    };

    run_watch_tick(root, &tree, &workspace, level, format, headless, true)?;

    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: DebounceEventResult| {
            let _ = tx.send(res);
        },
    )
    .map_err(|err| fail_msg(ods_core::io_failed("watch init", err)))?;

    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .map_err(|err| fail_msg(ods_core::io_failed("watch", err)))?;

    if !headless {
        eprintln!(
            "watching {} — renames map automatically (Ctrl+C to stop)",
            root.display()
        );
    } else {
        eprintln!("ods serve: watching {}", root.display());
    }
    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(Ok(_events)) => {
                if let Err(err) =
                    run_watch_tick(root, &tree, &workspace, level, format, headless, false)
                {
                    eprintln!("{}", err.message());
                }
            }
            Ok(Err(err)) => eprintln!("watch error: {err:?}"),
            Err(RecvTimeoutError::Timeout) => {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    eprintln!("ods serve: shutting down {}", root.display());
    Ok(())
}

fn serve_workspace(options: ServeOptions) -> Result<(), CliError> {
    match resolved_serve_mode(options.mode) {
        ServeMode::Watch => {
            if options.memory_report {
                // One graph load for the report only (serve path itself stays FM-only).
                let (docs, budget) = load_workspace_with_options(
                    &options.root,
                    load_options_graph(),
                )
                .map(|ws| (ws.documents.len(), ws.config.service.max_rss_mb))
                .unwrap_or((0, 10));
                print_memory_report("watch", docs, 0, budget);
            }
            watch_workspace(&options.root, LintLevel::Full, OutputFormat::Text, true)
        }
        ServeMode::Poll => poll_workspace(options),
        ServeMode::Auto => unreachable!("auto mode is resolved before serve"),
    }
}

fn poll_workspace(options: ServeOptions) -> Result<(), CliError> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let shutdown = install_shutdown_flag();
    let workspace = Rc::new(RefCell::new(
        load_workspace_with_options(&options.root, load_options_graph())
            .map_err(|err| fail_io("watch/serve", err))?,
    ));
    let tree = {
        let ws = workspace.borrow();
        Rc::new(RefCell::new(WatchTree::from_scan(
            scan_markdown_tree_with_code_paths(&options.root, &ws.ignore, &ws.code_paths)
                .map_err(|err| fail_io("watch/serve", err))?,
        )))
    };
    eprintln!("ods serve: polling {}", options.root.display());
    while !shutdown.load(Ordering::SeqCst) {
        run_watch_tick(
            &options.root,
            &tree,
            &workspace,
            LintLevel::Full,
            OutputFormat::Text,
            true,
            false,
        )?;
        if options.memory_report {
            let retained = tree.borrow().snapshot.files.len();
            let (docs, budget_mb) = {
                let ws = workspace.borrow();
                (ws.documents.len(), ws.config.service.max_rss_mb)
            };
            print_memory_report("poll", docs, retained, budget_mb);
        }
        sleep_checking_shutdown(Duration::from_secs(options.poll_secs), &shutdown);
    }
    eprintln!("ods serve: shutting down {}", options.root.display());
    Ok(())
}

/// Incremental watch tick: scan → renames → dirty parse/apply → lint (no double full load).
fn run_watch_tick(
    root: &Path,
    tree: &std::rc::Rc<std::cell::RefCell<WatchTree>>,
    workspace: &std::rc::Rc<std::cell::RefCell<Workspace>>,
    level: LintLevel,
    format: OutputFormat,
    headless: bool,
    force_full: bool,
) -> Result<(), CliError> {
    let (ignore, code_paths) = {
        let ws = workspace.borrow();
        (ws.ignore.clone(), ws.code_paths.clone())
    };

    let current = scan_markdown_tree_with_code_paths(root, &ignore, &code_paths)
        .map_err(|err| fail_io("watch/serve", err))?;
    let changes = {
        let watch = tree.borrow();
        observe_renames(&watch.effective_previous(), &current)
    };

    if !changes.is_empty() {
        let report = apply_path_changes(root, &changes).map_err(|err| fail_io("watch/serve", err))?;
        if matches!(format, OutputFormat::Text) && !headless {
            eprintln!("path map: {}", report.summary());
            for (from, to) in &report.moves {
                let from = from.strip_prefix(root).unwrap_or(from);
                let to = to.strip_prefix(root).unwrap_or(to);
                eprintln!("  move {} → {}", from.display(), to.display());
            }
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
        }
    } else if !force_full {
        let heal = heal_orphan_path_ids(root).map_err(|err| fail_io("watch/serve", err))?;
        if !heal.rewritten_files.is_empty() && matches!(format, OutputFormat::Text) && !headless {
            eprintln!("path id heal: {}", heal.summary());
        }
    }

    let prev_files = tree.borrow().snapshot.files.clone();
    let mut dirty: Vec<PathBuf> = current
        .files
        .iter()
        .filter(|(path, hash)| prev_files.get(*path) != Some(hash))
        .map(|(path, _)| path.clone())
        .collect();
    for change in &changes {
        let to = match change {
            ods_core::PathChange::FileMoved { to, .. }
            | ods_core::PathChange::DirMoved { to, .. } => to,
        };
        if !dirty.iter().any(|p| p == to) {
            dirty.push(to.clone());
        }
    }
    dirty.sort();
    dirty.dedup();

    let removed: Vec<PathBuf> = prev_files
        .keys()
        .filter(|p| !current.files.contains_key(*p))
        .cloned()
        .collect();

    let total = workspace.borrow().documents.len().max(1);
    let need_full =
        force_full || dirty.len() > WATCH_FULL_RELOAD_DIRTY || dirty.len() * 10 > total;

    if need_full {
        let fresh = load_workspace_with_options(root, load_options_graph())
            .map_err(|err| fail_io("watch/serve", err))?;
        *workspace.borrow_mut() = fresh;
    } else {
        if !removed.is_empty() {
            let refs: Vec<&Path> = removed.iter().map(PathBuf::as_path).collect();
            apply_document_removes(&mut workspace.borrow_mut(), &refs);
        }
        let existing: Vec<PathBuf> = dirty.into_iter().filter(|p| p.is_file()).collect();
        if !existing.is_empty() {
            // Frontmatter-only parse — never retain bodies on the serve path.
            let docs = parse_paths_parallel(root, &existing, false)
                .map_err(|err| fail_io("watch/serve", err))?;
            apply_document_upserts(&mut workspace.borrow_mut(), docs);
        }
    }

    // Soft RSS budget from ods.toml [service] max_rss_mb (default 10).
    // Enforcement is best-effort: strip in-memory bodies and warn; does not exit the process.
    {
        let budget_mb = {
            let ws = workspace.borrow();
            ws.config.service.max_rss_mb.max(1)
        };
        if ods_core::rss_over_budget(budget_mb) {
            ods_core::strip_workspace_bodies(&mut workspace.borrow_mut());
            if let Some(rss_kb) = ods_core::current_rss_kb() {
                let limit_kb = budget_mb.saturating_mul(1024);
                eprintln!(
                    "ods serve: warning rss_kb={rss_kb} exceeds service.max_rss_mb={budget_mb} (limit {limit_kb} KB); stripped document bodies"
                );
            }
        }
        let ws = workspace.borrow();
        let _store = ods_core::WorkspaceStore::from_workspace(&ws);
        let _ = _store.within_rss_budget(budget_mb);
    }

    {
        let ws = workspace.borrow();
        let diagnostics = lint_workspace_with_level(&ws, level);
        if !headless {
            print_diagnostics(&diagnostics, format);
            write_or_clear_ods_error_report(root, &diagnostics, format)?;
            if diagnostics.is_empty() && matches!(format, OutputFormat::Text) {
                println!(
                    "Everything is fine — graph and links are consistent. No update required."
                );
            }
        } else {
            write_or_clear_ods_error_report(root, &diagnostics, OutputFormat::Text)?;
            if !diagnostics.is_empty() {
                eprintln!(
                    "ods serve: {} diagnostic(s) in {}",
                    diagnostics.len(),
                    root.display()
                );
            }
        }
    }

    let (ignore, code_paths) = {
        let ws = workspace.borrow();
        (ws.ignore.clone(), ws.code_paths.clone())
    };
    let after = scan_markdown_tree_with_code_paths(root, &ignore, &code_paths)
        .map_err(|err| fail_io("watch/serve", err))?;
    let paired = paired_from_paths(&changes);
    tree.borrow_mut().commit_scan(after, &paired);
    Ok(())
}

fn print_memory_report(
    mode: &str,
    documents: usize,
    retained_snapshot_files: usize,
    max_rss_mb: u64,
) {
    let rss = ods_core::current_rss_kb()
        .map(|kb| kb.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    eprintln!(
        "ods serve: mode={mode} documents={documents} retained_snapshot_files={retained_snapshot_files} max_rss_mb={max_rss_mb} rss_kb={rss}"
    );
}
