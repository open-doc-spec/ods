---
description: "ODS skill references catalog — deep CLI recipes & token economy guides."
tags:
  - skill
  - ods
  - reference
owner: team:ods
ods:
  profile: note
  status: stable
---

# ODS Skill References

This directory contains practical operational guides and advanced query recipes for AI agents executing in ODS workspaces:

---

## Reference Guides

| Guide | Description |
|---|---|
| **[`cli-recipes.md`](cli-recipes.md)** | Advanced CLI query recipes, Boolean search combinations, section slicing, and DAG traversal options. |
| **[`token-economy.md`](token-economy.md)** | Token budgeting heuristics, prompt pruning strategies, and cost-reduction decision matrix (~95% savings). |

---

## Dynamic CLI Discovery

The `ods` CLI binary is self-documenting. Use the following commands for real-time schema and contract inspection:

| Goal | Command |
|---|---|
| **Schema Key Placement** | `ods schema keys` |
| **Machine-Readable JSON Schema** | `ods schema` |
| **Profile Heading Contracts & Policies** | `ods profile show <name>` |
| **Command Flags & Manuals** | `ods <command> --help` |
| **Workspace Health Diagnostic** | `ods doctor` |

---

## Normative Specifications (Source of Truth)

Authoritative normative specifications live in the satellite repository:
- **Repository**: [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec)
- **Key Dictionary**: `specs/keys.md`
- **Context Resolution**: `specs/context.md`
- **Profile Shapes & Key Policies**: `specs/profiles.md`
