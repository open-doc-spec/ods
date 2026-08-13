# Changelog — ods skill

## Unreleased

### Added
- AI discovery cold-start: `ods overview` / `summary`, `ods find --key` / `--tag-match`, `ods tag list` / `show`, `ods schema keys`, and unique `ods context --tag`/`--key` fallback. Skill §1 playbook updated.
- `ods bench` subcommand integration (`stats`, `strip`, `restore`, `run`) for frontmatter snapshot backups and ROI token & cost auditing.
- Initial `ods` Agent Skill.
- `SKILL.md`: condensed operational ODS spec (fields, IDs-as-paths, two-edge
  graph, indexes, resources, context, profiles, lint levels), full command
  reference, workflows with a compliance gate and git/non-git branching,
  MUST/MUST NOT guardrails, non-goals, and an evals roadmap.
- `scripts/bootstrap.sh`: single entrypoint — `install | update | ensure |
  status | doctor | check`. Installs/updates the release binary and keeps the
  background service running with zero manual release/download work. `check`
  probes workspace compliance (root `index.md` with `ods:`) and git tracking.
- `scripts/install-from-release.sh`: vendored, self-contained release installer.
- `references/intro.md`, `keys.md`, `core.md`, `scope.md`: synced from `specs/ods/` (legacy `spec.md` / `non-goals.md` are pointers).
- `evals/evals.json`: 8 starter cases (activation ±, authoring correctness,
  guardrail/non-goal enforcement, command choice, service model, output format,
  git-rename workflow).

### Follow-ups
- **Enable `gh skill update`:** publish `skills/ods/` into the skills registry
  repo (`staytunedllp/skills`). That publish is what lets other machines run
  `gh skill install ods` / `gh skill update`; authoring here is step one.
- Grow evals to ≥10 cases with `evals/fixtures/{good,bad}/` and run across
  models × reasoning-effort levels; add a script-backed validator.
