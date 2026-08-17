---
profile: note
status: stable
share: public
description: Comprehensive strategic and technical research report on integrating Google OKF v0.2 into ODS (Open Document Spec), scaling architecture, zero-Rust distribution, release pipelines, universal agent ecosystem, and 100K GitHub star growth plan.
owner: team:opendocify
tags: [okf, ods, research, scaling, agent-spec]
---

# Google OKF v0.2 & ODS Integration Research Report

## Executive Summary

This report establishes the architectural roadmap, integration specification, distribution strategy, and open-source growth strategy for incorporating **Google Open Knowledge Format (OKF) v0.2** natively into **Open Document Specs (ODS)** under the umbrella brand **Open Document Spec** (`opendocify.com`).

Google OKF v0.2 introduces critical standards for machine-generated and agent-maintained knowledge bundles (provenance, trust signals, temporal staleness, and attested computations). By natively combining ODS (developer & codebase documentation graphs) with OKF v0.2 (data, metric, and knowledge bundle graphs), Open Document Spec becomes the **universal document & knowledge specification for AI agents and human teams**.

---

## 1. Native OKF v0.2 Support & Brand Strategy (`ods` vs `ods` / `opendocify.com`)

### 1.1 Should we offer native support of Google OKF v0.2?
**YES.** Google OKF v0.2 and ODS solve complementary halves of knowledge representation:
* **ODS**: Optimizes software engineering repositories, file-to-code mapping (`code:`), dependency graphs (`depends:`, `related:`), context scoping (`ods context`), and structural section profiles (`profile:`).
* **OKF v0.2**: Optimizes data asset catalogs, metrics, business logic provenance (`sources:`), trust verification (`generated:`, `verified:`), temporal staleness (`stale_after:`), and executable compute validation (`Attested Computation`).

Natively supporting both specs in one engine positions Open Document Spec as the standard protocol across both engineering codebases and data/AI agent knowledge bases.

### 1.2 Dual-Spec Architecture
Open Document Spec's engine (`ods-core`) will natively recognize and validate two complementary specifications:

```
                          ┌────────────────────────────────────────┐
                          │   Open Document Spec Platform (opendocify.com) │
                          └───────────────────┬────────────────────┘
                                              │
                                   ┌──────────┴──────────┐
                                   ▼                     ▼
                       ┌───────────────────────┐ ┌───────────────────────┐
                       │   ODS Specification   │ │   OKF Specification   │
                       │  (Codebase Doc Graph) │ │  (Agent Knowledge)    │
                       └───────────────────────┘ └───────────────────────┘
```

* **Root Workspace Identification**:
  * ODS Workspace: Root `index.md` contains `ods: 0.1` (or `ods: 0.2`).
  * OKF Workspace / Bundle: Root `index.md` contains `okf_version: "0.2"` or `okf: 0.2`.
  * Hybrid Workspace: Root `index.md` contains both `ods: 0.2` and `okf: 0.2`.
* **Cross-Spec Interoperability**: OKF files in an ODS workspace can link to ODS files via standard Markdown relative links. Commands like `ods lint`, `ods index`, `ods context`, and `ods export` operate seamlessly across both formats.

### 1.3 Brand and Binary Naming (`ods` vs `ods`)
* **Company & Domain**: **Open Document Spec** (`opendocify.com`).
* **Specification Suite**: **ODS / ODS Core Specifications**.
* **CLI Binary Strategy**:
  * Maintain `ods` as the primary CLI binary name for backwards compatibility, zero breakage of existing installations, and muscle memory.
  * Provide `ods` and `opendocify` as native aliases / symlinks created during installation.
  * CLI command usage examples: `ods lint`, `ods lint`, `opendocify lint` execute identically.

---

## 2. Integration Mechanics: Native vs Optional

### 2.1 Why Native Support in `ods-core` Rust Engine?
Integrating OKF natively into the core Rust engine (`ods-core`) — rather than behind an optional plugin or feature flag — is essential for the following reasons:

1. **Zero-Friction AI Workflow**: Modern LLMs (Gemini 2.5/3, Claude 3.5/3.7, AGY CLI) generate OKF v0.2 knowledge documents out-of-the-box. Native parsing ensures agents and developers experience instant validation, indexing, and context extraction without secondary tooling.
2. **Unified Context Graph (`ods context`)**: Allows an AI agent to resolve a prompt context that spans from a high-level OKF metric concept (e.g. `Revenue`), through its `Attested Computation`, to the underlying ODS architectural docs and `code:` references in one traversal.
3. **Performance**: Rust-native parsing of both frontmatter conventions guarantees sub-millisecond linting across 100,000+ document repositories.

