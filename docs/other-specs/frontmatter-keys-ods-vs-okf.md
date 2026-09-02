---
profile: note
status: stable
share: public
description: Comparison of ODS vs Google OKF v0.2 frontmatter keys — which keys exist in which spec, purpose of each family, root markers, and seamless ods CLI.
owner: team:opendocify
tags: [ods, okf, frontmatter, keys, multi-spec, reference]
---

# Frontmatter keys: ODS vs OKF v0.2

This document is the **author-facing map** of which metadata keys belong to which Open Document Spec spec, what they are for, and how the **`ods`** CLI routes them.

| Spec | CLI | Best for |
|---|---|---|
| **ODS** (Open Document Spec) | bare `ods …` (**default** — no flag) | Codebase documentation graphs: profiles, depends/related, code refs, indexes |
| **OKF v0.2** (Open Knowledge Format) | `ods … --okf` | Knowledge/metric/asset catalogs: trust, provenance, freshness, attested computations |

**Tool** = **`ods`**. ODS is always the native product identity. **Native OKF** means the OKF engine ships in the same binary and is **activated only with `--okf`**. There is no `--ods` flag and no `ods okf` / `ods ods` namespaces.

```
ods lint         # ODS only (default)
ods lint --okf   # OKF only on pure OKF trees; ODS+OKF on hybrid when ODS is present
ods init         # ODS workspace (root ods.toml with spec)
ods init --okf   # OKF bundle (okf_version: "0.2")
```

**Source of truth for product naming:** [ods tool / keys / legacy cleanup plan](../plan/archive/odc_tool_keys_legacy_cleanup.md).

**Normative references**

