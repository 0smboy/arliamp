class Arliamp < Formula
  desc "Isolated cyber stage launcher for rliamp in Ghostty"
  homepage "https://github.com/0smboy/arliamp"
  url "https://github.com/0smboy/arliamp/archive/refs/tags/v1.tar.gz"
  sha256 "01159bd1541c666c08b960a98f35c0e33a1ae61fcc05f071cb17d01c9f77159c"
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