---

## 3. Zero-Rust Local Execution Architecture

To guarantee that users do **NOT** need Rust or Cargo installed on their local machine, Open Document Spec provides 4 zero-dependency distribution channels:

```
                              ┌─────────────────────────┐
                              │  Open Document Spec Core (Rust) │
                              └────────────┬────────────┘
                                           │
         ┌──────────────────┬──────────────┴───────┬──────────────────┐
         ▼                  ▼                      ▼                  ▼
┌──────────────────┐ ┌───────────────┐   ┌──────────────────┐ ┌───────────────┐
│ Standalone Binaries│ │ NPM Wrapper   │   │ Install Scripts  │ │ WebAssembly   │
│ (GitHub Releases)│ │(@opendocify/  │   │ (curl / iwr)     │ │ (@opendocify/ │
│                  │ │ cli)          │   │                  │ │  wasm)        │
└──────────────────┘ └───────────────┘   └──────────────────┘ └───────────────┘
```

1. **Prebuilt Static Binaries**: GitHub Actions (`release.yml`) cross-compiles static binaries for Linux (`x86_64`, `aarch64`, `musl`), macOS (`x86_64`, `apple-silicon`), and Windows (`x86_64-msvc`).
2. **One-Line Installer Scripts**:
   * macOS/Linux: `curl -fsSL https://opendocify.com/install.sh | sh`
   * Windows: `iwr -useb https://opendocify.com/install.ps1 | iex`
3. **Zero-Dependency NPM / NPX Package (`@opendocify/cli` / `ods-cli`)**:
   * Automatically fetches the appropriate prebuilt binary for the host OS during `npm i -g @opendocify/cli`.
   * On-demand execution without global install: `npx @opendocify/cli lint`.
4. **WebAssembly Bindings (`@opendocify/wasm`)**:
   * Compiles `ods-core` to WASM using `wasm-pack`.
   * Powers in-browser playground on `opendocify.com`, VSCode Web / GitHub.dev extensions, Edge Functions, and Node.js environments with **zero native binary downloads**.

---

## 4. Connecting Release Notes to the Website

To maintain a real-time, automated connection between GitHub releases and `opendocify.com`:

```
┌──────────────────┐      ┌─────────────────────────┐      ┌──────────────────┐
│  CHANGELOG.md /  │────► │ GitHub Release Published│────► │ Webhook Trigger  │
│  Git Tag Commit  │      │  (release.yml)          │      │  (Deploy Event)  │
└──────────────────┘      └─────────────────────────┘      └────────┬─────────┘
                                                                    │
                                                                    ▼
                                                           ┌──────────────────┐
                                                           │ opendocify.com/  │
                                                           │ releases Page    │
                                                           └──────────────────┘
```

1. **Single Source of Truth**: `CHANGELOG.md` in repository root.
2. **Automated Web Release Sync**:
   * Astro site (`app-web`) fetches release notes at build time via GitHub REST API (`/repos/open-doc-spec/ods/releases`) paired with local `CHANGELOG.md` parsing.
   * `.github/workflows/release.yml` triggers a repository dispatch hook to rebuild and redeploy `app-web` to Firebase Hosting / CDN whenever a release tag is published.
3. **CLI Terminal Release Notes**:
   * `ods release-notes` command renders formatted release notes directly in the user's terminal.
   * `ods update --check` summarizes available updates alongside change highlights.

---

## 5. Website Skill Download (.zip Distribution)

To make agent skill installation frictionless:

1. **Automated Skill Packaging**:
   * In `.github/workflows/release.yml` and `app-web` build pipeline, package `skills/ods/` and `skills/okf/` into compressed `.zip` and `.tar.gz` archives.
   * Assets published to `public/skills/ods.zip` and `public/skills/okf.zip` on `opendocify.com`.
2. **Download Endpoints**:
   * Direct URL: `https://opendocify.com/skills/ods.zip`
   * Direct terminal fetch: `curl -fsSL https://opendocify.com/skills/ods.zip -o ods-skill.zip`
3. **Web UI Download Button**:
   * Prominent "Download Skill (.zip)" button on `opendocify.com` documentation and skill landing pages for instant drag-and-drop into Cursor, Claude Desktop, AGY CLI (`~/.gemini/antigravity-cli/skills/`), and VS Code.

---

## 6. Version Synchronization Across Repo, CLI & Skill

### 6.1 Current Disconnect Analysis
Currently, version numbers are manually specified across `Cargo.toml` (`0.0.4`), `SKILL.md` (`v0.1.24`), `README.md`, and `package.json`.

