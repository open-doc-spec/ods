#!/usr/bin/env bash
# Open Document Spec skill bootstrap — install / update the `ods` binary (optional `ods`
# alias) and keep the background watch service running.
#
# This script is self-contained: it depends only on the vendored
# install-from-release.sh (next to it) plus `gh` (GitHub CLI) auth. It never
# builds from source and never requires the open-document-spec repo checkout.
#
# Usage:
#   bootstrap.sh                 # default: install -> check . -> ensure . (if compliant) -> doctor .
#   bootstrap.sh install
#   bootstrap.sh update
#   bootstrap.sh ensure [path]   # guarantee background service for a workspace
#   bootstrap.sh status [path]
#   bootstrap.sh doctor [path]
#   bootstrap.sh check [path]    # compliance + git probe, no mutation
#
# Env:
#   ODS_PREFIX    install dir for the binary (default: ~/.local/bin)
#   ODS_REPO      GitHub repo (default: StaytunedLLP/open-document-spec)
#   ODS_VERSION   pin a release tag (default: latest)
#   GH_TOKEN      required for the private repo (or run `gh auth login`)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${ODS_PREFIX:-$HOME/.local/bin}"
export PATH="${PREFIX}:${PATH}"

log()  { printf '==> %s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
die()  { printf 'error: %s\n' "$*" >&2; exit 1; }

have_cli() { command -v ods >/dev/null 2>&1 || command -v ods >/dev/null 2>&1; }
cli_bin() {
  if command -v ods >/dev/null 2>&1; then command -v ods
  else command -v ods
  fi
}

# Install the binary from the latest (or pinned) GitHub Release.
cmd_install() {
  if have_cli && [[ "${1:-}" != "--force" ]]; then
    log "checking installed ods against latest release: $(command -v ods) ($(ods --version 2>/dev/null || echo '?'))"
  else
    log "installing ods from release${ODS_VERSION:+ ${ODS_VERSION}}"
  fi
  bash "${SCRIPT_DIR}/install-from-release.sh" ${ODS_VERSION:+"${ODS_VERSION}"}
  hash -r 2>/dev/null || true
  have_cli || die "ods not on PATH after install; add ${PREFIX} to PATH"
  log "installed $(command -v ods) ($(ods --version))"
}

# Update in place. Prefer the binary's own self-update; fall back to reinstall.
cmd_update() {
  if ! have_cli; then
    log "ods not installed; installing instead"
    cmd_install
  else
    if ods update --check >/dev/null 2>&1; then
      log "ods binary is up to date"
    else
      log "updating ods"
      if ! ods update 2>/dev/null; then
        warn "self-update failed; reinstalling from release"
        cmd_install --force
      fi
    fi
  fi
  if find_workspace_root . >/dev/null 2>&1; then
    log "running workspace & machine migration (ods upgrade --write)"
    ods upgrade --write . 2>/dev/null || true
  fi
  log "now on $(ods --version)"
}

# Resolve the workspace root: walk up looking for an index.md whose frontmatter
# carries an `ods:` key. Prints the root path on success, empty on failure.
find_workspace_root() {
  local dir; dir="$(cd "${1:-.}" 2>/dev/null && pwd)" || return 1
  while :; do
    if [[ -f "${dir}/index.md" ]] && frontmatter_has_ods "${dir}/index.md"; then
      printf '%s\n' "${dir}"
      return 0
    fi
    [[ "${dir}" == "/" ]] && break
    dir="$(dirname "${dir}")"
  done
  return 1
}

# True if the file's leading YAML frontmatter block contains a top-level `ods:` key.
frontmatter_has_ods() {
  awk '
    NR==1 && $0!="---" { exit 1 }        # no frontmatter at all
    NR==1 { infm=1; next }
    infm && $0=="---" { exit 1 }         # end of frontmatter, key not found
    infm && /^ods[[:space:]]*:/ { exit 0 }
  ' "$1"
}

is_git() {
  git -C "${1:-.}" rev-parse --is-inside-work-tree >/dev/null 2>&1
}

# Compliance + git probe. No mutation.
cmd_check() {
  local path="${1:-.}" root
  if root="$(find_workspace_root "${path}")"; then
    printf 'compliant=true root=%s\n' "${root}"
  else
    printf 'compliant=false root=\n'
    printf 'hint: not an ODS workspace (no index.md with `ods:`). Run: ods init %s\n' "${path}"
  fi
  if is_git "${path}"; then
    printf 'git=true\n'
  else
    printf 'git=false\n'
  fi
}

# Guarantee the background watch service for a workspace (idempotent).
cmd_ensure() {
  local path="${1:-.}"
  have_cli || die "ods not installed; run: bootstrap.sh install"
  if ! find_workspace_root "${path}" >/dev/null; then
    warn "not an ODS workspace (no index.md with \`ods:\`). Run: ods init ${path}"
    warn "skipping service start on a non-workspace"
    return 0
  fi
  log "starting ods service for ${path}"
  ods start "${path}"
  ods start --status "${path}" 2>/dev/null || ods start --status
}

cmd_status() {
  local path="${1:-.}"
  have_cli || die "ods not installed; run: bootstrap.sh install"
  ods --version
  ods start --status "${path}" 2>/dev/null || ods start --status
}

cmd_doctor() {
  local path="${1:-.}"
  have_cli || die "ods not installed; run: bootstrap.sh install"
  ods doctor "${path}"
}

main() {
  local sub="${1:-default}"
  [[ $# -gt 0 ]] && shift || true
  case "${sub}" in
    install) cmd_install "$@" ;;
    update)  cmd_update "$@" ;;
    ensure)  cmd_ensure "$@" ;;
    status)  cmd_status "$@" ;;
    doctor)  cmd_doctor "$@" ;;
    check)   cmd_check "$@" ;;
    default)
      cmd_install
      cmd_check .
      if find_workspace_root . >/dev/null; then
        cmd_ensure .
        cmd_doctor .
      else
        warn "no ODS workspace at '.'; run 'ods init .' then 'bootstrap.sh ensure .'"
      fi
      cmd_update
      log "Open Document Spec (ods) is installed and running on your machine!"
      log "Version: $(ods --version)"
      ;;
    -h|--help|help)
      sed -n '2,26p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      ;;
    *) die "unknown subcommand '${sub}' (try: install|update|ensure|status|doctor|check)" ;;
  esac
}

main "$@"
