#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SAMPLE="${ROOT}/src/fixtures/ecommerce"
EXPORT_OUT="${TMPDIR:-/tmp}/ods-graph-local.md"

export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export ODS_AUTO_UPDATE=0
# legacy dual-read still honored by the binary; silence both during local gates
export ODC_AUTO_UPDATE=0

find_rustup() {
  if command -v rustup >/dev/null 2>&1; then
    command -v rustup
    return 0
  fi

  for candidate in "${HOME}/.cargo/bin/rustup" /opt/homebrew/bin/rustup /usr/local/bin/rustup; do
    if [ -x "${candidate}" ]; then
      echo "${candidate}"
      return 0
    fi
  done

  return 1
}

if RUSTUP="$(find_rustup 2>/dev/null)"; then
  TOOLCHAIN="${RUSTUP_TOOLCHAIN:-$("${RUSTUP}" show active-toolchain | awk '{print $1}')}"
  "${RUSTUP}" component add rustfmt clippy --toolchain "${TOOLCHAIN}" >/dev/null
  TOOLBIN="$(dirname "$("${RUSTUP}" which rustc --toolchain "${TOOLCHAIN}")")"
  export PATH="${TOOLBIN}:${PATH}"
fi

run() {
  echo "==> $*"
  "$@"
}

cd "${ROOT}"
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets --locked -- -D warnings
run cargo test --workspace --locked
run "${ROOT}/src/scripts/check-naming.sh"
run "${ROOT}/src/scripts/check-odc-residue.sh"
# Schema registry smoke (unit + ods schema CLI when binary available after tests)
if [ -x "${ROOT}/src/scripts/check-schema-keys.sh" ]; then
  # Use already-built test/debug binary; skip full recompile if schema tests already ran via cargo test
  run cargo test -p ods-core --lib spec::schema --locked -- --quiet
fi
if [ "${SKIP_RELEASE_BUILD:-}" != "true" ]; then
  run cargo build --workspace --release --locked
fi

cd "${ROOT}"
ODS=""
for candidate in \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods" \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods" \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods" \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods"; do
  if [ -x "${candidate}" ]; then
    ODS="${candidate}"
    break
  fi
done

if [ -z "${ODS}" ]; then
  echo "error: ods/ods binary not found" >&2
  find "${ROOT}" -name ods -o -name ods -type f 2>/dev/null | head >&2
  exit 1
fi

ODS_CMD=("${ODS}")

FIXTURES=(
  "${ROOT}/src/fixtures/ecommerce"
  "${ROOT}/src/fixtures/policy-handbook"
  "${ROOT}/src/fixtures/packs/engineering-pack"
)

for fixture in "${FIXTURES[@]}"; do
  if [ -d "${fixture}" ]; then
    run "${ODS_CMD[@]}" overview --check "${fixture}"
    run "${ODS_CMD[@]}" lint "${fixture}"
  fi
done

run "${ODS_CMD[@]}" export "${SAMPLE}" --out "${EXPORT_OUT}"
test -s "${EXPORT_OUT}"
grep -q "ODS workspace graph" "${EXPORT_OUT}"

# OKF smoke when binary is ods
if [[ "$(basename "${ODS}")" == "ods" ]]; then
  OKF_TMP=$(mktemp -d)
  run "${ODS}" init --okf "${OKF_TMP}"
  run "${ODS}" lint --okf "${OKF_TMP}"
  rm -rf "${OKF_TMP}"
fi

if [ -f "${ROOT}/src/action/scripts/test-action.sh" ]; then
  run "${ROOT}/src/action/scripts/test-action.sh"
fi

echo "local checks passed"