### 6.2 Streamlined Single-Source Versioning Strategy
1. **Version Script (`scripts/bump-version.sh`)**:
   * Command: `./scripts/bump-version.sh 0.2.0`
   * Atomically updates:
     * `Cargo.toml` (`workspace.package.version`)
     * `skills/ods/SKILL.md` (frontmatter `metadata.github-ref`, `description`, spec version)
     * `skills/ods/references/spec.md`
     * `app-web/package.json`
2. **CI Version Integrity Gate (`.github/workflows/pr.yml`)**:
   * Add a validation step in PR CI that fails if `Cargo.toml` version, `SKILL.md` tag reference, and `CHANGELOG.md` top entry mismatch.
3. **Release Dynamic Templating**:
   * During release packaging, `SKILL.md` frontmatter is dynamically injected with the exact Git release SHA and tag.

---

## 7. Universal AI Agent Format Support (`AGENTS.md`, Claude, AGY CLI, Skills)

### 7.1 Strategic Value for AI Users
As AI software engineering matures, agent instructions are heavily fragmented across `.claude/agents/`, `.gemini/subagents/`, `AGENTS.md`, `.cursorrules`, and custom skill folders.

```
                           ┌───────────────────────────┐
                           │ Open Document Spec Universal Graph│
                           │  (ODS/OKF Agent Graph)   │
                           └─────────────┬─────────────┘
                                         │
        ┌───────────────────┬────────────┴────────────┬───────────────────┐
        ▼                   ▼                         ▼                   ▼
┌───────────────┐   ┌───────────────┐         ┌───────────────┐   ┌───────────────┐
│  AGENTS.md    │   │ AGY Subagents │         │ Claude Agents │   │ Cursor Rules  │
│  (Universal)  │   │ (.gemini/)    │         │ (.claude/)    │   │ (.cursor/)    │
└───────────────┘   └───────────────┘         └───────────────┘   └───────────────┘
```

By providing first-class ODS profiles (`profile: agent`, `profile: skill`, `profile: subagent`), Open Document Spec brings structure, linting, and graph management to AI prompt instructions.

### 7.2 Key Benefits
1. **Elimination of AI Instruction Drift**: A single source of truth for agent rules. Updating an ODS agent document updates all exported agent formats automatically.
2. **Context-Optimized Prompts (`ods context --agent <name>`)**: Resolves bounded prompt context (loading only relevant rules, code references, and skills) to avoid token waste and prevent hallucinations.
3. **Prompt Linting & Dead-Link Detection**: `ods lint` validates that agent subagent calls reference existing agent files, tools exist on disk, and skill parameter contracts match.
4. **Universal Export (`ods agent sync`)**: Automatically compiles ODS agent definitions into native `.claude/`, `.gemini/`, `.cursor/`, and `AGENTS.md` formats.

---

## 8. ODS Gaps & OKF Shared Key Adoption

### 8.1 Gap Analysis in ODS
Comparing ODS v0.1 with Google OKF v0.2 reveals four critical metadata gaps in ODS:

| Feature Area | ODS v0.1 | OKF v0.2 | Recommendation for Open Document Spec |
|---|---|---|---|
| **Provenance** | Only `owner` | `sources:` (with `author`, `usage_count`, `last_modified`) | Adopt `sources:` into ODS core schema |
| **Freshness** | `status: draft\|stable` | `stale_after: YYYY-MM-DD` | Adopt `stale_after:` into ODS core schema |
| **Audit Trail** | None | `generated: {by, at}` & `verified: [{by, at}]` | Adopt `generated:` and `verified:` into ODS |
| **Computation** | None | `runtime:`, `parameters:`, `attester:`, `executor:` | Natively support OKF `Attested Computation` profile |

### 8.2 Unified Core Vocabulary
Instead of maintaining divergent frontmatter keys, Open Document Spec adopts a **Unified Common Frontmatter Standard**:

```yaml
---
# Shared Metadata
title: Customer Orders Dataset
description: One row per completed order.
status: stable                     # draft | stable | deprecated | archived
stale_after: 2026-12-31           # Absolute date staleness check

# Provenance & Trust
generated: { by: "agent:gemini-3.6", at: "2026-07-29T22:00:00Z" }
verified: [{ by: "human:ahormati", at: "2026-07-30T10:00:00Z" }]
sources:
  - id: ga4-export
    resource: https://analytics.google.com

# Document Graph (ODS Engine)
profile: dataset                   # ODS Profile or OKF Type alias
depends: [schemas/orders-v2]
related: [guides/data-privacy]
code:
  - path: src/db/orders.rs
    role: schema
---
```

---

## 9. Repository Compliance Audit Command (`ods audit` / `ods scan`)

