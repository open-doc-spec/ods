pub(crate) fn run_pack_command(args: &[String]) -> Result<ExitCode, CliError> {
    let subcommand = args.get(2).map(String::as_str).unwrap_or("list");

    match subcommand {
        "help" | "--help" | "-h" => {
            print_command_help("pack");
            Ok(ExitCode::from(0))
        }
        "list" => run_pack_list(args),
        "add" => run_pack_add(args),
        "sync" => run_pack_sync(args),
        "remove" => run_pack_remove(args),
        "init" => run_pack_init(args),
        "preview" => run_pack_preview(args),
        other if other.starts_with('-') => run_pack_list(args),
        other => Err(usage_msg(ods_core::unknown_subcommand(
            "pack",
            other,
            "ods pack add|sync|list|preview|remove|init",
        ))),
    }
}

fn extract_pack_path(args: &[String], skip_idx: usize) -> PathBuf {
    args.iter()
        .enumerate()
        .skip(skip_idx)
        .find(|(_, a)| !a.starts_with('-'))
        .map(|(_, a)| PathBuf::from(a))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn run_pack_list(args: &[String]) -> Result<ExitCode, CliError> {
    let path = extract_pack_path(args, 3);
    let root = resolve_root_path(path);
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;

    // Prefer ods.toml packs; fall back to legacy root index frontmatter.
    let mut packs = workspace.config.packs.clone();
    if packs.is_empty()
        && let Some(doc) = workspace
            .documents
            .iter()
            .find(|d| d.path == root.join("index.ods.md"))
        && let FrontmatterState::Parsed(fm) = &doc.frontmatter
    {
        packs = fm.packs.clone();
    }

    println!("ODS Workspace Packs (root: {}):", root.display());
    if packs.is_empty() {
        println!("  (no external packs imported)");
    } else {
        for pack in packs {
            let pack_path = root.join(&pack);
            let status = if pack_path.exists() {
                "installed"
            } else {
                "missing"
            };
            println!("  • {} [{}] ({})", pack, status, pack_path.display());
        }
    }

    println!("\nLoaded Custom Profile Schemas:");
    for (name, def) in &workspace.profiles.definitions {
        println!("  • profile: {} ({})", name, def.source.display());
    }

    Ok(ExitCode::from(0))
}

fn insert_pack_into_ods_toml(text: &str, pack_entry: &str) -> String {
    if text.contains("packs =") {
        if let Some(idx) = text.find("packs =") {
            let after = &text[idx..];
            if let Some(bracket) = after.find('[') {
                let abs = idx + bracket;
                let rest = &text[abs + 1..];
                if let Some(end) = rest.find(']') {
                    let abs_end = abs + 1 + end;
                    let insert = if text[abs + 1..abs_end].trim().is_empty() {
                        format!("\n  \"{pack_entry}\",\n")
                    } else {
                        format!("\n  \"{pack_entry}\",")
                    };
                    return format!("{}{}{}", &text[..abs_end], insert, &text[abs_end..]);
                }
            }
        }
    }
    if let Some(first_section_idx) = text.find("\n[") {
        format!("{}\npacks = [\"{pack_entry}\"]{}", &text[..first_section_idx], &text[first_section_idx..])
    } else {
        format!("{}\npacks = [\"{pack_entry}\"]\n", text.trim_end())
    }
}

fn run_pack_add(args: &[String]) -> Result<ExitCode, CliError> {
    let positionals = positional_args(args, 3);
    let (root_path, source) = match positionals.as_slice() {
        [ws, src] => (PathBuf::from(ws), src.clone()),
        [src] => (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), src.clone()),
        _ => return Err(usage_msg(ods_core::missing_required_arg("source", "ods pack add [root] <source>"))),
    };

    let auto_update = args
        .windows(2)
        .find(|w| w[0] == "--auto-update")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| String::from("daily"));

    let root = resolve_root_path(root_path);
    let toml_path = root.join("ods.toml");
    let root_index_path = if root.join("index.ods.md").exists() {
        Some(root.join("index.ods.md"))
    } else if root.join("index.md").exists() {
        Some(root.join("index.md"))
    } else {
        None
    };

    if !toml_path.exists() && root_index_path.is_none() {
        return Err(fail_msg(ods_core::root_index_missing()));
    }

    let mut pack_entry = source.clone();
    let pack_name = source.split('/').next_back().unwrap_or("pack").trim_end_matches(".git").to_string();

    // Check if source is a local directory or relative path
    let local_path = Path::new(&source);
    if local_path.exists() {
        if let Ok(rel) = local_path.canonicalize() {
            if let Ok(workspace_rel) = rel.strip_prefix(&root) {
                pack_entry = workspace_rel.to_string_lossy().replace('\\', "/");
            } else {
                pack_entry = source.replace('\\', "/");
            }
        }
    } else if source.contains('/') && !source.contains(':') && !source.starts_with('.') {
        // GitHub shorthand: owner/repo -> vendor/repo
        let vendor_dir = root.join("vendor").join(&pack_name);
        println!("Cloning GitHub shorthand pack '{}' into {}...", source, vendor_dir.display());
        let git_url = format!("https://github.com/{source}.git");
        let status = Command::new("git")
            .args(["clone", &git_url, &vendor_dir.to_string_lossy()])
            .status();
        if let Ok(st) = status {
            if st.success() {
                pack_entry = format!("vendor/{pack_name}");
            } else {
                println!("Warning: git clone failed for {git_url}. Registering path reference.");
                pack_entry = format!("vendor/{pack_name}");
            }
        } else {
            pack_entry = format!("vendor/{pack_name}");
        }
    } else if source.starts_with("http://") || source.starts_with("https://") || source.starts_with("git@") {
        // Remote Git URL
        let vendor_dir = root.join("vendor").join(&pack_name);
        println!("Cloning Git URL pack into {}...", vendor_dir.display());
        let _status = Command::new("git")
            .args(["clone", &source, &vendor_dir.to_string_lossy()])
            .status();
        pack_entry = format!("vendor/{pack_name}");
    }

    // Record pack entry in global config (~/.ods/odsconfig.toml)
    let workspace_str = root.to_string_lossy().into_owned();
    let entry = PackEntry {
        workspace: workspace_str,
        name: pack_name,
        path: pack_entry.clone(),
        source: source.clone(),
        auto_update,
        last_updated: current_iso_timestamp(),
    };
    let _ = save_pack_entry(entry);

    // Append pack_entry to root index frontmatter if present, or to ods.toml
    let toml_path = root.join("ods.toml");
    if toml_path.is_file() {
        let text = fs::read_to_string(&toml_path).map_err(|e| fail_io("pack", e))?;
        if !text.contains(&format!("\"{pack_entry}\"")) {
            let updated = insert_pack_into_ods_toml(&text, &pack_entry);
            fs::write(&toml_path, updated).map_err(|e| fail_io("pack", e))?;
        }
        println!("Added ODS Pack '{}' to ods.toml.", pack_entry);
    } else if let Some(ref p) = root_index_path {
        let text = fs::read_to_string(p).map_err(|e| fail_io("pack", e))?;
        if text.contains(&format!("- {pack_entry}")) || text.contains(&format!("- \"{pack_entry}\"")) {
            println!("Pack '{}' is already registered in root index.md.", pack_entry);
            return Ok(ExitCode::from(0));
        }

        let updated_text = insert_pack_into_root_index(&text, &pack_entry);
        fs::write(p, updated_text).map_err(|e| fail_io("pack", e))?;

        println!("Added ODS Pack '{}' to root index.md frontmatter.", pack_entry);
    } else {
        println!("Added ODS Pack '{}' to workspace.", pack_entry);
    }
    Ok(ExitCode::from(0))
}

