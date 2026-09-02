# Spec writing

- Normative dialect trees: `specs/ods/`, `specs/okf/`, `specs/skills/`
- One idea per file; one-word module names (`intro`, `keys`, `core`, `assets`, `scope`, …)
- End-user narrative → `intro.md`; key dictionary → `keys.md`; RFC model → `core.md`
- Prefer relative links inside a dialect folder
- Universal keys top-level; ODS 2.0 engine keys flat at top level (no nested `ods:` wrapper)
- Title is H1 only — no frontmatter `title:` in ODS docs
- After path renames: update site nav, sitemap, redirects, guide, skill references, `llms.txt`, CHANGELOG
