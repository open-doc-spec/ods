---
name: ods
description: >-
  Use this skill whenever working in an Open Document Spec (ODS) workspace (marked by `ods.toml`),
  or when performing documentation discovery, architecture analysis, PRD/ADR authoring, symbol-to-code
  navigation (`ods.code`), bounded AI context resolution (`ods context`), fine-grained section reading
  (`ods read`), or graph linting (`ods lint`). Activates on any coding, debugging, planning, or refactoring
  task to reduce token costs by ~95% using deterministic DAG loading (<5ms) instead of dumping whole repositories.
  ODS is the default native CLI engine (never use --ods). Supports extra specs: --okf (Google OKF v0.2)
  and --skills (Agent Skills).
tags:
  - skill
  - ods
  - documentation
  - ai-agent
owner: team:ods
ods:
  profile: skill
  status: stable
---

# ODS — Open Document Spec

## 0. Glossary & Core Concepts

| Term | Meaning |
|---|---|
| **`ods`** | Unified native Rust CLI binary and engine (`ods`) |
| **`ods lsp`** | Native JSON-RPC 2.0 Language Server built into `ods` for Zed, VS Code, Neovim, Cursor |
| **`ods:` frontmatter** | ODS format root/nested engine keys (`ods: { profile: rfc, status: draft }`) |
| **`custom_profiles`** | `ods.toml` array declaring custom profile schema definitions |
| **`ods overview`** | Compact ~100-token workspace snapshot (doc counts, profiles, status, top tags, graph density) |
| **`ods find --key`** | Precision search across frontmatter schema keys and custom keys (`--key-match and\|or`, `--status`) |
| **`ods read`** | Fine-grained section extraction (`--section <heading>`), summary outline (`--summary`), and soft token caps (`--max-tokens N`) |
| **`ods context`** | Bounded AI reading list: target doc + hard `depends` + `context.load` (not full-repo, not `related`) |
| **`ods export graph`** | Full-workspace graph snapshot — use rarely for audits, **not** for routine AI prompts |

ODS is plain Markdown with **permissive** YAML frontmatter, powered by a native Rust engine binary named **`ods`**. A **workspace** is any directory tree whose root **`ods.toml`** declares `spec = "0.1"`. Compliance is **compliant | non-compliant** (binary model; no Level ladder). Discovery follows a progressive CLI arc: `overview` → `find` / `tag` / `tree` → `read` / `context`.

---

## Multi-spec Flags (Locked)

| Flag | Meaning |
|---|---|
| *(none)* | **ODS** — default native product of this CLI |
| `--okf` | Enable **Google OKF v0.2** engine for this command |
| `--skills` | Enable **Agent Skills** package engine for this command |

There is **no** `--ods` flag and **no** `ods okf` / `ods ods` namespaces (`ods okf` is hard-removed; use flags).

---

## 1. 🎯 Activation Triggers for AI Coding Agents

An AI agent MUST activate and leverage this skill in the following scenarios:

1. **Workspace Contains `ods.toml`**: The workspace is an active ODS knowledge graph.
2. **Context Exploration & Task Planning**: Need to understand repository layout, architecture, or features before writing code without inflating prompt token costs.
3. **Locating Feature Logic**: Finding source code entrypoints, handlers, or tests tied to specifications (`ods.code`).
4. **Authoring Specifications**: Creating new PRDs (`profile: feature`), ADRs (`profile: decision`), Guides (`profile: guide`), Policies (`profile: policy`), Agent execution contracts (`profile: agent`), or Custom profiles.
5. **Renaming / Moving Documentation**: Moving files without breaking inbound markdown references (`ods mv`).
6. **Pre-Commit / CI Quality Verification**: Ensuring zero broken links, no circular dependencies, and valid frontmatter (`ods lint`).

---

## 2. ⚡ AI Goal-to-Command Quick Navigation Matrix

