---
description: "AI agent reading patterns, multi-workspace tracking, daemon execution modes, and large workspaces."
ods:
  profile: "note"
  status: "stable"
---

# Advanced Usage

## How Agents Should Read ODS Workspaces

1. Start at the root `ods.toml`.
2. Descend through child indexes toward the target document.
3. Read the target frontmatter first.
4. Load `depends` transitively up to `context.max-depth` (default 2).
5. Load top-level `load` entries: Document `.md` paths first, then Resource paths.
6. Skip paths matching `context.ignore` and workspace `ignore:`.
7. Prefer `stable` over `draft`, and `draft` over `deprecated` or `archived`.

That gives agents a deterministic path instead of a broad scan.

Use the CLI to debug the resolution:

```bash
ods context path/to/doc.md
```

---

## Global Multi-Workspace Tracking (`ods workspaces`)

ODS background daemons (`ods start` / `ods serve`) monitor multiple repositories registered in the user's global machine configuration file (`~/.ods/odsconfig.toml`):

```bash
# Register a workspace for background tracking
ods workspaces add /path/to/my-project

# List all tracked workspaces and pack entries
ods workspaces list

# Remove a workspace from tracking
ods workspaces remove /path/to/my-project

# Print machine config file path (~/.ods/odsconfig.toml)
ods workspaces path
```

---

## Daemon Execution Modes & Memory Tuning (`ods serve`)

`ods serve` supports three execution modes for handling filesystem updates across workspaces:

1. **`auto`** (default): Automatically selects `watch` mode, unless `ODS_LOW_MEMORY=1` is set in the environment, in which case it uses `poll` mode.
2. **`watch`**: Real-time OS event watcher (fsnotify/kqueue/inotify) for immediate rename mapping and index regeneration.
3. **`poll`**: Periodic polling interval (default 10s). Keeps a long-lived graph workspace between ticks; strips document bodies when RSS exceeds `[service].max_rss_mb` (default 10 MB).

```bash
# Low-memory polling configuration with RSS memory diagnostics
ODS_LOW_MEMORY=1 ODS_POLL_SECS=30 ods serve --mode poll --memory-report --root .
```

---

## Large Workspaces

Guidance for tens of thousands of Markdown files:

- Keep profiles small; register custom catalogs under `custom_profiles` in root `ods.toml`.
- Prefer path-derived IDs; pin explicit `id:` only when renames must not break external refs.
- Run `ods lint` and `ods lint` in CI rather than trusting hand-edited maps.
- Use `ods lint` while onboarding noisy trees; enable body-link and graph checks at lint when ready.
- Tooling skips `node_modules`, `target`, and similar trees by default and may honor `.gitignore`.
- Exclude implementation trees with root `ignore:` (for example `src`).
