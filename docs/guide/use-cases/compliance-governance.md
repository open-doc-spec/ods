---
description: "Using ODS policy profiles, status enums, and level-3 CI checks for enterprise governance."
ods:
  profile: "note"
  status: "stable"
---

# Compliance & Governance Use Case

Enterprise software teams use ODS to enforce documentation compliance, mandatory section headers, and audit trails.

---

## Key Controls

1. **Policy Profile (`profile: policy`)**: Enforces required H2 sections (`## Purpose`, `## Scope`, `## Requirements`, `## Enforcement`).
2. **Status Lifecycle (`status: draft | stable | deprecated | archived`)**: Enforces explicit document maturity.
3. **CI gate (`ods lint`)**: Fails pull requests if required sections or links are missing.
