---
ods:
  profile: note
  status: stable
---

# Open Document Spec (ODS)

[![CI](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/pr.yml/badge.svg)](https://github.com/StaytunedLLP/open-document-spec/actions/workflows/pr.yml)
[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-Open--Document--Spec-blue.svg?logo=github)](https://github.com/marketplace)
[![Compliance](https://img.shields.io/badge/ODS-Level--3-green.svg)](https://github.com/StaytunedLLP/open-document-spec)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

**Open Document Spec (ODS)** is a lightweight, human-first convention built on plain Markdown. It enriches Markdown repositories with machine-readable YAML frontmatter metadata, explicit graph relationships, and sub-5ms deterministic AI context loading—managed by the high-performance native **Open Document Spec CLI (`ods`)**.

Your files remain `.md` files forever.

### Related repositories (first-cut extract)

| Surface | Repository |
|---------|------------|
| Engine / CLI (this monorepo) | [open-doc-spec/ods](https://github.com/open-doc-spec/ods) |
| Normative specs | [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec) |
| End-user skill | [open-doc-spec/ods-skills](https://github.com/open-doc-spec/ods-skills) |
| GitHub Action | [open-doc-spec/ods-action](https://github.com/open-doc-spec/ods-action) |
| Benchmark fixtures | [open-doc-spec/ods-benchmarks](https://github.com/open-doc-spec/ods-benchmarks) |
| Site (private) | [open-doc-spec/opendocify.com](https://github.com/open-doc-spec/opendocify.com) |

In-tree `specs/`, `skills/ods/`, `app-web/`, `src/fixtures/benchmarks/`, and the composite action remain **mirrors** for first cut. Details: [`docs/maintainer/satellite-repos.md`](docs/maintainer/satellite-repos.md).

---

## 🧭 The 5W1H Framework

| Perspective | Summary & Details |
|---|---|
| ❓ **WHAT** | **A Graph-Based Markdown Specification Standard & Native Rust Engine**<br/>ODS adds lightweight, standardized YAML frontmatter metadata (`profile`, `status`, `depends`, `related`, `code`) to Markdown files, turning flat folders into an intelligent, validated, auto-healing document graph. |
| 💡 **WHY** | **Eliminate Documentation Drift & Reduce AI Token Costs by ~95%**<br/>Traditional docs rot as code evolves and flood AI prompts with irrelevant context. ODS computes deterministic, bounded reading lists (<5ms) to feed AI coding assistants (Antigravity, Claude, Cursor, ChatGPT) only what they need, saving up to ~95% in token costs. |
| 👥 **WHO** | **Cross-Functional Collaboration (Developers, Product Managers, Compliance & AI Agents)**<br/>Product Managers write PRDs, Legal/HR enforce policy section trees, Developers link source code symbols (`code:`), and AI Coding Assistants auto-index and validate workspaces without manual intervention. |
| 📍 **WHERE** | **Everywhere Markdown Lives (Git Repos, CI/CD, IDEs, AI Prompt Windows)**<br/>Runs locally in your terminal (`ods`), in your IDE via AI skills, in CI/CD via GitHub Marketplace Actions (`ods/action`), and outputs standard OASIS SARIF v2.1.0 reports for GitHub Security Code Scanning. |
| ⏰ **WHEN** | **From Day 1 Through Active Refactoring & Production Deployment**<br/>Use `ods init` on day 1, `ods new` for feature specs, `ods mv` to automatically heal graph links during refactoring, and `ods lint` on every pull request to guarantee zero broken links. |
| 🛠️ **HOW** | **Skill-First AI Automation & 4-Tier Novice-to-Expert Mastery Arc**<br/>Get started in 3 steps, or progress from complete beginner to enterprise architect using our 4-Tier Novice-to-Expert Mastery Path below. |

---

## 🎓 Novice-to-Expert Mastery Path

Progress from initial setup to enterprise documentation architecture across 4 structured learning tiers:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        ODS NOVICE-TO-EXPERT PROGRESSION ARC                            │
├───────────────────┬─────────────────────┬──────────────────────┬───────────────────────┤
│  TIER 1: NOVICE   │ TIER 2: PRACTITIONER│   TIER 3: POWER USER │ TIER 4: ARCHITECT     │
│  (Foundations)    │ (Day-to-Day Graph)  │   (Custom Catalogs)  │ (CI/CD & Governance)  │
├───────────────────┼─────────────────────┼──────────────────────┼───────────────────────┤
│ • Skill / Install │ • Graph (`depends`) │ • Custom Profiles    │ • Background Daemon   │
│ • `ods init`      │ • Code (`code:`)    │ • ODS Packs (`pack`) │ • GitHub Action CI    │
│ • Root `spec = "0.1"` │ • Auto-Index (`index`)│ • AI Scope (`context`)│ • SARIF Security     │
│ • `ods lint`      │ • Atomic `ods mv`   │ • Token ROI (`bench`)│ • Git Pre-Commit Hook │
└───────────────────┴─────────────────────┴──────────────────────┴───────────────────────┘
```

| Tier | Focus Area | Essential Commands & Keys | Learning Goal |
|---|---|---|---|
| **Tier 1: Novice** | Foundations & Validation | `ods init`<br/>`ods lint`<br/>`spec = "0.1"` | Install binary, initialize root `ods.toml`, write basic frontmatter (`profile`, `status`), run lint checks. |
| **Tier 2: Practitioner** | Document Graphs & Code Links | `depends:` / `related:`<br/>`code:`<br/>`ods overview` / `ods find`<br/>`ods mv` | Construct document dependency graphs, bind source code symbols, navigate the document graph, and perform atomic file moves. |
| **Tier 3: Power User** | Custom Schemas & AI Context | `custom_profiles (ods.toml):`<br/>`ods profile`<br/>`ods context`<br/>`ods find --key`<br/>`ods overview`<br/>`ods pack`<br/>`ods bench` | Register domain profile schemas, discover docs by key/tag, cold-start with workspace overview, export packs, and measure token ROI. |
| **Tier 4: Architect** | Automation & Enterprise Governance | `ods setup --git-hooks`<br/>`ods serve` / `ods start`<br/>`ods/action` (CI)<br/>`--format sarif`<br/>`ods lint --okf` | Run persistent OS background daemons, enforce CI pull request gates, output SARIF security annotations, and enable Google OKF via `ods lint --okf` (no namespaces; no `--ods` flag). |

---

## 🌟 Key Features & Capabilities

| Feature | Command / Key | Role & Description |
|---|---|---|
| ⚡ **Deterministic Bounded Context Graph** | `ods context <doc-id>` | Bounded AI reading scope (<5ms) following `depends:` + `context.load` (not full-repo dump). |
| 🔍 **Multi-criteria AI discovery** | `ods find` / `ods overview` / `ods schema keys` | Find by tag and frontmatter keys (`--key`, `--status`, …); cold-start snapshot; list registered schema keys. |
| 📋 **Custom Profile Schema Engine** | `ods profile` / `custom_profiles (ods.toml):` | Single-source profile schema registration in `ods.toml`, enforcing required/optional/forbidden keys and `H2`/`H3` section hierarchies. |
| 📊 **Workspace Document Telemetry** | `ods stats` | Reports document health score %, graph dependency density, profile distribution, and top taxonomy tags. |
| 🌳 **Visual Tree Representation** | `ods tree` | Displays visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs. |
| 🔄 **Smart Rename & Dependency Healing** | `ods mv <src> <dst>` | Renames files while automatically rewriting graph dependencies, relative body links, and code references. |
| 🔀 **Graph Change Diffing** | `ods diff [target]` | Compares document graph dependencies and frontmatter changes against git commits or branches. |
| 🪄 **Legacy Markdown Adoption** | `ods adopt <dir>` | Scans legacy non-ODS Markdown folders and automatically drafts frontmatter schemas (`status: draft`). |
| 🏷️ **Tag Taxonomy & Governance** | `ods tags` / `ods tag list\|show\|rename` | Observed tag catalog, per-tag doc list, and workspace-wide rename (dry-run; `--write`). |
| 📜 **JSON Schema Export** | `ods schema` / `ods schema keys` | Export JSON Schema for IDE autocomplete, or list registry key definitions. |
| 🛡️ **SARIF Security Reporting** | `ods lint --format sarif` | Outputs standard OASIS SARIF v2.1.0 format for GitHub Code Scanning / CI security integration. |
| ⚓ **Git Pre-Commit Hook Installer** | `ods setup --git-hooks` | Installs `.git/hooks/pre-commit` hook to catch broken links before commits. |
| 🧹 **Diagnostic & Report Cleaner** | `ods clean` | Cleans `.ods/ods-errors.md`, `.ods/coverage.md`, and diagnostic cache files. |
| 🌐 **Google OKF v0.2 Interoperability** | `ods lint --okf` / `ods init --okf` | Native OKF v0.2 in the same binary; **flag-only** (`--okf`). Agent Skills via `--skills`. |
| 🔒 **Secret & Pack Isolation** | `share: private` / `ods share` | Protects private secrets and exports sanitized public workspace packs (`ods pack`). |
| ⚙️ **Background OS Daemon & Watcher** | `ods setup` / `ods start` | FS watcher (not LSP). Editors use **`ods lsp`** (JSON-RPC). |
| 🤖 **Zero-Terminal AI Skill** | `skills/ods/SKILL.md` + `ods skill install` | Skill-first integration; `ods agents sync` for AGENTS.md. |
| 🚀 **GitHub Marketplace Action** | `uses: StaytunedLLP/open-document-spec@v1` | Automated CI linting and GitHub PR inline code annotations. |

---

## 👥 Who is ODS For?

ODS is built for cross-functional collaboration between technical and non-technical teams:

- 👩‍💼 **Product Managers & Business Owners**: Bind feature PRDs and acceptance criteria directly to code implementations and goals.
- ⚖️ **Legal, HR & Compliance**: Enforce mandatory policy section trees and isolate sensitive secrets with `share: private`.
- 🔬 **Researchers & Writers**: Build deterministic, machine-readable citation graphs across research papers and notes.
- 💻 **Developers & DevOps**: Connect technical specs to source code symbols with sub-5ms Rust graph traversal.

> **Zero Terminal Skills Needed**: Non-technical users can use ODS entirely through **AI Assistant Skills** (Antigravity, Claude, ChatGPT, Cursor). Simply tell your AI Assistant: _"Install ODS skill and index my documents folder."_

---

## 🚀 Quickstart in 3 Steps

### 1. Enable ODS (Primary: Skill-First for AI Assistants)

The recommended, zero-friction way to start with ODS is via the **ODS Skill** (`skills/ods/SKILL.md`) in your AI Coding Assistant.

Tell your AI Assistant: _"Install Open Document Spec skill"_. The skill detects your OS, downloads the native **`ods`** binary, registers the OS background service, and verifies workspace health:

```text
==> Open Document Spec is installed and running on your machine!
==> Version: ods v0.0.13
```

<details>
<summary><b>Optional: Manual Direct CLI Installation (Without AI Assistant)</b></summary>

**macOS / Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.sh | bash
```

**Windows (PowerShell)**:

```powershell
irm https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.ps1 | iex
```

</details>

### 2. Initialize & Start Service

```bash
mkdir my-docs && cd my-docs
ods init .      # Writes root ods.toml with spec = "0.1" spec marker
ods setup       # Verify workspace, check updates, and register background OS service
```

### 3. Scaffold, Validate & Track Health

```bash
ods profile init rfc               # Scaffold custom profile schema in .ods/profiles/rfc.md
ods profile list                   # List standard and registered custom profiles
ods new rfc docs/rfcs/001.md       # Scaffold new document from profile template
ods lint                           # Level-3 graph & section integrity check
ods stats                          # Workspace telemetry & Health score summary
ods tree                           # Display visual document hierarchy tree
ods context <doc-id-or-path>       # Bounded AI reading scope (<5ms)
ods mv <src.md> <dst.md>           # Rename file & auto-rewrite workspace deps
ods adopt docs/                    # Auto-draft frontmatter on legacy Markdown
```

---

## 📘 Custom Profiles Specification

Custom profiles define domain document schemas, profile key policies, and starter templates.

### Single-Source Registration in Root `ods.toml`

Custom profiles are registered explicitly in root `ods.toml` under `custom_profiles`:

```toml
# root ods.toml
spec = "0.1"

custom_profiles = [
  ".ods/profiles/rfc.md",
  "docs/profiles/api_endpoint.md"
]
```

### Custom Profile Schema Definition File (`docs/profiles/api_endpoint.md`)

```markdown
---
ods:
  custom_profile:
    name: api_endpoint
    required_keys:
      - service
      - endpoint_url
---

# API Endpoint Profile

## Overview

## Specification

### Request Payload

### Response Payload

## Verification & Testing
```

- **`ods.custom_profile.name`**: Names the profile represented by this registered definition file. Target documents select it with `ods.profile: api_endpoint`.
- **`required_keys`**: List of top-level frontmatter keys required on target documents during `ods lint`.
- **`optional_keys`**: List of top-level frontmatter keys documented as useful but not required.
- **`forbidden_keys`**: List of top-level frontmatter keys that target documents using the profile must not contain.
- Each of these three policy keys accepts one or more key names, with one `-` entry per name. If a policy has no entries, omit it; `[]` is an optional YAML notation for an empty list.
- **H2 & H3 Hierarchies**: `ods lint` validates parent `## H2` and child `### H3` section trees.

---

## 🛠️ Complete ODS CLI Command Reference

| Command | Syntax | Role & Description |
|---|---|---|
| `ods init` | `ods init [path]` | Initialize root `ods.toml` with `spec = "0.1"` spec marker. |
| `ods setup` | `ods setup [path] [--git-hooks]` | Verify workspace health, check updates, and register background OS service. `--git-hooks` installs `.git/hooks/pre-commit`. |
| `ods lint` | `ods lint [path] [ods lint\|3 \| --mode standard\|strict] [--fix] [--format text\|json\|sarif]` | Graph validation. `--fix` regenerates **indexes only** (does not create missing depends targets). Extra dialects: `--okf` / `--skills`. |
| `ods lint --okf` | `ods lint --okf [path] [--skip-frontmatter-keys] [--ignore-keys k1,k2]` | Validate Google OKF v0.2 knowledge bundles (`okf_version: "0.2"`). Supports key suppression flags and root `okf_lint` frontmatter policies. |
| `ods export graph` | `ods export graph [path] [--format text\|json\|md] [--spec ods\|okf]` | Export workspace knowledge graph in structured JSON for AI agents, Markdown snapshot, or text edge list (`--spec okf` exports Google OKF v0.2 bundle JSON). |
| `ods stats` | `ods stats [path]` | Display workspace document telemetry, graph density, profile distribution, and health score %. |
| `ods overview` | `ods overview [path]` (alias: `summary`) | Cold-start snapshot: counts, profile/status, top tags, custom keys (no lint health). |
| `ods completion` | `ods completion <bash\|zsh\|fish\|powershell>` | Generate shell autocompletion scripts for popular shells. |
| `ods schema` | `ods schema [keys] [--write] [--format text\|json]` | Export JSON Schema, or list registry keys (`ods schema keys`). |
| `ods tree` | `ods tree [path]` | Display visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs. |
| `ods diff` | `ods diff [target]` | Compare document graph dependencies and frontmatter changes against git commits or branches. |
| `ods clean` | `ods clean [path]` | Clean diagnostic reports (`.ods/ods-errors.md`), coverage files (`.ods/coverage.md`), and cache files. |
| `ods status` / `coverage` | `ods coverage [path]` | Display workspace health score and profile coverage breakdown. |
| `ods context` | `ods context <doc-id> [--max-tokens N] [--print] [--include-code] [--root <dir>]` | Bounded AI reading list (depends + context.load). Without id: unique `--tag`/`--key`/`--status` only. |
| `ods find` | `ods find [path] [--tag t] [--key expr] [query]` | Find docs by tag, schema/custom keys, and/or id/path query (`--format text\|json`). |
| `ods mv` | `ods mv <src> <dst>` | Move/rename Markdown file and auto-heal graph links and references. |
| `ods adopt` | `ods adopt [path]` | Auto-draft frontmatter on unindexed legacy Markdown files. |
| `ods profile` | `ods profile list/init` | List registered profiles or scaffold new custom profile schemas. |
| `ods tags` | `ods tags [path] [--all]` | List root-level project tags with use counts (`--all` includes unused default tags). Tags must be top-level frontmatter (not under `ods:`). |
| `ods tag list` / `show` | `ods tag list` · `ods tag show <tag>` | Observed tags with counts, or documents matching a tag (`--format text\|json`). |
| `ods tag rename` | `ods tag rename <old> <new> [--write]` | Rewrite a root-level tag across document frontmatter (dry-run; `--write` applies). |
| `ods share` | `ods share [path]` | Export public documentation pack while stripping `share: private` files. |
| `ods pack` | `ods pack <subcommand>` | Manage reusable ODS document packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| `ods bench` | `ods bench <subcommand>` | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| `ods doctor` | `ods doctor [path]` | Workspace health check (version, doc count, index freshness, profile conflicts, service status). |
| `ods update` | `ods update` | Binary self-update to latest released version from GitHub Releases. |

---

## 🤖 GitHub Marketplace Action Integration

Automate document graph verification in your GitHub CI/CD pipeline:

```yaml
name: Documentation Graph CI

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  validate-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify Open Document Spec Graph
        uses: StaytunedLLP/open-document-spec@v1
        with:
          version: 'latest'
          command: 'lint'
```

---

## 📚 Complete End-User Guide Navigation

- 🏁 **Tier 1 (Novice)**: [Introduction](/docs/guide/01-introduction) · [Quickstart](/docs/guide/02-quickstart) · [Adoption](/docs/guide/03-adoption)
- 🛠️ **Tier 2 (Practitioner)**: [Tooling Matrix](/docs/guide/04-tooling) · [Features](/docs/guide/features)
- 📋 **Tier 3 (Power User)**: [Profiles & Catalogs](/docs/guide/05-profiles) · [Advanced Workspaces](/docs/guide/06-advanced) · [ROI Calculator](/docs/guide/roi-calculator)
- 🏢 **Tier 4 (Architect)**: [Diagnostics](/docs/guide/07-troubleshooting-and-diagnostics) · [Enterprise Deployment](/docs/guide/08-enterprise-deployment) · [Use Cases](/docs/guide/use-cases)

### Specification (format meaning)

- **ODS**: [Intro](specs/ods/intro.md) · [Keys](specs/ods/keys.md) · [Core](specs/ods/core.md) · [all modules](specs/ods/ods.toml)
- **OKF** (`--okf`): [Intro](specs/okf/intro.md) · [Keys](specs/okf/keys.md)
- **Skills** (`--skills`): [Intro](specs/skills/intro.md) · [Keys](specs/skills/keys.md)
- Site: https://opendocify.com/spec

---

## 📄 License

Apache License 2.0. See [LICENSE](LICENSE).
