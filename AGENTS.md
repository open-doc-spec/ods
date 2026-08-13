# AGENTS.md

<!-- hand-maintained: do not overwrite with `ods agents sync` at this repo root -->

Rules for coding agents in the **Open Document Spec** monorepo.

## Product locks (always)

- Binary: **`ods`** — bare `ods <cmd>` is ODS (there is **no** `--ods` flag)
- Extra specs: **`--okf`**, **`--skills`** only — never namespaces `ods okf` / `ods ods`
- Workspace marker: **`ods.toml`** at repo root (not nested `index.ods.md`)
- Compliance: **compliant | non-compliant** only (no Level 0–3)
- Discovery: `ods overview` / `find` / `ls` / `tree` / `context` / `read` — progressive CLI, not folder indexes
- Editors: **`ods lsp`** · Watcher: `ods serve` / `ods start` (not LSP); serve target **≤10 MB** RSS (`service.max_rss_mb`)
- Specs live in satellite repo [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec)
- Subcommands: name after the verb (`ods profile init <name>` → name at argv index **3**)

## Keys are schema-driven

Engine SoT: `src/ods-core/src/spec/schema.rs`. Details: `.agents/rules/30-schema-keys.md`.

## User-facing errors are catalog-driven

CLI/engine user copy lives in **`src/ods-core/src/error/messages.rs`** (not ad-hoc `failure("…")` strings in commands).

- Shape: `error:` / `usage:` one-liner + `Next:` directive (optional `Hint:`)
- Exit: usage **2**, failure **1**
- When adding a failure path: add/reuse a catalog builder, then call `fail_msg` / `usage_msg` from the CLI


## Token & context reliability

- Cold-start: `ods overview` → `ods tag list` / `ods schema keys` → `ods find --key …` → `ods context <id>`.
- Prefer `ods read <id>` (`--section <heading>`, `--summary`, `--max-tokens N`) or `ods context <id>` (`--max-tokens N`, `--print`). Read **only** returned sections or paths.
- Context walks **depends + context.load** (not `related`). `--include-code` is opt-in.
- If context errors: **stop** — use `ods find <query>` / `--key`; do not dump the repo or full graph export.
- `ods export` defaults to `.ods/graph.md` (not routine AI prompts).
- Do not load `skills/ods/references/*` and `specs/ods/*` duplicates in one turn.

## Definition of Done

Do **not** claim CI-safe / commit-ready without:

1. `SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh` → exit 0  
2. If Rust/tests changed: `ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh`  
3. Keys touched → registry + `specs/*/keys.md` + tests aligned  
4. User-visible → release-docs skill checklist  

Full policy: `.agents/rules/40-quality-gates.md`. Multi-step work: `.agents/skills/quality-gate/SKILL.md`.

## Cheap handoff

```bash
.agents/hooks/scripts/pre-handoff.sh          # naming + odc residue
.agents/hooks/scripts/pre-handoff.sh --full   # + check-local
```

## Ownership

| Path | Owner |
|---|---|
| **This file** | Hand-maintained (this monorepo). **Do not** run `ods agents sync` at repo root — it skips overwrite when `.agents/rules/` exists. |
| `.agents/rules/*` | Always-on short rules |
| `.agents/skills/*` | Task skills (maintainer) |
| Satellite: `ods-skills` | **End-user** product skill: [open-doc-spec/ods-skills](https://github.com/open-doc-spec/ods-skills) |
| Satellite: `ods-spec` | Normative docs: [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec) |
| Satellite: `opendocify.com` | Site: [open-doc-spec/opendocify.com](https://github.com/open-doc-spec/opendocify.com) (**private**) |
| Satellite: `ods-benchmarks` | Fixture benchmarks: [open-doc-spec/ods-benchmarks](https://github.com/open-doc-spec/ods-benchmarks) |
| Satellite: `ods-action` | GitHub Action: [open-doc-spec/ods-action](https://github.com/open-doc-spec/ods-action) |
| Nested `**/AGENTS.md` | Crate/folder specifics |

Satellite map: `docs/maintainer/satellite-repos.md`.

## Useful commands

| Goal | Command |
|---|---|
| Local gate | `./src/scripts/check-local.sh` |
| Coverage ≥90% | `./src/scripts/coverage.sh` |
| Schema smoke | `./src/scripts/check-schema-keys.sh` |
| Lint ODS/OKF/Skills | `ods lint` / `--okf` / `--skills` |
| Context | `ods context <id> [--max-tokens N] [--print]` |
| Find docs | `ods find [--tag t] [--key k] [query]` |
| Tag catalog | `ods tag list` / `ods tag show <tag>` |
| Schema keys | `ods schema keys` |
| Workspace overview | `ods overview` |
| JSON Schema | `ods schema` |

Maintainer entry: `.agents/README.md`.
