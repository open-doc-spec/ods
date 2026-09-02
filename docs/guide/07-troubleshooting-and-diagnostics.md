---
description: "Diagnostic workflow, complete error catalog for .ods/ods-errors.md, git merge conflicts, and daemon troubleshooting."
ods:
  profile: "note"
  status: "stable"
---

# Troubleshooting and Diagnostics Guide

This guide provides a comprehensive reference for diagnosing workspace issues, resolving `ods lint` errors in `.ods/ods-errors.md`, handling Git merge conflicts, and troubleshooting background service daemons (`ods serve`).

---

## 0. CLI error shape (first call)

User-facing CLI failures use a fixed, short format so the next action is never ambiguous:

```text
error: <what failed — one short line>
Next: <exact command or action>
```

Usage mistakes (wrong args/flags) use `usage:` instead of `error:` (exit code **2** vs **1**).

Optional `Hint:` lines appear only when they prevent a second failure (for example, OKF markers found → pass `--okf`).

**Source of truth (code):** `src/ods-core/src/error/messages.rs` — do not invent alternate stderr copy in commands.

### Common CLI lifecycle messages

| Situation | You will see | Do this |
|---|---|---|
| Not an ODS workspace | `error: not an ODS workspace (no root ods.toml with spec)` | `Next: run \`ods init\`` (or `--okf` / `--skills` if Hint says so) |
| Forbidden `--ods` | `usage: unknown flag: --ods` | ODS is default — bare `ods <cmd>`; extras are `--okf` / `--skills` only |
| Unknown command | `usage: unknown command '…'` | `ods help`; may include `did you mean …?` |
| Context miss | `error: no document matched '…'` | `ods find <query>` / `ods find --key …` or a path-shaped id |
| Context filter ambiguous | `error: context filter matched N documents…` | Narrow with `ods find --tag` / `--key`, then `ods context <id>` |
| Missing context id | `usage: missing document id` | `ods context <id-or-path>` (or unique `--tag`/`--key`/`--status`) |
| Load failure | `error: could not load workspace at '…'` | Check path or `ods init` |
| Missing workspace marker | `error: missing ods.toml` | `ods init` then retry |
| Nothing to undo | `error: nothing to undo (no snapshot found)` | Snapshots come from bulk writes (adopt/fmt) |
| Update failed | `error: update failed: …` | Check GitHub network access or install from Releases |
| Service start/stop failed | `error: start service: …` / `stop service: …` | Permissions; `ods start --status`; guide § daemon troubleshooting |

Set `ODS_ERROR_CODES=1` to append stable message ids (for docs/automation); default human output stays code-free.

Lint diagnostics for ODS, OKF, and Skills also come from the same catalog (short fact strings in `ods lint` / `.ods/ods-errors.md`).

---

## 1. The Diagnostic Workflow

When an issue occurs or documents are modified, follow this 3-step diagnostic workflow:

```bash
# Step 1: Check overall workspace health and daemon service status
ods doctor

# Step 2: Validate graph relationships, schemas, and references
ods lint

# Step 3: Re-lint after fixes (discovery is overview/find/tree — no index lockfiles)
ods overview
ods lint
```

If `ods lint` detects errors, it writes a detailed diagnostic report to `.ods/ods-errors.md`. When all issues are resolved, `ods lint` automatically deletes that report and outputs a green confirmation message.

---

## 2. Complete `.ods/ods-errors.md` Lint Error Catalog

Below is the complete catalog of errors and warnings reported by `ods lint`, along with their root causes and step-by-step resolution actions.

### 1. YAML Frontmatter Parse Error
* **Severity**: Error (lint)
* **Diagnostic Message**: `Failed to parse YAML frontmatter: <syntax error details>`
* **Cause**: Invalid YAML syntax (e.g., unquoted string containing colons, bad indentation, tabs instead of spaces).
* **Resolution**: Fix the YAML formatting in the specified document. Use 2 spaces for indentation and wrap strings containing special characters in quotes.

### 2. Invalid Status Enum
* **Severity**: Error (lint)
* **Diagnostic Message**: `invalid status: <value> (allowed: draft|stable|deprecated|archived)` — may include `(did you mean \`draft\`? …)` for common aliases (`wip`, `done`, …)
* **Cause**: `status` in frontmatter is set to an unsupported value (e.g. `done`, `wip`, `COMPLETE`).
* **Resolution**: Change `status` to one of the four valid lowercase enums: `draft`, `stable`, `deprecated`, or `archived`.

