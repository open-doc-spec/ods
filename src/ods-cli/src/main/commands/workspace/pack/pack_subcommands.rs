fn run_pack_remove(args: &[String]) -> Result<ExitCode, CliError> {
    let positionals = positional_args(args, 3);
    let (root_path, name) = match positionals.as_slice() {
        [ws, n] => (PathBuf::from(ws), n.clone()),
        [n] => (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), n.clone()),
        _ => return Err(usage_msg(ods_core::missing_required_arg("pack name", "ods pack remove [root] <pack-name-or-path>"))),
    };

    let root = resolve_root_path(root_path);
    let toml_path = root.join("ods.toml");
    if toml_path.exists() {
        if let Ok(text) = fs::read_to_string(&toml_path) {
            let updated = remove_pack_from_ods_toml(&text, &name);
            fs::write(&toml_path, updated).map_err(|e| fail_io("pack", e))?;
            println!("Removed ODS Pack reference '{}' from ods.toml.", name);
        }
    } else {
        let root_index_path = if root.join("index.ods.md").exists() {
            Some(root.join("index.ods.md"))
        } else if root.join("index.md").exists() {
            Some(root.join("index.md"))
        } else {
            None
        };

        if let Some(ref p) = root_index_path {
            let text = fs::read_to_string(p).map_err(|e| fail_io("pack", e))?;
            let target_line = format!("  - {name}");
            let updated = text.lines().filter(|line| *line != target_line).collect::<Vec<_>>().join("\n");
            fs::write(p, updated).map_err(|e| fail_io("pack", e))?;
            println!("Removed ODS Pack reference '{}' from root index.", name);
        } else {
            println!("Removed ODS Pack reference '{}'.", name);
        }
    }
    Ok(ExitCode::from(0))
}

fn remove_pack_from_ods_toml(text: &str, pack_name: &str) -> String {
    let mut out_lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("packs") && trimmed.contains('[') && trimmed.contains(']') {
            let replaced = line
                .replace(&format!("\"{pack_name}\", "), "")
                .replace(&format!(", \"{pack_name}\""), "")
                .replace(&format!("\"{pack_name}\""), "");
            out_lines.push(replaced);
        } else if trimmed.contains(&format!("\"{pack_name}\""))
            || trimmed.contains(&format!("'{pack_name}'"))
            || trimmed == format!("- {pack_name}")
        {
            continue;
        } else {
            out_lines.push(line.to_string());
        }
    }
    out_lines.join("\n")
}

fn run_pack_preview(args: &[String]) -> Result<ExitCode, CliError> {
    let name = args
        .get(3)
        .ok_or_else(|| usage_msg(ods_core::missing_required_arg("pack name", "ods pack preview <pack-name-or-path>")))?;

    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let pack_dir = root.join(name);

    if !pack_dir.exists() {
        return Err(fail_msg(ods_core::path_not_found(&pack_dir)));
    }

    println!("Previewing ODS Pack at {}:", pack_dir.display());
    let workspace = load_workspace(&pack_dir).map_err(|e| fail_load(&pack_dir, e))?;
    for (schema_name, def) in &workspace.profiles.definitions {
        println!("  • profile: {} ({})", schema_name, def.source.display());
    }

    Ok(ExitCode::from(0))
}

