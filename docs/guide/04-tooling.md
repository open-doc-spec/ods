---
description: "CLI command matrix, service daemon vs watch comparison, CI integration, and updates."
ods:
  profile: "note"
  status: "stable"
---

# Tooling

Reference implementation: the single native **`ods` CLI**. ODS is the default engine (no `--ods` flag). Extra specs use flags only: `--okf`, `--skills`. Editor support is built in via **`ods lsp`** (JSON-RPC over stdio; not the same as `ods serve`).

---

## Production Checklist

| Check | Action |
| --- | --- |
| Version | `ods --version` |
| First-run setup | `ods setup` |
| Workspace | Root `ods.toml` with spec `spec = "0.1"` and CLI requirement `ods: ">=0.0.1"` |
| Local clean | `ods lint` |
| CI | `ods lint` + `ods lint` |
| Automation | `ods start` (background) or `ods watch` (foreground) |
| AI dump (optional) | `ods export` → `graph.md` |
| Doctor | `ods doctor` |
| Updates | `ods update` |

Happy path: [Quickstart Guide](/docs/quickstart).

---

## Commands Matrix (Novice to Expert)

| Command | Mastery Tier | Role & Syntax |
| --- | --- | --- |
| `ods init [path]` | 🏁 **Tier 1: Novice** | Make folder/repo ODS-compliant (writes root `ods.toml`). `--adopt` drafts frontmatter. No nested indexes. |
| `ods setup [path]` | 🏁 **Tier 1: Novice** | Set up machine background service for workspace, check updates, and run `ods doctor`. `--git-hooks` installs pre-commit hook. `--editor zed\|vscode\|nvim\|cursor` writes `ods lsp` config. |
| `ods lsp` | 🏁 **Tier 1: Novice** | JSON-RPC Language Server (stdio / `--port`); not the same as `ods serve`. |
| `ods lint` / `ods lint [path]` | 🏁 **Tier 1: Novice** | Validate graph & schemas (`ods lint\|3`, `--format text\|json\|sarif`, `--canonical-refs`). Generates or clears `.ods/ods-errors.md`. |
| `ods new <path>` | 🛠️ **Tier 2: Practitioner** | Scaffold a new Markdown document with inferred profile (`guide`, `feature`, etc.) and valid frontmatter. |
| `ods mv <from> <to>` | 🛠️ **Tier 2: Practitioner** | Offline document move + rewrite graph references workspace-wide. |
| `ods sync [path]` | 🛠️ **Tier 2: Practitioner** | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph references. |
| `ods adopt [path]` | 🛠️ **Tier 2: Practitioner** | Draft frontmatter for existing Markdown files (dry-run; `--write`). |
| `ods rm <path-or-id>` | 🛠️ **Tier 2: Practitioner** | Atomically delete document and scrub graph references (`depends`/`related`) workspace-wide. Alias: `ods remove`. |
| `ods status <path> <value>` | 🛠️ **Tier 2: Practitioner** | Set lifecycle status (`draft` \| `stable` \| `deprecated` \| `archived`). Writes nested `ods.status` when an `ods:` map exists. |
| `ods archive <path-or-id>` | 🛠️ **Tier 2: Practitioner** | Alias for `ods status <path> archived`. |
| `ods fmt [path]` | 🛠️ **Tier 2: Practitioner** | Reformat YAML frontmatter/body blank-line spacing. `--refs md-paths` converts extensionless IDs to relative `.md` paths. **`--migrate`** rewrites engine keys under `ods:`, hoists misplaced nested `tags`, and **preserves non-ODS keys**. |
| `ods stats [path]` | 🛠️ **Tier 2: Practitioner** | Display workspace document telemetry, graph density, profile distribution, and health score (`--format text\|json`). |
| `ods tree [path]` | 🛠️ **Tier 2: Practitioner** | Display visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs (`--format text\|json`). |
| `ods context [path] <id>` | 📋 **Tier 3: Power User** | Bounded AI reading list. Walks `depends` + `context.load`; `--include-related` / `--include-code` / `--include-private`; `--explain`; `--okf`. Without id: unique `--tag` / `--key` / `--status` only (multi-match → use `ods find`). |
| `ods read [root] <id-or-path>` | 📋 **Tier 3: Power User** | Fine-grained section extraction (`--section <heading>`), outline summary (`--summary`), and token cap controls (`--max-tokens N`, `--format text\|json`). |
| `ods undo [path]` | 📋 **Tier 3: Power User** | Restore latest frontmatter snapshot (`--list` shows ids under `~/.ods/backups/…`). Created mainly by `ods bench strip --write`; not a full git undo. |
| `ods profiles [path]` | 📋 **Tier 3: Power User** | List standard and custom profiles loaded in workspace and report schema conflicts. |
| `ods profile init <name>` | 📋 **Tier 3: Power User** | Scaffold `.ods/profiles/<name>.md` and **register** under root `custom_profiles (ods.toml):` (use `--no-register` to skip). |
| `ods profile show <name>` | 📋 **Tier 3: Power User** | Show profile layer, source, sections, and required/optional/forbidden keys. |
| `ods aliases` / `ods alias add` | 📋 **Tier 3: Power User** | List or add **section-heading** aliases in root `ods.toml` `[aliases]`. |
| `ods tags [path]` | 📋 **Tier 3: Power User** | List **top-level** document tags with counts (`--all` includes default unused tags). Tags must not live under `ods:`. |
| `ods tag list` / `ods tag show <tag>` | 📋 **Tier 3: Power User** | Observed tags with counts or docs for one tag (`--format text\|json`). Complements `ods tags` (which can include unused builtins via `--all`). |
| `ods tag rename <old> <new>` | 📋 **Tier 3: Power User** | Workspace-wide top-level tag rename (dry-run; `--write`). |
| `ods find [path]` | 📋 **Tier 3: Power User** | Find by tag (`--tag`, `--tag-match any\|all`), schema/custom keys (`--key`, exact match; `--status`/`--profile`/`--owner`), and/or ID/path query. |
| `ods schema [keys]` | 📋 **Tier 3: Power User** | List schema key definitions (`ods schema keys`) or export JSON Schema (`ods.schema.json`; `--write`, `--out PATH`). |
| `ods overview [path]` | 📋 **Tier 3: Power User** | AI cold-start snapshot (counts, profile/status, top tags, custom keys). Alias: `summary`. For lint health % use `ods stats`. |
| `ods diff [target]` | 📋 **Tier 3: Power User** | Compare document graph dependencies and frontmatter changes against git commits or branches (`--format text\|json`). |
| `ods graph [path]` | 📋 **Tier 3: Power User** | Print `depends`/`related` edges as `path -> edge` lines. |
| `ods export [path]` | 📋 **Tier 3: Power User** | Export single-file Markdown graph snapshot (`--out PATH`, `--include-private`). |
| `ods share [path]` | 📋 **Tier 3: Power User** | Publish share-filtered workspace/subtree directory (`--out DIR`, `--include-org`, `--include-private`). |
| `ods pack <subcommand>` | 📋 **Tier 3: Power User** | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| `ods bench <subcommand>` | 📋 **Tier 3: Power User** | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| `ods completion <shell>` | 🏢 **Tier 4: Enterprise Architect** | Generate shell autocompletion scripts (`bash`, `zsh`, `fish`, `powershell`). |
| `ods clean [path]` | 🏢 **Tier 4: Enterprise Architect** | Clean `.ods/ods-errors.md`, `.ods/coverage.md`, and diagnostic cache files. |
| `ods coverage [path]` | 🏢 **Tier 4: Enterprise Architect** | Documentation health % (`--write-report` → `.ods/coverage.md`; separate from lint `.ods/ods-errors.md`). |
| `ods start [path]` | 🏢 **Tier 4: Enterprise Architect** | Register + start **user OS service** (`systemd` / `launchd` / Windows Scheduled Task). `--status` checks status. |
| `ods stop [path]` | 🏢 **Tier 4: Enterprise Architect** | Stop running OS service. `--unregister` stops and removes registration completely. |
| `ods watch [path]` | 🏢 **Tier 4: Enterprise Architect** | Foreground live rename map + re-lint terminal loop. |
| `ods logs [path] [-f]` | 🏢 **Tier 4: Enterprise Architect** | Show background service logs (`~/.ods/logs/ods-serve.log`); `-f` follows. |
| `ods serve --root <path>` | 🏢 **Tier 4: Enterprise Architect** | Headless daemon loop executed by background service (`--mode auto\|watch\|poll`). |
| `ods workspaces <subcommand>` | 🏢 **Tier 4: Enterprise Architect** | Manage globally tracked ODS workspaces in `~/.ods/odsconfig.toml` (`add`, `remove`, `list`, `path`). |
| `ods disable [path]` | 🏢 **Tier 4: Enterprise Architect** | Opt-out / strip ODS metadata (dry-run; `--write`, `--keep-frontmatter`, `--remove-indexes`). Alias: `ods revert`. |
| `ods doctor [path]` | 🏢 **Tier 4: Enterprise Architect** | Workspace health check (version, doc count, `ods.toml`, profile conflicts, service status). |
| `ods update` | 🏢 **Tier 4: Enterprise Architect** | Self-update CLI binary & restart background user service (`--check`, `--force`, `--version <tag>`). |

