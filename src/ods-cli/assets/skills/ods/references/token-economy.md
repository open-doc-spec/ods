---
description: "Token budgeting, prompt pruning strategies, and cost-reduction heuristics for AI agents."
tags:
  - tokens
  - optimization
  - economics
  - ods
owner: team:ods
ods:
  profile: guide
  status: stable
---

# ODS Token Economy & Context Optimization Guide

This guide details how AI agents achieve up to **~95% token savings** and avoid context degradation using ODS deterministic context boundaries.

---

## 1. The Token Cost Problem

In standard LLM agent workflows, exploring an unfamiliar repository consumes tens of thousands of tokens per prompt:
* **Naive Directory Scans**: `find . -name "*.md"` + dumping 30–50 documents into the system prompt $\to$ **20,000–80,000 tokens**.
* **Context Degradation (Lost in the Middle)**: Dumping entire workspaces causes LLMs to hallucinate or miss critical architectural constraints.
* **ODS Bounded Retrieval**: Targeted query $\to$ DAG traversal $\to$ surgical section reading $\to$ **800–1,500 tokens** (**89%–97% savings**).

---

## 2. Progressive Context Resolution Arc

Follow this step-by-step token budget hierarchy:

```text
Turn 1: Orient (≤100 tokens)
   └─► ods overview
Turn 2: Search (≤50 tokens)
   └─► ods find --tag <name> / --key "status=stable"
Turn 3: Scope Context (≤1,000–4,000 tokens)
   └─► ods context <id> --print --max-tokens <BUDGET>
Turn 4: Target Slice (≤200 tokens)
   └─► ods read <id> --section "<HEADING>"
Turn 5: Code Jump (0 extra doc tokens)
   └─► Read declared symbols in ods.code
```

---

## 3. Decision Matrix: When to Use Which Command

| Situation | Recommended Command | Token Footprint | Why |
|---|---|---|---|
| **Cold start in new repo** | `ods overview` | ~80 tokens | Summarizes workspace layout, profiles, statuses, and top tags. |
| **Locating a specific concept** | `ods find --tag <tag>` | ~20 tokens | Returns exact document IDs without dumping file bodies. |
| **Need prerequisites for task** | `ods context <id> --print` | ~800 tokens | Reads target + strictly hard `depends`, pruned to max budget. |
| **Only need the algorithm/steps**| `ods read <id> --section "Steps"` | ~150 tokens | Extracts only the matching `##` section heading. |
| **Checking document headings** | `ods read <id> --summary` | ~60 tokens | Returns table of contents outline with line numbers. |
| **Locating implementation code** | Read `ods.code` symbols | ~0 extra tokens | Direct symbol jumping; eliminates codebase AST searches. |

---

## 4. Prompt Budgeting Best Practices

1. **Always Set `--max-tokens`**: When using `ods context --print`, specify `--max-tokens <N>` (e.g. `2000` or `4000`) to guarantee predictable prompt budgets.
2. **Exclude `related` by Default**: Soft references (`related:`) are for human discovery; do **not** auto-include them in agent prompts unless explicitly asked.
3. **Avoid Graph Dumps**: Do NOT dump `ods export graph` into routine prompt context — use it only for workspace-level structural audits.
