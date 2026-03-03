#!/bin/sh
set -eu

ARLIAMP_TAP="0smboy/arliamp"
RLIAMP_TAP="0smboy/rliamp"
UNIMATRIX_PKG="git+https://github.com/will8211/unimatrix.git"
PM="${ARLIAMP_PM:-auto}"

info() {
  printf '==> %s\n' "$*"
}

warn() {
  printf 'Warning: %s\n' "$*" >&2
}

die() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

ensure_ruby3_for_zb() {
  if has_cmd ruby && ruby -e 'exit(RUBY_VERSION.split(".").first.to_i >= 3 ? 0 : 1)'; then
    return 0
  fi

  if has_cmd brew; then
    info "Installing Ruby 3+ via Homebrew for zerobrew compatibility..."
    brew install ruby
  fi

  if [ -d /opt/homebrew/opt/ruby/bin ]; then
    PATH="/opt/homebrew/opt/ruby/bin:$PATH"
    export PATH
  fi
  if [ -d /usr/local/opt/ruby/bin ]; then
    PATH="/usr/local/opt/ruby/bin:$PATH"
    export PATH
  fi

  has_cmd ruby || die "Ruby 3+ is required for zerobrew."
  ruby -e 'exit(RUBY_VERSION.split(".").first.to_i >= 3 ? 0 : 1)' ||
    die "Ruby 3+ is required for zerobrew."
}

install_with_brew() {
  has_cmd brew || die "Homebrew not found."

  info "Installing tmux..."
  brew install tmux python

  info "Installing rliamp..."
  brew tap "$RLIAMP_TAP" "https://github.com/$RLIAMP_TAP"
  brew install "$RLIAMP_TAP/rliamp"

  info "Installing arliamp..."
  brew tap "$ARLIAMP_TAP" "https://github.com/$ARLIAMP_TAP"
  brew install "$ARLIAMP_TAP/arliamp"
}

install_with_zb() {
  has_cmd zb || die "zerobrew (zb) not found."
  ensure_ruby3_for_zb

  info "Installing tmux..."
  zb install tmux

  info "Installing rliamp..."
  zb install "$RLIAMP_TAP/rliamp"

  info "Installing arliamp..."
  zb install "$ARLIAMP_TAP/arliamp"
}

install_unimatrix() {
  py=""
  if has_cmd python3; then
    py="python3"
  elif has_cmd python; then
    py="python"
  fi

  [ -n "$py" ] || die "Python is required to install unimatrix."

  info "Installing unimatrix..."
  "$py" -m pip install --user --upgrade "$UNIMATRIX_PKG"
}

postflight() {
  if [ ! -d /Applications/Ghostty.app ]; then
    warn "Ghostty not found at /Applications/Ghostty.app (required at runtime)."
  fi

  if has_cmd python3; then
    user_base="$(python3 -c 'import site; print(site.USER_BASE)' 2>/dev/null || true)"
    if [ -n "$user_base" ] && [ -d "$user_base/bin" ]; then
      case ":$PATH:" in
      *:"$user_base/bin":*) ;;
      *) warn "Add $user_base/bin to PATH for unimatrix." ;;
      esac
    fi
  fi

  if has_cmd zb; then
    case ":$PATH:" in
    *:/opt/zerobrew/bin:*) ;;
    *) [ -d /opt/zerobrew/bin ] && warn "Add /opt/zerobrew/bin to PATH." ;;
    esac
  fi

  info "Install complete."
  info "Run: arliamp \"/path/to/music-or-url\""
}

select_pm() {
  case "$PM" in
  brew|zb) ;;
  auto)
    if has_cmd brew; then
      PM="brew"
    elif has_cmd zb; then
      PM="zb"
    else
      die "Neither Homebrew nor zerobrew found. Install one first."
    fi
    ;;
  *)
    die "Invalid ARLIAMP_PM='$PM' (use brew, zb, or auto)."
    ;;
  esac
}

main() {
  select_pm
  info "Selected package manager: $PM"

  case "$PM" in
  brew) install_with_brew ;;
  zb) install_with_zb ;;
  esac

  install_unimatrix
  postflight
}

main "$@"
