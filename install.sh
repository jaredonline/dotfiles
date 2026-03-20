#!/bin/bash
set -euo pipefail

DOTFILES="$(cd "$(dirname "$0")" && pwd)"

link_file() {
  local src="$DOTFILES/$1"
  local dst="$2"

  # Create parent directory if needed
  mkdir -p "$(dirname "$dst")"

  # Back up existing non-symlink files
  if [[ -e "$dst" && ! -L "$dst" ]]; then
    echo "Backing up $dst → ${dst}.backup"
    mv "$dst" "${dst}.backup"
  fi

  ln -sf "$src" "$dst"
  echo "Linked $dst → $src"
}

merge_json() {
  local base="$DOTFILES/$1"
  local override="$DOTFILES/$2"
  local dst="$3"

  mkdir -p "$(dirname "$dst")"

  # Back up existing file (symlink or regular)
  if [[ -L "$dst" ]]; then
    rm "$dst"
  elif [[ -e "$dst" ]]; then
    echo "Backing up $dst → ${dst}.backup"
    mv "$dst" "${dst}.backup"
  fi

  if [[ -f "$override" ]]; then
    jq -s '.[0] * .[1]' "$base" "$override" > "$dst"
    echo "Merged $base + $override → $dst"
  else
    cp "$base" "$dst"
    echo "Copied $base → $dst (no local override)"
  fi
}

# Core symlinks
link_file "zsh/.zshrc"              "$HOME/.zshrc"
link_file "zsh/.zshprofile"         "$HOME/.zshprofile"
link_file "zsh/.p10k.zsh"           "$HOME/.p10k.zsh"
link_file "git/.gitconfig"          "$HOME/.gitconfig"
link_file "tools/.gemrc"            "$HOME/.gemrc"
link_file "tools/.terraformrc"      "$HOME/.terraformrc"
link_file "gh/config.yml"           "$HOME/.config/gh/config.yml"
merge_json "claude/settings.json" "local/claude/settings.json" "$HOME/.claude/settings.json"
# Claude Code skills
for cmd in "$DOTFILES"/claude/commands/*.md; do
  link_file "claude/commands/$(basename "$cmd")" "$HOME/.claude/commands/$(basename "$cmd")"
done

# Tool setup (run once)
if command -v gh &>/dev/null && ! git config --global credential.https://github.com.helper &>/dev/null; then
  gh auth setup-git
fi

# Source local/private install if present
[[ -f "$DOTFILES/local/install.sh" ]] && source "$DOTFILES/local/install.sh"

echo ""
echo "Done! You may need to restart your shell."