fn insert_pack_into_root_index(text: &str, pack_entry: &str) -> String {
    if text.contains("packs:") {
        text.replace("packs:", &format!("packs:\n  - {pack_entry}"))
    } else if text.starts_with("---\n") || text.starts_with("---\r\n") {
        text.replacen("---", &format!("---\npacks:\n  - {pack_entry}"), 1)
    } else {
        format!("---\npacks:\n  - {pack_entry}\n---\n\n{text}")
    }
}

fn run_pack_sync(args: &[String]) -> Result<ExitCode, CliError> {
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;

    let mut packs = workspace.config.packs.clone();
    if packs.is_empty()
        && let Some(doc) = workspace
            .documents
            .iter()
            .find(|d| d.path == root.join("index.ods.md"))
        && let FrontmatterState::Parsed(fm) = &doc.frontmatter
    {
        packs = fm.packs.clone();
    }

    let registered_packs = load_registered_packs();
    println!("Synchronizing {} installed ODS Packs...", packs.len());

    for pack in packs {
        let pack_dir = root.join(&pack);
        let reg_entry = registered_packs
            .iter()
            .find(|p| p.workspace == root.to_string_lossy() && p.path == pack);

        let due = force || reg_entry.is_none_or(|e| pack_update_due(&e.last_updated, &e.auto_update));

        if pack_dir.join(".git").exists() && due {
            println!("Pulling updates for {}...", pack);
            let status = Command::new("git")
                .current_dir(&pack_dir)
                .args(["pull", "--ff-only"])
                .status();
            if let Ok(st) = status
                && st.success()
            {
                let name = pack.split('/').next_back().unwrap_or(&pack).to_string();
                let source = reg_entry.map_or_else(|| pack.clone(), |e| e.source.clone());
                let auto_update = reg_entry.map_or_else(|| "daily".to_string(), |e| e.auto_update.clone());
                let _ = save_pack_entry(PackEntry {
                    workspace: root.to_string_lossy().into_owned(),
                    name,
                    path: pack.clone(),
                    source,
                    auto_update,
                    last_updated: current_iso_timestamp(),
                });
            }
        } else if pack_dir.exists() {
            println!("Verified local pack path {} (up to date).", pack);
        } else {
            println!("Warning: Pack path {} does not exist.", pack_dir.display());
        }
    }

    println!("ODS Pack synchronization complete.");
    Ok(ExitCode::from(0))
}

include!("pack_subcommands.rs");

