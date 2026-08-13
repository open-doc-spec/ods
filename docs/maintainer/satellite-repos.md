---
profile: note
status: draft
share: public
description: First-cut mapping of monorepo trees to open-doc-spec satellite repositories.
---

# Satellite repositories

Product surfaces have been extracted into dedicated repositories under
[github.com/open-doc-spec](https://github.com/open-doc-spec). This monorepo remains
the **engine / CLI** source of truth (`ods`).

## Policy

| Rule | Detail |
|------|--------|
| Satellite SoT | Satellite repositories are the **sole source of truth** for their respective surfaces |
| Monorepo mirrors | In-tree mirrors have been **removed** |
| Install scripts | Canonical: monorepo `src/scripts/install.{sh,ps1}` — sync into site/skill repositories |

## Mapping

| Surface | Extracted Satellite (SoT) | Visibility |
|---------|---------------------------|------------|
| Site / domain app | [open-doc-spec/opendocify.com](https://github.com/open-doc-spec/opendocify.com) | **Private** |
| Normative specs | [open-doc-spec/ods-spec](https://github.com/open-doc-spec/ods-spec) | Public |
| Benchmark fixtures | [open-doc-spec/ods-benchmarks](https://github.com/open-doc-spec/ods-benchmarks) | Public |
| End-user skill | [open-doc-spec/ods-skills](https://github.com/open-doc-spec/ods-skills) | Public |
| GitHub Action | [open-doc-spec/ods-action](https://github.com/open-doc-spec/ods-action) | Public |
| Engine / CLI | **This repo** (`ods`) | Public |
| Org defaults | [open-doc-spec/.github](https://github.com/open-doc-spec/.github) | Public |

## Notes

- **Specs vs schema**: Markdown in `ods-spec` is normative documentation.
  Runtime keys remain schema-driven in `src/ods-core/src/spec/schema.rs`.
- **Benchmarks**: Fixtures live in `ods-benchmarks`; `ods bench` engine stays here.
- **Action consumers**: Use `uses: open-doc-spec/ods-action@v1`.
- **Skill**: Maintained in `open-doc-spec/ods-skills`.

Tracking parent: [open-doc-spec/ods#34](https://github.com/open-doc-spec/ods/issues/34).