---

### `ods serve` vs. `ods watch` Comparison

| Dimension | `ods serve` (Background OS Service) | `ods watch` (Foreground Terminal Watcher) |
| :--- | :--- | :--- |
| **Execution Architecture** | Headless OS Daemon (systemd user unit, launchd agent, Windows Scheduled Task). Registered via `ods start` / `ods setup`. | Interactive Foreground Process running in an open terminal tab (`ods watch .`). |
| **User Visibility** | Invisible ("Set and Forget"). Zero terminal output. | Displays live event logs, rename mappings, and lint warnings in stdout. |
| **Workspace Scope** | Global machine service tracking registered workspace paths (`~/.ods/odsconfig.toml`). | Single workspace directory tree (defaults to `.`). |
| **Process Lifecycle** | Automatically starts on OS boot/login. Runs persistently in background. | Runs until terminal tab is closed or `Ctrl+C` is pressed. |
| **Extra Responsibilities** | Background auto-update check (~daily) and remote Git pack auto-sync (`ods pack sync`). | Dedicated filesystem event watching and instant re-linting. |
| **Ideal For** | Everyday background automation for devs and non-tech users. | Active refactoring sessions, debugging graph renames, containerized environments. |

---

## Code References

ODS documents can declare implementation locations with `code:`:

