---
description: "Complete reference for frontmatter keys, ods.toml configuration, profiles vs packs, AI context, and CLI commands."
status: "stable"
order: 9
ods:
  profile: "note"
  status: "stable"
---

# ODS Features & Architecture Overview

Open Document Specs (ODS) is a lightweight, graph-native Markdown convention layer. This document provides a complete guide to all frontmatter keys, workspace mechanics, profiles, tags, code references, AI context rules, and ODS Packs.

---

## 1. Core Frontmatter Features

Frontmatter is split into **universal top-level** keys (any tool can read them) and **engine keys under `ods:`** (ODS-only). See the key dictionary [ods/keys.md](../../specs/ods/keys.md).

| Field | Type | Placement | Level | Purpose & Usage |
| :--- | :--- | :--- | :---: | :--- |
| **`description`** | string | **Top-level** | 1+ | **Summary**: One-line summary used for indexes and SSG meta. |
| **`tags`** | list of strings | **Top-level only** | 1+ | **Search Facets**: Free-form lowercase facets (`- customer-care`). Never under `ods:`. |
| **`owner`** | string \| list | **Top-level** | 1+ | **Maintainer**: Responsible individual or team (`support-team`). |
| **`profile`** | string | **Under `ods:`** | 1+ | **Document Classification**: Document kind (`guide`, `decision`, `feature`, `sop`, etc.). Defaults to `note`. |
| **`status`** | string | **Under `ods:`** | 1+ | **Document Lifecycle**: Maturity (`draft`, `stable`, `deprecated`, `archived`). Defaults to `draft`. |
| **`share`** | string | **Under `ods:`** | 1+ | **Visibility Control**: `public` (default), `org`, or `private`. |
| **`id`** | string | **Under `ods:`** | 2+ | **Explicit Identity**: Override the path-derived default ID. |
| **`depends`** | list of refs | **Under `ods:`** | 2+ | **Prerequisite Edges**: Graph prerequisite links to other documents. |
| **`related`** | list of refs | **Under `ods:`** | 2+ | **Association Edges**: Non-binding reference links. |
| **`resources`** | list | **Under `ods:`** | 2+ | **Native Asset Links**: Non-Markdown files attached to the document. |
| **`code`** | list | **Under `ods:`** | 2+ | **Implementation Map**: Fixed-role source code mappings (`path`, `symbol`, `role`). |
| **`context`** | map | **Under `ods:`** | 2+ | **Bounded AI Reading Scope**: `load`, `ignore`, `max-depth`. |

Misplaced nested `tags` under `ods:`: `ods lint` warns; repair with `ods fmt --migrate`.

---

## 2. Root `ods.toml` Configuration Keys

The root `ods.toml` file serves as the single workspace marker and policy home for an ODS workspace. Below is the complete reference of all supported configuration sections, keys, default values, and their exact operational effects.

### Top-Level Workspace Settings

| Key | Type | Default | Operational Effect & Behavior |
| :--- | :--- | :--- | :--- |
| **`spec`** | string | `"0.1"` | **Workspace Spec Marker & Version**: Declares the ODS workspace root boundary and specification version (`spec = "0.1"`). Essential for `ods` commands (`ods lint`, `ods find`, `ods context`, `ods overview`) to identify the workspace root. *(Serde alias: `ods`)* |
| **`ignore`** | array of strings | `[]` | **Workspace Scan Excludes**: Paths or glob pattern prefixes excluded from scanner operations (`["src/", "dist/", "target/"]`). Skips these paths during `ods lint`, `ods overview`, `ods find`, and graph context indexing. |
| **`custom_profiles`** | array of strings | `[]` | **Custom Profile Schemas**: Workspace-relative Markdown file paths registering custom profile schemas (`[".ods/profiles/rfc.md"]`). Loaded automatically during `ods profile list`, `ods profile show`, and document structure validation in `ods lint`. *(Serde alias: `custom-profiles`)* |
| **`packs`** | array of strings | `[]` | **Imported ODS Packs**: Relative folder paths to imported ODS Pack bundles (`["vendor/engineering-pack"]`). Automatically merges packed profiles, skills, and assets into workspace discovery. |

---

### Extra Spec Engines (`[specs.okf]` & `[specs.skills]`)

Declaratively enables multi-spec validation during bare `ods lint` without requiring explicit CLI flags (`--okf`, `--skills`).

| Section & Key | Type | Default | Operational Effect & Behavior |
| :--- | :--- | :--- | :--- |
| **`[specs.okf].enabled`** | boolean | `false` | When `true`, bare `ods lint` automatically runs OKF specification validation alongside standard ODS linting. |
| **`[specs.okf].check_keys`** | boolean | `true` | Validates frontmatter keys against the OKF schema registry. |
| **`[specs.okf].ignore_keys`** | array of strings | `[]` | List of frontmatter key names to ignore during OKF key linting. |
| **`[specs.skills].enabled`** | boolean | `false` | When `true`, bare `ods lint` automatically validates Agent Skill definitions alongside standard ODS linting. |
| **`[specs.skills].check_keys`** | boolean | `true` | Validates skill frontmatter keys against the Agent Skills schema registry. |
| **`[specs.skills].ignore_keys`** | array of strings | `[]` | List of skill frontmatter key names to ignore during skills key linting. |

---

### Service & Memory Ceiling (`[service]`)

Controls background daemon behavior for `ods serve` and `ods start`.

| Key | Type | Default | Operational Effect & Behavior |
| :--- | :--- | :--- | :--- |
| **`[service].mode`** | string | `"poll"` | **Watcher Mode**: Daemon change monitoring strategy. Options: `"poll"` (low memory polling), `"watch"` (filesystem event watching), or `"auto"` (auto-select based on system resources). |
| **`[service].poll_secs`** | integer | `2` | **Polling Interval**: Sleep duration in seconds between directory scans when `mode = "poll"`. |
| **`[service].max_rss_mb`** | integer | `10` | **Soft Memory Ceiling**: Maximum Resident Set Size in megabytes target for `ods serve` / `ods start` background processes (`<= 10 MB`). |

---

### Complete `ods.toml` Example

Below is a complete reference `ods.toml` showcasing all supported configuration blocks:

```toml
# ODS workspace configuration
spec = "0.1"

ignore = [
  "target",
  "dist",
  "node_modules"
]

custom_profiles = [
  ".ods/profiles/rfc.md",
  ".ods/profiles/sop.md"
]

packs = [
  "vendor/engineering-pack"
]

[specs.okf]
enabled = true
check_keys = true
ignore_keys = ["custom_okf_meta"]

[specs.skills]
enabled = true
check_keys = true
ignore_keys = []

[service]
mode = "poll"
poll_secs = 2
max_rss_mb = 10
```
