---
description: "Standard profiles, custom profiles, precedence rules, profile mapping matrix, and ODS Packs."
ods:
  profile: "note"
  status: "stable"
---

# Profiles, Catalogs, and ODS Packs

Profiles answer **what kind of document** a file is. They define expected H2 section headings checked during validation.

ODS uses a two-tiered **Standard (Built-in) vs. Workspace (Custom)** layering model for profiles and tags:

| Layer | Source | Enforcement |
| :--- | :--- | :--- |
| **Standard Profiles** | Built into the specification / `ods` binary (13 core profiles) | Always available; section lint for known shapes |
| **Custom Profiles** | Explicitly declared under `custom_profiles` in root `ods.toml`, or imported **ODS Packs** (`packs:`) | Additive catalogs; unknown name → error (no fallback to **Default Profile (`note`)**) |

Prefer **Standard Profiles** first (`agent`, `feature`, `guide`, `api`, `architecture`, `decision`, `sop`, `policy`, `meeting`, `faq`, `checklist`, `index`, `note`). Introduce a **Custom Profile** or **ODS Pack** only when a repeated document class is worth standardizing across teams.

---

## Resolution Precedence

When resolving profile definitions (Zero Folder Auto-Discovery):

```text
1. Standard Profiles (built-in 13 core profiles)           // always loaded first
2. Explicit custom profile catalog paths listed in custom_profiles (ods.toml): // workspace-local
3. Custom profiles in imported ODS Packs listed in packs:   // vendor / linked packs
```

- First definition of a name wins.
- Later duplicate definitions trigger a conflict warning (CI lint (compliant) should fail).
- Custom profiles are **additive**; they cannot overwrite or replace a Standard Profile name.
- Custom profiles are recognized **strictly** when explicitly declared in `custom_profiles (ods.toml):` or imported `packs:`. Unlisted directories are not auto-discovered.

---

## Custom profiles (one command)

```bash
ods profile init rfc              # scaffold .ods/profiles/rfc.md + register custom_profiles (ods.toml):
ods profile init rfc --no-register  # scaffold only
ods profile show rfc
ods profiles                      # list standard + custom
```

By default, `profile init` appends `.ods/profiles/<name>.md` under root **`custom_profiles (ods.toml):`**. No folder auto-discovery: unlisted paths are ignored.

Profile section titles are exact. Extra heading synonyms belong in the profile definition (pipe-alternatives such as `Goal | Objective`). Workspace `[aliases]` and the `ods alias` / `ods aliases` commands were removed.

---

## ODS Packs (`ods pack`)

A **Profile** defines a single document structural schema (`profile: decision`). A **Pack** (v1) is a reusable directory/repo whose primary engine effect is importing **custom profile catalogs** (`ods-profiles/` or pack root). Packs may also carry Markdown SOPs/guides as ordinary docs; **skills install** and template engines are separate surfaces (`ods skill install`, `ods init --skills`) — not automatic pack apply.

Workspaces declare custom profiles and imported ODS Packs in their root `ods.toml`:

```yaml
---
profile: index
spec = "0.1"
ignore:
  - src
custom_profiles (ods.toml):
  - .ods/profiles/rfc.md
packs:
  - vendor/engineering-pack
  - ../shared-company-pack
---
```

### Multi-Transport `ods pack add`

`ods pack add <source> [--auto-update frequency]` supports Git-native transport across five sources:

1. **GitHub Shorthand**: `ods pack add owner/repo` (Git clone into `vendor/repo`).
2. **HTTPS Git URL**: `ods pack add https://github.com/acme/pack.git` (Remote HTTPS clone).
3. **SSH Git URL**: `ods pack add git@github.com:acme/pack.git` (Remote SSH clone).
4. **Local Path / Link**: `ods pack add ../shared-company-pack` (Monorepo & local pack links/symlinks).
5. **File URL**: `ods pack add file:///opt/packs/security` (Air-gapped enterprise CI volume mounts).

---

## Profile Mapping Matrix ("Which Profile Should I Use?")

| Document Intent | Recommended Standard Profile | Custom Profile / Pack Option |
| :--- | :--- | :--- |
| **Product Requirements / PRD** | `feature` | — |
| **System Design / Technical Architecture** | `architecture` | — |
| **Interface / API Specification** | `api` | — |
| **Architecture Decision / Choice / ADR** | `decision` | — |
| **Standard Operating Procedure / Runbook** | `sop` | — |
| **Rules / Policy / Governance** | `policy` | — |
| **Tutorial / How-To Guide** | `guide` | — |
| **Audit / Release Gates** | `checklist` | — |
| **Meeting Notes / Discussion Summary** | `meeting` | — |
| **Frequently Asked Questions** | `faq` | — |
| **Directory Navigation** | `index` | — |
| **General Prose / Formal Spec / Note** | `note` | — |
| **Agent Instructions / Prompt Contract** | `agent` | Goal, Task, Scope, Non-Scope, Context, Inputs, Constraints, Priority, Steps, Output, Success Criteria, Failure Modes, Dependencies, Assumptions, Examples |

### Writing `agent.md` documents

Use the headings in the `agent` row as the document body contract. Keep the
instruction itself in Markdown under those headings; frontmatter should only
identify the document (`ods.profile: agent`, lifecycle status, and applicable
relationships). A short document may omit empty optional sections, while a
complex task should make dependencies, assumptions, failure modes, and success
criteria explicit. Use `depends` for document-level prerequisites and the
`Steps` section for ordered work; do not encode task logic in XML tags or
invent new frontmatter keys for prose instructions.


### Profile resolution order

1. Built-in standard profiles
2. Explicit custom profiles listed in root `custom_profiles (ods.toml):`
3. Custom profiles in imported `packs:`
