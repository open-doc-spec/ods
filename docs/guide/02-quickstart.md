---
description: "Installation options, workspace initialization, service background daemon, validation, and AI context commands."
ods:
  profile: "note"
  status: "stable"
---

# Quickstart

Open Document Spec is one native CLI binary: **`ods`**. ODS is the default engine (no `--ods` flag). Extra specs use **`--okf`** / **`--skills`**. The normal flow is install, run `ods setup`, initialize or adopt Markdown, optionally keep the user service running, then validate with bare **`ods lint`** / **`ods lint`**.

---

## 1. Setup & Installation

### Option 1: Primary Preference — Skill-First Setup for AI Assistants

The recommended, zero-friction entry point for ODS is via the **ODS Skill** in your AI Coding Assistant (Claude Code, Antigravity, Cursor, Codex, Windsurf, etc.).

When an AI assistant activates the ODS skill, the bundled cross-platform bootstrap script automatically:
1. Detects your host Operating System (macOS, Linux, Windows) and architecture (`x86_64`, `arm64`).
2. Downloads and verifies the matching native `ods` (legacy `ods`) release binary.
3. Registers and starts the persistent background OS user service (`systemd` user unit / `launchd` agent / Windows Scheduled Task).
4. Validates workspace health and prints status:

```text
==> Open Document Spec is installed and running on your machine!
==> Version: ods v0.0.x
```

---

### Option 2: Direct CLI Installation

If you are operating directly in a terminal without an AI coding skill, you can manually install the CLI binary:

**macOS / Linux**:

```bash
curl -fsSL https://opendocify.com/install.sh | bash
ods --version
```

**Windows (PowerShell)**:

```powershell
irm https://opendocify.com/install.ps1 | iex
ods --version
```

---

## 2. Initialize a Workspace

New documentation folder:

```bash
mkdir my-docs
cd my-docs
ods init .          # ODS default (writes root ods: marker)
```

Existing Markdown tree:

```bash
cd existing-docs
ods init . --adopt  # or: ods init . --adopt
```

OKF knowledge bundle:

```bash
ods init --okf .    # OKF v0.2 knowledge bundle (extra-spec flag)
```

`ods init` (ODS default) makes the folder ODS-compliant by creating a root `ods.toml` with `spec = "0.1"` and generating child index files.

### Multi-spec: when do I need a flag?

| Situation | Command |
| :--- | :--- |
| ODS workspace (default) | bare `ods lint`, `ods context …` |
| Pure OKF tree (`okf_version` only) | always pass **`--okf`** (e.g. `ods lint --okf`) |
| Hybrid (both markers) | bare = **ODS only**; pass **`--okf`** for ODS+OKF, or set root `specs.okf.enabled: true` once |
| Agent Skills package | **`--skills`** |

OKF ships **in the same binary** (native engine) but is **not** always on. There is no `--ods` flag.  
ODS lifecycle is **`ods.status`**; OKF uses top-level **`status`** — they are not auto-mapped.

---

## 3. Run Setup & Start Background Service

```bash
ods setup
```

`ods setup` checks release freshness, verifies the root spec header, starts/registers the background OS user service (`systemd` / `launchd` / `schtasks`), and runs `ods doctor`.

Direct service commands:

```bash
ods start .
ods start --status
ods stop .
ods stop --unregister .
```

Foreground alternative:

```bash
ods watch .
```

While `ods start` or `ods watch` runs, rename/move Markdown normally. ODS keeps path-shaped `id`, `depends`, `related`, body links, resource paths, context path entries, and generated `ods.toml` child lists current.

---

## 4. Validate Trust

```bash
ods lint
ods lint
```

Clean lint output:

```text
Everything is fine — graph and links are consistent. No update required.
```

---

## 5. Use AI Context

Preferred bounded reading list (what agents should load):

```bash
ods context <doc-id>
ods context <doc-id> --max-tokens 8000 --print
ods context <doc-id> --explain              # why each path was included
ods context <doc-id> --include-related      # also walk soft related: edges
ods context <doc-id> --okf                  # pure OKF, or hybrid ODS+OKF merge
```

| Frontmatter | Role in context |
| :--- | :--- |
| `ods.depends` | Structural prerequisites — **walked** (up to `context.max-depth`) |
| `ods.related` | Soft links — **not** walked unless `--include-related` |
| `ods.context.load` | Extra files (md or resources) that **must** load — use top-level **`load:`** in ODS 2.0 |
| `ods.context.ignore` | Skip noisy trees during expansion |
| `ods.code` | Only if you pass `--include-code` |

`ods export` writes a **full graph dump** (default `.ods/graph.md`) for humans/CI — not the primary AI prompt pack.

```bash
ods export
ods export --out ai/graph.md
```

Publishing a filtered subset for external hand-off:

```bash
ods share . --out ../shared-docs
```

---

## 6. Keep Current

```bash
ods update --check
ods update
```

`ods update` downloads the latest binary release from GitHub Releases and automatically restarts the background service so it runs with the updated binary.

---

## Next

- Existing repos: [Adopting ODS](/docs/adoption)
- CLI, service, CI, updates: [Tooling Reference](/docs/tooling)
- Profiles: [Profiles & Catalogs](/docs/profiles)
- FAQ: [FAQ & Troubleshooting](/docs/faq)
