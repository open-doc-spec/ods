use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ods"));
    command.env("ODS_AUTO_UPDATE", "0");
    command
}

#[test]
fn installs_complete_bundles_for_skill_agents() {
    for (agent, expected_directory) in [
        ("codex", ".codex/skills/ods"),
        ("gemini-cli", ".gemini/skills/ods"),
        ("claude-code", ".claude/skills/ods"),
        ("antigravity", ".gemini/config/skills/ods"),
    ] {
        let directory = tempdir().expect("temporary test directory");
        let output = ods_bin()
            .current_dir(directory.path())
            .args(["skill", "install", "--agent", agent, "--scope", "project"])
            .output()
            .expect("run ods skill install");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let installed = directory.path().join(expected_directory);
        assert!(installed.join("SKILL.md").is_file());
        assert!(installed.join("scripts/bootstrap.sh").is_file());
        assert!(installed.join("references/index.md").is_file());
        assert!(installed.join("references/lsp.md").is_file());
        assert!(
            !installed.join("evals/evals.json").exists(),
            "evals should not be installed into agent hosts (token waste)"
        );
        assert!(
            fs::read_to_string(installed.join("SKILL.md"))
                .unwrap()
                .contains("name: ods")
        );
    }
}

#[test]
fn installs_file_based_agents() {
    for (agent, expected_file, content_snippet) in [
        (
            "cursor",
            ".cursor/rules/ods.mdc",
            "Open Document Specs (ODS) Rules",
        ),
        ("copilot", ".github/copilot-instructions.md", "name: ods"),
        (
            "windsurf",
            ".windsurf/rules/ods.md",
            "trigger: model_decision",
        ),
    ] {
        let directory = tempdir().expect("temporary test directory");
        let output = ods_bin()
            .current_dir(directory.path())
            .args(["skill", "install", "--agent", agent, "--scope", "project"])
            .output()
            .expect("run ods skill install");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let installed = directory.path().join(expected_file);
        assert!(installed.is_file());
        assert!(
            fs::read_to_string(installed)
                .unwrap()
                .contains(content_snippet)
        );
    }
}

#[test]
fn respects_user_scope_with_mocked_home() {
    for (agent, expected_home_path) in [
        ("claude-code", ".claude/skills/ods/SKILL.md"),
        ("cursor", ".cursor/rules/ods.mdc"),
        ("antigravity", ".gemini/config/skills/ods/SKILL.md"),
    ] {
        let directory = tempdir().expect("temporary test directory");
        let home_dir = directory.path().join("mock_home");
        fs::create_dir_all(&home_dir).unwrap();

        let output = ods_bin()
            .current_dir(directory.path())
            .env("HOME", &home_dir)
            .env("USERPROFILE", &home_dir)
            .args(["skill", "install", "--agent", agent, "--scope", "user"])
            .output()
            .expect("run ods skill install");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let installed = home_dir.join(expected_home_path);
        assert!(
            installed.is_file(),
            "expected file not found: {:?}",
            installed
        );
    }
}

#[test]
fn default_scopes_are_applied() {
    // claude-code defaults to user scope
    {
        let directory = tempdir().expect("temporary test directory");
        let home_dir = directory.path().join("mock_home");
        fs::create_dir_all(&home_dir).unwrap();

        let output = ods_bin()
            .current_dir(directory.path())
            .env("HOME", &home_dir)
            .env("USERPROFILE", &home_dir)
            .args(["skill", "install", "--agent", "claude-code"])
            .output()
            .expect("run ods skill install");

        assert!(output.status.success());
        assert!(home_dir.join(".claude/skills/ods/SKILL.md").is_file());
    }

    // cursor defaults to project scope
    {
        let directory = tempdir().expect("temporary test directory");
        let output = ods_bin()
            .current_dir(directory.path())
            .args(["skill", "install", "--agent", "cursor"])
            .output()
            .expect("run ods skill install");

        assert!(output.status.success());
        assert!(directory.path().join(".cursor/rules/ods.mdc").is_file());
    }
}

#[test]
fn skill_help_lists_new_agents() {
    let output = ods_bin()
        .args(["skill", "help"])
        .output()
        .expect("run ods skill help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for agent in [
        "codex",
        "gemini-cli",
        "windsurf",
        "claude-code",
        "cursor",
        "antigravity",
        "copilot",
    ] {
        assert!(stdout.contains(agent), "missing {agent} from help output");
    }
}

#[test]
fn skill_command_errors_out_on_invalid_arguments() {
    let directory = tempdir().expect("temporary test directory");

    // Invalid subcommand
    let output1 = ods_bin()
        .current_dir(directory.path())
        .args(["skill", "invalid_sub"])
        .output()
        .unwrap();
    assert!(!output1.status.success());
    let err1 = String::from_utf8_lossy(&output1.stderr);
    assert!(
        err1.contains("unknown skill subcommand") || err1.contains("Next:"),
        "{err1}"
    );

    // Missing agent
    let output2 = ods_bin()
        .current_dir(directory.path())
        .args(["skill", "install"])
        .output()
        .unwrap();
    assert!(!output2.status.success());
    assert!(
        String::from_utf8_lossy(&output2.stderr).contains("missing value for --agent")
            || String::from_utf8_lossy(&output2.stderr).contains("--agent"),
        "{}",
        String::from_utf8_lossy(&output2.stderr)
    );

    // Invalid agent
    let output3 = ods_bin()
        .current_dir(directory.path())
        .args(["skill", "install", "--agent", "unknown"])
        .output()
        .unwrap();
    assert!(!output3.status.success());
    assert!(String::from_utf8_lossy(&output3.stderr).contains("unknown agent"));

    // Invalid scope
    let output4 = ods_bin()
        .current_dir(directory.path())
        .args([
            "skill", "install", "--agent", "cursor", "--scope", "invalid",
        ])
        .output()
        .unwrap();
    assert!(!output4.status.success());
    assert!(String::from_utf8_lossy(&output4.stderr).contains("invalid scope"));
}

#[test]
fn error_on_missing_home_directory_for_user_scope() {
    let directory = tempdir().unwrap();
    let output = ods_bin()
        .current_dir(directory.path())
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .args(["skill", "install", "--agent", "cursor", "--scope", "user"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not resolve home directory"));
}
