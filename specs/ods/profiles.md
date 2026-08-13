---
description: "Standard document profiles, custom catalogs, ODS packs, and section header mapping."
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
  related:
    - core.md
    - validation.md
---

# ODS · Profiles

Profiles define the **nature** of a document (guide, decision, feature, …) and the H2 sections tools should expect. They are structural validation schemas, not copy-paste templates.

Key: `ods.profile` — see [keys.md](keys.md).

---

## 1. Layers

| Layer | Profiles | Tags |
| :--- | :--- | :--- |
| **Standard (spec)** | Built-in profiles (always available) | Suggested tags only (never required) |
| **Workspace (custom)** | `custom-profiles:` paths and imported **packs** | Observed top-level `tags:` values |
| Unknown values | SHOULD warn; fall back to **`note`** for section checks | MUST NOT error |

Rules:

- `ods.profile` is optional; default is **`note`**. On ordinary documents it lives under the nested `ods:` map.
- **Canonical registration:** root `ods.toml` `custom_profiles` lists profile definition files. Legacy root-index `profiles:` / `custom-profiles:` MAY still be accepted during migration.
- **No folder auto-discovery:** tools MUST NOT implicitly scan `ods-profiles/` or `.ods/profiles/` unless those paths are listed or imported via packs.
- An **ODS Pack** is a versioned directory/repo with reusable profiles (`ods-profiles/`), skills, or templates. Remote packs sync under `~/.ods/packs/`; local packs use relative paths.
- Resolution order: Standard → explicit `custom-profiles:` paths → packs.
- Duplicate profile names: first wins with a warning; Level-3 SHOULD error.
- Profile catalogs are workspace utilities—they do not join the document graph.
- Profile id comes from definition frontmatter `name:` or the file stem.
- H2 headings in a profile definition declare expected sections; pipe alternatives allowed: `## Goal | Objective | Purpose`.
- Unrecognized profile → warning; treat as `note` until defined.
- Custom profiles are additive only—no inheritance hierarchies.

Root example:

```yaml
---
profile: index
ods: 0.1
custom-profiles:
  - docs/profiles/rfc.md
  - docs/profiles/api_endpoint.md
packs:
  - vendor/engineering-pack
---
```

---

## 2. Standard profiles

| Profile | Meaning | Expected sections |
| :--- | :--- | :--- |
| `note` | Free-form prose (default) | none |
| `agent` | AI instruction / task contract (`agent.md`) | Goal, Task, Scope, Non-Scope, Context, Inputs, Constraints, Priority, Steps, Output, Success Criteria, Failure Modes, Dependencies, Assumptions, Examples |
| `feature` | Capability / PRD-style | Goal, Scope, Requirements, Acceptance Criteria, Risks |
| `guide` | Tutorial / how-to | Overview, Prerequisites, Steps, Troubleshooting |
| `api` | Interface reference (artifact stays a resource) | Overview, Request, Response, Errors, Examples |
| `architecture` | System structure and why | Overview, Components, Data Flow, Trade-offs |
| `decision` | ADR / RFC outcome / policy choice | Context, Decision, Alternatives, Consequences |
| `sop` | Runbook / procedure | Purpose, Prerequisites, Steps, Validation, Rollback |
| `policy` | Rules people must follow | Purpose, Scope, Rules, Exceptions |
| `meeting` | Meeting notes | Attendees, Agenda, Decisions, Action Items |
| `faq` | Q&A | question/answer pairs; no fixed H2 list |
| `checklist` | Verifiable gates | Overview, Items, Verification, Notes |
| `index` | Navigation files | none |

There is no standard `specs` profile. Specification documents use `note`, `decision`, or `guide` by intent.

---

## 3. Workspace tag operations

`tags` is **universal top-level** frontmatter ([keys.md](keys.md)). MUST NOT appear under `ods:`.

- Free-form strings; normalize to lowercase.
- SHOULD NOT collide with lifecycle statuses or profile names.
- Tooling SHOULD support `ods tags`, `ods tag list` / `ods tag show`, find-by-tag and find-by-key (`ods find --tag` / `--key`), and `ods tag rename`.

---

## 4. Section aliases

| Canonical | Recognized aliases |
| :--- | :--- |
| Goal | Objective, Objectives, Purpose |
| Scope | In Scope, Boundaries |
| Requirements | Functional Requirements, Needs |
| Acceptance Criteria | Acceptance, Success Criteria, Definition of Done |
| Overview | Introduction, Summary, Background |
| Prerequisites | Requirements, Before You Begin |
| Steps | Instructions, Procedure, Process |
| Troubleshooting | Common Issues, FAQ |
| Context | Background |
| Alternatives | Options, Options Considered |
| Consequences | Outcome, Implications |
| Validation | Verification, Checks |
| Rollback | Recovery, Revert |
| Rules | Standards, Requirements |
| Action Items | Actions, Next Steps, TODO |
| Risks | Risks and Mitigations, Concerns |
| Trade-offs | Tradeoffs, Pros and Cons |
