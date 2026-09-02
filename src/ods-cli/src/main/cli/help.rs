// Top-level and per-command help (user-facing).
// `ods --help` is the non-technical entrypoint. `ods help <command>` and
// `ods <command> --help` share the same catalog.
fn print_help() {
    println!("{}", TOP_LEVEL_HELP);
}

fn print_ods_help() {
    print_help();
}

/// Print help for a command or alias. Returns `false` if the name is unknown.
fn print_command_help(name: &str) -> bool {
    let key = canonical_help_command(name);
    let Some(body) = command_help_text(key) else {
        return false;
    };
    println!("{body}");
    true
}

fn canonical_help_command(name: &str) -> &str {
    match name {
        "summary" => "overview",
        "profiles" => "profile",
        "remove" => "rm",
        "enable" => "init",
        "revert" => "disable",
        "sandbox" => "bench",
        "--version" | "-V" => "version",
        "--help" | "-h" => "help",
        other => other,
    }
}

fn command_help_text(name: &str) -> Option<&'static str> {
    Some(match name {
        "help" => HELP_HELP,
        "version" => HELP_VERSION,
        "init" => HELP_INIT,
        "setup" => HELP_SETUP,
        "lint" => HELP_LINT,
        "overview" => HELP_OVERVIEW,
        "find" => HELP_FIND,
        "read" => HELP_READ,
        "context" => HELP_CONTEXT,
        "tag" => HELP_TAG,
        "tags" => HELP_TAGS,
        "tree" => HELP_TREE,
        "graph" => HELP_GRAPH,
        "schema" => HELP_SCHEMA,
        "new" => HELP_NEW,
        "adopt" => HELP_ADOPT,
        "fmt" => HELP_FMT,
        "mv" => HELP_MV,
        "rm" => HELP_RM,
        "status" => HELP_STATUS,
        "archive" => HELP_ARCHIVE,
        "undo" => HELP_UNDO,
        "doctor" => HELP_DOCTOR,
        "stats" => HELP_STATS,
        "audit" => HELP_AUDIT,
        "coverage" => HELP_COVERAGE,
        "export" => HELP_EXPORT,
        "share" => HELP_SHARE,
        "diff" => HELP_DIFF,
        "sync" => HELP_SYNC,
        "clean" => HELP_CLEAN,
        "disable" => HELP_DISABLE,
        "profile" => HELP_PROFILE,
        "start" => HELP_START,
        "stop" => HELP_STOP,
        "serve" => HELP_SERVE,
        "watch" => HELP_WATCH,
        "logs" => HELP_LOGS,
        "lsp" => HELP_LSP,
        "completion" => HELP_COMPLETION,
        "pack" => HELP_PACK,
        "skill" => HELP_SKILL,
        "agents" => HELP_AGENTS,
        "workspaces" => HELP_WORKSPACES,
        "bench" => HELP_BENCH,
        "update" => HELP_UPDATE,
        "upgrade" => HELP_UPGRADE,
        "index" => HELP_INDEX,
        _ => return None,
    })
}

fn argv_wants_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "--help" || a == "-h")
}

fn command_accepts_help_subcommand(cmd: &str) -> bool {
    matches!(
        cmd,
        "skill"
            | "pack"
            | "agents"
            | "workspaces"
            | "profile"
            | "profiles"
            | "tag"
            | "bench"
    )
}

/// Handle `ods`, `ods help [cmd]`, `ods --help [cmd]`, `ods <cmd> --help`.
fn try_print_cli_help(args: &[String]) -> Option<Result<ExitCode, CliError>> {
    let first = args.get(1).map(String::as_str);
    match first {
        None => {
            print_help();
            Some(Ok(ExitCode::from(0)))
        }
        Some("help") | Some("--help") | Some("-h") => match args.get(2).map(String::as_str) {
            None | Some("-h") | Some("--help") | Some("help") => {
                print_help();
                Some(Ok(ExitCode::from(0)))
            }
            Some(cmd) => {
                if print_command_help(cmd) {
                    Some(Ok(ExitCode::from(0)))
                } else {
                    let suggestion = ods_core::suggest_command(cmd);
                    Some(Err(usage_msg(ods_core::unknown_command(cmd, suggestion))))
                }
            }
        },
        Some(cmd)
            if argv_wants_help(args)
                || (command_accepts_help_subcommand(cmd)
                    && args.get(2).map(String::as_str) == Some("help")) =>
        {
            if print_command_help(cmd) {
                Some(Ok(ExitCode::from(0)))
            } else {
                None
            }
        }
        _ => None,
    }
}

const TOP_LEVEL_HELP: &str = "\
ods — Open Document Spec

Turn a folder of Markdown files into a checked documentation workspace.
Writers can lint and find pages. Engineers can keep links, tags, and
status consistent. AI tools can load a small reading list instead of
the whole repo.

Usage:
  ods <command> [options]
  ods help <command>
  ods <command> --help

Getting started:
  init [path]                 Make this folder an ODS workspace (writes ods.toml)
  setup [path]                First-run check: doctor, optional editor + git hooks
  lint [path]                 Check documents and links (prints OK when clean)
  overview [path]             Snapshot of the workspace (good first command)

Discover:
  find [path] [query]         Find documents by tag, key, or name
  read <id>                   Read a section or summary (optional token budget)
  context <id>                Bounded reading list (depends + top-level load)
  tag list|show|rename        List tags, show docs, or rename a tag
  tag rename <old> <new>      Workspace-wide tag rename (dry-run; --write)
  tags [path]                 Tag counts (--all includes unused builtins)
  tree [path]                 Folder tree of documents
  graph [path]                Print depends/related edges
  schema [keys]               List keys or write JSON Schema

Author:
  new <path>                  Create a document with frontmatter
  adopt [path]                Draft frontmatter for existing Markdown (dry-run)
  fmt [path]                  Normalize frontmatter spacing
  mv <from> <to>              Move a file and rewrite references
  rm <path-or-id>             Delete a document and scrub references
  status <id> <value>         Set draft | stable | deprecated | archived
  archive <id>                Shortcut for status archived
  undo [path]                 Restore the latest frontmatter snapshot

