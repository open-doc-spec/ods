# Token & context (locked)

1. Bounded context first: `ods context <id>` → read **only** those paths.
2. Empty/error context = stop; never full-tree dump or full graph export for Q&A.
3. Context walks **depends + top-level `load`** only (not `related`). `--include-code` is opt-in.
4. Prefer `--max-tokens N` / `--print` for prompt packs.
5. Product skill (`skills/ods`) is progressive — open `references/*` only when needed.
6. Docs must not claim unimplemented flags; relative multi-segment ids are first-class.
7. Export defaults under `.ods/` — not for routine agent prompts.