### 9.1 Overview & Command Interface
To allow developers and CI/CD pipelines to scan a repository and identify Markdown files that do not conform to ODS/OKF standards, Open Document Spec introduces `ods audit` (alias `ods scan`).

```bash
# Scan repository for non-compliant markdown files
ods audit

# Enforce strict compliance in CI/CD pipeline (exits 1 on unadopted files)
ods audit --strict

# Output machine-readable JSON report
ods audit --format json
```

### 9.2 Execution Output Example

```
🔍 Open Document Spec Repository Audit Report
Workspace: /home/user/my-project

Summary:
  Total Markdown Files : 142
  ✅ ODS/OKF Compliant : 98  (69.0%)
  ⚠️ Unadopted Plain   : 38  (26.8%)
  ❌ Error / Malformed  : 6   (4.2%)

Errors Found:
  1. docs/api/users.md: Line 1: Missing required frontmatter block.
  2. docs/guides/setup.md: Line 4: Dangling dependency 'guides/old-setup'.
  3. metrics/revenue.md: Line 12: Invalid status 'active' (expected draft|stable|deprecated|archived).

Unadopted Files:
  - README.md
  - CONTRIBUTING.md
  - docs/internal/notes.md
  ... 35 more files.

💡 Quick Fix: Run 'ods adopt --write' to automatically generate frontmatter for plain markdown files.
```

---

## 10. GitHub Search Compliance Tracking & Query

### 10.1 How to Track ODS/OKF Compliant Repositories
Every ODS workspace root `index.md` contains the explicit root marker `ods: 0.1` (or `ods: 0.2`). Every OKF workspace root contains `okf_version: "0.2"` or `okf: 0.2`.

### 10.2 GitHub Code Search Queries

1. **Search for ODS Compliant Workspaces**:
   ```
   filename:index.md path:/ "ods:"
   ```

2. **Search for OKF Compliant Workspaces**:
   ```
   filename:index.md path:/ ("okf:" OR "okf_version:")
   ```

3. **Combined Open Document Spec Universal Search Query**:
   ```
   filename:index.md path:/ ("ods:" OR "okf:" OR "okf_version:" OR "ods:")
   ```

4. **GitHub Search API cURL Command**:
   ```bash
   curl -s -H "Accept: application/vnd.github+json" \
     "https://api.github.com/search/code?q=filename:index.md+path:/+%22ods:%22" \
     | jq '.total_count'
   ```

### 10.3 Automated Live Tracker Badge
`opendocify.com` will run an automated daily GitHub Search API worker to track total public compliant repositories and display a live badge:
`![ODS Compliant Repos](https://img.shields.io/badge/ODS--Compliant--Repos-12.4k-blue)`

---

## 11. Hyper-Scaling Strategy: 100K GitHub Stars & 1M Users Roadmap

To scale Open Document Spec to 100,000+ GitHub Stars and 1,000,000+ users:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      VIRAL DEV-TO-DEV FLYWHEEL                          │
├─────────────────────────────────────────────────────────────────────────┤
│ 1. Zero-Install NPX & WASM Demo ──► Instant User Delight                │
│ 2. Prepackaged AI Agent Skills   ──► Automatic Adoption by AI Tools    │
│ 3. GitHub Action Linter Gate     ──► PR Enforcement in Top Repositories│
│ 4. "Powered by Open Document Spec" Badge ──► Organic Star & Viral Discovery    │
└─────────────────────────────────────────────────────────────────────────┘
```

1. **AI Ecosystem Integration**: Publish official ODS skills across AGY CLI, Cursor Directory, Claude Artifacts, LlamaIndex, LangChain, and AutoGen.
2. **Interactive In-Browser WASM Playground**: `opendocify.com/play` allows developers to drop any repo URL or zip file to instantly visualize and fix their documentation graph using WebAssembly.
3. **GitHub Action Compliance Gate**: Provide `@opendocify/action` with zero setup so open-source maintainers can enforce `ods audit --strict` in their CI workflows.
4. **Token Cost Savings Calculator**: Position `ods context` as the #1 token-saving tool for developers using Claude 3.7 / Gemini 2.5 / GPT-4o, demonstrating 70%+ lower prompt costs.
5. **Community & Content Engine**: Benchmark reports on AI documentation drift, technical deep-dives on HackerNews and LlamaIndex, and open-source documentation awards.

---

## Conclusion & Action Plan

Integrating Google OKF v0.2 into Open Document Spec (`ods-core`) establishes a universal specification for code, knowledge, and AI agent memory. The strategy outlined above guarantees zero-friction local execution, unified versioning, real-time website release sync, and viral developer distribution.
