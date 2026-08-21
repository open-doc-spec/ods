---
name: multi-spec
description: >-
  ODS vs OKF vs Agent Skills flags, key placement, and hybrid workspace rules
  for the ods CLI.
---

# Multi-spec (maintainer)

| Dialect | Flag | Root marker | Keys home |
|---|---|---|---|
| ODS | *(none)* | `ods.toml` `spec = "0.1"` | `specs/ods/keys.md` |
| OKF | `--okf` | `okf_version: "0.2"` | `specs/okf/keys.md` |
| Skills | `--skills` | `SKILL.md` package | `specs/skills/keys.md` |

## Hard rules

- No `--ods` flag
- No `ods okf` / `ods skills` namespaces
- Universal ODS keys top-level; engine under `ods:`
- OKF `status` is top-level; ODS lifecycle is `ods.status` — do not conflate
- Hybrid: bare lint is ODS-first unless `specs.okf.enabled` / flags
- Key catalogs for all dialects: `src/ods-core/src/spec/schema.rs` (not scattered match arms)

## Commands

```bash
ods lint
ods lint --okf
ods lint --skills
ods init --okf ./bundle
ods init --skills ./my-skill
ods schema              # ODS JSON Schema from registry
ods schema --okf        # OKF key list from registry
ods schema --skills
```

Comparison doc: `docs/other-specs/frontmatter-keys-ods-vs-okf.md`  
CLI surface: `docs/other-specs/cli-multi-spec.md`  
Schema how-to: `docs/maintainer/schema-driven-keys.md`
