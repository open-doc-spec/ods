fn run_completion_command(args: &[String]) -> Result<ExitCode, CliError> {
    let shell = args
        .get(2)
        .map(|s| s.to_lowercase())
        .ok_or_else(|| usage_msg(ods_core::missing_required_arg("shell", "ods completion <bash|zsh|fish|powershell>")))?;

    match shell.as_str() {
        "bash" => {
            println!("{}", BASH_COMPLETION);
        }
        "zsh" => {
            println!("{}", ZSH_COMPLETION);
        }
        "fish" => {
            println!("{}", FISH_COMPLETION);
        }
        "powershell" => {
            println!("{}", POWERSHELL_COMPLETION);
        }
        other => {
            return Err(usage_msg(ods_core::invalid_choice(
                "shell",
                other,
                "bash|zsh|fish|powershell",
            )));
        }
    }

    Ok(ExitCode::from(0))
}

const BASH_COMPLETION: &str = r#"_ods_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="lint index profiles profile status find tag context graph mv fmt adopt new rm archive init disable doctor sync watch logs serve export start stop share bench audit coverage setup update upgrade workspaces skill pack stats overview summary completion schema tree diff clean"

    if [[ ${COMP_CWORD} -eq 1 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
}
complete -F _ods_completions ods
"#;

const ZSH_COMPLETION: &str = r#"#compdef ods
_ods() {
    local -a commands
    commands=(
        'lint:Validate workspace markdown files and graph consistency'
        'index:OKF only: generate or check OKF navigation indexes (ods index --okf)'
        'profiles:List or initialize document profile schemas'
        'status:Set document lifecycle status (draft|stable|deprecated|archived)'
        'find:Find documents by tag, profile, or query'
        'tag:Tag management and workspace-wide tag renaming'
        'context:Resolve bounded context for a target document'
        'graph:Export workspace dependency graph'
        'mv:Move document and automatically heal relative references'
        'fmt:Format frontmatter and Markdown structure'
        'adopt:Draft profile frontmatter for unindexed Markdown files'
        'new:Scaffold a new document with frontmatter'
        'rm:Remove document and scrub references'
        'archive:Archive document status (alias for status archived)'
        'init:Initialize ODS workspace'
        'disable:Strip ODS metadata / opt out of workspace'
        'doctor:Report workspace health and configuration status'
        'sync:Synchronize git status and workspace metadata'
        'watch:Watch file system and re-lint on changes'
        'logs:View service watcher logs'
        'serve:Run foreground language server / watcher'
        'export:Export graph visualization'
        'stats:Display workspace document telemetry and health metrics'
        'overview:Compact workspace snapshot for AI cold-start'
        'summary:Alias for overview'
        'completion:Generate shell autocompletion scripts'
        'schema:Export JSON Schema for frontmatter validation'
        'tree:Display visual hierarchy tree of workspace documents'
        'diff:Compare document graph changes against git commit/branch'
        'clean:Clean diagnostic reports and cache files'
    )
    _describe 'ods command' commands
}
_ods "$@"
"#;

const FISH_COMPLETION: &str = r#"complete -c ods -f
complete -c ods -n "__fish_use_subcommand" -a "lint" -d "Validate workspace consistency"
complete -c ods -n "__fish_use_subcommand" -a "index" -d "OKF only: generate/check OKF indexes (ods index --okf)"
complete -c ods -n "__fish_use_subcommand" -a "stats" -d "Display document telemetry and health score"
complete -c ods -n "__fish_use_subcommand" -a "overview" -d "AI cold-start workspace snapshot"
complete -c ods -n "__fish_use_subcommand" -a "summary" -d "Alias for overview"
complete -c ods -n "__fish_use_subcommand" -a "schema" -d "Export JSON Schema for frontmatter"
complete -c ods -n "__fish_use_subcommand" -a "tree" -d "Display visual document tree"
complete -c ods -n "__fish_use_subcommand" -a "diff" -d "Compare graph changes"
complete -c ods -n "__fish_use_subcommand" -a "clean" -d "Clean diagnostic reports"
"#;

const POWERSHELL_COMPLETION: &str = r#"Register-ArgumentCompleter -Native -CommandName ods -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @('lint', 'index', 'profiles', 'status', 'find', 'tag', 'context', 'graph', 'mv', 'fmt', 'adopt', 'new', 'rm', 'archive', 'init', 'disable', 'doctor', 'sync', 'watch', 'logs', 'serve', 'export', 'stats', 'overview', 'summary', 'completion', 'schema', 'tree', 'diff', 'clean')
    $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#;
