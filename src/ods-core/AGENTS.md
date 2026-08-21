# AGENTS.md — `src/ods-core`

- Domain folders: `graph/`, `lint/`, `parse/`, `index/`, `okf/`, `multi_spec/`, `spec/`, `error/`, …
- Functional style: pure data + free functions; IO at edges
- **User-facing messages SoT:** `error/messages.rs` (CLI lifecycle + high-volume lint diagnostics)
- Multi-spec: `multi_spec/` + flags only (no namespaces); `ScopeResolveError::message()` delegates to the catalog
- Frontmatter model: universal top-level tags; engine under nested `ods:`
- Navigation indexes (`index.md` / `index.ods.md`) are optional; they are not the workspace marker
- Root marker: `ods.toml` with `spec`; custom profiles key `custom_profiles`
- Spec docs: `specs/ods/` (not a runtime dependency — keep behavior aligned)

## Keys = schema registry

- **Source of truth:** `spec/schema.rs` → `SpecSchemaRegistry` (ODS / OKF / Skills + custom profiles)
- `validate_ods_frontmatter` + `generate_ods_json_schema` are registry-driven
- Do not reintroduce dead parallel key tables; unknown keys stay preserved
- Guide: `docs/maintainer/schema-driven-keys.md`
- Tests: unit tests in `schema.rs`; integration under `tests/` for lint enums + multi_spec
