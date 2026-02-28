class Arliamp < Formula
  desc "Isolated cyber stage launcher for rliamp in Ghostty"
  homepage "https://github.com/0smboy/arliamp"
  url "https://github.com/0smboy/arliamp/archive/refs/tags/v1.tar.gz"
  sha256 "3ade813b4f5b70ff62fb09d25c360b9a0b0c453f59aaa2b8eae3f9a580072f13"
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
