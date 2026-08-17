---
profile: plan
status: stable
share: public
description: Manual cutover checklist for the few existing ODS workspaces moving to Open Document Spec (ods) CLI.
---

# External repo cutover checklist (≈3 ODS workspaces)

Do this **per external repository** that already uses ODS root markers. This monorepo is already migrated.

## 0. Publish Open Document Spec release (once, this monorepo)

After merging multi-spec work to `main`:

1. Ensure CI (`pr.yml`) is green on `main`.
2. Trigger **`release.yml`** (`workflow_dispatch` or auto after CI success on main).
3. Confirm release assets include **`ods-v*-linux-x86_64.tar.gz`** (and platform peers) plus optional legacy **`ods-v*-…`**.
4. Smoke: `gh release view <tag> --json assets --jq '.assets[].name'`

Local dry-run without publishing:

```bash
./src/scripts/package-local-release.sh
# → dist-local/ods-vX.Y.Z-linux-x86_64.tar.gz
```

## 1. Install CLI

```bash
# After a GitHub Release publishes ods-* assets:
export GH_TOKEN="$(gh auth token)"   # private repo
curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
  https://raw.githubusercontent.com/open-doc-spec/ods/main/src/scripts/install.sh | bash
ods --version
```

## 2. CI

Replace:

```yaml
- run: ods lint
- run: ods index --check
```

With:

```yaml
- run: ods lint            # auto-detect (preferred)
- run: ods index --check
# or explicit: ods ods lint / ods ods index --check
# legacy argv0: ods lint (ODS only) if the `ods` symlink is on PATH
```

GitHub Action consumers: continue using the composite action; it installs **`ods`** and runs lint (ODS namespace when needed).

## 3. Root `index.md`

- Keep **`ods:`** (spec). CLI pin is **`ods: ">=x.y.z"`** — replace any legacy **`ods-cli:`** (`ods upgrade --write` can rewrite).
- Bump `ods:` if your policy requires a minimum CLI with multi-spec support.
- OKF roots use **`okf_version: "0.2"`** only (do not invent `okf:`).

## 4. Validate

```bash
ods lint .
ods doctor .
ods audit --write-report   # → .ods/ods-errors.md
ods upgrade --write        # ods-cli→ods, ~/.ods→~/.ods hints
```

## 5. Agents (optional)

```bash
ods agents sync .
# commit AGENTS.md if desired
```

## 6. OKF (only if that repo holds knowledge bundles)

```bash
ods okf init .    # only for new OKF roots — do not mix blindly
ods okf lint .
```

## Owners

List the three repos and owners here when known:

| Repo | Owner | Done |
|---|---|---|
| _(fill in)_ | | |
| | | |
| | | |
