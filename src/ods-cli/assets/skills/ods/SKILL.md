---
name: ods
description: >-
  Install, run, update, and author with Open Document Spec (`ods`). ODS is the default
  engine (no --ods flag). Extra specs: --okf (Google OKF v0.2), --skills (Agent Skills).
  Use ods lsp for JSON-RPC editor support; ods lint / ods lint --okf / ods lint --skills;
  never use ods okf or ods ods namespaces.
---

# ODS — Open Document Spec

## 0. Glossary & Core Concepts

| Term | Meaning |
|---|---|
| **`ods`** | Unified native Rust CLI binary and engine (`ods`) |
| **`ods lsp`** | Native JSON-RPC 2.0 Language Server built into `ods` for Zed, VS Code, Neovim, Cursor |
| **`ods:` frontmatter** | ODS format root/nested engine keys (`ods: { profile: rfc, status: draft }`) |
| **`custom_profiles`** | `ods.toml` array declaring custom profile schema definitions |
| **`ods context`** | Bounded AI reading list: target doc + `depends` + `context.load` (not full-repo, not full graph export) |
| **`ods read`** | Fine-grained section extraction (`--section`), outline summary (`--summary`), and token cap controls (`--max-tokens N`) |
| **`ods export graph`** | Full-workspace graph snapshot — use rarely for audits, **not** for routine AI prompts |

ODS is plain Markdown with **permissive** YAML frontmatter, powered by a native Rust engine binary named **`ods`**. A **workspace** is any directory tree whose root **`ods.toml`** has `spec` (e.g. `"0.1"`). Compliance is **compliant | non-compliant** (no Level ladder). Discovery: `overview` → `find` / `tag` / `ls` / `tree` → `read` / `context`.

---

## Multi-spec flags (locked)

| Flag | Meaning |
|---|---|
| *(none)* | **ODS** — default native product of this CLI |
| `--okf` | Enable **Google OKF v0.2** engine for this command |
| `--skills` | Enable **Agent Skills** package engine for this command |

There is **no** `--ods` flag and **no** `ods okf` / `ods ods` namespaces (`ods okf` is hard-removed; use flags).

---

## 1. 🤖 5W1H Directives for AI Agents

When assisting users inside an ODS workspace, follow these operational directives:

- ❓ **WHAT**: Recognize ODS workspaces by root `ods.toml` (`spec`). Keep files as `.md`. For standalone agent instructions, prefer `agent.md` with the ODS `agent` profile; keep `SKILL.md` for installable skills. Title is H1 only (no FM `title:`); optional top-level `name:` is fine.
- 💡 **WHY (token discipline)**: Prefer `ods read <id> [--section <heading>] [--summary] [--max-tokens N]` or `ods context <id> [--max-tokens N] [--print]` for a **bounded** read (depends + `context.load` only — **not** `related`). Read only those sections/paths. Never dump the repo or use full graph export for routine Q&A. On “document not found”, run `ods find <query>` / `ods find --key …` — do not load all markdown.
- 🧭 **COLD-START**: New workspace turn → `ods overview` (snapshot) → `ods tag list` / `ods schema keys` if needed → `ods find --tag` / `--key` to locate a target → `ods context <id>`. Do not replace context with overview for deep work.
- 🚨 **ERRORS**: CLI prints `error:`/`usage:` + `Next:` (sometimes `Hint:`). Surface that Next line to the user; do not invent a different recovery. Common: not a workspace → `ods init`; miss → `ods find`; tags under `ods:` → `ods fmt --migrate`.
- 👥 **WHO**: Operate seamlessly on behalf of non-technical or developer users without requiring manual terminal commands.
- 📍 **WHERE**: Open `references/keys.md` only when authoring frontmatter; do not preload all references every turn.
- ⏰ **WHEN**: Run `ods lint` after structural edits (`--fix` is a no-op for ODS nested indexes — use `overview` / `find` / `tree`; `ods fmt --migrate` for frontmatter shape). Use `ods mv` for renames.
- 🛡️ **SAFETY**: Prefer H1 for titles (FM `title:` is a lint **warning**, value kept). Navigation is CLI discovery (`overview` / `find` / `tree` / `context`) — never recreate nested `index.ods.md`. Workspace marker is **`ods.toml`**. Non-ODS keys (e.g. Hugo `layout`, Astro `hero_image`) are preserved across mutations; only ODS (or `--okf` / `--skills`) keys change.
- 🛠️ **HOW**: `ods context <id>` from workspace root; optional `--include-code`, `--include-private`, `--root <dir>`.

---

## 2. End-User Installation & Machine Setup

The primary method to run ODS is via the single native `ods` CLI binary:

