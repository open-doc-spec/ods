// Open Document Spec (ods) primary binary entrypoint
// Layout under main/: cli/ (dispatch) · commands/ (user commands) · support/ (helpers)
#![forbid(unsafe_code)]

mod service;
mod update;

// --- cli: entry + argv + exit ---
include!("main/cli/entry.rs");
include!("main/cli/help.rs");
include!("main/cli/cli_arg_parser.rs");
include!("main/cli/exit_code_helper.rs");

// --- commands (user-facing) ---
include!("main/commands/okf/okf_commands.rs");
include!("main/commands/upgrade_command.rs");
include!("main/commands/document/lint_and_index_commands.rs");
include!("main/commands/find_command.rs");
include!("main/commands/tag_command.rs");
include!("main/commands/lifecycle/context_graph_mv_commands.rs");
include!("main/commands/document/fmt_command.rs");
include!("main/commands/document/adopt_and_init_commands.rs");
include!("main/commands/lifecycle/lifecycle_commands.rs");
include!("main/commands/lifecycle/disable_command.rs");
include!("main/commands/profile/profile_commands.rs");
include!("main/commands/service/service_commands.rs");
include!("main/commands/lsp_command.rs");
include!("main/commands/setup_command.rs");
include!("main/commands/skill_command.rs");
include!("main/commands/update_command.rs");
include!("main/commands/service/watch_and_serve_runner.rs");
include!("main/commands/workspace/workspaces/workspaces_command.rs");
include!("main/commands/workspace/pack/pack_command.rs");
include!("main/commands/share_command.rs");
include!("main/commands/bench_command.rs");
include!("main/commands/stats_command.rs");
include!("main/commands/overview_command.rs");
include!("main/commands/completion_command.rs");
include!("main/commands/schema_command.rs");
include!("main/commands/tree_command.rs");
include!("main/commands/diff_command.rs");
include!("main/commands/clean_command.rs");
include!("main/commands/read_command.rs");
include!("main/commands/undo_command.rs");

// --- support (formatters, loaders, helpers) ---
include!("main/support/diagnostics_formatter.rs");
include!("main/support/doctor_reporter.rs");
include!("main/support/git_sync.rs");
include!("main/support/path_change_reporter.rs");
include!("main/support/workspace_light_loader.rs");
include!("main/support/process_memory.rs");
include!("main/support/graph_formatter.rs");
include!("main/support/alias_printer.rs");
