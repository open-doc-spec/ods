# AGENTS.md — `src/ods-core`

- Domain folders: `graph/`, `lint/`, `parse/`, `index/`, `okf/`, `multi_spec/`, `spec/`, `error/`, …
- Functional style: pure data + free functions; IO at edges
- **User-facing messages SoT:** `error/messages.rs` (CLI lifecycle + high-volume lint diagnostics)
- Multi-spec: `multi_spec/` + flags only (no namespaces); `ScopeResolveError::message()` delegates to the catalog
- Frontmatter model: **flat top-level keys** (ODS 2.0); engine keys are not nested under `ods:`
- Workspace default: `spec = "2.0"` in `ods.toml`
- Index canonical name: prefer `index.ods.md` (also accept `index.md`)
- **`ods.toml` `[service]`** — `max_rss_mb = 10` soft budget for `ods serve`/`watch`; graph commands load frontmatter only (`include_body: false`). Set `ODS_LOW_MEMORY=1` for single-thread parse; `ODS_MEM_REPORT=1` prints `rss_kb` on exit (regression tests).
- **Memory module:** `memory/mod.rs` — RSS sampling, `strip_workspace_bodies`, `DEFAULT_MAX_RSS_MB`
- Spec docs: `specs/ods/` (not a runtime dependency — keep behavior aligned)

## Keys = schema registry

- **Source of truth:** `spec/schema.rs` → `SpecSchemaRegistry` (ODS / OKF / Skills + custom profiles)
- `validate_ods_frontmatter` + `generate_ods_json_schema` are registry-driven
- Do not reintroduce dead parallel key tables; unknown keys stay preserved
- Guide: `docs/maintainer/schema-driven-keys.md`
- Tests: unit tests in `schema.rs`; integration under `tests/` for lint enums + multi_spec