Workspace:
  doctor [path]               Health check
  stats [path]                Counts, graph density, lint health %
  audit [path]                Inventory plain vs compliant Markdown
  coverage [path]             Documentation health %
  export [path]               Write graph snapshot (default .ods/graph.md)
  share --out DIR             Publish a share-filtered copy
  diff [git-rev]              Markdown changes vs a git revision
  sync [path]                 Apply git renames to the graph
  clean [path]                Remove generated .ods reports
  disable [path]              Opt out / strip ODS metadata (dry-run)

Profiles:
  profile list                List loaded profiles (alias: ods profiles)
  profile init <name>         Scaffold a custom profile
  profile show <name>         Show keys and sections

Automation:
  start [path]                Register and start the background watcher
  stop [path]                 Stop the background service
  serve [path]                Headless watch loop (--mode auto|watch|poll)
  watch [path]                Foreground watch + re-lint
  logs [-f]                   Show background service logs
  lsp                         Language Server for editors (JSON-RPC)
  completion <shell>          Completions: bash | zsh | fish | powershell

Packs, agents, and machine:
  pack …                      Import reusable documentation packs
  skill install               Install the ODS skill into an AI agent
  agents sync                 Write or refresh AGENTS.md
  workspaces …                Global workspace registry (~/.ods/odsconfig.toml)
  bench …                     Benchmark / snapshot frontmatter
  update                      Self-update this binary
  upgrade                     Workspace/machine cutover helper
  version, --version, -V      Print version and exit
  help, --help, -h            Show this help (or help for one command)

Global options (most commands):
  --help, -h                  Show help for this command
  --version, -V               Print version and exit
  --format text|json|sarif    Output format where supported (default: text)
  --okf                       Also run the OKF engine
  --skills                    Also run the Agent Skills engine
  --root <dir>                Workspace root (when the command supports it)

Examples:
  ods init
  ods lint
  ods find --tag api
  ods read checkout --section Overview
  ods serve --root . --mode poll
  ods serve --mode poll
  ods help lint

Environment:
  ODS_AUTO_UPDATE=0           Disable auto-update (default: on)
  ODS_LOW_MEMORY=1            serve --mode auto → poll
  ODS_SERVE_MODE              Default serve mode (auto|watch|poll)
  ODS_POLL_SECS               Default poll interval for serve
  GH_TOKEN / GITHUB_TOKEN     Optional token for GitHub rate limits

Learn more:
  ods help <command>          Flags, arguments, and examples for one command
  https://opendocify.com
";

const HELP_HELP: &str = "\
ods help — show usage for ods or one command

Usage:
  ods help
  ods help <command>
  ods --help
  ods <command> --help

Description:
  With no command, prints the full command map (same as `ods` or `ods --help`).
  With a command, prints that command’s arguments, flags, and examples.

Examples:
  ods help
  ods help find
  ods lint --help
";

const HELP_VERSION: &str = "\
ods version — print the installed CLI version

Usage:
  ods version
  ods --version
  ods -V

Description:
  Prints `ods <semver>` and exits. Does not check for updates.

See also:
  ods update --check
";

const HELP_INIT: &str = "\
ods init — make a folder an ODS workspace

Usage:
  ods init [path] [options]

Description:
  Writes `ods.toml` so this folder is an Open Document Spec workspace.
  Safe to re-run: already-initialized workspaces are left in place.
  Alias: ods enable

Arguments:
  [path]                      Folder to initialize (default: current directory)

Options:
  --adopt                     Also draft frontmatter on existing Markdown
  --okf                       Initialize an OKF knowledge bundle instead
  --skills                    Initialize an Agent Skills package instead
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods init
  ods init ./docs
  ods init --adopt

See also:
  ods setup, ods adopt, ods lint
";

const HELP_SETUP: &str = "\
ods setup — first-run machine and workspace check

Usage:
  ods setup [path] [options]

Description:
  Checks for updates, finds an ODS workspace, starts the user service
  when possible, and runs `ods doctor`. Use this after installing ods.

Arguments:
  [path]                      Folder to probe (default: current directory)

Options:
  --git-hooks                 Install .git/hooks/pre-commit (`ods lint`)
  --editor zed|vscode|nvim|cursor
                              Write Language Server config for `ods lsp`
  --help, -h                  Show this help

Examples:
  ods setup
  ods setup --git-hooks
  ods setup --editor zed

See also:
  ods doctor, ods start, ods lsp
";

const HELP_LINT: &str = "\
ods lint — check documents and links

Usage:
  ods lint [path] [options]

Description:
  Validates Markdown frontmatter, references, and workspace rules.
  Prints a success line when everything is clean.

Arguments:
  [path]                      Folder to check (default: current directory)

Options:
  --format text|json|sarif    Output format (default: text)
  --canonical-refs            Warn when document refs omit .md
  --okf                       Also lint OKF
  --skills                    Also lint Agent Skills packages
  --skip-frontmatter-keys     Do not require profile keys
  --ignore-keys <k1,k2>       Ignore specific frontmatter keys
  --fix                       No-op for ODS (nested indexes were removed)
  --help, -h                  Show this help

Examples:
  ods lint
  ods lint . --format json
  ods lint --okf

See also:
  ods doctor, ods fmt, ods overview
";

const HELP_OVERVIEW: &str = "\
ods overview — compact workspace snapshot

Usage:
  ods overview [path] [options]
  ods summary [path]

Description:
  Prints document counts, profile/status breakdown, top tags, custom
  keys, and graph stats. Useful as an AI or human cold-start.
  For lint health %, use `ods stats`.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods overview
  ods summary --format json

See also:
  ods stats, ods find, ods tree
";

const HELP_FIND: &str = "\
ods find — find documents by tag, key, or name

Usage:
  ods find [path] [--tag <name> ...] [--key <expr> ...] [<query>]

Description:
  Search the workspace by tag, schema/custom keys, and/or id/path/stem.
  Value match is exact (case-insensitive).

