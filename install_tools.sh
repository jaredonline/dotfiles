#!/bin/bash
[[ "$(uname -s)" == "Linux" ]] || { echo "install_tools.sh: Linux only" >&2; exit 2; }
set -euo pipefail

# Install dev tooling (LSPs, CLI utilities, bd) on Linux devboxes.
# Run once after initial setup — not needed on every install.sh run.

log() { echo "==> $*"; }

log "installing apt packages: silversearcher-ag, tig, jq"
sudo apt-get update && sudo apt-get install -y silversearcher-ag tig jq

log "installing oh-my-zsh"
if [[ ! -d "$HOME/.oh-my-zsh" ]]; then
  sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended
fi

log "installing powerlevel10k"
P10K_DIR="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k"
if [[ ! -d "$P10K_DIR" ]]; then
  git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "$P10K_DIR"
fi

log "installing eza via cargo"
cargo install eza

log "installing grpcurl"
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

log "installing Claude Code LSP plugin binaries"
go install golang.org/x/tools/gopls@latest
gem install ruby-lsp
rustup component add rust-analyzer
npm install -g typescript-language-server typescript pyright

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

log "install_tools.sh complete"
