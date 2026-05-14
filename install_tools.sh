#!/bin/bash
[[ "$(uname -s)" == "Linux" ]] || { echo "install_tools.sh: Linux only" >&2; exit 2; }
set -euo pipefail

# Install dev tooling (LSPs, CLI utilities, bd) on Linux devboxes.
# Run once after initial setup — not needed on every install.sh run.

log() { echo "==> $*"; }

# Run "$@" only if PREREQ is on PATH. Logs a SKIP line and returns 0
# (does not propagate the prereq-missing condition through set -e).
# Usage: try_install <prereq-cmd> <cmd> [args...]
try_install() {
    local prereq="$1"
    shift
    if ! command -v "$prereq" >/dev/null 2>&1; then
        log "SKIP: $prereq not on PATH (needed for: $*)"
        return 0
    fi
    "$@"
}

log "installing apt packages: silversearcher-ag, tig, jq"
sudo apt-get update && sudo apt-get install -y silversearcher-ag tig jq

# Pinned to 1.0.3: v1.0.4 has an unconditional JSONL auto-import on every
# write that adds ~45s tax per bd invocation. See gastownhall/beads#3880.
BD_VERSION="1.0.3"
log "checking bd version (want ${BD_VERSION})"
if ! bd --version 2>/dev/null | grep -qF "bd version ${BD_VERSION}"; then
  case "$(uname -m)" in
    x86_64|amd64)   BD_ARCH="amd64" ;;
    aarch64|arm64)  BD_ARCH="arm64" ;;
    *) echo "ERROR: unsupported architecture $(uname -m) for bd"; exit 1 ;;
  esac
  BD_TARBALL="beads_${BD_VERSION}_linux_${BD_ARCH}.tar.gz"
  BD_URL="https://github.com/steveyegge/beads/releases/download/v${BD_VERSION}/${BD_TARBALL}"
  BD_TMP="$(mktemp -d)"
  log "installing bd v${BD_VERSION} (linux_${BD_ARCH})"
  curl -sSL -o "$BD_TMP/$BD_TARBALL" "$BD_URL"
  tar -xzf "$BD_TMP/$BD_TARBALL" -C "$BD_TMP"
  BD_BIN="$(find "$BD_TMP" -type f -name bd | head -1)"
  if [[ -z "$BD_BIN" ]]; then
    echo "ERROR: bd binary not found in $BD_TARBALL — archive contents:"
    ls -laR "$BD_TMP"
    exit 1
  fi
  sudo install -m 0755 "$BD_BIN" /usr/local/bin/bd
  rm -rf "$BD_TMP"
fi

log "installing oh-my-zsh"
if [[ ! -d "$HOME/.oh-my-zsh" ]]; then
  sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended
fi

log "installing powerlevel10k"
P10K_DIR="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k"
if [[ ! -d "$P10K_DIR" ]]; then
  git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "$P10K_DIR"
fi

log "installing zsh-autosuggestions and zsh-syntax-highlighting plugins"
ZSH_CUSTOM="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}"
if [[ ! -d "$ZSH_CUSTOM/plugins/zsh-autosuggestions" ]]; then
  git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions \
    "$ZSH_CUSTOM/plugins/zsh-autosuggestions"
fi
if [[ ! -d "$ZSH_CUSTOM/plugins/zsh-syntax-highlighting" ]]; then
  git clone --depth=1 https://github.com/zsh-users/zsh-syntax-highlighting \
    "$ZSH_CUSTOM/plugins/zsh-syntax-highlighting"
fi

log "installing eza via cargo"
try_install cargo cargo install eza

log "installing grpcurl"
try_install go go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

log "installing Claude Code LSP plugin binaries"
try_install go go install golang.org/x/tools/gopls@latest
try_install gem gem install ruby-lsp
try_install rustup rustup component add rust-analyzer
try_install npm npm install -g typescript-language-server typescript pyright

log "install_tools.sh complete"