```yaml
code:
  - path: apps/web/src/routes/checkout.tsx
    symbol: CheckoutRoute
    role: entrypoint
  - path: apps/web/src/features/checkout/pricing.ts
    symbol: calculateCheckoutTotal
    role: implementation
  - path: apps/web/src/features/checkout/pricing.test.ts
    symbol: calculateCheckoutTotal
    role: test
```

`path` is required, `role` is required and fixed, and `symbol` is optional. `ods lint` validates paths at lint.

---

## CI Integration

In CI pipelines:

```bash
ods lint
ods lint --level 3
```

---

## Next

- [Profiles & Catalogs](/docs/profiles) · [Advanced Workspaces](/docs/advanced) · [FAQ](/docs/faq)


## Hybrid Workspaces & Declarative Multi-Spec Configuration

Bare `ods lint` runs **ODS only** by default. However, workspaces can declaratively enable extra specs (`okf`, `skills`) directly in the root `ods.toml`:

```toml
# root ods.toml
spec = "0.1"

[specs.okf]
enabled = true
check_keys = true
ignore_keys = ["runtime", "sources"]

[specs.skills]
enabled = true
check_keys = true
ignore_keys = []
```

When extra specs are set to `enabled: true` in root `ods.toml`, bare `ods lint` automatically validates those declared specs without requiring extra CLI flags. For full details on all supported root configuration options, see [ODS Features & Configuration Keys](/docs/guide/features.md#2-root-odstoml-configuration-keys).

### Imperative CLI Flags & Key Suppression

You can also enable extra specs or suppress frontmatter key requirements via CLI flags:
- Pass `--okf` or `--skills` to enable extra spec linting on the command line.
- Pass `--skip-frontmatter-keys` (or `--skip-keys`) to disable required frontmatter key validation during a lint run.
- Pass `--ignore-keys <key1,key2>` to ignore specific frontmatter keys.

Pure OKF trees can be linted with `ods lint --okf`; pure Agent Skills packages can be linted with `ods lint --skills`. Editors communicate via `ods lsp` (not `ods serve`).
