# arliamp

`arliamp` launches an isolated, full-screen Ghostty session for `rliamp` with a cyber stage wrapper (matrix background + centered player pane), without touching global Ghostty config.

## Requirements

- macOS with Ghostty installed at `/Applications/Ghostty.app`
- `tmux` in `PATH`
- `unimatrix` in `PATH`
- `rliamp` in `PATH`

## Usage

```bash
arliamp "/path/to/music-directory"
```

Inside the session:

- `v`: shader rotation `off -> static -> crazy -> off`
- `q`: quit `rliamp` and close the whole stage window

## Install (Homebrew / ZeroBrew)

```bash
# Homebrew
brew tap 0smboy/arliamp https://github.com/0smboy/arliamp
brew install 0smboy/arliamp/arliamp

# ZeroBrew
zb install 0smboy/arliamp/arliamp
```
