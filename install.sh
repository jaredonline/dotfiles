#!/bin/bash
set -euo pipefail

DOTFILES="$(cd "$(dirname "$0")" && pwd)"

# Resolve a path argument: absolute path passes through, relative path
# resolves against $DOTFILES for backward compatibility with internal callers.
_resolve_path() {
  local p="$1"
  if [[ "$p" == /* ]]; then
    echo "$p"
  else
    echo "$DOTFILES/$p"
  fi
}

link_file() {
  local src
  src="$(_resolve_path "$1")"
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
  local base
  base="$(_resolve_path "$1")"
  local override="$2"
  local dst="$3"

  # Resolve override only if non-empty
  if [[ -n "$override" ]]; then
    override="$(_resolve_path "$override")"
  fi

  mkdir -p "$(dirname "$dst")"

  # Back up existing file (symlink or regular)
  if [[ -L "$dst" ]]; then
    rm "$dst"
  elif [[ -e "$dst" ]]; then
    echo "Backing up $dst → ${dst}.backup"
    mv "$dst" "${dst}.backup"
  fi

  if [[ -n "$override" && -f "$override" ]]; then
    jq -s '.[0] * .[1]' "$base" "$override" > "$dst"
    echo "Merged $base + $override → $dst"
  else
    cp "$base" "$dst"
    echo "Copied $base → $dst (no override)"
  fi
}

merge_md() {
  local base
  base="$(_resolve_path "$1")"
  local override="$2"
  local dst="$3"

  if [[ -n "$override" ]]; then
    override="$(_resolve_path "$override")"
  fi

  mkdir -p "$(dirname "$dst")"

  # Back up existing file (symlink or regular)
  if [[ -L "$dst" ]]; then
    rm "$dst"
  elif [[ -e "$dst" ]]; then
    echo "Backing up $dst → ${dst}.backup"
    mv "$dst" "${dst}.backup"
  fi

  if [[ -n "$override" && -f "$override" ]]; then
    { cat "$base"; echo; cat "$override"; } > "$dst"
    echo "Merged $base + $override → $dst"
  else
    cp "$base" "$dst"
    echo "Copied $base → $dst (no override)"
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
merge_json "$DOTFILES/claude/settings.json" "" "$HOME/.claude/settings.json"
merge_md   "$DOTFILES/CLAUDE.md"            "" "$HOME/.claude/CLAUDE.md"
# Claude Code skills
for cmd in "$DOTFILES"/claude/commands/*.md; do
  link_file "claude/commands/$(basename "$cmd")" "$HOME/.claude/commands/$(basename "$cmd")"
done
# Symlink claude scripts
mkdir -p "$HOME/.claude/scripts"
for script in "$DOTFILES"/claude/scripts/*; do
  link_file "claude/scripts/$(basename "$script")" "$HOME/.claude/scripts/$(basename "$script")"
done

# Tool setup (run once)
if command -v gh &>/dev/null && ! git config --global credential.https://github.com.helper &>/dev/null; then
  gh auth setup-git
fi

# Cockpit setup
cockpit_init() {
  local cockpit_dir="${COCKPIT_DIR:-$HOME/ai-cockpit}"

  mkdir -p "$cockpit_dir/state/investigations"
  mkdir -p "$cockpit_dir/state/designs"
  mkdir -p "$cockpit_dir/state/explorations"
  mkdir -p "$cockpit_dir/local"

  if [[ ! -d "$cockpit_dir/.git" ]]; then
    echo "Initializing cockpit at $cockpit_dir"
    git -C "$cockpit_dir" init
    cat > "$cockpit_dir/.gitignore" << 'GITIGNORE'
.beads/
local/
.claude/
GITIGNORE
    echo "Cockpit initialized. Run 'bd init' in $cockpit_dir to set up beads."
  fi

  if [[ ! -f "$cockpit_dir/project-tree.json" ]]; then
    cat > "$cockpit_dir/project-tree.json" << 'JSON'
{
  "projects": []
}
JSON
  fi
}

build_dashboard() {
  if command -v cargo >/dev/null 2>&1; then
    echo "Building cockpit-dash..."
    mkdir -p "$HOME/.local/bin"
    (cd "$DOTFILES/cockpit-dash" && cargo build --release) 2>&1 | tail -5
    install -m 755 "$DOTFILES/cockpit-dash/target/release/cockpit-dash" "$HOME/.local/bin/cockpit-dash"
  else
    echo "Rust/cargo not installed — skipping cockpit-dash build"
  fi
}

build_krust() {
  if command -v cargo >/dev/null 2>&1; then
    echo "Building krust..."
    mkdir -p "$HOME/.local/bin"
    (cd "$DOTFILES/krust" && cargo build --release) 2>&1 | tail -5
    install -m 755 "$DOTFILES/krust/target/release/krust" "$HOME/.local/bin/krust"
  else
    echo "Rust/cargo not installed — skipping krust build"
  fi
}

setup_zellij() {
  local zellij_config="$HOME/.config/zellij"
  mkdir -p "$zellij_config/layouts"
  mkdir -p "$zellij_config/plugins"

  # Symlink config and layout
  link_file "zellij/config.kdl" "$zellij_config/config.kdl"
  link_file "zellij/layouts/cockpit.kdl" "$zellij_config/layouts/cockpit.kdl"

  # Download zjstatus plugin if missing
  if [[ ! -f "$zellij_config/plugins/zjstatus.wasm" ]]; then
    echo "Downloading zjstatus plugin..."
    curl -sL "https://github.com/dj95/zjstatus/releases/latest/download/zjstatus.wasm" \
      -o "$zellij_config/plugins/zjstatus.wasm" 2>&1 | tail -3
  fi
}

cockpit_init
build_dashboard
build_krust
setup_zellij

# Krust config dir (extra trees populate the config file)
mkdir -p "$HOME/.krust"

# Source any extra dotfile trees passed as args. Each tree must expose
# merge.sh at its root; helpers above are in scope, and $EXTRA_DIR points
# at the tree's absolute path.
for extra in "$@"; do
  abs="$(cd "$extra" 2>/dev/null && pwd)" || { echo "WARN: extra tree '$extra' not found — skipping"; continue; }
  if [[ ! -f "$abs/merge.sh" ]]; then
    echo "WARN: $abs/merge.sh not found — skipping"
    continue
  fi
  EXTRA_DIR="$abs" source "$abs/merge.sh"
done

echo ""
echo "Done! You may need to restart your shell."