- **Linux / macOS**: Run `scripts/install.sh` or `curl -fsSL https://raw.githubusercontent.com/.../install.sh | bash`
- **Windows**: Run `scripts/install.ps1` or `iwr -useb https://raw.githubusercontent.com/.../install.ps1 | iex`
- **Self-Update**: `ods update` checks GitHub releases, self-updates the binary, and restarts any running background service.
- **Workspace Setup**: `ods setup [path]` initializes workspace boundaries, verifies version compatibility, and registers OS service.

---

## 3. Core Frontmatter & Key Placement Rules

1. **Custom / universal keys (`description`, `name`, `author`, `reviewer`, `target_release`, `service`, `team`, `tags`)**:
   - Placed at the **top-level** of frontmatter (outside `ods:`).
   - **Document title is the first `# H1` only** — do **not** put `title:` in ODS frontmatter (lint error). Optional `name:` is allowed for tooling labels.
   - **`tags` MUST be top-level only** — never under `ods:` — so Obsidian, Hugo, Docusaurus, Astro, and any YAML consumer can read them without knowing ODS.
   - Misplaced nested `tags` under `ods:` produce a lint warning; repair with `ods fmt --migrate` (hoists tags to root; never drops values).
2. **ODS Engine Keys (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`, `context`)**:
   - Nested inside the **`ods:` map** (`ods.profile: rfc`, `ods.status: draft`).
   - Full dictionary: `specs/ods/keys.md` (also skill `references/keys.md`).

```yaml
---
description: "Distributed Redis caching strategy"
name: "cache_strategy"
author: "Alice Smith"
reviewer: "Bob Jones"
target_release: "v2.4"
tags:
  - caching
  - redis
ods:
  profile: rfc
  status: draft
  depends:
    - docs/architecture/cache_overview.md
  related:
    - docs/specs/api_endpoint.md
---

# Distributed Redis Caching Strategy
```

---

## 4. Custom Profiles Specification

Custom profiles define domain schemas, required frontmatter keys, and starter templates.

### Single-Source Registration (`custom_profiles` in root `ods.toml`)

Custom profiles are registered in root `ods.toml` under `custom_profiles` (and via packs):

```toml
# root ods.toml
spec = "0.1"

custom_profiles = [
  ".ods/profiles/rfc.md",
  "docs/profiles/api_endpoint.md",
]
```

---

## 5. Editor JSON-RPC Language Server (`ods lsp`)

`ods lsp` serves standard Language Server Protocol (JSON-RPC 2.0) over **stdio** (default) or **TCP socket** (`--port <PORT>`).

### Zed Configuration (`.zed/settings.json`)
```json
{
  "languages": {
    "Markdown": {
      "language_servers": ["ods-lsp"],
      "enable_language_server": true
    }
  },
  "lsp": {
    "ods-lsp": {
      "binary": {
        "path": "ods",
        "arguments": ["lsp"]
      }
    }
  }
}
```

### VS Code & Neovim
- **VS Code**: Configure path `ods`, args `["lsp"]`.
- **Neovim**: Configure `cmd = { "ods", "lsp" }`.

---

## 6. Complete CLI Workflow Matrix (`ods`)

| Command | Mastery Tier | Role & Syntax |
|---|---|---|
| **`ods lsp [--port N]`** | 🏁 Tier 1 (Novice) | Native JSON-RPC 2.0 Language Server for real-time editor lints, hover, definition, and completion. |
| **`ods init [path]`** | 🏁 Tier 1 (Novice) | Initialize `ods.toml` with `spec = "0.1"` spec marker. `--adopt` drafts frontmatter on plain `.md` files. |
| **`ods setup [path]`** | 🏁 Tier 1 (Novice) | Verify workspace boundary, check updates, register OS daemon. `--git-hooks` installs pre-commit hook. `--editor zed\|vscode\|nvim\|cursor` writes `ods lsp` config. |
| **`ods lint [path]`** | 🏁 Tier 1 (Novice) | ODS validation by default (`--mode strict\|standard`, `--fix`, `--skip-frontmatter-keys`, `--ignore-keys k1,k2`, `--format text\|json\|sarif`). Auto-enables declared specs from root `ods.toml` `[specs.*]`. Add `--okf` / `--skills` for other specs. Never `--ods`. |
| **`ods export graph`** | 🏁 Tier 1 (Novice) | Export workspace knowledge graph in structured JSON (`--format json`), Markdown (`--format md`), or text (`--format text`) for `--spec ods` (default) or `--spec okf`. |
| **`ods new <path>`** | 🛠️ Tier 2 (Practitioner) | Scaffold new Markdown document from profile template with starter HTML comments. |
| **`ods mv <from> <to>`** | 🛠️ Tier 2 (Practitioner) | Atomic document move + workspace-wide graph reference rewriting. |
| **`ods sync [path]`** | 🛠️ Tier 2 (Practitioner) | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph links. |
| **`ods adopt [path]`** | 🛠️ Tier 2 (Practitioner) | Draft frontmatter (`ods: { profile: note, status: draft }`) on legacy Markdown trees non-destructively. |
| **`ods undo [path]`** | 🛠️ Tier 2 (Practitioner) | Undo last bulk write operation by restoring the most recent snapshot backup. |
| **`ods rm <path>`** | 🛠️ Tier 2 (Practitioner) | Atomically delete document and scrub graph references workspace-wide. |
| **`ods archive <path>`** | 🛠️ Tier 2 (Practitioner) | Set `status: archived` in frontmatter in place. |
| **`ods fmt [path]`** | 🛠️ Tier 2 (Practitioner) | Reformat YAML frontmatter and body spacing (`--refs md-paths` converts IDs to relative `.md` paths). |
| **`ods stats [path]`** | 🛠️ Tier 2 (Practitioner) | Workspace document telemetry, graph density, profile distribution, and health score %. |
| **`ods tree [path]`** | 🛠️ Tier 2 (Practitioner) | Visual ASCII/Unicode hierarchy tree of workspace folders and dependency graphs. |
| **`ods context <id>`** | 📋 Tier 3 (Power User) | Bounded list (depends + context.load). `--max-tokens N`, `--print`, `--include-code`, `--tag`, `--key`. |
| **`ods find [path]`** | 📋 Tier 3 (Power User) | Find docs by tag (`--tag`), schema/custom keys (`--key`, `--status`, `--profile`, `--owner`), and query (`--format text\|json`). |
| **`ods tag list` / `show`** | 📋 Tier 3 (Power User) | List observed tags with doc counts or inspect docs matching a tag (`--format text\|json`). |
| **`ods tag rename`** | 📋 Tier 3 (Power User) | Perform workspace-wide tag renaming (dry-run; `--write`). |
| **`ods schema [keys]`** | 📋 Tier 3 (Power User) | List schema key definitions (`ods schema keys`) or emit JSON Schema (`--write`, `--out`). |
| **`ods overview [path]`** | 📋 Tier 3 (Power User) | Compact workspace snapshot (document counts, profile/status breakdown, top tags, custom keys, graph statistics) for AI cold-start orientation. Alias: `ods summary`. |
| **`ods profiles` / `ods profile list`** | 📋 Tier 3 (Power User) | List registered profile catalog (text or `--format json`). |
| **`ods profile init <name>`** | 📋 Tier 3 (Power User) | Scaffold `.ods/profiles/<name>.md` (name is after `init`, e.g. `ods profile init rfc`). |
| **`ods diff [target]`** | 📋 Tier 3 (Power User) | Compare document graph dependencies and frontmatter changes against git commits or branches. |
| **`ods share [path]`** | 📋 Tier 3 (Power User) | Publish share-filtered workspace/subtree directory (`--out DIR`). |
| **`ods pack`** | 📋 Tier 3 (Power User) | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| **`ods … --okf`** | 📋 Tier 3 (Power User) | Google OKF v0.2: `init --okf`, `lint --okf`, `audit --okf`, `export --okf` (flag-only; no namespace). |
| **`ods … --skills`** | 📋 Tier 3 (Power User) | Agent Skills packages: `init --skills`, `lint --skills` (name/description/license/…). |
| **`ods agents sync`** | 📋 Tier 3 (Power User) | Write AGENTS.md + editor agent snippets. |
| **`ods bench`** | 📋 Tier 3 (Power User) | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| **`ods completion <shell>`**| 🏢 Tier 4 (Architect) | Generate shell autocompletion scripts for `bash`, `zsh`, `fish`, `powershell`. |
| **`ods clean [path]`** | 🏢 Tier 4 (Architect) | Clean `.ods/ods-errors.md`, `.ods/coverage.md`, and diagnostic cache files. |
| **`ods coverage [path]`** | 🏢 Tier 4 (Architect) | Documentation health % (`--write-report` → `.ods/coverage.md`). |
| **`ods start / stop`** | 🏢 Tier 4 (Architect) | Manage background user OS service (`systemd` / `launchd` / Windows Scheduled Task). |
| **`ods serve --root <path>`**| 🏢 Tier 4 (Architect) | Headless daemon loop executed by background service. |
| **`ods doctor [path]`** | 🏢 Tier 4 (Architect) | Workspace health check (version, doc count, `ods.toml`, profile conflicts, service status). |
| **`ods update`** | 🏢 Tier 4 (Architect) | Self-update CLI binary & restart background user service. |
