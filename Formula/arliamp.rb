class Arliamp < Formula
  desc "Isolated cyber stage launcher for rliamp in Ghostty"
  homepage "https://github.com/0smboy/arliamp"
  url "https://github.com/0smboy/arliamp/archive/refs/tags/v1.2.0.tar.gz"
  sha256 "a95bb17d1f4f948b7160355fb4d35803df4754976dc57f4e25482e9fab4d04fc"
  license :cannot_represent

  depends_on "rust" => :build
  depends_on "tmux"

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
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
    output = shell_output("#{bin}/arliamp /definitely/not/found 2>&1", 1)
    assert_match "arliamp: path not found", output
  end
end
