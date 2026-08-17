#!/usr/bin/env bash
# Open Document Spec (ods) installer — downloads prebuilt `ods` (+ optional `ods` alias) from GitHub Releases.
#
# Supported platforms (auto-detected):
#   macOS  — Apple Silicon (arm64), Intel (x86_64)
#   Linux  — x86_64, arm64
#
# Windows: use install.ps1 instead.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/open-doc-spec/ods/main/src/scripts/install.sh | bash
#
# Options via environment variables:
#   ODS_VERSION   — pin a release tag, e.g. "v0.0.13"  (default: latest stable)
#   ODS_PREFIX    — directory to install binaries into  (default: ~/.local/bin)
#   ODS_NO_VERIFY — set to "1" to skip SHA256 checksum verification
#   GH_TOKEN / GITHUB_TOKEN — optional token (e.g. for higher API rate limits)
#
set -euo pipefail

REPO="open-doc-spec/ods"
API="https://api.github.com/repos/${REPO}"

# ── Helpers ───────────────────────────────────────────────────────────────────
info()  { echo "==> $*"; }
warn()  { echo "WARN: $*" >&2; }
fatal() { echo "error: $*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fatal "required command not found: $1 — please install it and retry"
}

strip_v() {
  printf '%s' "$1" | sed -E 's/^[vV]//'
}

version_key() {
  strip_v "$1" | awk -F. '{
    major=$1+0; minor=$2+0; patch=$3+0;
    printf "%010d.%010d.%010d\n", major, minor, patch
  }'
}

installed_ods_version() {
  if command -v ods >/dev/null 2>&1; then
    ods --version 2>/dev/null | awk '{print $2}' | head -1
  elif [ -x "${ODS_PREFIX:-${HOME}/.local/bin}/ods" ]; then
    "${ODS_PREFIX:-${HOME}/.local/bin}/ods" --version 2>/dev/null | awk '{print $2}' | head -1
  else
    printf ''
  fi
}

version_ge() {
  [ "$(version_key "$1")" \> "$(version_key "$2")" ] || [ "$(version_key "$1")" = "$(version_key "$2")" ]
}

github_token() {
  if [ -n "${GH_TOKEN:-}" ]; then
    printf '%s' "${GH_TOKEN}"
  elif [ -n "${GITHUB_TOKEN:-}" ]; then
    printf '%s' "${GITHUB_TOKEN}"
  else
    printf ''
  fi
}

# curl wrapper. Args: extra curl flags… then URL.
# Always sets User-Agent + Accept + optional Authorization.
api_curl() {
  local -a args=(
    -fsSL --tlsv1.2
    --connect-timeout 30 --max-time 300
    --retry 3 --retry-delay 2
    -H "User-Agent: ods-install"
  )
  local token
  token="$(github_token)"
  if [ -n "${token}" ]; then
    args+=(-H "Authorization: Bearer ${token}")
  fi
  curl "${args[@]}" "$@"
}

# Download a release asset by name.
# Tries direct GitHub Release URL first, falling back to API asset endpoint.
# Usage: download_asset <tag|latest> <filename> <output-path>
download_asset() {
  local tag="$1" filename="$2" out="$3"
  local direct_url="https://github.com/${REPO}/releases/download/${tag}/${filename}"

  if api_curl -o "${out}" "${direct_url}" 2>/dev/null; then
    return 0
  fi

  local release_json asset_id
  if [ "${tag}" = "latest" ]; then
    release_json=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/latest") \
      || return 1
  else
    release_json=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/tags/${tag}") \
      || return 1
  fi

  if command -v python3 >/dev/null 2>&1; then
    asset_id=$(printf '%s' "${release_json}" | FILENAME="${filename}" python3 -c '
import json, os, sys
name = os.environ["FILENAME"]
data = json.load(sys.stdin)
for a in data.get("assets", []):
    if a.get("name") == name:
        print(a["id"])
        raise SystemExit(0)
raise SystemExit(1)
' 2>/dev/null) || asset_id=""
  else
    asset_id=""
  fi

  # Fallback without python: id usually appears before name in GitHub JSON
  if [ -z "${asset_id}" ]; then
    asset_id=$(printf '%s' "${release_json}" \
      | tr '\n' ' ' \
      | sed -n "s/.*\"id\": *\([0-9][0-9]*\)[^}]*\"name\": *\"${filename}\".*/\1/p" \
      | head -1)
  fi
  [ -n "${asset_id}" ] || {
    warn "asset '${filename}' not found on release ${tag}"
    return 1
  }

  api_curl -H "Accept: application/octet-stream" \
    -o "${out}" \
    "${API}/releases/assets/${asset_id}"
}

# ── Arguments & Options ────────────────────────────────────────────────────────
FORCE="${ODS_FORCE:-0}"
VERSION_ARG=""

while [ $# -gt 0 ]; do
  case "$1" in
    -f|--force)
      FORCE=1
      shift
      ;;
    -v|--version)
      VERSION_ARG="$2"
      shift 2
      ;;
    -*)
      warn "unknown option: $1"
      shift
      ;;
    *)
      if [ -z "${VERSION_ARG}" ]; then
        VERSION_ARG="$1"
      fi
      shift
      ;;
  esac