| AI Agent Goal | Recommended CLI Command | Token Impact / Advantage |
|---|---|---|
| **Cold-Start Workspace Orientation** | `ods overview` (alias: `ods summary`) | **~100 tokens** vs dumping whole folder hierarchies |
| **Find Specs by Tag or Status** | `ods find --tag <name>` or `ods find --key "status=stable"` | Exact matching without full-text grep scans |
| **Find Complex Multi-Key Targets** | `ods find --key "status=draft AND owner=alice"` | Boolean filtering across custom & schema frontmatter |
| **Extract Specific Heading Only** | `ods read <id> --section "Prerequisites"` | Slices **only the required section**; ignores the rest |
| **Inspect Heading Structure Outline** | `ods read <id> --summary` | Returns headings + line numbers without body prose |
| **Assemble Exact AI Prompt Pack** | `ods context <id> --print --max-tokens 4000` | Injects target + topological `depends` within budget |
| **Explain Dependency Chain** | `ods context <id> --explain` | Explains why each prerequisite document was loaded |
| **Include Implementation Code** | `ods context <id> --include-code` | Appends bound source code paths declared in `ods.code` |
| **Inspect Tag Catalog** | `ods tag list` / `ods tag show <tag>` | High-signal taxonomy view with document counts |
| **Inspect Schema Dictionary** | `ods schema keys` | Instant look up of key placement (`TopLevel` vs `Nested`) |
| **Scaffold New Document** | `ods new <path>` | Scaffolds valid frontmatter & required `##` sections |
| **Atomic Document Rename** | `ods mv <old> <new>` | Atomically rewrites all inbound references in workspace |
| **Verify Knowledge Graph Integrity** | `ods lint` | Validates DAG acyclicity, schema keys, and sections |
| **Migrate Misplaced Frontmatter** | `ods fmt --migrate` | Non-destructively hoists tags and normalizes 3-tier keys |

---

## 3. 🤖 AI Agent Directives & Token Cost Reduction Protocol

To minimize token consumption by up to ~95%, avoid context degradation, and eliminate hallucinations, AI coding agents MUST follow this systematic interaction protocol:

### The 5-Phase Token-Discipline Playbook

```
1. Cold Start ──────► ods overview              (Compact ~100 token snapshot)
2. Discovery ───────► ods find --key / --tag    (Targeted doc resolution)
3. Reading ─────────► ods read --section / cap  (Extract only necessary tokens)
4. Context DAG ─────► ods context <id>          (Strict hard depends + load)
5. Grounding ───────► Jump to ods.code symbols  (Direct implementation jumping)
```

1. **Cold Start (Turn 1)**:
   - Run `ods overview` to orient on workspace layout, document counts, and top tags.
   - Do NOT run full-repo file listings, recursive searches, or dump `ods export graph` into context.

2. **Target Discovery**:
   - Locate relevant documents using targeted queries:
     * By schema / custom key: `ods find --key "status=stable" --profile guide`
     * By tag: `ods tag show <tag>` or `ods find --tag <tag>`
     * By key schema registry: `ods schema keys`

3. **Fine-Grained Reading & Token Capping**:
   - Extract only the specific section needed: `ods read <id> --section "Prerequisites"`
   - Get a structural summary outline: `ods read <id> --summary`
   - Cap token ceiling: `ods read <id> --max-tokens 400 [--format json]`

4. **Bounded Knowledge Graph Expansion (`ods context`)**:
   - **List DAG targets**: `ods context <id>`
   - **Assemble AI Prompt Pack**: `ods context <id> --print --max-tokens 4000` (prints full file contents in topological DAG order under budget)
   - **Include Source Code**: `ods context <id> --include-code` (expands bound code files declared in `ods.code`)
   - **Explain Inclusion**: `ods context <id> --explain` (shows exact dependency reasoning)
   - Traverses **strictly** hard `depends` and `context.load` up to `ods.context.max-depth` (default 2 hops).
   - **`related` references are soft links and are NEVER auto-loaded into context** unless `--include-related` is passed.

5. **Direct Source Code Jumping (`ods.code`)**:
   - Jump directly to declared source symbols (`role: entrypoint`, `role: implementation`, `role: test`) without performing expensive codebase AST scans.

6. **Self-Healing & Quality Verification**:
   - After modifying documents, run `ods lint` to verify graph acyclicity and 3-tier key validity.
   - Use `ods mv <from> <to>` when renaming or moving files to atomically heal all inbound references across the workspace.

