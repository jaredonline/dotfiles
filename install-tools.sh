#!/bin/bash
set -uo pipefail

# Install Claude Code LSP plugin binaries.
# Run this once after initial setup — not needed on every install.sh run.

echo "Installing Claude Code LSP binaries..."

command -v go &>/dev/null && go install golang.org/x/tools/gopls@latest || true
command -v gem &>/dev/null && gem install ruby-lsp || true
command -v rustup &>/dev/null && rustup component add rust-analyzer || true
command -v npm &>/dev/null && npm install -g typescript-language-server typescript pyright || true

echo ""
echo "Done!"
