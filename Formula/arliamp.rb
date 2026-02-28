class Arliamp < Formula
  desc "Isolated cyber stage launcher for rliamp in Ghostty"
  homepage "https://github.com/0smboy/arliamp"
  url "https://github.com/0smboy/arliamp/releases/download/v1/arliamp-v1-src.tar.gz"
  sha256 "9aa7620c6b18e88df9e4ec12a9a599d72f6f3dbc709eafaa48000fe6cccec419"
  license :cannot_represent

  depends_on "rust" => :build
  depends_on "tmux"

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  def caveats
    <<~EOS
      arliamp runtime dependencies:
        - Ghostty app at /Applications/Ghostty.app
        - unimatrix executable in PATH
        - rliamp executable in PATH
    EOS
  end

  test do
    output = shell_output("#{bin}/arliamp 2>&1", 1)
    assert_match "Usage: arliamp <music-directory>", output
  end
end
