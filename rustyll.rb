# Homebrew Formula for Rustyll
# To publish: Copy this file to your homebrew-tap repository

class Rustyll < Formula
  desc "Blazing fast, Jekyll-compatible static site generator written in Rust"
  homepage "https://rustyll.better-web.org"
  url "https://github.com/betterwebinit/rustyll/archive/refs/tags/v0.8.0.tar.gz"
  sha256 "" # Will be calculated after GitHub release
  license "AGPL-3.0"
  head "https://github.com/betterwebinit/rustyll.git", branch: "main"

  depends_on "rust" => :build

  def install
    cd "rustyll" do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    system "#{bin}/rustyll", "--version"
    system "#{bin}/rustyll", "new", "test-site"
    assert_predicate testpath/"test-site/_config.yml", :exist?
  end
end