Arguments:
  [path]                      Workspace root (default: current directory)
  [query]                     Match document id, path, or filename stem

Options:
  --tag <name>                Filter by tag (repeatable)
  --tag-match any|all         Tag combination (default: any)
  --key <expr>                Filter by key/value (comma values; AND/OR text)
  --key-match and|or          Combine multiple --key flags (default: and)
  --status <status>           Shortcut for --key status=<status>
  --profile <profile>         Shortcut for --key profile=<profile>
  --owner <owner>             Shortcut for --key owner=<owner>
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods find --tag caching
  ods find --key status=draft,stable
  ods find --key \"status=draft AND owner=alice\"
  ods find checkout

See also:
  ods tag list, ods read, ods context
";

const HELP_READ: &str = "\
ods read — read a document, section, or outline

Usage:
  ods read [root] <id-or-path> [options]

Description:
  Prints document prose, one heading section, or an outline summary.
  Use --max-tokens to keep AI prompts small.

Arguments:
  [root]                      Workspace root (default: current directory)
  <id-or-path>                Document id or path inside the workspace

Options:
  --section <heading>         Extract a section by title or slug
  --summary                   Outline only (headings and metadata)
  --max-tokens <N>            Soft token limit (bytes / 4)
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods read checkout
  ods read checkout --section Overview
  ods read checkout --summary --max-tokens 400

See also:
  ods context, ods find
";

const HELP_CONTEXT: &str = "\
ods context — bounded reading list for one document

Usage:
  ods context [root] <id-or-path> [options]
  ods context --tag <name>    (when that tag matches exactly one doc)

Description:
  Builds a small reading list: the target plus `depends` and
  top-level `load`. Does not walk `related` unless --include-related.
  Code edges stay off unless --include-code.

Arguments:
  [root]                      Workspace root, or use --root
  <id-or-path>                Target document (omit only with unique filters)

Options:
  --root <dir>                Workspace root (default: current directory)
  --tag <name>                When no id: require this tag (unique match)
  --key <expr>                When no id: key filter (unique match)
  --status <status>           When no id: shortcut for --key status=<status>
  --include-private           Include share: private documents
  --include-code              Expand code: edges
  --include-related           Also walk soft related: edges
  --explain                   Show why each path was included
  --max-tokens <N>            Cap estimated tokens (bytes / 4)
  --print                     Print file contents under the budget
  --format text|json          Output format (default: text)
  --okf                       Include OKF neighborhood (pure OKF or hybrid)
  --help, -h                  Show this help

Examples:
  ods context checkout
  ods context checkout --print --max-tokens 2000
  ods context --tag rfc-001

See also:
  ods read, ods find, ods export
";

const HELP_TAG: &str = "\
ods tag — list, inspect, or rename tags

Usage:
  ods tag list [path] [--format text|json]
  ods tag show [path] <tag> [--format text|json]
  ods tag rename [path] <old> <new> [--write]

Description:
  Works on observed top-level document tags (not keys under `ods:`).
  Rename is a dry-run until you pass --write.

Subcommands:
  list                        Tags in the workspace with document counts
  show <tag>                  Documents that carry one tag
  rename <old> <new>          Rewrite a tag across the workspace

Options:
  --format text|json          Output format (default: text)
  --write                     Apply rename (default: dry-run)
  --help, -h                  Show this help

Examples:
  ods tag list
  ods tag show api
  ods tag rename old-name new-name --write

See also:
  ods tags, ods find --tag
";

const HELP_TAGS: &str = "\
ods tags — tag counts for the workspace

Usage:
  ods tags [path] [options]

Description:
  Lists top-level tags with how many documents use them.
  Pass --all to include unused builtin tags.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --all                       Include unused builtin tags
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods tags
  ods tags --all

See also:
  ods tag list, ods find --tag
";

const HELP_TREE: &str = "\
ods tree — document folder tree

Usage:
  ods tree [path] [options]

Description:
  Prints an ASCII tree of Markdown documents under the workspace.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --depth <N>                 Max path depth (default: 2)
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods tree
  ods tree --depth 4

See also:
  ods overview, ods find
";

const HELP_GRAPH: &str = "\
ods graph — print depends/related edges

Usage:
  ods graph [path] [options]

Description:
  Prints graph edges as `path -> edge` lines. For a saved snapshot,
  use `ods export`.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods graph
  ods graph --format json

See also:
  ods export, ods context
";

const HELP_SCHEMA: &str = "\
ods schema — list keys or write JSON Schema

Usage:
  ods schema [keys] [options]

Description:
  Bare `ods schema` prints JSON Schema for frontmatter.
  `ods schema keys` lists registered keys, placement, and types.

Subcommands:
  keys                        List registered schema keys (text by default)

Options:
  --write, -w                 Save schema to .ods/<dialect>.schema.json
  --out <file>, -o <file>     Save schema to this path
  --okf                       Use the OKF schema
  --skills                    Use the Skills schema
  --spec ods|okf|skills       Select dialect
  --format text|json          Default: json for schema, text for keys
  --help, -h                  Show this help

Examples:
  ods schema keys
  ods schema --write
  ods schema --okf --out okf.schema.json

See also:
  ods lint, ods find --key
";

const HELP_NEW: &str = "\
ods new — create a document with frontmatter

Usage:
  ods new <path> [options]

Description:
  Scaffolds a Markdown file with inferred or explicit profile and a
  valid frontmatter block.

Arguments:
  <path>                      New file path (relative to the workspace)

Options:
  --profile, -p <name>        Profile to use (otherwise inferred)
  --title, -t <title>         Document title
  --help, -h                  Show this help

Examples:
  ods new docs/api.md
  ods new docs/rfc-42.md --profile rfc --title \"Auth RFC\"

See also:
  ods adopt, ods status
";

const HELP_ADOPT: &str = "\
ods adopt — draft frontmatter for existing Markdown

Usage:
  ods adopt [path] [options]

Description:
  Proposes profile frontmatter for Markdown that is not yet adopted.
  Dry-run by default; pass --write to change files.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --write                     Write drafted frontmatter
  --okf                       Adopt an OKF bundle instead
  --format text|json|sarif    Also print lint diagnostics
  --help, -h                  Show this help