---

## 4. End-User Installation & Machine Setup

The primary method to run ODS is via the single native `ods` CLI binary:

- **Linux / macOS**: Run `scripts/install.sh` or `curl -fsSL https://raw.githubusercontent.com/open-doc-spec/ods/main/src/scripts/install.sh | bash`
- **Windows**: Run `scripts/install.ps1` or `iwr -useb https://raw.githubusercontent.com/open-doc-spec/ods/main/src/scripts/install.ps1 | iex`
- **Self-Update**: `ods update` checks GitHub releases, self-updates the binary, and restarts background services.
- **Skill Installation**: `ods skill install --agent <antigravity|claude-code|cursor|codex|gemini-cli>` installs the agent skill bundle or rule file.

---

## 5. GitHub Actions CI Gate Integration

Reference the official reusable workflow from `open-doc-spec/ods-action` on PRs and pushes:

```yaml
jobs:
  docs-quality-gate:
    name: ODS Document Integrity Gate
    uses: open-doc-spec/ods-action/.github/workflows/ods.yml@main
    with:
      runs-on: 'ubuntu-latest'
      path: '.'
      doctor: true
      strict-fmt: true
```

---

## 6. Core Frontmatter & 3-Tier Key Placement Rules

1. **Top-Level Universal Keys** (`description`, `tags`, `owner`, `author`, `reviewer`, `created`, `updated`, `name`):
   - Placed at the root level of YAML frontmatter (outside `ods:`).
   - **Document title is the first `# H1` Markdown prose line only** — do NOT put `title:` in frontmatter (lint warning).
   - **`tags` MUST be top-level only** (`tags: [auth, security]`) — never under `ods:` — ensuring universal SSG compatibility (Obsidian, Hugo, Astro, Docusaurus).

