// Global machine configuration (TOML) and `ods workspaces` command.
//
// Registry: ~/.ods/odsconfig.toml

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackEntry {
    pub workspace: String,
    pub name: String,
    pub path: String,
    pub source: String,
    pub auto_update: String, // "hourly", "daily", "weekly", "never"
    pub last_updated: String,
}

/// Path to the global machine configuration file: `~/.ods/odsconfig.toml`.
pub fn registry_path() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(&home).join(".ods/odsconfig.toml")
}

/// Load registered workspace paths from machine config.
pub fn load_registry_paths() -> Vec<String> {
    let path = registry_path();
    if let Ok(content) = fs::read_to_string(&path) {
        return parse_workspace_paths(&content);
    }
    Vec::new()
}





pub(crate) fn save_pack_entry(entry: PackEntry) -> Result<(), CliError> {
    let mut packs = load_registered_packs();
    packs.retain(|p| !(p.workspace == entry.workspace && p.name == entry.name));
    packs.push(entry);
    save_config_with_packs(&load_registry_paths(), &packs)
}



include!("workspaces_config.rs");

/// Check if a path is inside a registered workspace from the global registry.
fn is_registered_workspace(root: &Path) -> bool {
    let paths = load_registry_paths();
    let abs_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    for ws in &paths {
        let ws_path = PathBuf::from(ws);
        let abs_ws = fs::canonicalize(&ws_path).unwrap_or(ws_path);
        if abs_root.starts_with(&abs_ws) {
            return true;
        }
    }
    false
}

/// Guard: ensures the given root is a valid ODS workspace (has marker or is registered).
fn require_ods_workspace(root: &Path) -> Result<(), CliError> {
    if ods_core::ods_enabled(root) {
        return Ok(());
    }

    if is_registered_workspace(root) {
        return Ok(());
    }

    Err(fail_msg(
        ods_core::not_ods_workspace(false, false)
            .hint("or run `ods workspaces add` to track it in ~/.ods/odsconfig.toml without init"),
    ))
}

fn run_workspaces_command(args: &[String]) -> Result<ExitCode, CliError> {
    let subcommand = args.get(2).map(String::as_str).unwrap_or("list");

    match subcommand {
        "--help" | "-h" | "help" => {
            print_command_help("workspaces");
            Ok(ExitCode::from(0))
        }
        "add" => {
            let raw_path = args
                .get(3)
                .map(PathBuf::from)
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let abs_path = fs::canonicalize(&raw_path)
                .map_err(|e| fail_msg(ods_core::io_failed("resolve path", e)))?;
            let path_str = abs_path.to_string_lossy().into_owned();

            let mut paths = load_registry_paths();
            if paths.contains(&path_str) {
                println!("{} is already tracked", abs_path.display());
            } else {
                paths.push(path_str);
                save_registry_paths(&paths)?;
                println!("added {} to tracked ODS workspaces", abs_path.display());
                println!(
                    "config: {}",
                    registry_path().display()
                );
            }
            Ok(ExitCode::from(0))
        }
        "remove" => {
            let raw_path = args
                .get(3)
                .map(PathBuf::from)
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let abs_path = fs::canonicalize(&raw_path)
                .map_err(|e| fail_msg(ods_core::io_failed("resolve path", e)))?;
            let path_str = abs_path.to_string_lossy().into_owned();

            let mut paths = load_registry_paths();
            if let Some(pos) = paths.iter().position(|w| w == &path_str) {
                paths.remove(pos);
                save_registry_paths(&paths)?;
                println!("removed {} from tracked ODS workspaces", abs_path.display());
            } else {
                println!("{} is not currently tracked", abs_path.display());
            }
            Ok(ExitCode::from(0))
        }
        "list" => {
            let paths = load_registry_paths();
            let packs = load_registered_packs();
            if paths.is_empty() && packs.is_empty() {
                println!("no tracked ODS workspaces or packs");
                println!(
                    "run 'ods workspaces add [path]' to register a workspace"
                );
            } else {
                println!("tracked ODS workspaces ({}):", paths.len());
                for ws in &paths {
                    let marker = if ods_core::ods_enabled(Path::new(ws)) {
                        "✓"
                    } else {
                        "○"
                    };
                    println!("  {marker} {ws}");
                }
                if !packs.is_empty() {
                    println!("\ntracked ODS packs ({}):", packs.len());
                    for p in &packs {
                        println!("  • {} (source: {}, auto_update: {})", p.name, p.source, p.auto_update);
                    }
                }
                println!();
                println!("✓ = has root ods.toml with spec");
                println!("○ = registered but no local ods.toml marker");
            }
            Ok(ExitCode::from(0))
        }
        "path" => {
            println!("{}", registry_path().display());
            Ok(ExitCode::from(0))
        }
        other => Err(usage_msg(ods_core::unknown_subcommand(
            "workspaces",
            other,
            "ods workspaces add|remove|list|path",
        ))),
    }
}

include!("workspaces_tests.rs");

