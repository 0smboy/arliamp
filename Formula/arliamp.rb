class Arliamp < Formula
  desc "Isolated cyber stage launcher for rliamp in Ghostty"
  homepage "https://github.com/0smboy/arliamp"
  url "https://github.com/0smboy/arliamp/releases/download/v1/arliamp-v1-src.tar.gz"
  sha256 "a0138e2c3586c69a88efa30d9b0d4cd7ab5a126722a9593bffb5fe27b9bbc8f7"
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
