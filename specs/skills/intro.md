---
description: "Agent Skills packages: purpose, layout, and how the ods CLI validates them."
ods:
  profile: "note"
  status: "stable"
  related:
    - keys.md
---

# Skills · Intro

**Agent Skills** are reusable instruction packages for AI agents: a directory with a `SKILL.md` (YAML frontmatter + Markdown body), optional `scripts/`, `references/`, and `assets/`.

For standalone agent instruction docs that live as normal Markdown files, prefer `agent.md` with the ODS `agent` profile. Keep `SKILL.md` as the installable package entrypoint.

The `ods` CLI treats Agent Skills as an **extra dialect** (flag only):

```bash
ods init --skills ./my-skill
ods lint --skills ./my-skill
ods lint --skills                 # packages under cwd
ods skill install --agent <name>  # install into a host agent
```

There is no `ods skills` namespace. Product skill for this repo: `skills/ods/SKILL.md`.

Upstream format notes: [agentskills.io](https://agentskills.io/llms.txt) and `docs/other-specs/agentskills.md`.

---

## Directory layout

```
skill-name/
├── SKILL.md          # Required: metadata + instructions
├── scripts/          # Optional: executable helpers
├── references/       # Optional: deeper docs
├── assets/           # Optional: templates, static files
└── ...
```

- `name` in frontmatter MUST match the parent directory name.
- Body over ~500 lines SHOULD warn (progressive disclosure—prefer `references/`).

---

## When to use

| Use **Skills** when… | Use **ODS docs** when… |
| :--- | :--- |
| You package **agent procedures** for install into hosts | You document a **product/repo** for humans + agents |
| Entry point is `SKILL.md` | Entry point is workspace `index.ods.md` + documents |
| Lint with `ods lint --skills` | Lint with bare `ods lint` |

---

## What to read next

- [keys.md](keys.md) — SKILL.md frontmatter fields
- Repo example: `skills/ods/`
- Multi-spec CLI: `docs/other-specs/cli-multi-spec.md`
