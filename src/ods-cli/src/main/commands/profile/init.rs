fn run_profile_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    // argv: ods profile init <name>  → name at index 3
    let profile_name = args
        .get(3)
        .filter(|a| !a.starts_with('-'))
        .or_else(|| {
            args.get(2)
                .filter(|a| a.as_str() != "init" && !a.starts_with('-'))
        })
        .ok_or_else(|| {
            usage_msg(ods_core::missing_required_arg(
                "name",
                "ods profile init <name> [--no-register]",
            ))
        })?;

    let no_register = args.iter().any(|a| a == "--no-register");
    let register = !no_register;

    // Optional root path after name: ods profile init rfc /path
    // Prefer first non-flag positional after name that looks like a path (not a flag).
    let root = args
        .iter()
        .skip(4)
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(root);
    let profiles_dir = root.join(".ods").join("profiles");
    fs::create_dir_all(&profiles_dir).map_err(|err| fail_io("profile", err))?;

    let file_path = profiles_dir.join(format!("{profile_name}.md"));
    let rel_register = format!(".ods/profiles/{profile_name}.md");
    let created = if file_path.exists() {
        println!(
            "profile definition already exists at {}",
            file_path.display()
        );
        false
    } else {
        let template = format!(
            "---
ods:
  custom_profile:
    name: {profile_name}
    required_keys:
      - owner
    optional_keys: []
    forbidden_keys: []
---

# {profile_name} Profile

## Overview

## Specification

### Details

## Verification & Testing
"
        );
        fs::write(&file_path, template).map_err(|err| fail_io("profile", err))?;
        println!(
            "scaffolded custom profile definition at {}",
            file_path.display()
        );
        true
    };

    if register {
        match register_custom_profile_in_root(&root, &rel_register) {
            Ok(RegisterResult::Registered(path)) => {
                println!("registered in {} under custom_profiles:", path.display());
                println!("  - {rel_register}");
            }
            Ok(RegisterResult::AlreadyRegistered(path)) => {
                println!(
                    "already registered in {} under custom_profiles:",
                    path.display()
                );
                println!("  - {rel_register}");
            }
            Ok(RegisterResult::NoRootIndex) => {
                println!(
                    "warning: missing ods.toml — profile not registered"
                );
                println!("Next: ods init  then re-run: ods profile init {profile_name}");
            }
            Err(e) => return Err(e),
        }
    } else {
        println!(
            "skipped registration (--no-register). Add to root ods.toml:\n  custom_profiles = [\"{rel_register}\"]"
        );
    }

    if created || register {
        println!("use in a document:");
        println!("  ods:");
        println!("    profile: {profile_name}");
        println!("    status: draft");
        println!("Next: ods lint");
    }

    Ok(ExitCode::from(0))
}

enum RegisterResult {
    Registered(PathBuf),
    AlreadyRegistered(PathBuf),
    NoRootIndex,
}

fn register_custom_profile_in_root(
    root: &Path,
    rel_entry: &str,
) -> Result<RegisterResult, CliError> {
    let toml_path = root.join("ods.toml");
    if !toml_path.is_file() {
        return Ok(RegisterResult::NoRootIndex);
    }

    let text = fs::read_to_string(&toml_path).map_err(|e| fail_io("profile", e))?;
    if profile_entry_already_listed(&text, rel_entry) {
        return Ok(RegisterResult::AlreadyRegistered(toml_path));
    }

    let updated = insert_custom_profile_into_ods_toml(&text, rel_entry);
    fs::write(&toml_path, updated).map_err(|e| fail_io("profile", e))?;
    Ok(RegisterResult::Registered(toml_path))
}

fn profile_entry_already_listed(text: &str, rel_entry: &str) -> bool {
    text.contains(&format!("\"{rel_entry}\""))
        || text.contains(&format!("'{rel_entry}'"))
        || text.lines().any(|l| l.trim() == rel_entry || l.trim() == format!("- {rel_entry}"))
}

fn insert_custom_profile_into_ods_toml(text: &str, rel_entry: &str) -> String {
    if text.contains("custom_profiles") {
        // Append entry before closing bracket of array if present.
        if let Some(idx) = text.find("custom_profiles") {
            let after = &text[idx..];
            if let Some(bracket) = after.find('[') {
                let abs = idx + bracket;
                let rest = &text[abs + 1..];
                if let Some(end) = rest.find(']') {
                    let abs_end = abs + 1 + end;
                    let insert = if text[abs + 1..abs_end].trim().is_empty() {
                        format!("\n  \"{rel_entry}\",\n")
                    } else {
                        format!("\n  \"{rel_entry}\",")
                    };
                    return format!("{}{}{}", &text[..abs_end], insert, &text[abs_end..]);
                }
            }
        }
    }
    if let Some(first_section_idx) = text.find("\n[") {
        format!("{}\ncustom_profiles = [\"{rel_entry}\"]{}", &text[..first_section_idx], &text[first_section_idx..])
    } else {
        format!("{}\ncustom_profiles = [\"{rel_entry}\"]\n", text.trim_end())
    }
}
