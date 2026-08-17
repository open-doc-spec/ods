---
description: "ODS skill references catalog — self-documenting CLI discovery & editor setup."
---

# ODS Skill References

The `ods` CLI binary is self-documenting and schema-driven. Rather than maintaining duplicate spec files that can drift, agents and developers should query the CLI directly for real-time schemas, profiles, and command usage.

---

## 1. Dynamic CLI Discovery Commands

| Goal | Command | Description |
|---|---|---|
| **Frontmatter Key Placement** | `ods schema keys` | Instant table showing whether a key belongs at the `TopLevel` or `NestedEngineMap` (`ods:`), its type, and rules. |
| **JSON Schema** | `ods schema` | Full machine-readable JSON Schema for validation and tooling. |
| **Profile Sections & Policies** | `ods profile show <name>` | Inspect required `##` headings, required keys, optional keys, and forbidden keys for any profile. |
| **Command Flags & Usage** | `ods <command> --help` | Complete argument signatures, options, and operational hints. |
| **Tag Taxonomy** | `ods tag list` / `ods tag show <tag>` | Inspect all tags across the workspace with document counts. |
| **Workspace Health Check** | `ods doctor` | Diagnostic health check across workspace markers, profiles, and services. |

---

## 2. Editor & IDE Integration

- **[`lsp.md`](lsp.md)**: Configuration guide for the native JSON-RPC 2.0 Language Server (`ods lsp`) in Zed, VS Code, Cursor, Neovim, and Helix.

---

## 3. Normative Specifications (Source of Truth)

Authoritative normative specification documents are maintained in the satellite repository:
- **Repository**: [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec)
- **Key Dictionary**: `specs/keys.md`
- **Core Model**: `specs/core.md`
- **Context Resolution Algorithm**: `specs/context.md`
- **Profile Shapes & Key Policies**: `specs/profiles.md`
- **Validation Rules**: `specs/validation.md`
- **Asset Bindings**: `specs/assets.md`