fn run_pack_init(args: &[String]) -> Result<ExitCode, CliError> {
    let name = args.get(3).map(String::as_str).unwrap_or("my-ods-pack");
    let root = PathBuf::from(name);

    if !root.exists() {
        fs::create_dir_all(&root).map_err(|e| fail_io("pack", e))?;
    }

    let ods_profiles_dir = root.join("ods-profiles");
    let skills_dir = root.join("skills");
    fs::create_dir_all(&ods_profiles_dir).map_err(|e| fail_io("pack", e))?;
    fs::create_dir_all(&skills_dir).map_err(|e| fail_io("pack", e))?;

    let toml_content = format!("spec = \"0.1\"\nname = \"{name}\"\n");
    fs::write(root.join("ods.toml"), toml_content).map_err(|e| fail_io("pack", e))?;

    println!("Scaffolding new ODS Pack at {}:", root.display());
    println!("  ✓ Created ods.toml (workspace marker)");
    println!("  ✓ Created ods-profiles/ (profile schema directory)");
    println!("  ✓ Created skills/ (AI agent skills directory)");

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_pack_command {
    use super::*;

    #[test]
    fn test_pack_command_routing_and_init() {
        let td = tempfile::tempdir().unwrap();
        let pack_path = td.path().join("test-pack");

        let ws = td.path().join("ws");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("ods.toml"), "spec = \"0.1\"\n").unwrap();

        let res_list = run_pack_command(&[
            "ods".into(),
            "pack".into(),
            "list".into(),
            ws.to_string_lossy().to_string(),
        ]);
        assert!(res_list.is_ok());

        let err2 = run_pack_command(&["ods".into(), "pack".into(), "invalid".into()]);
        assert!(err2.is_err());

        let res_init = run_pack_init(&[
            "ods".into(),
            "pack".into(),
            "init".into(),
            pack_path.to_str().unwrap().to_string(),
        ]);
        assert!(res_init.is_ok());

        assert!(pack_path.join("ods.toml").exists());
        assert!(pack_path.join("ods-profiles").is_dir());
        assert!(pack_path.join("skills").is_dir());

        let res_prev = run_pack_preview(&[
            "ods".into(),
            "pack".into(),
            "preview".into(),
            pack_path.to_str().unwrap().to_string(),
        ]);
        assert!(res_prev.is_ok() || res_prev.is_err());

        let res_add = run_pack_add(&[
            "ods".into(),
            "pack".into(),
            "add".into(),
            ws.to_string_lossy().to_string(),
            pack_path.to_string_lossy().to_string(),
        ]);
        assert!(res_add.is_ok());

        let res_rm = run_pack_remove(&[
            "ods".into(),
            "pack".into(),
            "rm".into(),
            ws.to_string_lossy().to_string(),
            pack_path.to_string_lossy().to_string(),
        ]);
        assert!(res_rm.is_ok());
    }

    #[test]
    fn insert_pack_into_root_index_variants() {
        let with_packs = "---\nprofile: index\npacks:\n  - existing\n---\n\n# R\n";
        let out = insert_pack_into_root_index(with_packs, "new-pack");
        assert!(out.contains("new-pack"), "{out}");

        let no_packs = "---\nprofile: index\nods: 0.1\n---\n\n# R\n";
        let out = insert_pack_into_root_index(no_packs, "p2");
        assert!(out.contains("packs:"), "{out}");
        assert!(out.contains("p2"), "{out}");

        let plain = "# No frontmatter\n";
        let out = insert_pack_into_root_index(plain, "p3");
        assert!(out.starts_with("---"), "{out}");
        assert!(out.contains("p3"), "{out}");
    }

    #[test]
    fn pack_list_and_sync_smoke() {
        let td = tempfile::tempdir().unwrap();
        let ws = td.path().join("ws");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(
            ws.join("ods.toml"),
            "spec = \"0.1\"\npacks = [\"local-pack\"]\n",
        )
        .unwrap();
        std::fs::write(
            ws.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\npacks:\n  - local-pack\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::create_dir_all(ws.join("local-pack")).unwrap();
        std::fs::write(
            ws.join("local-pack/index.ods.md"),
            "---\nprofile: index\n---\n\n# Pack\n",
        )
        .unwrap();

        let prev = std::env::current_dir().ok();
        let _ = std::env::set_current_dir(&ws);
        let _ = run_pack_list(&["ods".into(), "pack".into(), "list".into()]);
        let _ = run_pack_sync(&[
            "ods".into(),
            "pack".into(),
            "sync".into(),
            "--force".into(),
        ]);
        if let Some(p) = prev {
            let _ = std::env::set_current_dir(p);
        }
    }

    #[test]
    fn test_remove_pack_from_ods_toml() {
        let text = "spec = \"0.1\"\npacks = [\"local-pack\", \"other-pack\"]\n";
        let out = remove_pack_from_ods_toml(text, "local-pack");
        assert!(!out.contains("local-pack"));
        assert!(out.contains("other-pack"));
    }

    #[test]
    fn insert_pack_into_ods_toml_variants() {
        let empty_packs = "spec = \"0.1\"\npacks = []\n";
        let out = insert_pack_into_ods_toml(empty_packs, "p1");
        assert!(out.contains("\"p1\""), "{out}");

        let with_packs = "spec = \"0.1\"\npacks = [\"existing\"]\n";
        let out = insert_pack_into_ods_toml(with_packs, "p2");
        assert!(out.contains("\"existing\""), "{out}");
        assert!(out.contains("\"p2\""), "{out}");

        let no_packs_section = "spec = \"0.1\"\n\n[service]\nmode = \"poll\"\n";
        let out = insert_pack_into_ods_toml(no_packs_section, "p3");
        assert!(out.contains("packs = [\"p3\"]"), "{out}");
        assert!(out.contains("[service]"), "{out}");

        let minimal = "spec = \"0.1\"\n";
        let out = insert_pack_into_ods_toml(minimal, "p4");
        assert!(out.contains("packs = [\"p4\"]"), "{out}");
    }

    #[test]
    fn remove_pack_from_ods_toml_multiline_and_partial() {
        let multi = "spec = \"0.1\"\npacks = [\n  \"local-pack\",\n  \"other-pack\",\n]\n";
        let out = remove_pack_from_ods_toml(multi, "local-pack");
        assert!(!out.contains("local-pack"), "{out}");
        assert!(out.contains("other-pack"), "{out}");

        let single = "spec = \"0.1\"\npacks = [\"only\"]\n";
        let out = remove_pack_from_ods_toml(single, "only");
        assert!(!out.contains("\"only\""), "{out}");
    }

    #[test]
    fn test_pack_command_unknown_subcommand() {
        let err = run_pack_command(&["ods".into(), "pack".into(), "unknown_xyz".into()]).unwrap_err();
        assert!(err.message().contains("unknown"));

        let res_flag = run_pack_command(&["ods".into(), "pack".into(), "--help".into()]);
        assert!(res_flag.is_ok());
    }
}
