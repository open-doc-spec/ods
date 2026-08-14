use std::io as std_io;
use toml_edit::{Array as TomlArray, DocumentMut, Item as TomlItem, Value as TomlValue};

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
                println!("warning: missing ods.toml — profile not registered");
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
    let (updated, already_registered) = insert_custom_profile_into_ods_toml(&text, rel_entry)
        .map_err(|err| fail_io("profile registration", err))?;
    if already_registered {
        return Ok(RegisterResult::AlreadyRegistered(toml_path));
    }

    fs::write(&toml_path, updated).map_err(|e| fail_io("profile", e))?;
    Ok(RegisterResult::Registered(toml_path))
}

fn insert_custom_profile_into_ods_toml(
    text: &str,
    rel_entry: &str,
) -> std_io::Result<(String, bool)> {
    let mut document = text.parse::<DocumentMut>().map_err(|err| {
        std_io::Error::new(
            std_io::ErrorKind::InvalidData,
            format!("invalid ods.toml: {err}"),
        )
    })?;

    let item = document
        .entry("custom_profiles")
        .or_insert(TomlItem::Value(TomlValue::Array(TomlArray::new())));
    let array = item.as_array_mut().ok_or_else(|| {
        std_io::Error::new(
            std_io::ErrorKind::InvalidData,
            "custom_profiles in ods.toml must be an array",
        )
    })?;

    if array.iter().any(|value| value.as_str() == Some(rel_entry)) {
        return Ok((text.to_string(), true));
    }

    array.push(rel_entry);
    Ok((document.to_string(), false))
}

#[cfg(test)]
mod tests {
    use super::insert_custom_profile_into_ods_toml;

    #[test]
    fn registration_ignores_comments_and_updates_root_array() {
        let source =
            "# custom_profiles = [\"comment-only\"]\nspec = \"0.1\"\n[service]\nmode = \"poll\"\n";
        let (updated, already) =
            insert_custom_profile_into_ods_toml(source, ".ods/profiles/incident.md").unwrap();

        assert!(!already);
        assert!(updated.contains("custom_profiles = [\".ods/profiles/incident.md\"]"));
        assert!(updated.contains("# custom_profiles = [\"comment-only\"]"));
    }

    #[test]
    fn registration_reports_non_array_custom_profiles() {
        let error = insert_custom_profile_into_ods_toml(
            "spec = \"0.1\"\ncustom_profiles = \"not-an-array\"\n",
            ".ods/profiles/incident.md",
        )
        .expect_err("non-array custom_profiles must fail");

        assert!(error.to_string().contains("must be an array"));
    }
}
