# Contributing to Open Document Spec (ODS)

Thank you for improving ODS. This repository holds the **Rust reference implementation** (`ods` CLI), maintainer tooling, and **first-cut mirrors** of product surfaces that are also extracted to satellite repos under [open-doc-spec](https://github.com/open-doc-spec).

See **`docs/maintainer/satellite-repos.md`** for SoT vs mirror policy (site, specs, skill, benchmarks, action). Prefer satellite PRs for those surfaces after seed merges; keep monorepo trees in sync for first cut.

## Development setup

```bash
# From repository root
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D clippy::correctness -D clippy::suspicious
```

Binaries land under `.artifacts/target/release/` (see `.cargo/config.toml`).

Install locally:

```bash
cargo install --path src/ods-cli --bin ods --locked --force
```

## Layout

| Path | Role |
| --- | --- |
| `specs/` | Normative specification (mirror; SoT: [ods-spec](https://github.com/open-doc-spec/ods-spec)) |
| `docs/guide/` | End-user guide (keep in sync with site content) |
| `docs/plan/` | Historical plans (CLI UX is **flag-only**; see banners) |
| `docs/other-specs/` | OKF / Agent Skills key maps & notes |
| `docs/maintainer/` | Coverage, functional style, **satellite-repos.md** |
| `CHANGELOG.md` | Optional manual release history |
| `skills/ods` | AI assistant skill mirror (SoT: [ods-skills](https://github.com/open-doc-spec/ods-skills)) |
| `app-web/` | Site mirror (SoT: [opendocify.com](https://github.com/open-doc-spec/opendocify.com), private) |
| `src/ods-core` | Core library (ODS + OKF + Skills engines) |
| `src/ods-cli` | Primary binary `ods` |
| `src/ods-test-support` | Test workspace support library |
| `src/scripts/` | install, check-local, coverage, smoke-ods (**canonical installers**) |
| `src/action/` + `action.yml` | GitHub action mirror (SoT: [ods-action](https://github.com/open-doc-spec/ods-action)) |
| `src/fixtures/benchmarks/` | Bench fixtures mirror (SoT: [ods-benchmarks](https://github.com/open-doc-spec/ods-benchmarks)) |

### CLI crate layout (`src/ods-cli/src/`)

| Path | Role |
| --- | --- |
| `main.rs` | Thin bin entrypoint (`include!` of main/*) |
| `main/cli/` | Entry, dispatch, argv parser, exit codes |
| `main/commands/` | Reorganized commands (`document/`, `profile/`, `workspace/`, `lifecycle/`, `service/`) |
| `main/support/` | Formatters, loaders, git helpers |
| `service/`, `update/` | OS service + self-update implementation |
| `tests/*.test.rs` | Integration tests (declared in `Cargo.toml`) |

### Core crate layout (`src/ods-core/src/`)

Semantic domain folders only (no loose domain `.rs` at crate root except `lib.rs`):

`model/`, `parse/`, `graph/`, `lint/`, `index/`, `lifecycle/`, `mutate/`, `mv/`, `observe/`, `fs/`, `pipeline/`, `profiles/`, `tags/`, `share/`, `bench/`, `okf/`, `multi_spec/`, `spec/`.

New engine code should follow [functional style](docs/maintainer/functional-style.md): data + free functions, no `*Manager` types.

**Product naming:** **`ods`** is the tool; bare commands = ODS (no `--ods`); extra specs via **`--okf`** / **`--skills`** only (no namespaces).

Coverage: see [docs/maintainer/coverage.md](docs/maintainer/coverage.md). Run `./src/scripts/coverage.sh`. CI enforces a line floor (**90%** as of 2026-08-04, with T3 excludes).

### Touchpoint rule

CLI surface or multi-spec changes must update: `specs/`, `docs/guide/`, `docs/other-specs/`, `skills/ods/`, tests, help strings, and `CHANGELOG.md` in the same change set when practical.

## Tests & coverage

- Prefer unit tests next to pure logic and integration tests under `src/*/tests/`.
- Production bar: **workspace ≥90% lines** (T3 network/OS/watch excluded); **`ods-core` ≥90%**; CLI orchestration ≥80%.
- CI enforces a coverage floor (`--fail-under-lines`, currently **88%**) with the same T3 ignore list as `./src/scripts/coverage.sh`.
- Always keep `ods index --check .` and `ods lint .` green at repo root.

```bash
cargo install cargo-llvm-cov --locked
./src/scripts/coverage.sh
# or:
IGNORE_T3='(asset_downloader\.rs|update/installer\.rs|update/binary_replacer\.rs|update/http_helpers\.rs|service/launchers\.rs|service_commands\.rs|watch_and_serve_runner\.rs|okf_watch\.rs|github_release\.rs|setup_command\.rs|lsp_command\.rs|git_sync\.rs)'
cargo llvm-cov --workspace --locked --ignore-filename-regex "$IGNORE_T3" --fail-under-lines 90
```

## CI

Workflows in `.github/workflows/` consist of **two workflows** total:

| Workflow | When | Notes |
| --- | --- | --- |
| `pr.yml` | PRs (open/sync/labels), push to `main`, manual | Unified quality gate: Linux & Windows tests (`fmt`/`clippy`/`test`/`ods`/`cov`) |
| `release.yml` | After `pr.yml` succeeds on main | Multi-OS binary builds, GitHub Release tag `vX.Y.Z` publish |

**Do not create release tags by hand.** Releases are cut automatically when CI is green on main.

## Pull requests

1. Keep changes focused (spec vs guide vs code).
2. Prefer [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(ods): …`, `fix(watch): …`, `docs: …`, `ci: …`, `chore: …`
3. Update `docs/guide/` if you add a user-visible feature.

## Naming gate

```bash
./src/scripts/check-naming.sh
```

Fails if product docs/scripts reintroduce removed namespace CLI forms or the old crates intermediate directory path.