- ODS: [`specs/ods/intro.md`](../../specs/ods/intro.md), [`specs/ods/keys.md`](../../specs/ods/keys.md), [`specs/ods/core.md`](../../specs/ods/core.md)
- OKF (CLI dialect): [`specs/okf/keys.md`](../../specs/okf/keys.md); upstream [Google OKF SPEC v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- Implementation plans: [ODS migration](../plan/archive/ods_to_odc_migration_and_cli_architecture.md), [Native OKF](../plan/archive/okf_native_support.md)

---

## 1. When to use which spec

| Use ODS when… | Use OKF when… |
|---|---|
| Docs describe **software systems** (services, guides, RFCs, APIs) | Docs describe **data/knowledge assets** (tables, metrics, playbooks, policies) |
| You need **typed graph edges** (`depends`, `related`) and **code bindings** | You need **trust signals** (who generated/verified) and **staleness** |
| Progressive disclosure via **CLI discovery** (`overview` / `find` / `context`) + profiles | Progressive disclosure via OKF **index.md** / **log.md** conventions |
| Workspace root is an engineering repo (`ods.toml`) | Bundle is a knowledge pack (often agent-maintained) |

| Command on hybrid | Behavior |
|---|---|
| bare `lint` / `doctor` / `audit` | **ODS only** (or **ODS + OKF** if `ods.toml` has `[specs.okf] enabled = true`) |
| same with `--okf` | **ODS + OKF** |
| pure OKF tree without `--okf` | Error with hint to pass `--okf` |
| ODS-only cmds (`mv`, `tags`, …) | ODS engine |

### Frontmatter Key Linting & Opt-Out Management

By default, OKF validation enforces required keys such as `type` and `runtime` (for `Attested Computation`). If your team or workflow does not require frontmatter key enforcement for OKF concepts, you can suppress key linting via:

1. **Declarative root `ods.toml`**:
   ```toml
   [specs.okf]
   enabled = true
   check_keys = false
   ignore_keys = ["runtime", "sources"]
   ```
2. **CLI Flags**:
   - `ods lint --okf --skip-frontmatter-keys` (disables key requirement checks)
   - `ods lint --okf --ignore-keys runtime,sources` (ignores specific keys)

There is no namespace form. Extra specs always use flags.

---

## 2. Root markers (workspace / bundle identity)

| Key | Spec | Where | Purpose |
|---|---|---|---|
| `spec = "0.1"` | ODS | Root `ods.toml` | Declares ODS workspace boundary and spec version |
| `okf_version: "0.2"` | OKF | Root `index.md` only | Declares OKF bundle targets v0.2 |
| Both | Hybrid | `ods.toml` + OKF root `index.md` | Bare commands = ODS; pass `--okf` to include OKF (no namespaces) |

**Reserved filenames (OKF):** `index.md`, `log.md` are bundle conventions.

**ODS:** optional navigation `index.md` files may use `profile: index`; they are **not** the workspace marker. Workspace policy (`spec`, `ignore`, `packs`, `custom_profiles`, `[specs.*]`) lives in `ods.toml`. Scalar `ods: 0.1` on a document is legacy, not the project boundary.

---

## 3. Key comparison table (availability)

Legend: **Yes** = first-class in that spec · **No** = not part of that spec’s model · **Related** = similar idea under a different shape.

| Key / concept | ODS | OKF v0.2 | Purpose (short) |
|---|---|---|---|
| **Root `ods.toml` `spec`** | Yes | No | ODS workspace version / boundary (e.g. `"0.1"`) |
| **Nested `ods.profile`** | Yes | No | Document shape (guide, api, feature, …) |
| **Nested `ods.status`** | Yes | Related* | ODS doc lifecycle (`draft` / `stable` / …) |
| **Nested `ods.id`** | Yes | No† | Stable id override (path is default id) |
| **Nested `ods.share`** | Yes | No | Publish/share policy for export |
| **Nested `ods.depends`** | Yes | No† | Hard dependency edges |
| **Nested `ods.related`** | Yes | No† | Soft relationship edges |
| **Nested `ods.resources`** | Yes | Related | Non-code resource refs |
| **Nested `ods.code`** | Yes | No | Code path bindings + roles |
| **Nested `ods.context`** | Yes | No | Context pack for `ods context` |
| **Universal top-level** (`description`, `owner`, `tags`, …) — **never under `ods:`** | Yes (SSG/CMS-friendly; any tool can read) | Partial (`title`, `description`, `tags` recommended) | Human/agent display metadata; ODS forbids nesting these under `ods:` |
| **`type`** | No | **Required** | Concept kind (Metric, BigQuery Table, …) |
| **`resource`** | Via resources/code | Yes optional | Canonical URI of underlying asset |
| **`sources`** (+ credibility) | No (v1) | Yes | Provenance: what this concept was built from |
| **`usage_window`** | No | Yes | Window framing `usage_count` |
| **`generated` `{by,at}`** | No (v1) | Yes | Who/what wrote the concept, when |
| **`verified` `[{by,at}]`** | No (v1) | Yes | Independent confirmations; trust tier |
| **Top-level `status`** | No (use `ods.status`) | Yes | `draft` \| `stable` \| `deprecated` |
| **`stale_after`** | No (v1) | Yes | Absolute freshness deadline |
| **`runtime`** | No | Attested Computation | How the computation runs |
| **`parameters`** | No | Attested Computation | Named typed holes for the runtime |
| **`computation`** | No | Attested Computation | Path to computation file (alt to body) |
| **`executor`** | No | Attested Computation | How to run + receipt shape |
| **`attester`** | No | Attested Computation | Deterministic check of a run receipt |
| **`okf_version`** | No | Root | Bundle OKF version |
| **`timestamp`** (legacy) | No | Read fallback | v0.1; prefer `generated.at` |
| **Markdown links** | Yes | Yes | Relationships (OKF: untyped edges in prose) |
| **Unknown custom keys** | Preserve | Preserve | Extensions; must not be rejected |

\* **Status:** ODS keeps lifecycle under **`ods.status`**. OKF uses **top-level `status`**. Do not assume they are the same field in the engine without an explicit mapping.

† **IDs and edges:** OKF concept id is the path without `.md`. Relationships are ordinary markdown links, not `depends`/`related` arrays.

---

## 4. Purpose of key families

### 4.1 ODS engine keys (nested under `ods:`)

| Family | Purpose |
|---|---|
| **profile / status** | Tell tools and agents *what kind of doc* this is and whether it is ready |
| **id / share** | Stable identity and publish boundaries |
| **depends / related** | Build a **queryable graph** for reading order and impact |
| **code / resources** | Bind docs to real implementation and artifacts |
| **context** | Define bounded packs for prompt assembly (`ods context`) |
| **Root ods / ods** | Mark the tree as an ODS workspace and gate CLI version |

**Goal:** efficient, structured **engineering documentation** for humans and coding agents.

**Placement rule:** Universal keys (`tags`, `description`, `owner`, …) stay at the **common top level** so any technology can use them. Engine keys stay under **`ods:`**. Do not put `tags` under `ods:` — lint warns; `ods fmt --migrate` hoists them. See [ods/keys.md](../../specs/ods/keys.md).

### 4.2 OKF identity & description

| Key | Purpose |
|---|---|
| **type** | Route/filter concepts (only required field) |
| **title / description / tags** | Discovery, indexes, previews |
| **resource** | Point at the real system object (table URI, API, …) |

### 4.3 OKF provenance (`sources`)

Records **what materials** a concept was derived from, with optional credibility signals (`author`, `usage_count`, `last_modified`) — **not** a stored credibility score. Footnotes `[^id]` join body claims to `sources[].id`.

### 4.4 OKF trust (`generated`, `verified`)

| Key | Purpose |
|---|---|
| **generated** | How the content was produced (agent/human/process) and when it last meaningfully changed |
| **verified** | Independent confirmations; consumers derive **trust tiers** (unverified → machine-confirmed → human-reviewed) |

Absence of `verified` is meaningful (unverified) but **not** a hard failure under OKF conformance.

### 4.5 OKF lifecycle (`status`, `stale_after`)

| Key | Purpose |
|---|---|
| **status** | draft → stable → deprecated |
| **stale_after** | Absolute date; concept is stale when `today >= stale_after` |

### 4.6 OKF Attested Computation

Answers: *was this number produced the way we said it must be?*  
Frontmatter carries the **contract** (`runtime`, `parameters`, `executor`, `attester`, optional `computation` path). Open Document Spec v1 **lints the contract**; it does not execute attesters.

### 4.7 Shared conventions

| Convention | Purpose |
|---|---|
| YAML frontmatter + Markdown body | Human-readable, agent-parseable, git-diffable |
| Preserve unknown keys | Allow extensions without forking the specs |
| Progressive disclosure indexes | Cheap navigation before loading full docs |

---

## 5. CLI mapping

| Goal | Command |
|---|---|
| Create ODS workspace | `ods init` |
| Create OKF bundle | `ods init --okf` |
| Validate ODS | `ods lint` |
| Validate OKF | `ods lint --okf` |
| Inventory non-compliant Markdown | `ods audit --write-report` → `.ods/ods-errors.md` |
| Draft frontmatter on plains | `ods adopt` / `ods adopt --okf --write` |
| Resolve reading list | `ods context` / `ods context --okf` |
| Binary self-update | `ods update` |
| Machine/workspace forward helpers | `ods upgrade` |

---

## 6. Example side-by-side

### ODS document (sketch)

```yaml
---
title: Checkout Setup
owner: team:payments
ods:
  profile: guide
  status: stable
  depends:
    - products/payments/index
  code:
    - path: src/checkout/mod.rs
      role: implementation
---
```

### OKF concept (sketch)

```yaml
---
type: Metric
title: Revenue
description: Recognized revenue for a fiscal year.
tags: [finance, revenue]
status: stable
generated: { by: reference_agent/gemini-2.5-pro, at: 2026-06-20T22:53:05Z }
verified: { by: human:reviewer, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31
sources:
  - id: rev-policy
    resource: https://wiki.example/finance/revenue-recognition
    title: Revenue recognition policy
---
```

Same file format family (Markdown + YAML). **Different keys, different lint rules.** One tool (`ods`) routes by root markers or explicit namespace.

---

## 7. What v1 will not do

- Merge ODS and OKF into one frontmatter dialect  
- Invent `okf:` as a CLI pin (Google uses `okf_version:`)  
- Rename nested ODS engine keys from `ods:` to `ods:`  
- Run OKF executors/attesters  
- Silently map `ods.status` ↔ OKF `status` without a future explicit design  

If Open Document Spec later adopts shared optional keys into ODS (e.g. `stale_after`, `sources`), that is a **separate ODS version bump** documented in `specs/` — not implied by native OKF support alone.

---

## 8. See also

- ODS modules: [`specs/ods/`](../../specs/ods/) (`intro`, `keys`, `core`, `graph`, `validation`, …)  

- Plans: [Tool / keys / legacy cleanup](../plan/archive/odc_tool_keys_legacy_cleanup.md), [ODS migration](../plan/archive/ods_to_odc_migration_and_cli_architecture.md), [Native OKF](../plan/archive/okf_native_support.md)  
- Upstream OKF: [SPEC.md v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
