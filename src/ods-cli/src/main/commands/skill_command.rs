fn run_skill_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("help");
    if sub == "help" || sub == "-h" || sub == "--help" {
        print_skill_help();
        return Ok(ExitCode::from(0));
    }
    if sub != "install" {
        return Err(usage_msg(ods_core::unknown_subcommand(
            "skill",
            sub,
            "ods skill install --agent <name> | ods skill help",
        )));
    }

    let agent = parse_flag_val(args, "--agent").ok_or_else(|| {
        usage_msg(
            ods_core::missing_flag_value("--agent", "`ods skill install --agent claude-code`"),
        )
    })?;

    let scope_val = parse_flag_val(args, "--scope");
    let scope = match scope_val.as_deref() {
        Some("project") => "project",
        Some("user") => "user",
        Some(other) => {
            return Err(usage_msg(
                ods_core::UserMsg::new(
                    "invalid_skill_scope",
                    ods_core::ErrorStage::Argv,
                    format!("invalid scope: {other}"),
                )
                .next("use --scope project or --scope user"),
            ));
        }
        None => {
            // Default scopes per agent
            match agent.as_str() {
                "claude-code" | "antigravity" | "codex" | "gemini-cli" => "user",
                "cursor" | "copilot" | "windsurf" => "project",
                _ => "project",
            }
        }
    };

    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| fail_msg(ods_core::home_dir_unresolved()))?;

    let target = match agent.as_str() {
        "claude-code" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".claude/skills/ods")
            } else {
                PathBuf::from(".claude/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "cursor" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".cursor/rules/ods.mdc")
            } else {
                PathBuf::from(".cursor/rules/ods.mdc")
            };
            SkillInstallTarget::File {
                path,
                content: CURSOR_MDC_TEMPLATE.as_bytes(),
            }
        }
        "antigravity" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".gemini/config/skills/ods")
            } else {
                PathBuf::from(".gemini/config/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "codex" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".codex/skills/ods")
            } else {
                PathBuf::from(".codex/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "gemini-cli" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".gemini/skills/ods")
            } else {
                PathBuf::from(".gemini/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "windsurf" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".codeium/windsurf/memories/global_rules.md")
            } else {
                PathBuf::from(".windsurf/rules/ods.md")
            };
            SkillInstallTarget::File {
                path,
                content: WINDSURF_RULE_TEMPLATE.as_bytes(),
            }
        }
        "copilot" => {
            if scope == "user" {
                eprintln!("warning: GitHub Copilot only reads workspace-level instructions. Writing to project scope instead.");
            }
            SkillInstallTarget::File {
                path: PathBuf::from(".github/copilot-instructions.md"),
                content: SKILL_BUNDLE[0].1,
            }
        }
        other => {
            return Err(usage_msg(
                ods_core::UserMsg::new(
                    "unknown_skill_agent",
                    ods_core::ErrorStage::Argv,
                    format!("unknown agent: {other}"),
                )
                .next(
                    "use --agent claude-code|cursor|antigravity|codex|gemini-cli|windsurf|copilot",
                ),
            ));
        }
    };

    let dest_path = match target {
        SkillInstallTarget::Bundle(path) => {
            install_skill_bundle(&path)?;
            path
        }
        SkillInstallTarget::File { path, content } => {
            write_install_file(&path, content)?;
            path
        }
    };

    println!(
        "✓ ODS skill successfully installed for agent '{}' under '{}' scope (path: {})",
        agent,
        scope,
        dest_path.display()
    );

    Ok(ExitCode::from(0))
}

enum SkillInstallTarget {
    Bundle(PathBuf),
    File {
        path: PathBuf,
        content: &'static [u8],
    },
}

fn install_skill_bundle(destination: &Path) -> Result<(), CliError> {
    for (relative_path, contents) in SKILL_BUNDLE {
        write_install_file(&destination.join(relative_path), contents)?;
    }
    Ok(())
}

fn write_install_file(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            fail_msg(ods_core::io_failed("create directory", e))
        })?;
    }

    fs::write(path, contents)
        .map_err(|e| fail_msg(ods_core::io_failed("write skill file", e)))
}

fn print_skill_help() {
    println!(
        "ods skill <command> [flags]

Commands:
  install                  Install ODS skill or rules configuration into an AI agent's directory.
  help                     Print this help message.

Flags:
  --agent <name>           The AI agent target (claude-code, cursor, antigravity, codex, gemini-cli, windsurf, copilot)
  --scope <project|user>   Install to project workspace or global home directory (optional)"
    );
}

const CURSOR_MDC_TEMPLATE: &str = r#"---
description: ODS markdown authoring — bounded context, not full-repo dumps.
globs: ["**/*.md"]
alwaysApply: false
---
# Open Document Specs (ODS) Rules

When editing Markdown in an ODS workspace:
1. Keep engine keys under `ods:` (`profile`, `status`, `depends`, `related`, `context`).
2. **Token discipline**: run `ods context <id>` and read **only** the returned paths. Never load the whole tree or `ods export graph` for routine answers.
3. Use `ods mv <src> <dst>` when renaming docs; run `ods lint` after structural edits.
"#;

const WINDSURF_RULE_TEMPLATE: &str = r#"---
trigger: model_decision
description: ODS markdown — use bounded context, avoid full-repo token waste.
---
# Open Document Specs (ODS) Rules

When editing Markdown in an ODS workspace:
1. Keep ODS metadata consistent (`ods.profile`, `ods.status`, `depends`, `related`).
2. Run `ods context <id>` and read only those paths — do not dump the repository.
3. Use `ods mv` for renames; `ods lint` after structural edits.
"#;

/// Agent-facing skill bundle: progressive references only (no evals — those burn host context).
const SKILL_BUNDLE: &[(&str, &[u8])] = &[
    ("SKILL.md", include_bytes!("../../../assets/skills/ods/SKILL.md")),
    ("index.md", include_bytes!("../../../assets/skills/ods/index.md")),
    ("references/index.md", include_bytes!("../../../assets/skills/ods/references/index.md")),
    ("references/intro.md", include_bytes!("../../../assets/skills/ods/references/intro.md")),
    ("references/keys.md", include_bytes!("../../../assets/skills/ods/references/keys.md")),
    ("references/core.md", include_bytes!("../../../assets/skills/ods/references/core.md")),
    ("references/scope.md", include_bytes!("../../../assets/skills/ods/references/scope.md")),
    ("references/lsp.md", include_bytes!("../../../assets/skills/ods/references/lsp.md")),
    ("scripts/bootstrap.ps1", include_bytes!("../../../assets/skills/ods/scripts/bootstrap.ps1")),
    ("scripts/bootstrap.sh", include_bytes!("../../../assets/skills/ods/scripts/bootstrap.sh")),
    ("scripts/install-from-release.sh", include_bytes!("../../../assets/skills/ods/scripts/install-from-release.sh")),
];
