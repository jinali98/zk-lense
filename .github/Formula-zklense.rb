# Formula file for Homebrew tap
# This file should be placed at: Formula/zklense.rb in the gihanrcg/homebrew-zklense repository
# Repository: https://github.com/gihanrcg/homebrew-zklense

class Zklense < Formula
  desc "A command-line tool for profiling, building, and deploying zero-knowledge proofs built with Noir for Solana Blockchain"
  homepage "https://github.com/jinali98/zk-lense"
  version "0.1.0"
  license "MIT OR Apache-2.0"

  # Determine the correct binary based on architecture
  if Hardware::CPU.intel?
    url "https://github.com/jinali98/zk-lense/releases/download/v0.1.0/zklense-x86_64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_INTEL_SHA256"
  else
    url "https://github.com/jinali98/zk-lense/releases/download/v0.1.0/zklense-aarch64-apple-darwin.tar.gz"
    sha256 "REPLACE_WITH_ARM_SHA256"
  end

  def install
    bin.install "zklense"
  end

  test do
    system "#{bin}/zklense", "--version"
  end
end