Examples:
  ods adopt
  ods adopt --write

See also:
  ods init --adopt, ods fmt --migrate
";

const HELP_FMT: &str = "\
ods fmt — normalize frontmatter and optional key layout

Usage:
  ods fmt [path] [options]

Description:
  Rewrites YAML frontmatter/body blank-line spacing.
  --migrate moves engine keys under `ods:` and preserves non-ODS keys.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --migrate                   Rewrite engine keys under ods:
  --refs md-paths             Convert extensionless ids to relative .md paths
  --okf                       Also format OKF
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods fmt
  ods fmt --migrate

See also:
  ods lint, ods adopt
";

const HELP_MV: &str = "\
ods mv — move a document and rewrite references

Usage:
  ods mv [root] <from> <to> [options]
  ods mv --root <dir> <from> <to>

Description:
  Moves a Markdown file and updates graph references workspace-wide.

Arguments:
  [root]                      Workspace root (or use --root)
  <from>                      Source path or id
  <to>                        Destination path

Options:
  --root <dir>                Workspace root
  --dry-run                   Print what would change
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods mv docs/old.md docs/new.md
  ods mv old-id docs/new.md --dry-run

See also:
  ods sync, ods rm
";

const HELP_RM: &str = "\
ods rm — delete a document and scrub references

Usage:
  ods rm [root] <path-or-id> [options]
  ods remove [root] <path-or-id>

Description:
  Deletes the file and removes it from depends/related across the workspace.

Arguments:
  [root]                      Workspace root (or use --root)
  <path-or-id>                Document to delete

Options:
  --root <dir>                Workspace root
  --dry-run                   Print what would change
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods rm docs/old.md
  ods rm old-id --dry-run

See also:
  ods mv, ods archive
";

const HELP_STATUS: &str = "\
ods status — set document lifecycle status

Usage:
  ods status <path-or-id> <draft|stable|deprecated|archived>

Description:
  Writes nested `ods.status` when an `ods:` map exists.
  Alias: `ods archive <path-or-id>` sets archived.

Arguments:
  <path-or-id>                Document path or id
  <status>                    draft | stable | deprecated | archived

Options:
  --help, -h                  Show this help

Examples:
  ods status checkout stable
  ods archive old-promo

See also:
  ods archive, ods find --status
";

const HELP_ARCHIVE: &str = "\
ods archive — mark a document archived

Usage:
  ods archive <path-or-id>

Description:
  Shortcut for `ods status <path-or-id> archived`. Does not delete the file.

Arguments:
  <path-or-id>                Document path or id

Options:
  --help, -h                  Show this help

Examples:
  ods archive docs/old-promo.md

See also:
  ods status, ods rm
";

const HELP_UNDO: &str = "\
ods undo — restore the latest frontmatter snapshot

Usage:
  ods undo [path] [options]

Description:
  Restores the newest snapshot under ~/.ods/backups/<repo-hash>/.
  Snapshots are created mainly by `ods bench strip --write`.
  This is not a general git undo.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --list                      List snapshot ids (newest last)
  --help, -h                  Show this help

Examples:
  ods undo --list
  ods undo

See also:
  ods bench restore
";

const HELP_DOCTOR: &str = "\
ods doctor — workspace health check

Usage:
  ods doctor [path] [options]

Description:
  Reports version, document count, ods.toml, profile conflicts, and
  service status.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --okf                       Run OKF doctor instead
  --help, -h                  Show this help

Examples:
  ods doctor
  ods doctor --format json

See also:
  ods setup, ods lint, ods stats
";

const HELP_STATS: &str = "\
ods stats — workspace telemetry and health %

Usage:
  ods stats [path] [options]

Description:
  Document counts, profile mix, graph density, and lint health score.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods stats
  ods stats --format json

See also:
  ods overview, ods coverage
";

const HELP_AUDIT: &str = "\
ods audit — inventory plain vs compliant Markdown

Usage:
  ods audit [path] [options]

Description:
  Classifies Markdown as plain, invalid, partial, or compliant.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --write-report              Write a report file
  --report-path <path>        Report destination
  --fail-on any|plain|invalid Exit 1 when that class is present
  --format text|json          Output format (default: text)
  --okf                       Audit an OKF bundle
  --help, -h                  Show this help

Examples:
  ods audit
  ods audit --write-report --fail-on invalid

See also:
  ods coverage, ods lint
";

const HELP_COVERAGE: &str = "\
ods coverage — documentation health percent

Usage:
  ods coverage [path] [options]

Description:
  Percent of documents that parse and lint clean.
  Separate from lint’s `.ods/ods-errors.md`.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --write-report              Write `.ods/coverage.md`
  --summary                   Hide the per-file list
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods coverage
  ods coverage --write-report --summary

See also:
  ods stats, ods audit
";

const HELP_EXPORT: &str = "\
ods export — write a graph snapshot

Usage:
  ods export [path] [options]
  ods export graph --out PATH

Description:
  Writes a Markdown graph dump (default `.ods/graph.md`).
  This is a snapshot, not a routine AI prompt. Prefer `ods context`.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --out PATH                  Output file (default: .ods/graph.md)
  --include-private           Include share: private / org documents
  --format text|json          json prints the graph to stdout
  --okf                       Export an OKF bundle
  --help, -h                  Show this help

Examples:
  ods export
  ods export --out .ods/graph.md
  ods export --format json

See also:
  ods graph, ods context, ods share
";

const HELP_SHARE: &str = "\
ods share — publish a share-filtered copy

Usage:
  ods share [path] --out DIR [options]

Description:
  Copies documents allowed by share visibility into DIR.
  Does not run git for you.

Arguments:
  [path]                      Workspace or subtree (default: current directory)

Options:
  --out DIR                   Destination directory (required)
  --include-org               Include share: org documents
  --include-private           Include share: private documents
  --help, -h                  Show this help

Examples:
  ods share --out ./public
  ods share docs --out ./public --include-org

See also:
  ods export
";

