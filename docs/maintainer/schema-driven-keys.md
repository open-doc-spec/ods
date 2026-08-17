# Schema-driven keys (maintainer)

## Source of truth

`ods-core` → `src/ods-core/src/spec/schema.rs` → `SpecSchemaRegistry`

| Dialect id | Flag / default | Normative docs |
|---|---|---|
| `ods` | default (no flag) | `specs/ods/keys.md` |
| `okf` | `--okf` | `specs/okf/keys.md` |
| `skills` | `--skills` | `specs/skills/keys.md` |
| custom profiles | `ods.custom_profile.required_keys` on profile definitions | `specs/ods/profiles.md` |

## What is schema-driven today

- **Key catalog** for all three dialects (placement, type, required, description, aliases)
- **Lint** enum checks for ODS `status` / `share` via `validate_ods_frontmatter`
- **`ods schema`** JSON Schema emission from the registry (`ods schema`; `ods schema --okf` / `--skills` list keys)

Parse still fills a typed `Frontmatter` (complex nested `code`/`resources`/`context`). Unknown keys remain preserved.

## Adding or changing a key (lightning path)

1. Edit `register_*_schema()` in `schema.rs` (or add `register_<dialect>_schema()`).
2. If parse must populate a typed field, extend `Frontmatter` + parse once.
3. Add a unit test in `schema.rs` tests + one integration lint/CLI test.
4. Update `specs/<dialect>/keys.md` (and skill references if ODS public keys).
5. Run `cargo test -p ods-core --lib spec::schema` and `ods lint` on fixtures.

## Adding a new dialect

1. `register_<name>_schema()` + include in `with_defaults()`.
2. Thin engine module under `multi_spec/` or sibling (detect/lint/init).
3. CLI flag in `ExtraSpecs` / `parse_extra_spec_flags` (never `--ods`).
4. `specs/<name>/{intro,keys}.md` + guide touchpoints.
5. Tests: schema completeness, lint, CLI flag matrix.

Do **not** grow new giant `match key` tables in parse/lint for catalog metadata.
