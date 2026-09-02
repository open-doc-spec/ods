#!/usr/bin/env bash
# Smoke: registry unit tests + schema CLI generation for ODS/OKF/Skills.
# Not a full keys.md parser — catches broken registry / CLI wiring.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "==> schema unit tests"
cargo test -p ods-core --lib spec::schema --locked -- --quiet

echo "==> schema CLI (build if needed)"
ODS=""
for c in \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods" \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods"; do
  if [ -x "$c" ]; then ODS="$c"; break; fi
done
if [ -z "$ODS" ]; then
  cargo build -p ods-cli --bin ods --locked
  ODS="${ROOT}/.artifacts/target/debug/ods"
  [ -x "$ODS" ] || ODS="${ROOT}/target/debug/ods"
fi

out="$("$ODS" schema)"
echo "$out" | grep -q 'tags' || { echo "error: ods schema missing tags"; exit 1; }
echo "$out" | grep -q 'load' || { echo "error: ods schema missing load key"; exit 1; }

keys_out="$("$ODS" schema keys)"
echo "$keys_out" | grep -q 'load' || { echo "error: schema keys missing load"; exit 1; }
echo "$keys_out" | grep -q 'entity' || { echo "error: schema keys missing entity (2.1)"; exit 1; }

okf="$("$ODS" schema --okf)"
echo "$okf" | grep -q 'okf_version' || { echo "error: schema --okf missing okf_version"; exit 1; }

skills="$("$ODS" schema --skills)"
echo "$skills" | grep -q 'description' || { echo "error: schema --skills missing description"; exit 1; }

echo "check-schema-keys: OK"