const HELP_DIFF: &str = "\
ods diff — Markdown changes vs a git revision

Usage:
  ods diff [git-rev] [options]

Description:
  Runs `git diff` for `*.md` against HEAD or the revision you pass.

Arguments:
  [git-rev]                   Git revision (default: HEAD)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods diff
  ods diff main

See also:
  ods sync, ods graph
";

const HELP_SYNC: &str = "\
ods sync — apply git renames to the graph

Usage:
  ods sync [path] [options]

Description:
  Reads `git status` renames and rewrites document references.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods sync

See also:
  ods mv, ods watch
";

const HELP_CLEAN: &str = "\
ods clean — remove generated report files

Usage:
  ods clean [path] [options]

Description:
  Deletes `.ods/ods-errors.md`, `.ods/coverage.md`, and
  `.ods/ods.schema.json` when present.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods clean

See also:
  ods lint, ods coverage, ods schema
";

const HELP_DISABLE: &str = "\
ods disable — opt out / strip ODS metadata

Usage:
  ods disable [path] [options]
  ods revert [path]

Description:
  Dry-run by default. Pass --write to apply.
  Alias: ods revert

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --write                     Apply the changes
  --keep-frontmatter          Leave document frontmatter in place
  --remove-indexes            Delete leftover index files if present
  --remove-root-index         Also remove a legacy root index
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods disable
  ods disable --write

See also:
  ods init, ods doctor
";

const HELP_PROFILE: &str = "\
ods profile — list, create, or inspect profiles

Usage:
  ods profile list [path]
  ods profiles [path]
  ods profile init <name> [path] [--no-register]
  ods profile show <name> [path]

Description:
  Profiles define required sections and keys for a document type.
  `profile init` writes `.ods/profiles/<name>.md` and registers it
  in ods.toml unless you pass --no-register.

Subcommands:
  list                        List standard and custom profiles
  init <name>                 Scaffold a custom profile
  show <name>                 Source, sections, required/optional/forbidden keys

Options:
  --no-register               Do not append to custom_profiles in ods.toml
  --format text|json          Output format for list
  --help, -h                  Show this help

Examples:
  ods profiles
  ods profile init rfc
  ods profile show note

See also:
  ods new --profile
";

const HELP_START: &str = "\
ods start — register and start the background watcher

Usage:
  ods start [path] [options]

Description:
  Installs and starts the user OS service (launchd / systemd / Scheduled Task)
  that runs `ods serve` for this workspace.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --status                    Print installed/running and exit
  --help, -h                  Show this help

Examples:
  ods start
  ods start --status

See also:
  ods stop, ods serve, ods setup
";

const HELP_STOP: &str = "\
ods stop — stop the background watcher

Usage:
  ods stop [path] [options]

Description:
  Stops the user OS service for this workspace.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --unregister                Stop and remove the service registration
  --help, -h                  Show this help

Examples:
  ods stop
  ods stop --unregister

See also:
  ods start, ods logs
";

const HELP_SERVE: &str = "\
ods serve — headless watch loop

Usage:
  ods serve [path] [options]
  ods serve --root <path> --mode auto|watch|poll

Description:
  Long-running daemon used by `ods start`. Not a Language Server
  (use `ods lsp` in editors). Prefer `ods watch` for a live terminal.

Arguments:
  [path]                      Workspace root (or --root)

Options:
  --root <path>               Workspace root (default: current directory)
  --mode auto|watch|poll      auto uses watch, or poll if ODS_LOW_MEMORY=1
  --poll-secs <N>             Poll interval in seconds (default: 10)
  --memory-report             Print RSS diagnostics
  --okf                       Serve an OKF bundle
  --help, -h                  Show this help

Examples:
  ods serve --root . --mode watch
  ods serve --root . --mode poll

Environment:
  ODS_LOW_MEMORY=1            Force poll when --mode auto
  ODS_SERVE_MODE              Default mode
  ODS_POLL_SECS               Default poll interval

See also:
  ods start, ods watch, ods lsp
";

const HELP_WATCH: &str = "\
ods watch — foreground watch and re-lint

Usage:
  ods watch [path] [options]

Description:
  Watches the folder in this terminal and re-lints on changes.
  Stops on Ctrl+C. For background use, prefer `ods start`.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --format text|json|sarif    Lint output format
  --okf                       Watch an OKF bundle
  --help, -h                  Show this help

Examples:
  ods watch
  ods watch --okf

See also:
  ods serve, ods start, ods lint
";

const HELP_LOGS: &str = "\
ods logs — background service logs

Usage:
  ods logs [options]

Description:
  Prints `~/.ods/logs/ods-serve.log` from `ods start` / `ods serve`.

Options:
  -f, --follow                Follow new lines (Ctrl+C to stop)
  --help, -h                  Show this help

Examples:
  ods logs
  ods logs -f

See also:
  ods start, ods serve
";

const HELP_LSP: &str = "\
ods lsp — Language Server for editors

Usage:
  ods lsp [--port <n>]

Description:
  JSON-RPC Language Server for Markdown in an ODS workspace.
  Default: stdio (what editors expect). Not the same as `ods serve`.

Options:
  --port <n>                  Listen on 127.0.0.1:<n> instead of stdio
  --help, -h                  Show this help

Examples:
  ods lsp
  ods setup --editor zed

See also:
  ods setup --editor, ods lint
";

const HELP_COMPLETION: &str = "\
ods completion — shell completion script

Usage:
  ods completion <bash|zsh|fish|powershell>

Description:
  Prints a completion script for the given shell. Redirect it into
  your shell’s completion directory or eval it from your rc file.

Arguments:
  <shell>                     bash | zsh | fish | powershell

Options:
  --help, -h                  Show this help

Examples:
  ods completion zsh
  ods completion bash > ~/.local/share/bash-completion/completions/ods
";

const HELP_PACK: &str = "\
ods pack — import reusable documentation packs

Usage:
  ods pack list [path]
  ods pack add [path] <source> [--auto-update hourly|daily|weekly|never]
  ods pack sync [path]
  ods pack preview [path] <source>
  ods pack remove [path] <name>
  ods pack init <dir>

