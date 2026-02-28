# arliamp
<img width="3492" height="2286" alt="4fdff31e-d4e4-4e8d-ba04-388cadc5d591" src="https://github.com/user-attachments/assets/dc9a2eb1-c358-4b38-b2c0-79e66b8c0d70" />


`arliamp` launches an isolated, full-screen Ghostty session for `rliamp` with a cyber stage wrapper (matrix background + centered player pane), without touching global Ghostty config.

## Install arliamp

```bash
# Homebrew
brew tap 0smboy/arliamp https://github.com/0smboy/arliamp
brew install 0smboy/arliamp/arliamp

# ZeroBrew (requires Ruby 3 in PATH)
brew install ruby
export PATH="/opt/homebrew/opt/ruby/bin:$PATH"
zb install 0smboy/arliamp/arliamp
export PATH="/opt/zerobrew/bin:$PATH"
```

## Runtime Requirements

- macOS with Ghostty installed at `/Applications/Ghostty.app`
- `tmux` in `PATH`
- `unimatrix` in `PATH`:

```bash
pip install git+https://github.com/will8211/unimatrix.git
```

- `rliamp` in `PATH` (source: [0smboy/rliamp](https://github.com/0smboy/rliamp)):

```bash
# from rliamp tap
brew tap 0smboy/rliamp https://github.com/0smboy/rliamp
brew install 0smboy/rliamp/rliamp

# or
zb install 0smboy/rliamp/rliamp
```

## Usage

```bash
arliamp "/path/to/music-directory"
```

Inside the session:

- `v`: shader rotation `off -> static -> crazy -> off`
- `q`: quit `rliamp` and close the whole stage window

## Release Assets Policy

For each GitHub release:

- Asset 1: compiled binary (`arliamp-macos-aarch64`)
- Asset 2: compiled binary (`arliamp-macos-x86_64`)
- Asset 3+: source packages (`Source code (zip)` / `Source code (tar.gz)` provided by GitHub)

If you want manual install without brew, download the binary for your CPU and place it in a directory in your `PATH`, for example `/usr/local/bin/arliamp`.

## ZeroBrew Notes

- `zb` source builds execute a Ruby shim. On macOS, system Ruby (`/usr/bin/ruby`, 2.6) is too old.
- Use Ruby 3 first in `PATH` before running `zb install`:

```bash
brew install ruby
export PATH="/opt/homebrew/opt/ruby/bin:$PATH"
ruby -v
zb install 0smboy/arliamp/arliamp
```

If `arliamp` is installed but `command not found`, ensure:

```bash
export PATH="/opt/zerobrew/bin:$PATH"
hash -r
which arliamp
```
