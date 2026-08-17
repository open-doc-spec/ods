---
description: "Enterprise ODS Pack distribution, multi-repo governance, CI/CD pipelines, security controls, and ROI modeling."
ods:
  profile: "note"
  status: "stable"
---

# Enterprise Deployment and Governance Guide

This guide describes patterns for deploying and governing Open Document Spec (ODS) across enterprise engineering organizations, scaling from single repositories to multi-repo monorepos and thousands of developers.

---

## 1. Enterprise ODS Pack Architecture & Distribution

An **ODS Pack** is a reusable workspace bundle containing custom document profiles (`ods-profiles/`), AI agent skills (`skills/`), standard operating procedures (SOPs), and architectural guidelines.

### Creating a Company-Wide Pack (`ods pack init`)

```bash
ods pack init acme-engineering-pack
cd acme-engineering-pack
```

This scaffolds the canonical ODS Pack layout:
```text
acme-engineering-pack/
├── ods.toml                 # ODS workspace root marker
├── ods-profiles/            # Custom YAML schema definitions
│   ├── rfc.md
│   └── postmortem.md
└── skills/                  # AI agent skills and prompts
    └── security-audit/
        └── SKILL.md
```

### Hosting & Distributing Packs via Private Git Repositories

```bash
ods pack add git@github.com:acme-org/engineering-pack.git --auto-update daily
```

This clones the pack into `vendor/engineering-pack` and appends `- vendor/engineering-pack` to the project's root `ods.toml` `packs:` key.

---

## 2. Security, Access Control & Data Isolation (`share` Cascading)

Ods provides **3-Tier Visibility Control** (`share: public | org | private`):

| Share Level | Meaning | AI Context (`ods context`) | Graph Snapshot (`ods export`) | Directory Export (`ods share`) |
| :--- | :--- | :--- | :--- | :--- |
| **`public`** (default) | Open documentation | Included | Included | Copied to output directory |
| **`org`** | Internal team documentation | Excluded by default | Excluded by default | Excluded by default (use `--include-org`) |
| **`private`** | Strictly confidential IP & secrets | Excluded by default | Excluded by default | Excluded by default (use `--include-private`) |

---

## 3. Production CI/CD Pipelines & Quality Gates

Enforce Level-3 specification compliance and graph integrity across all pull requests (PRs) using continuous integration.

```yaml
name: ODS Compliance & Graph Integrity

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  ods-check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Repository
        uses: actions/checkout@v7

      - name: Check Index Lockfile Freshness
        run: ods lint

      - name: Graph and schema lint
        run: ods lint
```