Description:
  Packs are extra profile catalogs (and related files) imported into
  a workspace via ods.toml `packs = […]`.

Subcommands:
  list                        Packs declared for this workspace
  add <source>                Add a local path or remote pack
  sync                        Update registered packs
  preview <source>            Show what a pack would add
  remove <name>               Unregister a pack
  init <dir>                  Scaffold a new pack folder

Options:
  --auto-update <interval>    hourly | daily | weekly | never (default: daily)
  --help, -h                  Show this help

Examples:
  ods pack list
  ods pack add ./vendor/engineering-pack
  ods pack init ./my-pack

See also:
  ods profile list
";

const HELP_SKILL: &str = "\
ods skill — install the ODS skill into an AI agent

Usage:
  ods skill install --agent <name> [--scope project|user]
  ods skill help

Description:
  Writes the ODS agent skill or editor rules so coding agents use
  bounded `ods context` instead of dumping the repo.

Subcommands:
  install                     Install skill or rules for one agent
  help                        Show this help

Options:
  --agent <name>              claude-code, cursor, antigravity, codex,
                              gemini-cli, windsurf, copilot
  --scope project|user        project = this repo; user = home directory
  --help, -h                  Show this help

Examples:
  ods skill install --agent claude-code
  ods skill install --agent cursor --scope project
";

const HELP_AGENTS: &str = "\
ods agents — write agent instruction files

Usage:
  ods agents sync [path]
  ods agents help

Description:
  Writes AGENTS.md (unless this repo hand-maintains it) and refreshes
  small Claude/Cursor snippets that point at it.

Subcommands:
  sync [path]                 Write or refresh agent files
  help                        Show this help

Options:
  --help, -h                  Show this help

Examples:
  ods agents sync

See also:
  ods skill install
";

const HELP_WORKSPACES: &str = "\
ods workspaces — global workspace registry

Usage:
  ods workspaces list
  ods workspaces add [path]
  ods workspaces remove [path]
  ods workspaces path

Description:
  Tracks ODS folders in ~/.ods/odsconfig.toml so the machine service
  can watch more than the current directory.

Subcommands:
  list                        Registered workspaces (default)
  add [path]                  Register a folder (default: current directory)
  remove [path]               Unregister a folder
  path                        Print the config file path

Options:
  --help, -h                  Show this help

Examples:
  ods workspaces add
  ods workspaces list

See also:
  ods start, ods setup
";

const HELP_BENCH: &str = "\
ods bench — snapshot frontmatter and estimate token savings

Usage:
  ods bench stats [path]
  ods bench strip [path] [--write] [--full] [--indexes] [--profiles] [--path FILE]
  ods bench restore [path] [--snapshot ID]
  ods bench run [path] [--prompt TEXT] [--agent NAME]
  ods bench agent [path] [--prompt TEXT] [--agent NAME]

Description:
  Strip/restore use snapshots under ~/.ods/backups/.
  Alias: ods sandbox

Subcommands:
  stats / roi                 Token-efficiency report
  strip                       Snapshot, then strip frontmatter (dry-run)
  restore                     Restore a snapshot
  run / agent                 Simulated agent prompt comparison

Options:
  --write                     Apply strip
  --full                      Broader strip
  --indexes / --profiles      Also strip indexes or profile files
  --path <file>               Limit strip to one file
  --snapshot <id>             Restore this snapshot
  --prompt <text>             Benchmark prompt
  --agent / --llm <name>      Agent profile name
  --format text|json          Output format
  --help, -h                  Show this help

Examples:
  ods bench stats
  ods bench strip --write
  ods undo --list

See also:
  ods undo, ods stats
";

const HELP_UPDATE: &str = "\
ods update — self-update this binary

Usage:
  ods update [options]
  ods update <tag>

Description:
  Downloads the latest GitHub Release (or a specific tag) and replaces
  the installed `ods` binary. Restarts the user service when it is active.

Options:
  --check                     Report whether an update is available (exit 1 if yes)
  --force                     Reinstall even if the version matches
  --version <tag>             Install this release tag
  --help, -h                  Show this help

Examples:
  ods update --check
  ods update
  ods update v0.1.5

Environment:
  ODS_AUTO_UPDATE=0           Disable background auto-update
  GH_TOKEN / GITHUB_TOKEN     Optional token for GitHub rate limits

See also:
  ods version, ods upgrade
";

const HELP_UPGRADE: &str = "\
ods upgrade — workspace and machine cutover helper

Usage:
  ods upgrade [path] [options]

Description:
  Dry-run by default. Reports ODS/OKF roots and optional frontmatter
  migrate steps. Pass --write to apply safe machine copies.

Arguments:
  [path]                      Workspace root (default: current directory)

Options:
  --check                     Exit 1 if work remains
  --write                     Apply safe machine steps
  --migrate-fm                Also migrate frontmatter (with --write)
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods upgrade
  ods upgrade --check

See also:
  ods update, ods fmt --migrate
";

const HELP_INDEX: &str = "\
ods index — OKF navigation indexes only

Usage:
  ods index --okf [path] [--check]

Description:
  ODS no longer generates nested indexes. For ODS discovery use:
    ods overview · ods find · ods tree · ods context
  `ods index --okf` generates or checks OKF navigation indexes.

Arguments:
  [path]                      OKF bundle root (default: current directory)

Options:
  --okf                       Required for this command
  --check                     Exit 1 if indexes are out of date
  --format text|json          Output format (default: text)
  --help, -h                  Show this help

Examples:
  ods index --okf
  ods index --okf --check

See also:
  ods overview, ods find, ods tree
";

#[cfg(test)]
mod test_help_catalog {
    use super::*;

