
# Functional style for Open Document Spec (`ods-core` / `ods`)

Prefer **data + free functions** over OOP service objects.

## Rules

1. **Data is inert** — `Document`, `Workspace`, `Diagnostic` hold values only.
2. **Behavior is free functions** — `load_workspace`, `lint_workspace`, `parse_paths_parallel`, `apply_document_upserts`.
3. **Pipelines** — discover → parse → index → lint → report.
4. **Effects at the edge** — FS/stdout live in loaders and CLI; pure stages take `&Workspace` / docs in and return values out.
5. **No new** `*Manager` / `*Service` / `*Factory` types with mutable self as the primary API.
6. **`impl` only** for trivial getters, `Default`, `Display`, small inherent helpers.
7. **Mutation** only inside explicit `apply_*` (watch incremental) or `rebuild_indexes`.

## Scale (10K)

- Parallel parse via `rayon` (`ODS_JOBS` to cap threads, legacy fallback `ODC_JOBS`).
- Graph ops use `load_options_graph()` (`include_body: false`; streaming parse keeps frontmatter + `##`/`###` lines only).
- Watch holds one `Workspace` and applies dirty paths — no double full reload per tick.
- No disk DB/cache required for ≤~10K.

## Modules

- `ods-core/src/pipeline/` — discover, parse_stage, apply