done

# ── Dependency check ──────────────────────────────────────────────────────────
need_cmd curl
need_cmd tar

# ── Platform detection → short asset id (os-arch) ─────────────────────────────
OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  Linux-x86_64)              ASSET="linux-x86_64"  ;;
  Linux-aarch64|Linux-arm64) ASSET="linux-arm64"   ;;
  Darwin-arm64)              ASSET="macos-arm64"   ;;
  Darwin-x86_64)             ASSET="macos-x86_64"  ;;
  *)
    fatal "unsupported platform: ${OS}-${ARCH}
  Supported: Linux x86_64/arm64, macOS arm64/x86_64
  Windows: use src/scripts/install.ps1 (PowerShell)
  Build from source: cargo install --path src/ods-cli --bin ods"
    ;;
esac

# ── Version resolution ────────────────────────────────────────────────────────
VERSION="${VERSION_ARG:-${ODS_VERSION:-}}"
if [ -z "${VERSION}" ]; then
  info "Resolving latest ODS release..."
  if ! API_RESPONSE=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/latest" 2>/dev/null); then
    fatal "could not reach GitHub API — check network connection"
  fi
  VERSION=$(printf '%s' "${API_RESPONSE}" \
    | grep '"tag_name"' | head -1 \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  [ -n "${VERSION}" ] || fatal "could not resolve latest release tag for ${REPO}"
fi
info "Installing ODS ${VERSION} for ${ASSET}"

INSTALLED_VERSION="$(installed_ods_version)"
if [ "${FORCE}" != "1" ] && [ -n "${INSTALLED_VERSION}" ] && version_ge "${INSTALLED_VERSION}" "${VERSION}"; then
  info "ods ${INSTALLED_VERSION} is up to date (latest $(strip_v "${VERSION}"))"
  info "Use --force (or export ODS_FORCE=1) to force re-installation."
  command -v ods >/dev/null 2>&1 && ods --version || "${ODS_PREFIX:-${HOME}/.local/bin}/ods" --version
  exit 0
fi

# ── Filenames ─────────────────────────────────────────────────────────────────
FILENAME="ods-${VERSION}-${ASSET}.tar.gz"

# ── Temp workspace ────────────────────────────────────────────────────────────
TMPDIR_ODS=$(mktemp -d)
trap 'rm -rf "${TMPDIR_ODS}"' EXIT

# ── Download archive ──────────────────────────────────────────────────────────
info "Downloading ${FILENAME}..."
if ! download_asset "${VERSION}" "${FILENAME}" "${TMPDIR_ODS}/${FILENAME}"; then
  fatal "download failed for archive on ${VERSION}
Check that version exists at: https://github.com/${REPO}/releases"
fi

# ── Checksum verification ─────────────────────────────────────────────────────
if [ "${ODS_NO_VERIFY:-0}" != "1" ]; then
  info "Verifying checksum..."
  if ! download_asset "${VERSION}" "SHA256SUMS" "${TMPDIR_ODS}/SHA256SUMS"; then
    fatal "could not download SHA256SUMS for ${VERSION}"
  fi

  EXPECTED=$(grep " ${FILENAME}$" "${TMPDIR_ODS}/SHA256SUMS" | awk '{print $1}')
  [ -n "${EXPECTED}" ] || fatal "no checksum found for '${FILENAME}' in SHA256SUMS"

  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
  else
    warn "no sha256sum or shasum found — skipping checksum verification"
    ACTUAL="${EXPECTED}"
  fi

  [ "${EXPECTED}" = "${ACTUAL}" ] \
    || fatal "checksum mismatch!
  Expected: ${EXPECTED}
  Got:      ${ACTUAL}
  The downloaded file may be corrupt or tampered with."
  info "Checksum OK"
fi

# ── Extract ───────────────────────────────────────────────────────────────────
info "Extracting..."
tar xzf "${TMPDIR_ODS}/${FILENAME}" -C "${TMPDIR_ODS}"
BIN_SRC=""
for try in "${TMPDIR_ODS}/ods-${VERSION}-${ASSET}"; do
  if [ -f "${try}/ods" ]; then BIN_SRC="${try}/ods"; break; fi
done
if [ -z "${BIN_SRC}" ]; then
  FOUND=$(find "${TMPDIR_ODS}" -type f -name ods 2>/dev/null | head -1 || true)
  [ -n "${FOUND}" ] && BIN_SRC="${FOUND}"
fi
[ -n "${BIN_SRC}" ] && [ -f "${BIN_SRC}" ] || fatal "binary 'ods' not found in archive"

# ── Install ───────────────────────────────────────────────────────────────────
PREFIX="${ODS_PREFIX:-${HOME}/.local/bin}"
mkdir -p "${PREFIX}"
install -m 755 "${BIN_SRC}" "${PREFIX}/ods"

echo ""
info "Installed successfully:"
echo "    ${PREFIX}/ods  (primary)"

# ── PATH Auto-Configuration ───────────────────────────────────────────────────
case ":${PATH}:" in
  *":${PREFIX}:"*) ;;
  *)
    export PATH="${PREFIX}:${PATH}"
    SHELL_PROFILE=""
    if [ -n "${ZSH_VERSION:-}" ] || [ -f "${HOME}/.zshrc" ]; then
      SHELL_PROFILE="${HOME}/.zshrc"
    elif [ -f "${HOME}/.bashrc" ]; then
      SHELL_PROFILE="${HOME}/.bashrc"
    elif [ -f "${HOME}/.profile" ]; then
      SHELL_PROFILE="${HOME}/.profile"
    fi

    if [ -n "${SHELL_PROFILE}" ]; then
      if ! grep -q "PATH=.*${PREFIX}" "${SHELL_PROFILE}" 2>/dev/null; then
        echo "" >> "${SHELL_PROFILE}"
        echo "# Open Document Spec (ods) CLI PATH" >> "${SHELL_PROFILE}"
        echo "export PATH=\"${PREFIX}:\$PATH\"" >> "${SHELL_PROFILE}"
        info "Added '${PREFIX}' to PATH in ${SHELL_PROFILE}"
      fi
    fi
    ;;
esac

# ── Instant Verification ──────────────────────────────────────────────────────
info "Verifying installation..."
if "${PREFIX}/ods" --version >/dev/null 2>&1; then
  VER_OUT="$("${PREFIX}/ods" --version)"
  info "ODS installed and verified: ${VER_OUT}"
else
  fatal "installed binary at ${PREFIX}/ods failed execution check"
fi

# ── Next steps ────────────────────────────────────────────────────────────────
echo ""
echo "  Verify installation:"
echo "    ${PREFIX}/ods --version"
echo ""
echo "  Get started:"
echo "    ods lint"
echo "    ods export"
echo ""
echo "  Keep tools current (opt-out: ODS_AUTO_UPDATE=0):"
echo "    ods update              # update binary & restart background service"
echo ""
echo "  Guide: https://github.com/${REPO}/blob/main/README.md"
echo "  Changelog: https://github.com/${REPO}/blob/main/CHANGELOG.md"
echo ""