    const CATALOG_COMMANDS: &[&str] = &[
        "help",
        "version",
        "init",
        "setup",
        "lint",
        "overview",
        "find",
        "read",
        "context",
        "tag",
        "tags",
        "tree",
        "graph",
        "schema",
        "new",
        "adopt",
        "fmt",
        "mv",
        "rm",
        "status",
        "archive",
        "undo",
        "doctor",
        "stats",
        "audit",
        "coverage",
        "export",
        "share",
        "diff",
        "sync",
        "clean",
        "disable",
        "profile",
        "start",
        "stop",
        "serve",
        "watch",
        "logs",
        "lsp",
        "completion",
        "pack",
        "skill",
        "agents",
        "workspaces",
        "bench",
        "update",
        "upgrade",
        "index",
    ];

    #[test]
    fn every_catalog_entry_has_usage_and_help_flag() {
        for name in CATALOG_COMMANDS {
            let body = command_help_text(name).expect(name);
            assert!(body.contains("Usage:"), "{name} missing Usage:");
            assert!(
                body.contains("--help") || *name == "help" || *name == "version",
                "{name} should document --help"
            );
        }
    }

    #[test]
    fn aliases_resolve_to_catalog() {
        assert_eq!(canonical_help_command("summary"), "overview");
        assert_eq!(canonical_help_command("profiles"), "profile");
        assert_eq!(canonical_help_command("remove"), "rm");
        assert_eq!(canonical_help_command("enable"), "init");
        assert_eq!(canonical_help_command("revert"), "disable");
        assert_eq!(canonical_help_command("sandbox"), "bench");
        assert!(print_command_help("summary"));
        assert!(!print_command_help("not-a-real-command"));
        assert!(!print_command_help("alias"));
        assert!(!print_command_help("aliases"));
        assert!(command_help_text("alias").is_none());
        assert!(command_help_text("aliases").is_none());
        assert!(!TOP_LEVEL_HELP.contains("ods alias"));
        assert!(!TOP_LEVEL_HELP.contains("ods aliases"));
        let gone = try_print_cli_help(&["ods".into(), "help".into(), "aliases".into()]);
        assert!(gone.unwrap().is_err());
    }

    #[test]
    fn top_level_help_covers_pinned_strings() {
        assert!(TOP_LEVEL_HELP.contains("setup [path]"));
        assert!(TOP_LEVEL_HELP.contains("serve --mode poll"));
        assert!(TOP_LEVEL_HELP.contains("ODS_LOW_MEMORY=1"));
        assert!(TOP_LEVEL_HELP.contains("tag list|show|rename"));
        for cmd in [
            "lint", "profiles", "tags", "find", "context", "graph", "mv", "fmt", "adopt", "doctor",
            "sync", "watch", "update", "export", "start", "stop", "serve", "init",
        ] {
            assert!(TOP_LEVEL_HELP.contains(cmd), "missing {cmd}");
        }
        assert!(!TOP_LEVEL_HELP.contains("ods-lsp"));
        assert!(!TOP_LEVEL_HELP
            .to_ascii_lowercase()
            .contains("zed extension"));
    }

    #[test]
    fn try_print_help_routes() {
        let none = try_print_cli_help(&["ods".into()]);
        assert!(none.unwrap().is_ok());

        let help = try_print_cli_help(&["ods".into(), "help".into()]);
        assert!(help.unwrap().is_ok());

        let cmd = try_print_cli_help(&["ods".into(), "lint".into(), "--help".into()]);
        assert!(cmd.unwrap().is_ok());

        let nested = try_print_cli_help(&["ods".into(), "help".into(), "find".into()]);
        assert!(nested.unwrap().is_ok());

        let skill = try_print_cli_help(&["ods".into(), "skill".into(), "help".into()]);
        assert!(skill.unwrap().is_ok());

        let unknown = try_print_cli_help(&["ods".into(), "help".into(), "nope-xyz".into()]);
        assert!(unknown.unwrap().is_err());

        assert!(try_print_cli_help(&["ods".into(), "lint".into()]).is_none());
    }

    #[test]
    fn argv_help_and_subcommand_helpers() {
        assert!(argv_wants_help(&["ods".into(), "lint".into(), "-h".into()]));
        assert!(!argv_wants_help(&[
            "ods".into(),
            "logs".into(),
            "-f".into()
        ]));
        for cmd in [
            "skill",
            "pack",
            "agents",
            "workspaces",
            "profile",
            "profiles",
            "tag",
            "bench",
        ] {
            assert!(command_accepts_help_subcommand(cmd), "{cmd}");
        }
        assert!(!command_accepts_help_subcommand("read"));
        assert!(!command_accepts_help_subcommand("lint"));
    }

    #[test]
    fn print_ods_help_and_remaining_aliases() {
        print_ods_help();
        for alias in [
            "summary",
            "profiles",
            "remove",
            "enable",
            "revert",
            "sandbox",
            "--version",
            "-V",
            "--help",
            "-h",
            "version",
            "help",
        ] {
            assert!(print_command_help(alias), "{alias}");
        }
    }

    #[test]
    fn try_print_help_covers_flag_and_subcommand_forms() {
        for args in [
            vec!["ods".into(), "--help".into()],
            vec!["ods".into(), "-h".into()],
            vec!["ods".into(), "help".into(), "--help".into()],
            vec!["ods".into(), "help".into(), "-h".into()],
            vec!["ods".into(), "help".into(), "help".into()],
            vec!["ods".into(), "--help".into(), "lint".into()],
            vec!["ods".into(), "-h".into(), "serve".into()],
            vec!["ods".into(), "pack".into(), "help".into()],
            vec!["ods".into(), "profiles".into(), "help".into()],
            vec!["ods".into(), "tag".into(), "help".into()],
            vec!["ods".into(), "bench".into(), "help".into()],
            vec!["ods".into(), "agents".into(), "help".into()],
            vec!["ods".into(), "workspaces".into(), "help".into()],
            vec!["ods".into(), "lint".into(), "-h".into()],
        ] {
            let handled = try_print_cli_help(&args);
            assert!(
                handled.as_ref().is_some_and(|r| r.is_ok()),
                "expected help for {args:?}"
            );
        }

        let unknown_cmd_help =
            try_print_cli_help(&["ods".into(), "nope-xyz".into(), "--help".into()]);
        assert!(unknown_cmd_help.is_none());

        assert!(try_print_cli_help(&["ods".into(), "version".into()]).is_none());
    }

