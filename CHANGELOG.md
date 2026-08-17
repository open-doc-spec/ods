# Changelog

All notable changes to **Open Document Spec (`ods`)** are documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/),
and this project follows [Semantic Versioning](https://semver.org/).

GitHub Releases use GitHub’s auto-generated notes. Edit this file by hand when useful.

## [Unreleased]

### Breaking
- Workspace marker is **`ods.toml`** only (no nested `index.ods.md` / no ODS `ods index` generation).
- **`ods alias` / `ods aliases` and `ods.toml` `[aliases]` removed.** Profile section titles are exact (plus pipe-alternatives in the profile file). Help no longer lists those commands.
- Compliance is **compliant | non-compliant** (no Level 0–3 ladder).
- `ods serve` product RSS budget default **10 MB** (`service.max_rss_mb`).
- Skill `references/{intro,keys,core,scope}` are pointers to `specs/ods/*` (no forked copies).

### Fixed
- **Non-destructive frontmatter:** `ods fmt --migrate` preserves unknown keys nested under `ods:` (no longer drops foreign nested blocks). Third-party top-level keys (Hugo/Astro/etc.) remain untouched on migrate, tag rewrite, spacing normalize, and disable strip.
- **CLI honesty after index removal:** help/completions no longer claim ODS `index` lockfile generation; `ods lint --fix` documents no-op (discovery is `overview`/`find`/`tree`); error catalog points at **`ods.toml`** not root `index.ods.md`.
- Removed accidental committed workspace graph dumps (`graph.md`, `src/ods-cli/graph.md`).
- `ROOT_ODS_KEYS` strip list no longer duplicated `ods`; includes packs/specs for legacy root-policy disable.
- Docs/skill residual “regenerate indexes” / root-index wording cleaned; profile CLI split into `profile/{inspect,init,aliases,tests}.rs`; workspace config keys registered as `WorkspaceConfigOnly`.
- **Schema-driven disable strip lists** (`document_disable_strip_keys` / `workspace_policy_strip_keys`); shared `ChildGuard` for serve/watch test teardown (no leaked processes).

### Added
- **`ods read [root] <id-or-path>`:** Fine-grained section extraction (`--section <heading>`), outline summary (`--summary`), and soft token cap controls (`--max-tokens N`, `--format text|json`). Prevents path traversal out of workspace.
- **`ods find --key <expr>` & multi-criteria search:** query documents by schema keys and custom profile keys (`--key`, `--key-match and|or`, `--tag-match any|all`, `--status`, `--profile`, `--owner`). Supports comma multi-values (`--key status=draft,stable`), comma multi-keys, and simple `AND`/`OR` expressions. Value match is **exact** (case-insensitive).
- **`ods tag list` & `ods tag show <tag>`:** list observed workspace tags with document counts or inspect documents carrying a specific tag (`--format text|json`). (`ods tags` / `ods tag rename` unchanged.)
- **`ods schema keys`:** inspect registered schema key definitions, placements (`TopLevel`, `NestedEngineMap`), key types, and descriptions in text or JSON. Bare `ods schema` still exports JSON Schema.
- **`ods overview` (alias: `ods summary`):** compact workspace snapshot (document counts, profile/status breakdown, top tags, custom keys, graph statistics) for AI cold-start. Use `ods stats` for lint health %.
- **`ods context` filter fallback:** when the positional id is omitted, `--tag` / `--key` / `--status` may resolve a target **only if the match is unique**; multi-match fails with a short id list and `Next: ods find …`. Classic `ods context <id>` is unchanged.
- **`ods profile init --register` (default):** scaffolds `.ods/profiles/<name>.md` and appends it under root `custom-profiles:` (use `--no-register` to skip). **`ods profile show <name>`** prints layer, sections, and required/optional/forbidden keys.
- **`ods status <path-or-id> <draft|stable|deprecated|archived>`** lifecycle setter; **`ods archive`** remains an alias for `status … archived`.
- **`ods context --explain`** / **`--include-related`**; hybrid **`--okf`** merges OKF link neighborhood after ODS depends/load; respects root `specs.okf.enabled`.
- **`ods undo --list`** lists machine backup snapshots; help clarifies undo is snapshot/bench restore, not full git undo.
- Guide clarity: multi-spec flag rules in quickstart; context depends/related/load recipe; packs v1 = profile catalogs (honest scope); `--okf` command matrix in CLI help.

### Changed
- **Repo ownership URLs:** install, self-update, SARIF `informationUri`, and README now point at **`open-doc-spec/ods`** (Action: `open-doc-spec/ods-action`).
- **Production CLI help:** `ods` / `ods --help` groups every command by task with usage, examples, and environment notes for writers and engineers. `ods help <command>` and `ods <command> --help` share one catalog (arguments, flags, examples, see-also). `lsp`, `watch`, `serve`, `start`, and `logs` now print help and exit instead of blocking.
- **Directive CLI errors (catalog):** user-facing failures live in `src/ods-core/src/error/messages.rs`. First-call shape is `error:`/`usage:` + `Next:` (optional `Hint:`). Full CLI long-tail (argv/load/mutate/service/pack/bench/update/…), ODS **+ OKF + Skills** lint diagnostics, and service/update failures use the catalog; bare `failure(e.to_string())` / free-form usage dumps removed from command paths. Guide `07-troubleshooting`, skill, and agent rules updated.
- **Agent instructions:** root `AGENTS.md` (hand-maintained; `ods agents sync` will not clobber when `.agents/rules/` exists), rules `30-schema-keys` / `40-quality-gates`, skill `quality-gate`, pre-handoff hooks, and `check-schema-keys` smoke for reliable future iterations.
- **Schema-driven keys:** `SpecSchemaRegistry` now registers full **ODS + OKF + Skills** key catalogs (aligned with `specs/*/keys.md`). Lint enum checks (`status`/`share`) and `ods schema` JSON emission are driven from the registry so adding/updating dialect keys is a schema change + tests.
- **CI coverage floor** raised **88 → 90** lines (T3 excludes unchanged). Local: `ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh`.
- **Legacy `odc` residue gate:** `./src/scripts/check-odc-residue.sh`; test fixtures no longer embed `odc:` pins; install scripts install **ods** only (legacy env dual-read for `ODC_*` kept where noted).
- **Specs IA overhaul:** multi-spec tree under top-level **`specs/{ods,okf,skills}/`** (single location; former `src/specs` + root symlink removed). ODS modules renamed to one-word files (`core`, `assets`, `scope`, …) with new end-user **`intro.md`** and frontmatter **`keys.md`**. Site routes are `/spec/ods/...`, `/spec/okf/...`, `/spec/skills/...` (legacy flat `/spec/spec` etc. retired). `docs/other-specs/` is guides/comparisons only.
- **Test coverage elevation (production readiness):** workspace line coverage raised from ~**76.8%** raw / ~**77%** with T3 excludes toward the **≥90%** bar (T3-excluded). CI floor **73 → 88 → 90** using shared T3 `--ignore-filename-regex` (network download, OS service install, long-running watch, GitHub release client). New tests: multi-spec/skills engine, graph JSON export, LSP protocol surface, CLI agents/schema/stats/tree/clean/completion/audit/pack/upgrade/bench, schema-driven enum lint, and high-ROI core unit tests. Reports via `./src/scripts/coverage.sh` → `.artifacts/coverage/`. See `docs/maintainer/coverage.md`.
- **CLI multi-spec UX (flag-only):** ODS is the default native engine (no `--ods` flag). Extra specs use `--okf` and `--skills` only.
- **Bare hybrid lint** runs **ODS only**; pass `--okf` to also lint OKF. Pure OKF trees require `ods lint --okf`.
- **Agent Skills:** native parse/lint/init via `ods init --skills` and `ods lint --skills`.
- **LSP:** ranges, `didClose`, multi-spec hover/completion, `source` tags; `ods setup --editor`.
- **Pack init** no longer writes legacy `odc:` pin keys.
- **Docs/skill/agents/specs/fixtures** aligned to flag-only surface; `odc:` stripped from test fixtures.
- **Single `SpecKind`** shared by schema registry + descriptors; `profiles` module folderized.
- **`ods-core` layout:** semantic folders for all major domains (`graph/`, `mutate/`, `model/`, `fs/`, `lint/`, `index/`, `lifecycle/`, `mv/`, `parse/`, `tags/`, `share/`, `bench/`, `profiles/`, `multi_spec/`); public API re-exports preserved.
- **CI coverage floor** raised 70 → **73** lines (measured workspace ~74.65%).
- **Install / smoke / bootstrap / skill scripts** use bare `ods` + `--okf` / `--skills` only (no namespaces).
- **Machine config** path: write `~/.ods/odsconfig.toml`; load prefers modern file and still reads legacy `odcconfig.toml` if only that exists.
- **Crate layout:** packages moved from `src/crates/*` to `src/ods-core`, `src/ods-cli`, `src/ods-test-support`.
- **Dead code:** removed unused `SpecKeyProcessor` / parallel `SpecDescriptor` key tables; keep `SpecSchemaRegistry` only.

### Fixed
- **`ods context` token-waste regression:** `find_workspace_root` no longer collapses relative document ids (e.g. `specs/ods/core`) to an empty root, which made `ods context <id>` silently return **zero paths** (exit 0) and pushed agents to dump full trees / full graph exports. Roots are absolutized before ancestor walks; empty paths are rejected.
- **`ods context` CLI:** document id is not treated as the workspace root; supports `ods context <id>`, `ods context <workspace> <id>`, and `ods context --root <dir> <id>`. Missing ids now **error** (non-zero) instead of silent empty success. Query matching accepts path-shaped ids, `.md` paths, unique stems, and absolute paths under the workspace.
- **Agent skill / install templates:** stop recommending full `ods export graph` for routine AI prompts; Cursor/Windsurf rules prefer bounded context only (no always-on encyclopedias).
- **End-user friction wave:** `--max-tokens` / `--print` / `--include-code` on context; private-skip warnings; export defaults to `.ods/graph.md`; skill install ships current references (no evals); `ods find` by id/path/stem; honest `lint --fix` message; status “did you mean” hints; FM `title:` warning; real `resolve_context` bench averages; `--help` on context/lint/find; help text uses `index.ods.md`.
- Workspace error messages reference `~/.ods/odsconfig.toml` (not `odcconfig`).
- JSON Schema status enum aligned to SPEC: `draft|stable|deprecated|archived`.
- Hybrid workspaces: bare lint/doctor/audit are **ODS-only**; pass `--okf` to include OKF.
- `ods watch` log banner / `ods logs` path; `ods archive` nested status; coverage report path.

### Added
- **`multi_spec` engine selection** — `ExtraSpecs` / `Detected` / `resolve_engines`.
- **Agent Skills engine** — name/description/license/compatibility/metadata/allowed-tools.
- **`ods agents sync`** — AGENTS.md + editor snippets.
- **OKF via flags** — init/lint/index/context/export/fmt/doctor/audit/adopt/watch/serve `--okf`.
- **`ods setup --editor zed|vscode|nvim|cursor`** — write `ods lsp` config.

### Removed
- **`ods okf <cmd>` namespace** — hard-removed; use `ods <cmd> --okf` only.

### Breaking (historical rename notes)

- Root CLI pin key renamed: `ods-cli:` → **`ods:`** (tool is Open Document Spec). Spec marker remains `ods:`. OKF remains `okf_version:`.
- Lint health report path: root **`ods-error.md`** → **`.ods/ods-errors.md`**.
- Machine config / logs prefer **`~/.ods/`**.
- Legacy binary **`ods`** still ships (same code); bare `ods lint` ≡ ODS engine only.
- `ods audit` (OKF: `ods audit --okf`) — compliance inventory; `--write-report` → `.ods/ods-errors.md`
- Plans & key map: `docs/plan/archive/odc_tool_keys_legacy_cleanup.md`, migration + OKF plans, `docs/other-specs/frontmatter-keys-ods-vs-okf.md`
- `ods bench` — benchmarking and frontmatter snapshot/restore system (`stats`, `strip`, `restore`, `run`). Allows teams to take machine-level JSON snapshots (`~/.ods/backups/<repo-hash>/`), temporarily strip frontmatter, index lockfiles, and profiles to test AI task performance without ODS, restore workspace artifacts losslessly (`ods bench restore`), and calculate token & API cost ROI metrics (~94% token savings). Added `--full` flag for complete baseline isolation.
- `ods workspaces` — manage globally tracked ODS workspace paths (`add`, `remove`, `list`, `path`) stored in human-readable TOML at `~/.ods/odsconfig.toml` (legacy `~/.ods/workspaces.toml` is auto-migrated on read).
- `ods share [path] --out DIR` — publish a `share`-filtered copy of a workspace or subtree as a real directory (`--include-org`, `--include-private`), ready to `git init`/push yourself. `share:` set on any `index.md` now also acts as a directory-level default that cascades to descendants (a document's own `share` still wins).

- **Comprehensive Workspace Test & Line Coverage Elevation**:
  - Elevated workspace line coverage from **78.84%** baseline to **82.31%** across 11,900 lines with **100% green test execution** (`cargo test --workspace` passing 200+ unit & integration tests).
  - Target core engine modules (`ods-core`) elevated to **90%–100%** line coverage:
    - 100%: `okf/audit`, `okf/model`, `pipeline/apply`
    - 95–99%: `okf/lint` (99.40%), `share` (99.05%), `model` (99.03%), `adopt` (98.80%), `parse/frontmatter` (98.14%), `fs/scanner` (97.50%), `profiles` (96.58%), `tags/suggestions` (96.46%), `mv/migrator` (96.12%), `okf/bundle` (95.83%), `pipeline/discover` (95.65%)
    - 90–94%: `okf/init` (94.74%), `mv/healer` (94.12%), `tags/aliases` (93.33%), `export` (92.86%), `lifecycle/init_and_disable` (91.91%), `lint/checker` (91.59%), `index/generator` (91.56%), `okf/index` (90.46%)
  - Added unit and integration tests covering:
    - Profile scaffolding templates (`decision`, `sop`, `api`, `meeting`, `faq`) & AlreadyExists/NotFound error paths (`scaffold_and_remove.rs`)
    - Custom hand-authored index profile preservation during index pruning (`index/generator.rs`)
    - Root ODS/ODS metadata validation and non-root ODS key prohibition (`lint/canonical.rs`)
    - Resource indexing and single-quoted ODS version frontmatter unquoting (`index/checker.rs`)
    - Code path line-number suffix error validation and extra index entry warnings (`lint/helpers.rs`)
    - `okf doctor` (text/JSON), `okf audit --format json`, `okf adopt --write`, `okf index --check` (`okf_commands.rs`)
    - `tag rename` text & JSON output formatters (`tag_command.rs`)
    - `workspaces list` text & JSON output formatters (`workspaces_command.rs`)
    - `pack add` and `pack rm` (`pack_command.rs`)
    - Git status porcelain rename detection inside git repositories (`update_command.rs`)
    - `PathChangeReport` human-readable summary and issue detection (`mv/remover.rs`)
- **Test coverage:** policy in `docs/maintainer/coverage.md`; scripts `coverage.sh` + `coverage-100-check.sh`; CI floor **75%** lines. New tests: OKF audit/model/init/parse (full surface), pipeline apply, model `CodeRole`/`ods` pin, CLI okf/upgrade/share/export/help. Engine (`ods-core`) ~**84%** lines; first modules at **100%**: `okf/audit`, `okf/model`, `pipeline/apply`.
- **10K-oriented performance (no disk cache):**
  - Functional pipeline modules (`ods-core/src/pipeline/`): discover → parallel parse (`rayon`) → index.
  - Graph commands (`lint`, `doctor`, `index`, `context`, `graph`, `find`, `tags`, `profiles`) load with `include_body: false` (note bodies dropped; **`index.md` bodies kept** for child-list rules).
  - `ods watch` / `serve` keep a long-lived workspace and reparse **dirty paths only** (full reload only on first tick or large dirty sets). `ODC_JOBS` caps parse threads.
  - Lint report `.ods/ods-errors.md` capped at 500 diagnostics + summary.
- **Codebase Refactoring & <300 Line File Modularization**:
  - Restructured and modularized 100% of Rust files in `src/` (`ods-core` & `ods`) to be strictly under 300 lines of code (0 files exceeding 300 lines).
  - Extracted sub-modules semantically across test suites, entry dispatchers, configuration helpers, watch handlers, and rename algorithms (`entry_dispatch.rs`, `okf_watch.rs`, `agents_command.rs`, `pack_subcommands.rs`, `rename_pairing.rs`, `classifier_rewriter.rs`, `workspaces_config.rs`, `lint_tests.rs`).
  - Renamed generic test file names to explicit semantic names (`parse_refs_and_links.test.rs`, `cli_service_lifecycle.test.rs`, `cli_frontmatter_commands.test.rs`, `cli_workspaces_command.test.rs`).
  - Resolved all merge conflicts with `origin/main` branch (`v0.0.5`), aligning lockfiles, documentation, and dependencies.
- Auto-update respects `ODC_AUTO_UPDATE` as well as `ODS_AUTO_UPDATE`.

## [0.0.1] - 2026-07-19

### Added

- `ods setup` — first-run/update/service/doctor workflow for end users.
- Private-release-safe install scripts for macOS/Linux and Windows.
- Release verification for all six platform assets plus `SHA256SUMS`.
- End-user quickstart and tooling docs for CLI-only install, setup, service, CI, and AI context.

### Changed

- Root workspace ignores internal skill artifacts and removed extension source.
- Installer scripts skip downloads when the installed `ods` version is already current.
- `scripts/install-from-release.sh` and the skill installer now share the same GitHub API asset flow.

### Removed

- Tracked `ods-lsp`, Zed extension, and `install-zed-ods-lsp.sh` remnants.
- Obsolete cleanup script and removed-component placeholder document.

## [0.1.15] - 2026-07-17

### Added

- `ods export` — write full workspace graph to `graph.md` for AI review
- `ods start` / `ods stop` / `ods serve` — background user service (Linux/macOS/Windows)
- Watch rename pairing with pending removals (split delete/create batches)
- Path-shaped `id:` orphan heal when filename and id drift

### Changed

- Single product binary **`ods` only** (version `0.1.15`)
- Workspace members: `ods`, `ods-core`, `ods-test-support`
- Self-update installs `ods` only

### Removed

- `ods-lsp` language server from product/CI/release path
- Zed extension packaging from release workflow

### Documentation

- README end-to-end install → init → start → lint → export
- `docs/FEATURE_MATRIX.md` keys × auto-update matrix
- Quickstart aligned with CLI-only product

## [0.1.12] - 2026-07-16

### Notes

- Prior workspace package version line; see git history and GitHub tags for details.