### 3. Invalid Share Enum
* **Severity**: Error (lint)
* **Diagnostic Message**: `invalid share value: <value> (allowed: public|org|private)`
* **Cause**: `share` in frontmatter is set to an unsupported value.
* **Resolution**: Change `share` to one of the three valid lowercase enums: `public`, `org`, or `private`.

### 4. Profile Expected Section Warning
* **Severity**: Warning (lint)
* **Diagnostic Message**: `missing expected section: <Section>`
* **Cause**: Document frontmatter declares `profile: <name>`, but the Markdown body lacks one or more `## H2` section headings specified by that profile.
* **Resolution**: Add the required `## <Section>` heading to your document, or switch to `profile: note` if the document has a custom structure.

### 5. Dangling `depends` or `related` Reference
* **Severity**: Error (lint)
* **Diagnostic Message**: `dangling reference: <ref>`
* **Cause**: A path listed in `depends:` or `related:` does not resolve to an existing document file.
* **Resolution**: Fix the path spelling, update the reference to point to the renamed document, or remove the entry. Run `ods fmt --refs md-paths` to auto-normalize Document refs to relative `.md` paths.

### 6. Duplicate Document ID
* **Severity**: Error (lint)
* **Diagnostic Message**: `duplicate document id: <id>`
* **Cause**: Two distinct Markdown documents specify the exact same explicit `id:` in their frontmatter, or have conflicting path-derived IDs.
* **Resolution**: Remove the explicit `id:` override from one file (letting it use its relative path ID), or change the `id:` value to be unique.

### 7. Dependency Cycle Detected (`depends`)
* **Severity**: Error (lint)
* **Diagnostic Message**: `depends cycle detected: doc-a -> doc-b -> doc-a`
* **Cause**: Document A depends on Document B, which transitively depends back on Document A. `depends` forms a directed acyclic graph (DAG).
* **Resolution**: Remove the circular dependency. Move optional or bi-directional references from `depends:` into `related:`.

### 8. Missing Resource Path
* **Severity**: Error (lint)
* **Diagnostic Message**: `missing resource: <path>`
* **Cause**: A non-Markdown asset listed under `resources:` (`- path: assets/diagram.png`) does not exist on disk.
* **Resolution**: Check the file relative path and extension, move the asset to the expected location, or remove the `resources:` entry.

### 9. Missing Code Path or Invalid Role
* **Severity**: Error (lint for role, lint for path)
* **Diagnostic Message**: `missing code path: <path>` (or role validation message)
* **Cause**: A `code:` item specifies an unknown role or points to a source code file that does not exist.
* **Resolution**: Ensure `role` is one of the fixed roles (`entrypoint`, `implementation`, `test`, `schema`, `migration`, `config`, `infrastructure`, `pipeline`). Verify the target source file path.

### 10. Stale Index Bullet List
* **Severity**: Error (lint)
* **Diagnostic Message**: `index missing children: …` / `index has extra entries: …`
* **Cause**: Files were added, deleted, or renamed with broken graph refs.
* **Resolution**: Run `ods overview` / `ods find` to renavigate the document graph automatically.

### 11. Dangling Body Markdown Link
* **Severity**: Error (lint)
* **Diagnostic Message**: `dangling markdown link in body: <link>`
* **Cause**: Standard Markdown link in prose points to a non-existent file or relative path.
* **Resolution**: Update the target link path or create the missing file.

### 12. Tags nested under `ods:`
* **Severity**: Warning
* **Diagnostic Message**: `tags must be top-level frontmatter (not under ods:) …; run: ods fmt --migrate`
* **Cause**: Tags were placed under a nested `ods:` map (legacy). Use flat top-level keys in ODS 2.0 or run `ods fmt --migrate`.
* **Resolution**: Run `ods fmt --migrate` (hoists tags; never drops values).

---

## 3. Git Operations & Merge Conflict Resolution

### Merge Conflicts in `ods.toml`
Root `ods.toml` is the workspace marker (spec, ignore, packs, aliases, specs). Merges between branches can conflict on that file — resolve it like any TOML config, then re-validate.

**Resolution**:
```bash
# Resolve the conflict in ods.toml (ours/theirs or manual edit), then:
ods lint
git add ods.toml
```

### Reconciling Git Renames (`ods sync`)
If files were renamed using standard `git mv` or an IDE refactoring tool while `ods serve` / `ods watch` was **not** running in the background:

```bash
ods sync
ods lint
```