    #[test]
    fn command_help_guards_print_and_exit() {
        for args in [
            vec!["ods".into(), "clean".into(), "--help".into()],
            vec!["ods".into(), "completion".into(), "-h".into()],
            vec!["ods".into(), "diff".into(), "--help".into()],
            vec!["ods".into(), "fmt".into(), "--help".into()],
            vec!["ods".into(), "adopt".into(), "--help".into()],
            vec!["ods".into(), "init".into(), "--help".into()],
            vec!["ods".into(), "tags".into(), "--help".into()],
            vec!["ods".into(), "coverage".into(), "--help".into()],
            vec!["ods".into(), "stats".into(), "--help".into()],
            vec!["ods".into(), "tree".into(), "--help".into()],
            vec!["ods".into(), "share".into(), "--help".into()],
            vec!["ods".into(), "doctor".into(), "--help".into()],
            vec!["ods".into(), "sync".into(), "--help".into()],
            vec!["ods".into(), "watch".into(), "--help".into()],
            vec!["ods".into(), "serve".into(), "--help".into()],
            vec!["ods".into(), "export".into(), "--help".into()],
            vec!["ods".into(), "start".into(), "--help".into()],
            vec!["ods".into(), "stop".into(), "--help".into()],
            vec!["ods".into(), "logs".into(), "--help".into()],
            vec!["ods".into(), "new".into(), "--help".into()],
            vec!["ods".into(), "rm".into(), "--help".into()],
            vec!["ods".into(), "archive".into(), "--help".into()],
            vec!["ods".into(), "lsp".into(), "--help".into()],
            vec!["ods".into(), "upgrade".into(), "--help".into()],
        ] {
            let handled = try_print_cli_help(&args);
            assert!(
                handled.as_ref().is_some_and(|r| r.is_ok()),
                "expected help for {args:?}"
            );
        }

        assert!(run_clean_command(&["ods".into(), "clean".into(), "--help".into()]).is_ok());
        assert!(
            run_completion_command(&["ods".into(), "completion".into(), "--help".into()]).is_ok()
        );
        assert!(run_diff_command(&["ods".into(), "diff".into(), "--help".into()]).is_ok());
        assert!(run_fmt_command(&["ods".into(), "fmt".into(), "--help".into()]).is_ok());
        assert!(run_adopt_command(&["ods".into(), "adopt".into(), "--help".into()]).is_ok());
        assert!(run_init_command(&["ods".into(), "init".into(), "--help".into()]).is_ok());
        assert!(run_tags_command(&["ods".into(), "tags".into(), "--help".into()]).is_ok());
        assert!(run_coverage_command(&["ods".into(), "coverage".into(), "--help".into()]).is_ok());
        assert!(run_stats_command(&["ods".into(), "stats".into(), "--help".into()]).is_ok());
        assert!(run_tree_command(&["ods".into(), "tree".into(), "--help".into()]).is_ok());
        assert!(run_share_command(&["ods".into(), "share".into(), "--help".into()]).is_ok());
        assert!(run_doctor_command(&["ods".into(), "doctor".into(), "--help".into()]).is_ok());
        assert!(run_sync_command(&["ods".into(), "sync".into(), "--help".into()]).is_ok());
        assert!(run_watch_command(&["ods".into(), "watch".into(), "--help".into()]).is_ok());
        assert!(run_serve_command(&["ods".into(), "serve".into(), "--help".into()]).is_ok());
        assert!(run_export_command(&["ods".into(), "export".into(), "--help".into()]).is_ok());
        assert!(run_start_command(&["ods".into(), "start".into(), "--help".into()]).is_ok());
        assert!(run_stop_command(&["ods".into(), "stop".into(), "--help".into()]).is_ok());
        assert!(run_logs_command(&["ods".into(), "logs".into(), "--help".into()]).is_ok());
        assert!(run_new_command(&["ods".into(), "new".into(), "--help".into()]).is_ok());
        assert!(run_rm_command(&["ods".into(), "rm".into(), "--help".into()]).is_ok());
        assert!(run_archive_command(&["ods".into(), "archive".into(), "--help".into()]).is_ok());
        assert!(run_lsp_command(&["ods".into(), "lsp".into(), "--help".into()]).is_ok());
        assert!(run_upgrade_command(&["ods".into(), "upgrade".into(), "--help".into()]).is_ok());
        assert!(run_update_command(&["ods".into(), "update".into(), "--help".into()]).is_ok());
        assert!(
            run_profile_list_command(&["ods".into(), "profile".into(), "--help".into()]).is_ok()
        );
        assert!(run_profile_init_command(&[
            "ods".into(),
            "profile".into(),
            "init".into(),
            "--help".into()
        ])
        .is_ok());
        assert!(run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "--help".into()
        ])
        .is_ok());
        assert!(run_mv_command(&["ods".into(), "mv".into(), "--help".into()]).is_ok());
        assert!(run_read_command(&["ods".into(), "read".into(), "--help".into()]).is_ok());
        assert!(run_schema_command(&["ods".into(), "schema".into(), "--help".into()]).is_ok());
        assert!(run_undo_command(&["ods".into(), "undo".into(), "--help".into()]).is_ok());
        assert!(run_status_command(&["ods".into(), "status".into(), "--help".into()]).is_ok());
        assert!(run_ods_audit_command(&["ods".into(), "audit".into(), "--help".into()]).is_ok());
        assert!(run_pack_command(&["ods".into(), "pack".into(), "--help".into()]).is_ok());
        assert!(run_bench_command(&["ods".into(), "bench".into(), "--help".into()]).is_ok());
        assert!(run_setup_command(&["ods".into(), "setup".into(), "--help".into()]).is_ok());
        assert!(run_skill_command(&["ods".into(), "skill".into(), "--help".into()]).is_ok());
        assert!(dispatch_agents_command(&["ods".into(), "agents".into(), "--help".into()]).is_ok());
        assert!(
            run_workspaces_command(&["ods".into(), "workspaces".into(), "--help".into()]).is_ok()
        );
    }
}