2. **Nested ODS Engine Keys (`ods:`)**:
   - Nested inside the **`ods:`** block (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`, `context`).

```yaml
---
description: "Distributed Redis session management"
owner: team:security
tags:
  - caching
  - auth
ods:
  profile: guide
  status: stable
  depends:
    - ../crypto/jwt-spec.md
  code:
    - path: src/auth/session.ts
      role: entrypoint
      symbol: createSession
    - path: tests/auth/session.test.ts
      role: test
  context:
    max-depth: 2
    load:
      - ../schemas/session-payload.json
---

# Session Management

## Overview
How session tokens are signed, validated, and revoked.
```

---

## 7. Standard Document Profiles & Execution Contracts

Standard document profiles define required `##` H2 heading structures:

| Profile | Primary Purpose | Expected Heading Sections |
|---|---|---|
| `note` | Free-form note / knowledge base (default) | *(none required)* |
| `guide` | Step-by-step how-to tutorial | Overview, Prerequisites, Steps, Troubleshooting |
| `feature` | PRD / feature specification | Goal, Scope, Requirements, Acceptance Criteria, Risks |
| `decision` | Architecture Decision Record (ADR) | Context, Decision, Alternatives, Consequences |
| `sop` | Standard operating procedure / runbook | Purpose, Prerequisites, Steps, Validation, Rollback |
| `api` | API endpoint / RPC contract | Overview, Request, Response, Errors, Examples |
| `architecture`| High-level architecture & system flow | Overview, Components, Data Flow, Trade-offs |
| `policy` | Governance & team rules | Purpose, Scope, Rules, Exceptions |
| `meeting` | Sync notes and action items | Attendees, Agenda, Decisions, Action Items |
| `checklist` | Verifiable deployment or release checklist | Overview, Items, Verification, Notes |
| `agent` | Autonomous agent task instruction (`agent.md`) | Goal, Task, Scope, Non-Scope, Context, Inputs, Constraints, Priority, Steps, Output, Success Criteria, Failure Modes, Dependencies, Assumptions, Examples |
| `skill` | Reusable agent tool skill contract (`SKILL.md`) | Purpose, Capability, Activation, Scope, Non-Scope, Inputs, Outputs, Workflow, Rules, Priority, Validation, Eval, Resources, Tools, Lifecycle, Traceability |

---

## 8. Custom Profiles & Key Policies

Custom profiles allow organizations to enforce custom section shapes and frontmatter key requirements.

### Registration in `ods.toml`
```toml
spec = "0.1"
custom_profiles = [".ods/profiles"]
```

### Profile Definition with Key Policies (`.ods/profiles/incident.md`)
```markdown
---
ods:
  custom_profile:
    name: incident
    required_keys:
      - service
      - tracker
    optional_keys:
      - owner
    forbidden_keys:
      - title
---

# Incident Profile

## Impact

## Root Cause

## Remediation
```

* **Nested structures**: Required keys support complex nested objects (e.g. `tracker: { provider: github, issue: 123 }`) and lists (`tags: []`).
* **Strict validation**: Explicit `null` values fail required key checks; forbidden keys produce lint diagnostics.

---

## 9. Complete CLI Workflow Matrix (`ods`)

| Command | Role & Syntax |
|---|---|
| **`ods lsp [--port N]`** | Native JSON-RPC 2.0 Language Server for real-time editor diagnostics, hover, definition, and completions. |
| **`ods init [path]`** | Initialize workspace root `ods.toml` (`spec = "0.1"`). Use `--adopt` to non-destructively draft frontmatter. |
| **`ods setup [path]`** | Verify workspace boundaries, check updates, install pre-commit hooks (`--git-hooks`), configure editor LSP (`--editor`). |
| **`ods lint [path]`** | Validate workspace knowledge graph, DAG acyclicity, profile sections, and frontmatter keys (`--fix`, `--format text\|json\|sarif`). |
| **`ods overview [path]`** | Compact workspace summary (document counts, profile/status breakdown, top tags, graph stats) for AI cold-starts. Alias: `ods summary`. |
| **`ods read <id>`** | Fine-grained section extraction (`--section <H2>`), outline summary (`--summary`), and soft token caps (`--max-tokens N`, `--format json\|text`). |
| **`ods context <id>`** | Bounded reading list (traversing hard `depends` + `context.load`). Options: `--max-tokens N`, `--print`, `--include-code`. |
| **`ods find [query]`** | Multi-criteria query engine (`--key "status=draft,stable"`, `--key-match and\|or`, `--tag <tag>`, `--profile <p>`, `--owner <o>`). |
| **`ods tag list` / `show`** | Inspect workspace tag taxonomy with document counts or filter documents matching a specific tag. |
| **`ods schema keys`** | Inspect registered frontmatter schema keys, placement (`TopLevel` vs `NestedEngineMap`), and types. |
| **`ods new <path>`** | Scaffold new document from profile template with starter HTML comments. |
| **`ods mv <from> <to>`** | Atomic document rename/move + workspace-wide reference and link rewriting. |
| **`ods fmt [path]`** | Reformat YAML frontmatter spacing and migrate misplaced keys (`--migrate`). |
| **`ods profile init <name>`** | Scaffold `.ods/profiles/<name>.md` and register it under `custom_profiles` in `ods.toml`. |
| **`ods profile show <name>`** | Inspect custom or standard profile layers, required sections, and key rules. |
| **`ods export graph`** | Export full workspace graph in Markdown, JSON, or text format. |
| **`ods stats [path]`** | Telemetry, graph density, profile distribution, and health score %. |
| **`ods tree [path]`** | Visual ASCII tree of workspace folders and dependency links. |
| **`ods status <id> <st>`** | Update document lifecycle status (`draft`, `stable`, `deprecated`, `archived`). |
| **`ods skill install`** | Install ODS skill configuration for AI agents (`--agent antigravity\|claude-code\|cursor\|codex\|gemini-cli`). |
| **`ods doctor [path]`** | Run workspace health check (version, `ods.toml`, profile conflicts, service status). |
| **`ods update`** | Self-update CLI binary to latest GitHub release. |
| **`ods … --okf`** | Google OKF v0.2 commands (`init --okf`, `lint --okf`, `export --okf`). |
| **`ods … --skills`** | Agent Skills package commands (`init --skills`, `lint --skills`). |
